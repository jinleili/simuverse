use crate::util::{AnyTexture, BufferObj};
use wgpu::{PrimitiveTopology, ShaderModule, TextureFormat};

pub struct BufferlessFullscreenNode {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

#[allow(dead_code)]
impl BufferlessFullscreenNode {
    pub fn new(
        device: &wgpu::Device,
        format: TextureFormat,
        uniforms: Vec<&BufferObj>,
        storage_buffers: Vec<&BufferObj>,
        textures: Vec<&crate::util::AnyTexture>,
        samplers: Vec<&wgpu::Sampler>,
        shader_module: &ShaderModule,
        color_blend_state: Option<wgpu::BlendState>,
        use_depth_stencil: bool,
    ) -> Self {
        let pipeline_vertex_buffers = [];
        let blend_state = if color_blend_state.is_some() {
            color_blend_state
        } else {
            Some(wgpu::BlendState::ALPHA_BLENDING)
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bufferless fullscreen pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: &pipeline_vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            // the bufferless vertices are in clock-wise order
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: if use_depth_stencil {
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            },
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let bind_group = create_bind_group(
            device,
            uniforms,
            storage_buffers,
            textures,
            samplers,
            &pipeline.get_bind_group_layout(0),
        );

        Self {
            bind_group,
            pipeline,
        }
    }

    pub fn draw(
        &self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("bufferless rpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        self.draw_by_pass(&mut rpass);
    }

    pub fn draw_by_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}

pub fn create_bind_group(
    device: &wgpu::Device,
    uniforms: Vec<&BufferObj>,
    storage_buffers: Vec<&BufferObj>,
    textures: Vec<&AnyTexture>,
    samplers: Vec<&wgpu::Sampler>,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let mut entries: Vec<wgpu::BindGroupEntry> = vec![];
    let mut b_index = 0_u32;
    for i in 0..uniforms.len() {
        let buffer_obj = uniforms[i];
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: buffer_obj.buffer.as_entire_binding(),
        });
        b_index += 1;
    }

    for i in 0..storage_buffers.len() {
        let buffer_obj = storage_buffers[i];
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: buffer_obj.buffer.as_entire_binding(),
        });
        b_index += 1;
    }

    for i in 0..textures.len() {
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: wgpu::BindingResource::TextureView(&textures[i].tex_view),
        });
        b_index += 1;
    }

    for i in 0..samplers.len() {
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: wgpu::BindingResource::Sampler(samplers[i]),
        });
        b_index += 1;
    }

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &entries,
        label: None,
    })
}
