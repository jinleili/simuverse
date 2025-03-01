use super::{OBSTACLE_RADIUS, d2q9_node::D2Q9Node};
use crate::{
    FieldAnimationType, SettingObj, Simulator,
    fluid::LbmUniform,
    node::{BindGroupData, BufferlessFullscreenNode, ComputeNode},
    util::BufferObj,
};
use alloc::vec;
use wgpu::TextureFormat;

use crate::create_shader_module;

// 通用的流體模擬，產生外部依賴的流體量
pub struct FluidSimulator {
    lattice: wgpu::Extent3d,
    lattice_pixel_size: u32,
    pre_pos: glam::Vec2,
    fluid_compute_node: D2Q9Node,
    _curl_cal_node: ComputeNode,
    particle_update_node: ComputeNode,
    _render_node: BufferlessFullscreenNode,
    particle_render: BufferlessFullscreenNode,
}

impl FluidSimulator {
    pub fn new(
        app: &app_surface::AppSurface,
        canvas_size: glam::UVec2,
        canvas_buf: &BufferObj,
        setting: &SettingObj,
    ) -> Self {
        let device = &app.device;
        let fluid_compute_node = D2Q9Node::new(app, canvas_size, setting);
        let lattice = fluid_compute_node.lattice;

        let curl_shader =
            create_shader_module(device, "lbm/curl_update", Some("curl_update_shader"));
        // iOS cannot create R32Float texture, R16Float cannot use to storage texture
        // Need enable TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES feature
        let curl_texture_format = TextureFormat::Rgba16Float;
        // let curl_texture_format = TextureFormat::R16Float;
        let curl_tex = crate::util::load_texture::empty(
            device,
            curl_texture_format,
            wgpu::Extent3d {
                width: lattice.width,
                height: lattice.height,
                depth_or_array_layers: 1,
            },
            None,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            Some("curl_tex"),
        );
        let bind_group_data = BindGroupData {
            workgroup_count: fluid_compute_node.workgroup_count,
            uniforms: vec![
                &fluid_compute_node.lbm_uniform_buf,
                &fluid_compute_node.fluid_uniform_buf,
            ],
            storage_buffers: vec![&fluid_compute_node.info_buf],
            inout_tv: vec![
                (&fluid_compute_node.macro_tex, None),
                (&curl_tex, Some(wgpu::StorageTextureAccess::WriteOnly)),
            ],
            ..Default::default()
        };
        let curl_cal_node = ComputeNode::new(device, &bind_group_data, &curl_shader);

        let render_shader = create_shader_module(device, "lbm/present", Some("lbm present shader"));
        let sampler = crate::util::load_texture::bilinear_sampler(device);

        let render_node = BufferlessFullscreenNode::new(
            device,
            app.config.format,
            &BindGroupData {
                uniforms: vec![
                    &fluid_compute_node.fluid_uniform_buf,
                    setting.particles_uniform.as_ref().unwrap(),
                ],
                storage_buffers: vec![canvas_buf],
                inout_tv: vec![(&fluid_compute_node.macro_tex, None), (&curl_tex, None)],
                samplers: vec![&sampler],
                ..Default::default()
            },
            &render_shader,
            None,
        );

        let update_shader = create_shader_module(
            device,
            "lbm/particle_update",
            Some("particle_update_shader"),
        );
        let bind_group_data = BindGroupData {
            workgroup_count: setting.particles_workgroup_count,
            uniforms: vec![
                &fluid_compute_node.lbm_uniform_buf,
                &fluid_compute_node.fluid_uniform_buf,
                setting.particles_uniform.as_ref().unwrap(),
            ],
            storage_buffers: vec![setting.particles_buf.as_ref().unwrap(), canvas_buf],
            inout_tv: vec![(&fluid_compute_node.macro_tex, None)],
            ..Default::default()
        };
        let particle_update_node = ComputeNode::new(device, &bind_group_data, &update_shader);

        let particle_shader = create_shader_module(device, "present", None);
        let particle_render = BufferlessFullscreenNode::new(
            device,
            app.config.format,
            &BindGroupData {
                uniforms: vec![
                    &fluid_compute_node.fluid_uniform_buf,
                    setting.particles_uniform.as_ref().unwrap(),
                ],
                storage_buffers: vec![canvas_buf],
                ..Default::default()
            },
            &particle_shader,
            None,
        );

        FluidSimulator {
            lattice,
            lattice_pixel_size: fluid_compute_node.lattice_pixel_size,
            pre_pos: glam::Vec2::ZERO,
            fluid_compute_node,
            _curl_cal_node: curl_cal_node,
            particle_update_node,
            _render_node: render_node,
            particle_render,
        }
    }
}

impl Simulator for FluidSimulator {
    fn on_click(&mut self, app: &app_surface::AppSurface, pos: glam::Vec2) {
        if pos.x <= 0.0 || pos.y <= 0.0 {
            return;
        }
        let x = pos.x as u32 / self.lattice_pixel_size;
        let y = pos.y as u32 / self.lattice_pixel_size;
        let half_size = OBSTACLE_RADIUS as u32;
        if x < half_size
            || x >= self.lattice.width - (half_size + 2)
            || y < half_size
            || y >= self.lattice.height - (half_size + 2)
        {
            return;
        }
        self.fluid_compute_node.add_obstacle(&app.queue, x, y);
    }

    fn touch_begin(&mut self, _app: &app_surface::AppSurface) {
        self.pre_pos = glam::Vec2::ZERO;
    }

    fn touch_move(&mut self, app: &app_surface::AppSurface, pos: glam::Vec2) {
        if pos.x <= 0.0 || pos.y <= 0.0 {
            self.pre_pos = glam::Vec2::ZERO;
            return;
        }
        let dis = pos.distance(self.pre_pos);
        if (self.pre_pos.x == 0.0 && self.pre_pos.y == 0.0) || dis > 300.0 {
            self.pre_pos = pos;
            return;
        }

        self.fluid_compute_node
            .add_external_force(&app.queue, pos, self.pre_pos);

        self.pre_pos = pos;
    }

    fn update_uniforms(&mut self, app: &app_surface::AppSurface, setting: &crate::SettingObj) {
        // 通过外部参数来重置流体粒子碰撞松解时间 tau = (3.0 * x + 0.5), x：[0~1] 趋大，松解时间趋快
        let tau = 3.0 * setting.fluid_viscosity + 0.5;
        let fluid_ty = if setting.animation_type == FieldAnimationType::LidDrivenCavity {
            1
        } else {
            0
        };
        let uniform_data = LbmUniform::new(
            tau,
            fluid_ty,
            (self.lattice.width * self.lattice.height) as i32,
        );
        app.queue.write_buffer(
            &self.fluid_compute_node.lbm_uniform_buf.buffer,
            0,
            bytemuck::bytes_of(&uniform_data),
        );
    }

    fn update_by(
        &mut self,
        _app: &app_surface::AppSurface,
        _control_panel: &mut crate::ControlPanel,
    ) {
    }

    fn update_workgroup_count(
        &mut self,
        _app: &app_surface::AppSurface,
        workgroup_count: (u32, u32, u32),
    ) {
        self.particle_update_node.workgroup_count = workgroup_count;
    }

    fn reset(&mut self, app: &app_surface::AppSurface) {
        self.fluid_compute_node
            .reset_lattice_info(&app.device, &app.queue);

        self.pre_pos = glam::Vec2::ZERO;
    }

    fn compute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("fluid solver"),
            ..Default::default()
        });

        for _ in 0..1 {
            self.fluid_compute_node.compute_by_pass(&mut cpass, 0);
            self.particle_update_node.compute_by_pass(&mut cpass);
            // self.curl_cal_node.dispatch(&mut cpass);

            self.fluid_compute_node.compute_by_pass(&mut cpass, 1);
            self.particle_update_node.compute_by_pass(&mut cpass);
            // self.curl_cal_node.dispatch(&mut cpass);
        }
    }

    fn draw_by_rpass<'b, 'a: 'b>(
        &'a mut self,
        _app: &app_surface::AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
        _setting: &mut crate::SettingObj,
    ) {
        // setting.particles_uniform_data.is_only_update_pos = 0;
        // setting.update_particles_uniform(app);

        // draw macro_tex
        // self.render_node.draw_rpass(rpass);

        // draw paticles
        self.particle_render.draw_by_pass(rpass);
    }
}
