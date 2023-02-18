use super::BindingGroupSetting;
use crate::util::vertex::Vertex;
use crate::util::Plane;
use crate::util::{AnyTexture, BufferObj, MVPUniform};
use app_surface::math::{Position, Rect, Size};
use bytemuck::Pod;
use std::ops::{Deref, DerefMut};
use wgpu::util::DeviceExt;
use wgpu::StorageTextureAccess;

#[allow(dead_code)]
pub struct NodeAttributes<'a, T: Vertex + Pod> {
    pub view_size: Size<f32>,
    pub vertices_and_indices: Option<(Vec<T>, Vec<u32>)>,
    pub vertex_buffer_layouts: Option<Vec<wgpu::VertexBufferLayout<'a>>>,
    pub uniform_buffers: Vec<&'a BufferObj>,
    pub storage_buffers: Vec<&'a BufferObj>,
    pub tex_views: Vec<(&'a AnyTexture, Option<StorageTextureAccess>)>,
    pub samplers: Vec<&'a wgpu::Sampler>,
    // 动态 uniform
    pub dynamic_uniforms: Vec<(&'a BufferObj, wgpu::ShaderStages)>,

    pub tex_rect: Option<Rect>,
    pub corlor_format: Option<wgpu::TextureFormat>,
    pub color_blend_state: Option<wgpu::BlendState>,
    pub primitive_topology: wgpu::PrimitiveTopology,
    pub cull_mode: Option<wgpu::Face>,
    pub use_depth_stencil: bool,
    pub shader_module: &'a wgpu::ShaderModule,
    pub shader_stages: Vec<wgpu::ShaderStages>,
}

pub struct ViewNodeBuilder<'a, T: Vertex + Pod> {
    pub attributes: NodeAttributes<'a, T>,
}

impl<'a, T: Vertex + Pod> Deref for ViewNodeBuilder<'a, T> {
    type Target = NodeAttributes<'a, T>;
    fn deref(&self) -> &NodeAttributes<'a, T> {
        &self.attributes
    }
}

impl<'a, T: Vertex + Pod> DerefMut for ViewNodeBuilder<'a, T> {
    fn deref_mut(&mut self) -> &mut NodeAttributes<'a, T> {
        &mut self.attributes
    }
}

#[allow(dead_code)]
impl<'a, T: Vertex + Pod> ViewNodeBuilder<'a, T> {
    pub fn new(
        tex_views: Vec<(&'a AnyTexture, Option<StorageTextureAccess>)>,
        shader_module: &'a wgpu::ShaderModule,
    ) -> Self {
        ViewNodeBuilder {
            attributes: NodeAttributes {
                view_size: (1.0, 1.0).into(),
                vertices_and_indices: None,
                vertex_buffer_layouts: None,
                uniform_buffers: vec![],
                storage_buffers: vec![],
                tex_views,
                samplers: vec![],
                dynamic_uniforms: vec![],
                tex_rect: None,
                corlor_format: None,
                color_blend_state: Some(wgpu::BlendState::ALPHA_BLENDING),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                use_depth_stencil: false,
                shader_module,
                shader_stages: vec![],
            },
        }
    }

    pub fn with_primitive_topology(mut self, primitive_topology: wgpu::PrimitiveTopology) -> Self {
        self.primitive_topology = primitive_topology;
        self
    }

    pub fn with_cull_mode(mut self, cull_mode: Option<wgpu::Face>) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    pub fn with_vertices_and_indices(mut self, vertices_and_indices: (Vec<T>, Vec<u32>)) -> Self {
        self.vertices_and_indices = Some(vertices_and_indices);
        self
    }

    pub fn with_vertex_buffer_layouts(
        mut self,
        layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    ) -> Self {
        self.vertex_buffer_layouts = Some(layouts);
        self
    }

    pub fn with_view_size(mut self, size: Size<f32>) -> Self {
        self.view_size = size;
        self
    }

    pub fn with_uniform_buffers(mut self, buffers: Vec<&'a BufferObj>) -> Self {
        self.uniform_buffers = buffers;
        self
    }

    pub fn with_dynamic_uniforms(
        mut self,
        uniforms: Vec<(&'a BufferObj, wgpu::ShaderStages)>,
    ) -> Self {
        self.dynamic_uniforms = uniforms;
        self
    }

    pub fn with_storage_buffers(mut self, buffers: Vec<&'a BufferObj>) -> Self {
        self.storage_buffers = buffers;
        self
    }

    pub fn with_tex_views(
        mut self,
        views: Vec<(&'a AnyTexture, Option<StorageTextureAccess>)>,
    ) -> Self {
        self.tex_views = views;
        self
    }

    pub fn with_samplers(mut self, samplers: Vec<&'a wgpu::Sampler>) -> Self {
        self.samplers = samplers;
        self
    }

    pub fn with_tex_rect(mut self, rect: Rect) -> Self {
        self.tex_rect = Some(rect);
        self
    }

    pub fn with_color_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.corlor_format = Some(format);
        self
    }

    pub fn with_color_blend_state(mut self, blend_state: Option<wgpu::BlendState>) -> Self {
        self.color_blend_state = blend_state;
        self
    }

    pub fn with_use_depth_stencil(mut self, bl: bool) -> Self {
        self.use_depth_stencil = bl;
        self
    }

    pub fn with_shader_stages(mut self, states: Vec<wgpu::ShaderStages>) -> Self {
        self.shader_stages = states;
        self
    }

    pub fn build(self, device: &wgpu::Device) -> ViewNode {
        debug_assert!(
            self.shader_stages.len()
                >= self.uniform_buffers.len()
                    + self.samplers.len()
                    + self.storage_buffers.len()
                    + self.tex_views.len(),
            "shader_stages count less than binding resource count"
        );
        ViewNode::frome_attributes::<T>(self.attributes, device)
    }
}

#[allow(dead_code)]
pub struct ViewNode {
    pub vertex_buf: Option<BufferObj>,
    pub index_buf: wgpu::Buffer,
    pub index_count: usize,
    pub bg_setting: BindingGroupSetting,
    pub dy_uniform_bg: Option<super::DynamicUniformBindingGroup>,
    pub pipeline: wgpu::RenderPipeline,
    view_width: f32,
    view_height: f32,
    pub clear_color: wgpu::Color,
}

#[allow(dead_code)]
impl ViewNode {
    fn frome_attributes<T: Vertex + Pod>(
        attributes: NodeAttributes<T>,
        device: &wgpu::Device,
    ) -> Self {
        let corlor_format = if let Some(format) = attributes.corlor_format {
            format
        } else {
            wgpu::TextureFormat::Bgra8Unorm
        };

        let stages: Vec<wgpu::ShaderStages> = if attributes.shader_stages.len() > 0 {
            attributes.shader_stages
        } else {
            let mut stages: Vec<wgpu::ShaderStages> = vec![wgpu::ShaderStages::VERTEX];
            let uniform_buffers_len = if attributes.uniform_buffers.len() > 0 {
                attributes.uniform_buffers.len()
            } else {
                1
            };
            for _ in 0..(uniform_buffers_len
                + attributes.storage_buffers.len()
                + attributes.tex_views.len()
                + attributes.samplers.len())
            {
                stages.push(wgpu::ShaderStages::FRAGMENT);
            }
            stages
        };

        let sampler = crate::util::load_texture::default_sampler(device);
        let new_samplers: Vec<&wgpu::Sampler> = if attributes.tex_views.len() > 0 {
            if attributes.samplers.len() > 0 {
                attributes.samplers
            } else {
                vec![&sampler]
            }
        } else {
            vec![]
        };
        // 如果没有设置 mvp, 且设置了 view_size, 则设置一个全屏的 mvp
        let (p_matrix, vm_matrix, _factor) =
            crate::util::matrix_helper::perspective_mvp(attributes.view_size);
        let mvp = MVPUniform {
            mvp_matrix: (p_matrix * vm_matrix).to_cols_array_2d(),
        };
        let mvp_buf = BufferObj::create_uniform_buffer(device, &mvp, Some("mvp uniform"));
        let uniform_buffers =
            if attributes.uniform_buffers.len() == 0 && attributes.view_size.width > 0.0 {
                vec![&mvp_buf]
            } else {
                attributes.uniform_buffers
            };
        let bg_setting = BindingGroupSetting::new(
            device,
            uniform_buffers,
            attributes.storage_buffers,
            attributes.tex_views,
            new_samplers,
            stages,
        );

        // Create the vertex and index buffers
        let (vertex_buf, index_data) = if let Some(vi) = attributes.vertices_and_indices {
            let vertex_buf = if std::mem::size_of_val(&vi.0[0]) > 0 {
                Some(BufferObj::create_buffer(
                    device,
                    Some(&vi.0),
                    None,
                    wgpu::BufferUsages::VERTEX,
                    Some("vertex_buf"),
                ))
            } else {
                None
            };
            (vertex_buf, vi.1)
        } else {
            let factor = crate::util::matrix_helper::fullscreen_factor(attributes.view_size);
            let rect = Rect::new(2.0 * factor.1, 2.0 * factor.2, Position::zero());
            let plane = Plane::new_by_rect(rect, 1, 1);
            if let Some(rect) = attributes.tex_rect {
                let (vertex_data, index_data) = plane.generate_vertices_by_texcoord2(rect, None);
                let vertex_buf = BufferObj::create_buffer(
                    device,
                    Some(&vertex_data),
                    None,
                    wgpu::BufferUsages::VERTEX,
                    Some("vertex_buf"),
                );

                (Some(vertex_buf), index_data)
            } else {
                let (vertex_data, index_data) = plane.generate_vertices();
                let vertex_buf = BufferObj::create_buffer(
                    device,
                    Some(&vertex_data),
                    None,
                    wgpu::BufferUsages::VERTEX,
                    Some("vertex_buf"),
                );
                (Some(vertex_buf), index_data)
            }
        };
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let default_layout_attributes = T::vertex_attributes(0);
        let vertex_buffer_layouts = if let Some(layouts) = attributes.vertex_buffer_layouts {
            layouts
        } else {
            if std::mem::size_of::<T>() > 0 {
                vec![wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<T>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &default_layout_attributes,
                }]
            } else {
                vec![]
            }
        };
        let (dy_uniform_bg, pipeline_layout) = if attributes.dynamic_uniforms.len() > 0 {
            let dy_bg = super::DynamicUniformBindingGroup::new(device, attributes.dynamic_uniforms);
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bg_setting.bind_group_layout, &dy_bg.bind_group_layout],
                push_constant_ranges: &[],
            });
            (Some(dy_bg), pipeline_layout)
        } else {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bg_setting.bind_group_layout],
                push_constant_ranges: &[],
            });
            (None, pipeline_layout)
        };

        // Create the render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("image_view pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &attributes.shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &attributes.shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: corlor_format,
                    blend: attributes.color_blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: attributes.primitive_topology,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: attributes.cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            // ??????
            depth_stencil: if attributes.use_depth_stencil {
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

        ViewNode {
            view_width: attributes.view_size.width,
            view_height: attributes.view_size.height,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bg_setting,
            dy_uniform_bg,
            pipeline,
            clear_color: wgpu::Color::BLACK,
        }
    }

    // 视口的宽高发生变化
    pub fn resize(&mut self, queue: &wgpu::Queue, tex_rect: Option<Rect>) {
        if let Some(buf) = &self.vertex_buf {
            let vertex_data = if let Some(rect) = tex_rect {
                let (vertex_data, _) = Plane::new(1, 1).generate_vertices_by_texcoord(rect);
                vertex_data
            } else {
                let (vertex_data, _) = Plane::new(1, 1).generate_vertices();
                vertex_data
            };
            queue.write_buffer(&buf.buffer, 0, bytemuck::cast_slice(&vertex_data))
        }
    }

    pub fn draw(
        &self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) {
        self.draw_by_offset(frame_view, encoder, load_op, 0);
    }

    pub fn draw_by_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        self.draw_rpass_by_offset(rpass, 0, 1);
    }

    pub fn draw_by_instance_count<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'b>,
        instance_count: u32,
    ) {
        self.draw_rpass_by_offset(rpass, 0, instance_count);
    }

    pub fn draw_by_offset(
        &self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        load_op: wgpu::LoadOp<wgpu::Color>,
        offset_index: u32,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
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
        self.set_rpass(&mut rpass);
        self.draw_rpass_by_offset(&mut rpass, offset_index, 1);
    }

    pub fn draw_rpass_by_offset<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'b>,
        offset_index: u32,
        instance_count: u32,
    ) {
        self.set_rpass(rpass);
        if let Some(node) = &self.dy_uniform_bg {
            rpass.set_bind_group(
                1,
                &node.bind_group,
                &[256 * offset_index as wgpu::DynamicOffset],
            );
        }
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..instance_count);
    }

    fn set_rpass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bg_setting.bind_group, &[]);
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        if self.vertex_buf.is_some() {
            rpass.set_vertex_buffer(0, self.vertex_buf.as_ref().unwrap().buffer.slice(..));
        }
    }
}
