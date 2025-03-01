use super::{BendingConstraintObj, MeshColoringObj, StretchConstraintObj};
use crate::util::vertex::PosParticleIndex;
use alloc::{vec, vec::Vec};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleBufferObj {
    pub pos: [f32; 4],
    pub old_pos: [f32; 4],
    pub accelerate: [f32; 4],
    // uv and invert_mass
    // 为了字节对齐
    pub uv_mass: [f32; 4],
    // 与之相连的4个粒子的索引，用于计算法线
    pub connect: [i32; 4],
}

// Every interior vertex is defined by the joining of four or eight identical triangles
// that meet isotropically in x and y.This topology guarantees more symmetric behavior.
// ┌──────┬──────┬──────┐
// │     /│\     │     /│
// │   /  │  \   │   /  │
// │ /    │    \ │ /    │
// ├──────┼──────┼──────┤
// │\     │     /│\     │
// │  \   │   /  │  \   │
// │    \ │ /    │    \ │
// ├──────┼──────┼──────┤
// │     /│\     │     /│
// │   /  │  \   │   /  │
// │ /    │    \ │ /    │
// └──────┴──────┴──────┘

pub struct ClothFabric {
    pub horizontal_num: usize,
    pub vertical_num: usize,
    pub vertices: (Vec<PosParticleIndex>, Vec<u32>),
    pub particles: Vec<ParticleBufferObj>,
    pub stretch_constraints: (Vec<MeshColoringObj>, Vec<StretchConstraintObj>),
    pub bend_constraints: (Vec<MeshColoringObj>, Vec<BendingConstraintObj>),
}
impl ClothFabric {
    pub fn gen_fabric(
        horizontal_num: usize,
        vertical_num: usize,
        horizontal_pixel: f32,
        vertical_pixel: f32,
        a_pixel_on_ndc: f32,
    ) -> Self {
        let mut vertex_data: Vec<PosParticleIndex> =
            Vec::with_capacity(horizontal_num * vertical_num);
        let mut index_data: Vec<u32> = Vec::new();
        for h in 0..vertical_num {
            let mh = h % 2;
            let offset = (horizontal_num * h) as u32;
            for w in 0..horizontal_num {
                vertex_data.push(PosParticleIndex::new([w as u32, h as u32, 0]));
                if h == 0 || w == 0 {
                    continue;
                }
                let mw = w % 2;
                let current: u32 = offset + w as u32;
                let left = current - 1;
                // 找到上一行同一行位置的索引
                let top: u32 = current - horizontal_num as u32;
                if (mh == 0 && mw == 0) || (mh == 1 && mw == 1) {
                    index_data.append(&mut vec![top, top - 1, left, left, current, top]);
                } else {
                    index_data.append(&mut vec![current, top, top - 1, top - 1, left, current]);
                }
            }
        }

        let mut particles = Vec::with_capacity(horizontal_num * vertical_num);

        let horizontal_step = horizontal_pixel / (horizontal_num - 1) as f32 * a_pixel_on_ndc;
        let vertical_step = vertical_pixel / (vertical_num - 1) as f32 * a_pixel_on_ndc;
        let uv_x_step = 1.0 / (horizontal_num - 1) as f32;
        let uv_y_step = 1.0 / (vertical_num - 1) as f32;

        let tl_x = (-horizontal_step) * ((horizontal_num - 1) as f32 / 2.0);
        let tl_y = vertical_step * ((vertical_num - 1) as f32 / 2.0);
        for h in 0..vertical_num {
            for w in 0..horizontal_num {
                let p = [
                    tl_x + horizontal_step * w as f32,
                    tl_y - vertical_step * h as f32,
                    0.0,
                    0.0,
                ];
                // 上边两个角固定：粒子质量为 无穷大
                // 每个顶点的质量等于与之相连的每个三角形质量的 1/3 之后
                let invert_mass = if h == 0 && (w == 0 || w == horizontal_num - 1) {
                    0.0
                } else if w == 0 || w == (horizontal_num - 1) || h == (vertical_num - 1) {
                    // 边界上的点，只有两个三角形与之相连
                    0.2
                } else {
                    0.1
                };
                particles.push(ParticleBufferObj {
                    pos: p,
                    old_pos: p,
                    // 重力加速度不能太小，会导致布料飘来飘去，没有重量感
                    // accelerate: [0.0, 0.0, 0.0, 0.0],
                    accelerate: [0.0, -3.98, 0.0, 0.11],
                    uv_mass: [uv_x_step * w as f32, uv_y_step * h as f32, invert_mass, 0.0],
                    connect: [0; 4],
                })
            }
        }
        // 与粒子直接相邻的其它粒子
        cal_connected_particles(&mut particles, horizontal_num, vertical_num);

        let stretch_constraints = super::gen_cloth_constraints::generate_stretch_constraints(
            horizontal_num,
            vertical_num,
            &particles,
        );

        // 目前 bend constraints 的生成花费了 80% 的总计算时间
        // let bend_constraints = super::gen_cloth_constraints::generate_bend_constraints(
        //     horizontal_num as i32,
        //     vertical_num as i32,
        //     &particles,
        // );
        let bend_constraints = super::gen_cloth_constraints::generate_bend_constraints2(
            horizontal_num,
            vertical_num,
            &particles,
        );

        Self {
            horizontal_num,
            vertical_num,
            vertices: (vertex_data, index_data),
            particles,
            stretch_constraints,
            bend_constraints,
        }
    }
}

fn cal_connected_particles(
    particles: &mut [ParticleBufferObj],
    horizontal_num: usize,
    vertical_num: usize,
) {
    for h in 0..vertical_num {
        for w in 0..horizontal_num {
            let index0 = h * horizontal_num + w;
            let particle0 = &mut particles[index0];

            if h == 0 {
                if w == 0 {
                    // 左上角
                    particle0.connect[0] = index0 as i32 + 1;
                    particle0.connect[1] = (index0 + horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else if w == horizontal_num - 1 {
                    // 右上角
                    particle0.connect[0] = (index0 + horizontal_num) as i32;
                    particle0.connect[1] = (index0 - 1) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else {
                    particle0.connect[0] = index0 as i32 + 1;
                    particle0.connect[1] = (index0 + horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[1];
                    particle0.connect[3] = (index0 - 1) as i32;
                }
            } else if h == vertical_num - 1 {
                if w == 0 {
                    // 左下角
                    particle0.connect[0] = (index0 - horizontal_num) as i32;
                    particle0.connect[1] = (index0 + 1) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else if w == horizontal_num - 1 {
                    // 右下角
                    particle0.connect[0] = (index0 - 1) as i32;
                    particle0.connect[1] = (index0 - horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else {
                    // 底边
                    particle0.connect[0] = (index0 - 1) as i32;
                    particle0.connect[1] = (index0 - horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[1];
                    particle0.connect[3] = (index0 + 1) as i32;
                }
            } else if w == 0 {
                // 左竖边
                particle0.connect[0] = (index0 - horizontal_num) as i32;
                particle0.connect[1] = (index0 + 1) as i32;
                particle0.connect[2] = particle0.connect[1];
                particle0.connect[3] = (index0 + horizontal_num) as i32;
            } else if w == horizontal_num - 1 {
                // 右竖边
                particle0.connect[0] = (index0 + horizontal_num) as i32;
                particle0.connect[1] = (index0 - 1) as i32;
                particle0.connect[2] = particle0.connect[1];
                particle0.connect[3] = (index0 - horizontal_num) as i32;
            } else {
                particle0.connect[0] = (index0 - horizontal_num) as i32;
                particle0.connect[1] = (index0 + 1) as i32;
                particle0.connect[2] = (index0 + horizontal_num) as i32;
                particle0.connect[3] = (index0 - 1) as i32;
            }
        }
    }
}
