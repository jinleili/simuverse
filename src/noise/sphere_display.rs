use crate::{
    create_shader_module,
    geometries::Sphere,
    node::{ViewNode, ViewNodeBuilder},
    util::{
        load_texture::{self},
        BufferObj,
    },
};
use app_surface::AppSurface;
use wgpu::ShaderStages;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TexDimention {
    xy: [f32; 2],
}
pub struct SphereDisplay {
    gen_tex_node: ViewNode,
    p_matrix: glam::Mat4,
    mv_matrix: glam::Mat4,
    mvp_uniform: crate::MVPMatUniform,
    mvp_buf: BufferObj,
}

impl SphereDisplay {
    pub fn new(
        app: &AppSurface,
        uniform_buf: &BufferObj,
        permulation_buf: &BufferObj,
        gradient_buf: &BufferObj,
    ) -> Self {
        let (p_matrix, mut mv_matrix, _factor) =
            crate::util::matrix_helper::perspective_mvp((&app.config).into());
        let transelate = glam::Mat4::from_translation(glam::Vec3::new(0., 0., -1.));
        mv_matrix *= transelate;

        let height = 948;
        let width = (height as f32 * std::f32::consts::PI).ceil() as u32;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let texture = load_texture::empty(
            &app.device,
            format,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            None,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            None,
        );

        let normal: [[f32; 4]; 4] = mv_matrix.inverse().transpose().to_cols_array_2d();
        let mvp_uniform = crate::MVPMatUniform {
            mv: mv_matrix.to_cols_array_2d(),
            proj: p_matrix.to_cols_array_2d(),
            mvp: (p_matrix * mv_matrix).to_cols_array_2d(),
            mv_no_rotation: mv_matrix.to_cols_array_2d(),
            normal,
        };
        let mvp_buf = BufferObj::create_uniform_buffer(&app.device, &mvp_uniform, Some("mvp_buf"));

        let sphere_tex_shader = create_shader_module(&app.device, "noise/sphere_tex", None);

        let (vertices, indices) = Sphere::new(1.0, 50, 34).generate_vertices();

        // generate sphere textue
        let builder = ViewNodeBuilder::<crate::util::vertex::PosNormalUv>::new(
            vec![(&texture, Some(wgpu::StorageTextureAccess::WriteOnly))],
            &sphere_tex_shader,
        )
        .with_uniform_buffers(vec![&mvp_buf, uniform_buf])
        .with_vertices_and_indices((vertices, indices))
        .with_storage_buffers(vec![permulation_buf, gradient_buf])
        .with_shader_stages(vec![
            ShaderStages::VERTEX,
            ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ShaderStages::FRAGMENT,
            ShaderStages::FRAGMENT,
            ShaderStages::FRAGMENT,
            ShaderStages::FRAGMENT,
        ])
        .with_color_format(app.config.format);
        let gen_tex_node = builder.build(&app.device);

        Self {
            gen_tex_node,
            p_matrix,
            mv_matrix,
            mvp_uniform,
            mvp_buf,
        }
    }

    pub fn gen_texture(&self, _app: &AppSurface) {
        // let mut encoder = app
        //     .device
        //     .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        //         label: Some("gen_texture Encoder"),
        //     });
        // self.gen_tex_node.draw(
        //     &self.canvas.tex_view,
        //     &mut encoder,
        //     wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        // );
        // app.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn draw_by_pass<'a, 'b: 'a>(
        &'b mut self,
        app: &AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
    ) {
        self.mv_matrix *= glam::Mat4::from_rotation_y(0.005);
        self.mvp_uniform.mv = self.mv_matrix.to_cols_array_2d();
        self.mvp_uniform.mvp = (self.p_matrix * self.mv_matrix).to_cols_array_2d();
        self.mvp_uniform.normal = self.mv_matrix.inverse().transpose().to_cols_array_2d();
        app.queue.write_buffer(
            &self.mvp_buf.buffer,
            0,
            bytemuck::bytes_of(&self.mvp_uniform),
        );

        self.gen_tex_node.draw_by_pass(rpass);
    }
}
