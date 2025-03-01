use crate::util::BufferObj;
use alloc::{vec, vec::Vec};
use wgpu::util::DeviceExt;

use crate::{TrajectoryUniform, create_shader_module};

pub struct ParticleRenderNode {
    trajectory_views: Vec<wgpu::TextureView>,
    bind_group: wgpu::BindGroup,
    fade_out_pipeline: wgpu::RenderPipeline,
    update_pipeline: wgpu::RenderPipeline,
    compose_pipeline: wgpu::RenderPipeline,
    vertices_buf: wgpu::Buffer,
}

#[allow(dead_code)]
impl ParticleRenderNode {
    pub fn new(app: &app_surface::AppSurface, point_size: f32, canvas_size: glam::UVec2) -> Self {
        let device = &app.device;
        let sampler = crate::util::load_texture::bilinear_sampler(device);
        // Render pipeline is incompatible with render pass
        // Incompatible color attachment: [Rgba8Unorm] != [Bgra8Unorm]
        let format = app.config.format;
        let trajectory_tex = crate::util::load_texture::empty(
            device,
            format,
            wgpu::Extent3d {
                width: canvas_size.x,
                height: canvas_size.y,
                depth_or_array_layers: 2,
            },
            Some(wgpu::TextureViewDimension::D2Array),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some("trajectory_tex"),
        );

        let mut trajectory_views: Vec<wgpu::TextureView> = vec![];
        for i in 0..2 {
            trajectory_views.push(
                trajectory_tex
                    .tex
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some("trajectory"),
                        format: Some(format),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: i,
                        array_layer_count: Some(1),
                        ..Default::default()
                    }),
            );
        }
        let uniform_data = TrajectoryUniform {
            screen_factor: [2.0 / canvas_size.x as f32, 2.0 / canvas_size.y as f32],
            trajectory_view_index: 0,
            bg_view_index: 1,
        };
        let uniform_buf = BufferObj::create_uniform_buffer(
            device,
            &uniform_data,
            Some("particle render uniform_buf"),
        );
        let layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(0),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
        let entries: Vec<wgpu::BindGroupEntry> = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&trajectory_tex.tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ];

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layouts,
            label: None,
        });

        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: None,
        });

        let half_size = point_size / 2.0;
        let vertex_buffer_data = [
            half_size, half_size, half_size, -half_size, -half_size, half_size, -half_size,
            -half_size,
        ];
        let vertices_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let trajectory_shader = create_shader_module(
            device,
            "lbm/trajectory_present",
            Some("trajectory_present shader"),
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let fade_out_pipeline = generate_pipeline(
            device,
            vec![Some(format.into())],
            &render_pipeline_layout,
            &trajectory_shader,
            ("main", "fs_fadeout"),
            true,
        );
        let update_pipeline = generate_pipeline(
            device,
            vec![Some(format.into())],
            &render_pipeline_layout,
            &trajectory_shader,
            ("vs_update", "fs_update"),
            false,
        );
        let compose_pipeline = generate_pipeline(
            device,
            vec![Some(format.into())],
            &render_pipeline_layout,
            &trajectory_shader,
            ("main", "fs_compose"),
            true,
        );

        ParticleRenderNode {
            trajectory_views,
            bind_group,
            fade_out_pipeline,
            update_pipeline,
            compose_pipeline,
            vertices_buf,
        }
    }

    pub fn update_trajectory<'a, 'b: 'a>(
        &'b self,
        encoder: &mut wgpu::CommandEncoder,
        particles_buf: &'b BufferObj,
        particle_count: i32,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("update trajectory"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.trajectory_views[1],
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        // fade out previous frame trajectory
        rpass.set_pipeline(&self.fade_out_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..3, 0..1);

        // update trajectory
        rpass.set_pipeline(&self.update_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, particles_buf.buffer.slice(..));
        rpass.set_vertex_buffer(1, self.vertices_buf.slice(..));
        rpass.draw(0..4, 0..particle_count as u32);
    }

    pub fn draw_rpass<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'b>,
        _particles_buf: &'b BufferObj,
        _particle_count: i32,
    ) {
        // compose
        rpass.set_pipeline(&self.compose_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}

fn generate_pipeline(
    device: &wgpu::Device,
    targets: Vec<Option<wgpu::ColorTargetState>>,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    entry_points: (&'static str, &'static str),
    is_bufferless: bool,
) -> wgpu::RenderPipeline {
    let vertex_attr_array =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32, 3 => Float32];
    let buffers: Vec<wgpu::VertexBufferLayout> = if is_bufferless {
        vec![]
    } else {
        vec![
            wgpu::VertexBufferLayout {
                array_stride: 4 * 6,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &vertex_attr_array,
            },
            wgpu::VertexBufferLayout {
                array_stride: 2 * 4,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![4 => Float32x2],
            },
        ]
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(entry_points.0),
            buffers: &buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(entry_points.1),
            targets: &targets,
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
