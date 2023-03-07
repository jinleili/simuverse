use app_surface::math::{Position, Size};
use std::usize;

pub mod framework;

mod egui_lib;
pub(crate) use egui_lib::*;

mod egui_layer;
pub use egui_layer::EguiLayer;

pub(crate) mod geometries;

pub(crate) mod node;
pub mod noise;

mod setting;
pub use setting::*;

mod field_simulator;
pub use field_simulator::FieldSimulator;

mod fluid;
pub use fluid::FluidSimulator;

mod field_velocity_code;
pub use field_velocity_code::get_velocity_code_snippet;

pub mod pbd;

pub mod util;
use util::shader::{create_shader_module, insert_code_then_create};
use util::vertex::{PosColor as PosTangent, PosOnly};

pub static DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MVPMatUniform {
    mv: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    mvp: [[f32; 4]; 4],
    mv_no_rotation: [[f32; 4]; 4],
    normal: [[f32; 4]; 4],
    u_time: f32,
    _padding: [f32; 3],
}

pub trait Simulator {
    fn update_uniforms(&mut self, _app: &app_surface::AppSurface, _setting: &crate::SettingObj) {}

    fn on_click(&mut self, _app: &app_surface::AppSurface, _pos: Position) {}

    fn touch_begin(&mut self, _app: &app_surface::AppSurface) {}

    fn touch_move(&mut self, _app: &app_surface::AppSurface, _pos: Position) {}

    fn touch_end(&mut self, _app: &app_surface::AppSurface) {}

    fn reset(&mut self, _app: &app_surface::AppSurface) {}

    fn resize(&mut self, _app: &app_surface::AppSurface) -> bool {
        false
    }

    fn update_by(&mut self, app: &app_surface::AppSurface, control_panel: &mut crate::ControlPanel);
    fn update_workgroup_count(
        &mut self,
        app: &app_surface::AppSurface,
        workgroup_count: (u32, u32, u32),
    );

    fn compute(&mut self, _encoder: &mut wgpu::CommandEncoder) {}

    fn draw_by_rpass<'b, 'a: 'b>(
        &'a mut self,
        app: &app_surface::AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
        setting: &mut crate::SettingObj,
    );
}

#[derive(Clone, Copy, PartialEq)]
pub enum SimuType {
    Field = 0,
    Fluid,
    Noise,
    PBDynamic,
    D3Fluid,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FieldAnimationType {
    Basic = 0,
    JuliaSet,
    Spirl,
    BlackHole,
    Poiseuille,
    LidDrivenCavity,
    Custom,
}

impl FieldAnimationType {
    pub fn from_u32(ty: u32) -> Self {
        match ty {
            0 => FieldAnimationType::Basic,
            1 => FieldAnimationType::JuliaSet,
            2 => FieldAnimationType::Spirl,
            3 => FieldAnimationType::BlackHole,
            4 => FieldAnimationType::Poiseuille,
            5 => FieldAnimationType::LidDrivenCavity,
            _ => FieldAnimationType::Custom,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum ParticleColorType {
    #[default]
    MovementAngle = 0,
    Speed = 1,
    Uniform = 2,
}

impl ParticleColorType {
    pub fn from_u32(ty: u32) -> Self {
        match ty {
            0 => ParticleColorType::MovementAngle,
            1 => ParticleColorType::Speed,
            _ => ParticleColorType::Uniform,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FieldUniform {
    // 场格子数
    pub lattice_size: [i32; 2],
    // 格子所占像素数
    pub lattice_pixel_size: [f32; 2],
    // 画布像素数
    pub canvas_size: [i32; 2],
    // 投影屏幕宽高比
    pub proj_ratio: [f32; 2],
    // 单个像素在 NDC 空间中的大小
    pub ndc_pixel: [f32; 2],
    // 0: pixel speed, field simulator used
    // 1: lbm lattice speed, fluid simulator used. Its value is usually no greater than 0.2
    pub speed_ty: i32,
    // 用于字节对齐
    pub _padding: f32,
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleUniform {
    // particle uniform color
    pub color: [f32; 4],
    // total particle number
    pub num: [i32; 2],
    pub point_size: i32,
    pub life_time: f32,
    pub fade_out_factor: f32,
    pub speed_factor: f32,
    // particle color type
    // 0: uniform color; 1: use velocity as particle color, 2: angle as color
    pub color_ty: i32,
    // 1: not draw on the canvas
    pub is_only_update_pos: i32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TrajectoryParticle {
    pub pos: [f32; 2],
    pub pos_initial: [f32; 2],
    pub life_time: f32,
    pub fade: f32,
}

impl TrajectoryParticle {
    pub fn zero() -> Self {
        TrajectoryParticle {
            pos: [0.0, 0.0],
            pos_initial: [0.0, 0.0],
            life_time: 0.0,
            fade: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle3D {
    pub pos: [f32; 4],
    // initial position, use to reset particle position
    pos_initial: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TrajectoryUniform {
    screen_factor: [f32; 2],
    // which view particles position will drawing to.
    trajectory_view_index: i32,
    bg_view_index: i32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Pixel {
    pub alpha: f32,
    // absolute velocity
    pub speed: f32,
    // density
    pub rho: f32,
}

use rand::{prelude::Distribution, Rng};

const MAX_PARTICLE_COUNT: usize = 205000;
fn get_particles_data(
    canvas_size: Size<u32>,
    count: i32,
    life_time: f32,
) -> (wgpu::Extent3d, (u32, u32, u32), Vec<TrajectoryParticle>) {
    let ratio = canvas_size.width as f32 / canvas_size.height as f32;
    let x = (count as f32 * ratio).sqrt().ceil();
    let particles_size = wgpu::Extent3d {
        width: x as u32,
        height: (x * (1.0 / ratio)).ceil() as u32,
        depth_or_array_layers: 1,
    };
    let workgroup_count = (
        (particles_size.width + 15) / 16,
        (particles_size.height + 15) / 16,
        1,
    );

    let mut particles = init_trajectory_particles(canvas_size, particles_size, life_time);
    if MAX_PARTICLE_COUNT > particles.len() {
        for _i in 0..(MAX_PARTICLE_COUNT - particles.len()) {
            particles.push(TrajectoryParticle::zero());
        }
    }
    (particles_size, workgroup_count, particles)
}

fn init_trajectory_particles(
    canvas_size: Size<u32>,
    num: wgpu::Extent3d,
    life_time: f32,
) -> Vec<TrajectoryParticle> {
    let mut data: Vec<TrajectoryParticle> = vec![];
    let mut rng = rand::thread_rng();
    let step_x = canvas_size.width as f32 / (num.width - 1) as f32;
    let step_y = canvas_size.height as f32 / (num.height - 1) as f32;
    let unif_x = rand::distributions::Uniform::new_inclusive(-step_x, step_x);
    let unif_y = rand::distributions::Uniform::new_inclusive(-step_y, step_y);
    let unif_life = rand::distributions::Uniform::new_inclusive(
        0.0,
        if life_time <= 0.0 { 1.0 } else { life_time },
    );

    for x in 0..num.width {
        let pixel_x = step_x * x as f32;
        for y in 0..num.height {
            let pos = [
                pixel_x + unif_x.sample(&mut rng),
                step_y * y as f32 + unif_y.sample(&mut rng),
            ];
            let pos_initial = if life_time <= 1.0 {
                [rng.gen_range(0.0..step_x), pos[1]]
            } else {
                pos
            };
            data.push(TrajectoryParticle {
                pos,
                pos_initial,
                life_time: if life_time <= 1.0 {
                    0.0
                } else {
                    unif_life.sample(&mut rng)
                },
                fade: 0.0,
            });
        }
    }

    data
}

pub fn generate_circle_plane(r: f32, fan_segment: usize) -> (Vec<PosOnly>, Vec<u32>) {
    // WebGPU 1.0 not support Triangle_Fan primitive
    let mut vertex_list: Vec<PosOnly> = Vec::with_capacity(fan_segment + 2);
    let z = 0.0_f32;
    vertex_list.push(PosOnly { pos: [0.0, 0.0, z] });
    vertex_list.push(PosOnly { pos: [r, 0.0, z] });

    let mut index_list: Vec<u32> = Vec::with_capacity(fan_segment * 3);

    let step = (std::f32::consts::PI * 2.0) / fan_segment as f32;
    for i in 1..=fan_segment {
        let angle = step * i as f32;
        vertex_list.push(PosOnly {
            pos: [r * angle.cos(), r * angle.sin(), z],
        });
        index_list.push(0);
        index_list.push(i as u32);
        if i == fan_segment {
            index_list.push(1);
        } else {
            index_list.push(i as u32 + 1);
        }
    }
    (vertex_list, index_list)
}

// 光盘平面
pub fn generate_disc_plane(
    min_r: f32,
    max_r: f32,
    fan_segment: usize,
) -> (Vec<PosTangent>, Vec<u32>) {
    // WebGPU 1.0 not support Triangle_Fan primitive
    let mut vertex_list: Vec<PosTangent> = Vec::with_capacity(fan_segment);
    let z = 0.0_f32;
    vertex_list.push(PosTangent {
        pos: [min_r, 0.0, z],
        color: [0.0, 1.0, z, 1.0],
    });
    vertex_list.push(PosTangent {
        pos: [max_r, 0.0, z],
        color: [0.0, 1.0, z, 1.0],
    });

    let tangent_r = 1.0;
    let tan_offset_angle = std::f32::consts::FRAC_PI_2;

    let mut index_list: Vec<u32> = Vec::with_capacity(fan_segment * 6);

    let step = (std::f32::consts::PI * 2.0) / fan_segment as f32;
    for i in 1..fan_segment {
        let angle = step * i as f32;
        // 切线只表达大小与方向，可以任意平移，so, Z 与平面的 Z 坐标无关
        let tangent = [
            tangent_r * (angle + tan_offset_angle).cos(),
            tangent_r * (angle + tan_offset_angle).sin(),
            0.0,
            1.0,
        ];
        vertex_list.push(PosTangent {
            pos: [min_r * angle.cos(), min_r * angle.sin(), z],
            color: tangent,
        });
        vertex_list.push(PosTangent {
            pos: [max_r * angle.cos(), max_r * angle.sin(), z],
            color: tangent,
        });
        let index = i as u32 * 2;
        index_list.append(&mut vec![
            index - 2,
            index - 1,
            index,
            index,
            index - 1,
            index + 1,
        ]);
    }
    let index = (fan_segment - 1) as u32 * 2;
    index_list.append(&mut vec![index, index + 1, 0, 0, index + 1, 1]);

    (vertex_list, index_list)
}
