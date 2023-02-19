use wgpu::{PushConstantRange, ShaderModule, StorageTextureAccess};

use super::{BindingGroupSetting, DynamicUniformBindingGroup};
use crate::util::{AnyTexture, BufferObj};

use core::ops::Range;
use std::vec::Vec;

pub struct ComputeNode {
    pub bg_setting: BindingGroupSetting,
    pub dy_uniform_bg: Option<DynamicUniformBindingGroup>,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::ComputePipeline,
    pub group_count: (u32, u32, u32),
}

impl ComputeNode {
    pub fn new(
        device: &wgpu::Device,
        group_count: (u32, u32, u32),
        uniforms: Vec<&BufferObj>,
        storage_buffers: Vec<&BufferObj>,
        inout_tv: Vec<(&AnyTexture, Option<StorageTextureAccess>)>,
        shader_module: &ShaderModule,
    ) -> Self {
        ComputeNode::new_with_push_constants(
            device,
            group_count,
            uniforms,
            storage_buffers,
            inout_tv,
            shader_module,
            None,
        )
    }

    #[allow(dead_code)]
    pub fn new_with_dynamic_uniforms(
        device: &wgpu::Device,
        group_count: (u32, u32, u32),
        uniforms: Vec<&BufferObj>,
        dynamic_uniforms: Vec<&BufferObj>,
        storage_buffers: Vec<&BufferObj>,
        inout_tv: Vec<(&AnyTexture, Option<StorageTextureAccess>)>,
        shader_module: &ShaderModule,
    ) -> Self {
        let mut visibilitys: Vec<wgpu::ShaderStages> = vec![];
        for _ in 0..(uniforms.len() + storage_buffers.len() + inout_tv.len()) {
            visibilitys.push(wgpu::ShaderStages::COMPUTE);
        }
        let bg_setting = BindingGroupSetting::new(
            device,
            uniforms,
            storage_buffers,
            inout_tv,
            vec![],
            visibilitys,
        );
        let mut dy_uniforms: Vec<(&BufferObj, wgpu::ShaderStages)> = vec![];
        for obj in dynamic_uniforms {
            dy_uniforms.push((obj, wgpu::ShaderStages::COMPUTE));
        }
        let dy_uniform_bg = DynamicUniformBindingGroup::new(device, dy_uniforms);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &bg_setting.bind_group_layout,
                &dy_uniform_bg.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: shader_module,
            entry_point: "cs_main",
        });

        ComputeNode {
            bg_setting,
            dy_uniform_bg: Some(dy_uniform_bg),
            pipeline_layout,
            pipeline,
            group_count,
        }
    }

    pub fn new_with_push_constants(
        device: &wgpu::Device,
        group_count: (u32, u32, u32),
        uniforms: Vec<&BufferObj>,
        storage_buffers: Vec<&BufferObj>,
        inout_tv: Vec<(&AnyTexture, Option<StorageTextureAccess>)>,
        shader_module: &ShaderModule,
        push_constants: Option<Vec<(wgpu::ShaderStages, Range<u32>)>>,
    ) -> Self {
        let mut visibilitys: Vec<wgpu::ShaderStages> = vec![];
        for _ in 0..(uniforms.len() + storage_buffers.len() + inout_tv.len()) {
            visibilitys.push(wgpu::ShaderStages::COMPUTE);
        }
        let bg_setting = BindingGroupSetting::new(
            device,
            uniforms,
            storage_buffers,
            inout_tv,
            vec![],
            visibilitys,
        );

        let mut ranges: Vec<PushConstantRange> = vec![];
        if let Some(constants) = push_constants {
            for (stage, range) in constants.iter() {
                ranges.push(wgpu::PushConstantRange {
                    stages: *stage,
                    range: range.clone(),
                })
            }
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bg_setting.bind_group_layout],
            push_constant_ranges: &ranges,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: shader_module,
            entry_point: "cs_main",
        });

        ComputeNode {
            bg_setting,
            dy_uniform_bg: None,
            pipeline_layout,
            pipeline,
            group_count,
        }
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        self.compute_by_offsets(encoder, None);
    }

    pub fn compute_by_pass<'a, 'b: 'a>(&'b self, cpass: &mut wgpu::ComputePass<'a>) {
        self.dispatch_by_offsets(cpass, None);
    }

    pub fn compute_by_offsets(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        offsets: Option<Vec<Vec<wgpu::DynamicOffset>>>,
    ) {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        self.dispatch_by_offsets(&mut cpass, offsets);
    }

    pub fn dispatch_by_offsets<'a, 'b: 'a>(
        &'b self,
        cpass: &mut wgpu::ComputePass<'a>,
        offsets: Option<Vec<Vec<wgpu::DynamicOffset>>>,
    ) {
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.bg_setting.bind_group, &[]);
        if let Some(offsets) = offsets {
            for os in offsets {
                cpass.set_bind_group(1, &self.dy_uniform_bg.as_ref().unwrap().bind_group, &os);
                cpass.dispatch_workgroups(
                    self.group_count.0,
                    self.group_count.1,
                    self.group_count.2,
                );
            }
        } else {
            cpass.dispatch_workgroups(self.group_count.0, self.group_count.1, self.group_count.2);
        }
    }
}
