use crate::util::BufferObj;
use std::vec::Vec;

#[allow(dead_code)]
pub struct DynamicUniformBindingGroup {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DynamicUniformBindingGroup {
    pub fn new(device: &wgpu::Device, uniforms: Vec<(&BufferObj, wgpu::ShaderStages)>) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![];
        let mut entries: Vec<wgpu::BindGroupEntry> = vec![];

        let mut b_index = 0;
        for i in 0..uniforms.len() {
            let buffer_obj = uniforms[i];

            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: buffer_obj.1,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    // min_binding_size: buffer_obj.0.min_binding_size,
                    min_binding_size: None,
                },
                count: None,
            });
            // 对于动态 uniform buffer, 必须指定 buffer slice 大小
            // make sure that in your BindingResource::Buffer, you're slicing with .slice(..size_of::<Whatever>() as BufferAddress)
            // and not .slice(..)
            // for dynamic uniform buffers, BindingResource::Buffer specifies a "window" into the buffer that is then offset by your dynamic offset value
            entries.push(wgpu::BindGroupEntry {
                binding: b_index,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer_obj.0.buffer,
                    offset: 0,
                    // size: buffer_obj.0.min_binding_size,
                    size: wgpu::BufferSize::new(
                        device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress,
                    ),
                }),
            });
            b_index += 1;
        }
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layouts,
            label: None,
        });

        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: None,
        });

        DynamicUniformBindingGroup { bind_group_layout, bind_group }
    }
}
