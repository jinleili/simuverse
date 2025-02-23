use crate::util::BufferObj;
use crate::{FieldAnimationType, ParticleColorType, ParticleUniform, SimuType, get_particles_data};

pub struct SettingObj {
    canvas_size: glam::UVec2,
    pub simu_type: SimuType,
    pub animation_type: FieldAnimationType,
    pub color_ty: ParticleColorType,
    pub fluid_viscosity: f32,

    pub particles_count: i32,
    pub particles_uniform_data: ParticleUniform,
    pub particles_uniform: Option<BufferObj>,
    pub particles_buf: Option<BufferObj>,
    pub particles_size: wgpu::Extent3d,
    pub particles_workgroup_count: (u32, u32, u32),
}

impl SettingObj {
    pub fn new(
        simu_type: SimuType,
        animation_type: FieldAnimationType,
        color_ty: ParticleColorType,
        particles_count: i32,
        particle_lifetime: f32,
        point_size: i32,
    ) -> Self {
        SettingObj {
            canvas_size: (0_u32, 0_u32).into(),
            simu_type,
            animation_type,
            fluid_viscosity: 0.02,
            color_ty,
            particles_count,
            particles_size: wgpu::Extent3d {
                width: 0,
                height: 0,
                depth_or_array_layers: 1,
            },
            particles_workgroup_count: (0, 0, 1),
            particles_buf: None,
            particles_uniform: None,
            particles_uniform_data: ParticleUniform {
                color: [1.0; 4],
                num: [0; 2],
                point_size,
                life_time: particle_lifetime,
                fade_out_factor: 0.96,
                speed_factor: if simu_type == SimuType::Field {
                    1.0
                } else {
                    4.15
                },
                color_ty: color_ty as i32,
                is_only_update_pos: 1,
            },
        }
    }

    pub fn update_field_type(&mut self, app: &app_surface::AppSurface, ty: SimuType) -> bool {
        if self.simu_type != ty {
            self.simu_type = ty;
            self.particles_uniform_data.speed_factor = if self.simu_type == SimuType::Field {
                1.0
            } else {
                4.15
            };
            self.update_particles_uniform(app);
            true
        } else {
            false
        }
    }

    pub fn update_canvas_size(&mut self, app: &app_surface::AppSurface, canvas_size: glam::UVec2) {
        self.canvas_size = canvas_size;
        self.update_particles_data(app);
    }

    pub fn update_particles_count(&mut self, app: &app_surface::AppSurface, count: i32) -> bool {
        if self.particles_count == count {
            return false;
        }
        self.particles_count = count;
        self.update_particles_data(app);
        true
    }

    pub fn update_particle_life(&mut self, app: &app_surface::AppSurface, lifetime: f32) {
        if (self.particles_uniform_data.life_time - lifetime).abs() < 1.0 {
            return;
        }
        self.particles_uniform_data.life_time = lifetime;
        self.update_particles_data(app);
    }

    pub fn update_particle_color(
        &mut self,
        app: &app_surface::AppSurface,
        color_type: crate::ParticleColorType,
    ) {
        self.color_ty = color_type;
        self.particles_uniform_data.color_ty = color_type as i32;
        self.update_particles_uniform(app);
    }

    pub fn update_particle_point_size(&mut self, app: &app_surface::AppSurface, point_size: i32) {
        if self.particles_uniform_data.point_size == point_size {
            return;
        }
        self.particles_uniform_data.point_size = point_size;
        self.update_particles_uniform(app);
    }

    pub fn update_particles_uniform(&self, app: &app_surface::AppSurface) {
        app.queue.write_buffer(
            &self.particles_uniform.as_ref().unwrap().buffer,
            0,
            bytemuck::bytes_of(&self.particles_uniform_data),
        );
    }

    fn update_particles_data(&mut self, app: &app_surface::AppSurface) {
        let (particles_size, particles_workgroup_count, particles) = get_particles_data(
            self.canvas_size,
            self.particles_count,
            self.particles_uniform_data.life_time,
        );
        self.particles_size = particles_size;
        self.particles_workgroup_count = particles_workgroup_count;
        self.particles_uniform_data.num = [
            self.particles_size.width as i32,
            self.particles_size.height as i32,
        ];
        if let Some(buf) = self.particles_buf.as_ref() {
            self.update_particles_uniform(app);
            app.queue
                .write_buffer(&buf.buffer, 0, bytemuck::cast_slice(&particles));
        } else {
            self.particles_buf = Some(BufferObj::create_buffer(
                &app.device,
                Some(&particles),
                None,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
                Some("particles_buf"),
            ));
            self.particles_uniform = Some(BufferObj::create_uniform_buffer(
                &app.device,
                &self.particles_uniform_data,
                Some("particle_uniform"),
            ));
        }
    }
}
