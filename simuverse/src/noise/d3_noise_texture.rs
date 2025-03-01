use crate::{
    create_shader_module,
    node::{BindGroupData, ComputeNode},
    noise::{create_gradient_buf, create_permulation_buf},
};
use alloc::vec;

pub struct D3NoiseTexture {
    pub tex: crate::util::AnyTexture,
}

impl D3NoiseTexture {
    pub fn create(app: &app_surface::AppSurface) -> Self {
        let tex = crate::util::load_texture::empty(
            &app.device,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 64,
            },
            Some(wgpu::TextureViewDimension::D3),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            None,
        );

        let dispatch_group_count = (8, 8, 8);

        let permulation_buf = create_permulation_buf(&app.device);
        let gradient_buf = create_gradient_buf(&app.device);
        let shader = create_shader_module(&app.device, "noise/3d_noise_tex", None);
        let noise_node = ComputeNode::new(
            &app.device,
            &BindGroupData {
                workgroup_count: dispatch_group_count,
                storage_buffers: vec![&permulation_buf, &gradient_buf],
                inout_tv: vec![(&tex, Some(wgpu::StorageTextureAccess::WriteOnly))],
                ..Default::default()
            },
            &shader,
        );

        let mut encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("3d noise encoder"),
            });
        noise_node.compute(&mut encoder);
        app.queue.submit(Some(encoder.finish()));

        Self { tex }
    }
}
