use crate::node::{BindGroupData, ComputeNode, ViewNode, ViewNodeBuilder};
use crate::util::AnyTexture;
use crate::util::{BufferObj, vertex::PosParticleIndex};

use super::{ClothFabric, ClothUniform, MeshColoringObj};

use alloc::{vec, vec::Vec};
use app_surface::AppSurface;

pub struct Cloth {
    mvp_uniform_data: crate::MVPMatUniform,
    mvp_buf: BufferObj,
    cloth_uniform_data: ClothUniform,
    cloth_uniform_buf: BufferObj,

    stretch_mesh_coloring: Vec<MeshColoringObj>,
    bend_mesh_coloring: Vec<MeshColoringObj>,

    // 外力
    external_force_node: ComputeNode,

    // 预测位置并重置约束的 lambda 等参数
    predict_and_reset: ComputeNode,
    stretch_solver: ComputeNode,
    bend_solver: ComputeNode,
    display_node: ViewNode,
    frame_count: usize,
    // 迭代次数
    pbd_iter_count: usize,
    delta_time: f32,
}

impl Cloth {
    pub fn new(app_view: &AppSurface, fabric: ClothFabric, _texture: Option<&AnyTexture>) -> Self {
        let viewport_size =
            glam::Vec2::new(app_view.config.width as f32, app_view.config.height as f32);
        let mvp_uniform_data = Self::get_mvp_uniform_data(viewport_size);
        let mvp_buf = BufferObj::create_uniform_buffer(&app_view.device, &mvp_uniform_data, None);

        //static const float MODE_COMPLIANCE[eModeMax] = {
        //  0.0f,            // Miles Macklin's blog (http://blog.mmacklin.com/2016/10/12/xpbd-slides-and-stiffness/)
        //  0.00000000004f, // 0.04 x 10^(-9) (M^2/N) Concrete
        //  0.00000000016f, // 0.16 x 10^(-9) (M^2/N) Wood
        //  0.000000001f,   // 1.0  x 10^(-8) (M^2/N) Leather
        //  0.000000002f,   // 0.2  x 10^(-7) (M^2/N) Tendon
        //  0.0000001f,     // 1.0  x 10^(-6) (M^2/N) Rubber
        //  0.00002f,       // 0.2  x 10^(-3) (M^2/N) Muscle
        //  0.0001f,        // 1.0  x 10^(-3) (M^2/N) Fat
        //};
        let pbd_iter_count = 15;
        let delta_time = 0.016 / pbd_iter_count as f32;
        let cloth_uniform_data = ClothUniform {
            num_x: fabric.horizontal_num as i32,
            num_y: fabric.vertical_num as i32,
            gravity: -70.0 * 0.7,
            damping: 0.01,
            compliance: 0.0000000016 / (delta_time * delta_time),
            stiffness: 0.05,
            dt: delta_time,
        };
        let cloth_uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &cloth_uniform_data,
            Some("cloth uniform"),
        );
        // dynamit uniform
        let dynamic_offset =
            app_view.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let stretch_coloring_buf = BufferObj::create_empty_uniform_buffer(
            &app_view.device,
            fabric.stretch_constraints.0.len() as u64 * dynamic_offset,
            0,
            true,
            Some("stretch_coloring_buf"),
        );
        let mut offset = 0;
        for mc in fabric.stretch_constraints.0.iter() {
            app_view.queue.write_buffer(
                &stretch_coloring_buf.buffer,
                offset,
                bytemuck::cast_slice(&mc.get_push_constants_data()),
            );
            offset += dynamic_offset;
        }

        let bend_coloring_buf = BufferObj::create_empty_uniform_buffer(
            &app_view.device,
            fabric.bend_constraints.0.len() as u64 * dynamic_offset * pbd_iter_count as u64,
            0,
            true,
            Some("bend_coloring_buf"),
        );
        offset = 0;
        for i in 0..pbd_iter_count {
            for mc in fabric.bend_constraints.0.iter() {
                app_view.queue.write_buffer(
                    &bend_coloring_buf.buffer,
                    offset,
                    bytemuck::bytes_of(&mc.get_bending_dynamic_uniform(i)),
                );
                offset += dynamic_offset;
            }
        }

        let mut particle_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &fabric.particles,
            Some("particle buf"),
        );

        let velocity_buf = BufferObj::create_empty_storage_buffer(
            &app_view.device,
            16,
            false,
            Some("velocity_buf"),
        );

        let external_force_shader = crate::util::shader::create_shader_module(
            &app_view.device,
            "pbd/cloth_external_force",
            None,
        );
        let bind_group_data = BindGroupData {
            workgroup_count: (1, 1, 1),
            uniforms: vec![&cloth_uniform_buf],
            storage_buffers: vec![&velocity_buf, &particle_buf],
            ..Default::default()
        };
        let external_force_node =
            ComputeNode::new(&app_view.device, &bind_group_data, &external_force_shader);

        let constraint_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &fabric.stretch_constraints.1,
            Some("constraint_buf"),
        );

        let bend_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &fabric.bend_constraints.1,
            Some("bend_constraints_buf"),
        );
        let predict_and_reset_shader = crate::util::shader::create_shader_module(
            &app_view.device,
            "pbd/xxpbd/cloth_predict",
            None,
        );
        let mut bind_group_data = BindGroupData {
            workgroup_count: (
                ((fabric.horizontal_num * fabric.vertical_num + 31) as f32 / 32.0).floor() as u32,
                1,
                1,
            ),
            uniforms: vec![&cloth_uniform_buf],
            storage_buffers: vec![&particle_buf, &constraint_buf],
            ..Default::default()
        };

        let predict_and_reset = ComputeNode::new(
            &app_view.device,
            &bind_group_data,
            &predict_and_reset_shader,
        );

        let constraint_solver_shader = crate::util::shader::create_shader_module(
            &app_view.device,
            "pbd/xxpbd/cloth_stretch_solver",
            None,
        );

        bind_group_data.dynamic_uniforms = vec![(&stretch_coloring_buf)];
        bind_group_data.workgroup_count = (0, 0, 0);
        let stretch_solver = ComputeNode::new_with_dynamic_uniforms(
            &app_view.device,
            &bind_group_data,
            &constraint_solver_shader,
        );

        let bend_solver_shader = crate::util::shader::create_shader_module(
            &app_view.device,
            "pbd/xxpbd/cloth_bending_solver",
            None,
        );
        let bind_group_data = BindGroupData {
            uniforms: vec![&cloth_uniform_buf],
            dynamic_uniforms: vec![&bend_coloring_buf],
            storage_buffers: vec![&particle_buf, &bend_constraints_buf],
            ..Default::default()
        };
        let bend_solver = ComputeNode::new_with_dynamic_uniforms(
            &app_view.device,
            &bind_group_data,
            &bend_solver_shader,
        );
        #[cfg(target_arch = "wasm32")]
        let texture = _texture.unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        let (cloth_texture, _) = pollster::block_on(crate::util::load_texture::from_path(
            "cloth_500x500.png",
            app_view,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            false,
        ));
        #[cfg(not(target_arch = "wasm32"))]
        let texture = &cloth_texture;

        let sampler = app_view
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let display_shader =
            crate::util::shader::create_shader_module(&app_view.device, "pbd/cloth_display", None);
        particle_buf.read_only = true;
        let bind_group_data = BindGroupData {
            uniforms: vec![&mvp_buf, &cloth_uniform_buf],
            storage_buffers: vec![&particle_buf],
            inout_tv: vec![(texture, None)],
            samplers: vec![&sampler],
            visibilitys: vec![
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::ShaderStages::FRAGMENT,
            ],
            ..Default::default()
        };
        let display_node_builder =
            ViewNodeBuilder::<PosParticleIndex>::new(bind_group_data, &display_shader)
                .with_use_depth_stencil(true)
                .with_polygon_mode(wgpu::PolygonMode::Fill)
                .with_cull_mode(None)
                .with_vertices_and_indices((fabric.vertices.0, fabric.vertices.1))
                .with_color_format(app_view.config.format);

        let display_node = display_node_builder.build(&app_view.device);

        Self {
            mvp_buf,
            mvp_uniform_data,
            cloth_uniform_buf,
            cloth_uniform_data,

            external_force_node,

            stretch_mesh_coloring: fabric.stretch_constraints.0,
            bend_mesh_coloring: fabric.bend_constraints.0,
            predict_and_reset,
            stretch_solver,
            bend_solver,
            display_node,
            frame_count: 0,
            pbd_iter_count: pbd_iter_count as usize,
            delta_time,
        }
    }

    pub fn update_by(&mut self, app: &AppSurface, control_panel: &mut crate::ControlPanel) {
        let new_damping = control_panel.pbd_setting.damping * 0.015;
        let new_gravity = control_panel.pbd_setting.gravity * -35.0 - 35.0;
        let compliance =
            control_panel.pbd_setting.compliance * 0.00000016 / (self.delta_time * self.delta_time);
        let stiffness = control_panel.pbd_setting.stiffness;
        if (new_damping - self.cloth_uniform_data.damping).abs() > 0.00001
            || (new_gravity - self.cloth_uniform_data.gravity).abs() > 0.00001
            || (compliance - self.cloth_uniform_data.compliance).abs() > 0.00000000001
            || (stiffness - self.cloth_uniform_data.stiffness).abs() > 0.00001
        {
            self.cloth_uniform_data.damping = new_damping;
            self.cloth_uniform_data.gravity = new_gravity;
            self.cloth_uniform_data.compliance = compliance;
            self.cloth_uniform_data.stiffness = stiffness;
            app.queue.write_buffer(
                &self.cloth_uniform_buf.buffer,
                0,
                bytemuck::bytes_of(&self.cloth_uniform_data),
            );
        }
    }

    pub fn resize(&mut self, app: &app_surface::AppSurface) -> bool {
        self.mvp_uniform_data = Self::get_mvp_uniform_data(glam::Vec2::new(
            app.config.width as f32,
            app.config.height as f32,
        ));
        app.queue.write_buffer(
            &self.mvp_buf.buffer,
            0,
            bytemuck::bytes_of(&self.mvp_uniform_data),
        );
        true
    }

    pub fn compute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.step_solver(encoder);
    }

    pub fn draw_by_rpass<'b, 'a: 'b>(
        &'a mut self,
        _app: &app_surface::AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
        _setting: &mut crate::SettingObj,
    ) {
        self.display_node.draw_by_pass(rpass);
    }

    fn step_solver(&mut self, encoder: &mut wgpu::CommandEncoder) {
        // 重用 cpass 在 macOS 上不能提升性能， 但是在 iOS 上提升明显
        // 64*64，8 约束，迭代20 ：Xs Max, 12ms -> 8ms
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("solver pass"),
            ..Default::default()
        });

        let dynamic_offset = 256;
        for i in 0..self.pbd_iter_count {
            // 下一次迭代的开始，先更新粒子速度
            self.predict_and_reset.compute_by_pass(&mut cpass);

            cpass.set_pipeline(&self.stretch_solver.pipeline);
            cpass.set_bind_group(0, &self.stretch_solver.bg_setting.bind_group, &[]);
            let mut index = 0;
            for mc in self.stretch_mesh_coloring.iter() {
                if let Some(bg) = &self.stretch_solver.dy_uniform_bg {
                    cpass.set_bind_group(1, &bg.bind_group, &[index * dynamic_offset]);
                }
                cpass.dispatch_workgroups(mc.thread_group.0, mc.thread_group.1, 1);
                index += 1;
            }

            let bending_dynamic_uniform_offset =
                (i * self.bend_mesh_coloring.len() * dynamic_offset as usize)
                    as wgpu::DynamicOffset;
            cpass.set_pipeline(&self.bend_solver.pipeline);
            cpass.set_bind_group(0, &self.bend_solver.bg_setting.bind_group, &[]);
            index = 0;
            for mc in self.bend_mesh_coloring.iter() {
                if let Some(bg) = &self.bend_solver.dy_uniform_bg {
                    cpass.set_bind_group(
                        1,
                        &bg.bind_group,
                        &[bending_dynamic_uniform_offset + index * dynamic_offset],
                    );
                }
                cpass.dispatch_workgroups(mc.thread_group.0, mc.thread_group.1, 1);
                index += 1;
            }
        }

        if self.frame_count > 10 {
            self.external_force_node.compute_by_pass(&mut cpass);
        }

        self.frame_count += 1;
    }

    fn get_mvp_uniform_data(viewport: glam::Vec2) -> crate::MVPMatUniform {
        let (proj_mat, mut mv_mat, _factor) = crate::util::matrix_helper::perspective_mvp(viewport);
        mv_mat *= glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, -0.4));
        crate::MVPMatUniform {
            mv: mv_mat.to_cols_array_2d(),
            mv_no_rotation: mv_mat.to_cols_array_2d(),
            proj: proj_mat.to_cols_array_2d(),
            mvp: (proj_mat * mv_mat).to_cols_array_2d(),
            normal: mv_mat.inverse().transpose().to_cols_array_2d(),
            u_time: 0.0,
            _padding: [0.0; 3],
        }
    }
}
