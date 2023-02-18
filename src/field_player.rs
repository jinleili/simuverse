use crate::util::math::Size;
use crate::util::node::{BufferlessFullscreenNode, ComputeNode};
use crate::util::BufferObj;
use crate::{SettingObj, FieldUniform, Player};
use app_surface::AppSurface;
use wgpu::{CommandEncoderDescriptor};

use crate::{create_shader_module, insert_code_then_create};

pub struct FieldPlayer {
    canvas_size: Size<u32>,
    field_uniform_data: FieldUniform,
    field_uniform: BufferObj,
    field_buf: BufferObj,
    field_workgroup_count: (u32, u32, u32),
    code_snippet: String,
    trajectory_update_shader: wgpu::ShaderModule,
    field_setting_node: ComputeNode,
    particles_update_node: ComputeNode,
    render_node: BufferlessFullscreenNode,
    frame_num: usize,
}

impl FieldPlayer {
    pub fn new(
        app: &app_surface::AppSurface,
        canvas_format: wgpu::TextureFormat,
        canvas_size: Size<u32>,
        canvas_buf: &BufferObj,
        setting: &SettingObj,
    ) -> Self {
        let pixel_distance = 4;
        let field_size: crate::util::math::Size<u32> = (
            canvas_size.width / pixel_distance,
            canvas_size.height / pixel_distance,
        )
            .into();

        let field_workgroup_count = (
            (field_size.width + 15) / 16,
            (field_size.height + 15) / 16,
            1,
        );
        let (_, sx, sy) = crate::util::utils::matrix_helper::fullscreen_factor(
            (canvas_size.width as f32, canvas_size.height as f32).into(),
        );

        let field_uniform_data = FieldUniform {
            lattice_size: [field_size.width as i32, field_size.height as i32],
            lattice_pixel_size: [pixel_distance as f32; 2],
            canvas_size: [canvas_size.width as i32, canvas_size.height as i32],
            proj_ratio: [sx, sy],
            ndc_pixel: [
                sx * 2.0 / canvas_size.width as f32,
                sy * 2.0 / canvas_size.height as f32,
            ],
            speed_ty: 0,
            _padding: 0.0,
        };
        let field_uniform = BufferObj::create_uniform_buffer(
            &app.device,
            &field_uniform_data,
            Some("field_uniform"),
        );
        let field_buf = BufferObj::create_empty_storage_buffer(
            &app.device,
            (field_size.width * field_size.height * 16) as u64,
            false,
            Some("field buf"),
        );

        let code_snippet = crate::get_velocity_code_snippet(setting.animation_type);
        let setting_shader =
            insert_code_then_create(&app.device, "field_setting", Some(&code_snippet), None);

        let field_setting_node = ComputeNode::new(
            &app.device,
            field_workgroup_count,
            vec![&field_uniform],
            vec![&field_buf],
            vec![],
            &setting_shader,
        );

        let trajectory_update_shader = create_shader_module(&app.device, "trajectory_update", None);
        let particles_update_node = ComputeNode::new(
            &app.device,
            setting.particles_workgroup_count,
            vec![&field_uniform, &setting.particles_uniform.as_ref().unwrap()],
            vec![
                &field_buf,
                &setting.particles_buf.as_ref().unwrap(),
                canvas_buf,
            ],
            vec![],
            &trajectory_update_shader,
        );

        let render_shader = create_shader_module(&app.device, "present", None);
        let render_node = BufferlessFullscreenNode::new(
            &app.device,
            canvas_format,
            vec![&field_uniform, &setting.particles_uniform.as_ref().unwrap()],
            vec![canvas_buf],
            vec![],
            vec![],
            &render_shader,
            None,
            false,
        );
        let mut instance = FieldPlayer {
            canvas_size,
            field_uniform_data,
            field_uniform,
            field_buf,
            field_workgroup_count,
            code_snippet,
            trajectory_update_shader,
            field_setting_node,
            particles_update_node,
            render_node,
            frame_num: 0,
        };

        instance.reset(app);
        instance
    }

    pub fn update_field_by_cpass<'c, 'b: 'c>(&'b self, cpass: &mut wgpu::ComputePass<'c>) {
        self.field_setting_node.dispatch(cpass);
    }
}

impl Player for FieldPlayer {
    fn reset(&mut self, app: &app_surface::AppSurface) {
        let mut encoder = app
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("update_field encoder"),
            });
        self.field_setting_node.compute(&mut encoder);
        app.queue.submit(Some(encoder.finish()));
    }

    fn update_by(&mut self, app: &AppSurface, control_panel: &mut crate::ControlPanel) {
        if !control_panel.is_code_snippet_changed() {
            return;
        }

        let setting_shader = insert_code_then_create(
            &app.device,
            "field_setting",
            Some(&control_panel.wgsl_code),
            None,
        );

        self.field_setting_node = ComputeNode::new(
            &app.device,
            self.field_workgroup_count,
            vec![&self.field_uniform],
            vec![&self.field_buf],
            vec![],
            &setting_shader,
        );
        self.reset(app);
    }

    fn update_workgroup_count(
        &mut self,
        _app: &app_surface::AppSurface,
        workgroup_count: (u32, u32, u32),
    ) {
        self.particles_update_node.group_count = workgroup_count;
    }

    fn compute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.particles_update_node.compute(encoder);
    }

    fn draw_by_rpass<'b, 'a: 'b>(
        &'a mut self,
        _app: &app_surface::AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
        _setting: &mut crate::SettingObj,
    ) {
        self.render_node.draw_rpass(rpass);
        self.frame_num += 1;
    }
}
