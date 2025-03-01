#![allow(dead_code)]

use alloc::{vec, vec::Vec};

pub trait Vertex {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute>;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexEmpty {}
impl Vertex for VertexEmpty {
    fn vertex_attributes(_offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosOnly {
    pub pos: [f32; 3],
}

impl Vertex for PosOnly {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![wgpu::VertexAttribute {
            shader_location: offset,
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
        }]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosTangent {
    pub pos: [f32; 3],
    pub tangent: [f32; 3],
}

impl Vertex for PosTangent {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                shader_location: offset,
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float32x3,
                offset: 4 * 3,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosColor {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex for PosColor {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                shader_location: offset,
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float32x4,
                offset: 4 * 3,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosUv {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex for PosUv {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                shader_location: offset,
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float32x2,
                offset: 4 * 3,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosUv2 {
    pub pos: [f32; 3],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
}

impl Vertex for PosUv2 {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                shader_location: offset,
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float32x2,
                offset: 4 * 3,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 2,
                format: wgpu::VertexFormat::Float32x2,
                offset: 4 * 5,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosNormalUv {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex for PosNormalUv {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                shader_location: offset,
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float32x3,
                offset: 4 * 3,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 2,
                format: wgpu::VertexFormat::Float32x2,
                offset: 4 * 6,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosParticleIndex {
    pos: [u32; 3],
}

#[allow(dead_code)]
impl PosParticleIndex {
    pub fn new(pos: [u32; 3]) -> Self {
        PosParticleIndex { pos }
    }
}

impl Vertex for PosParticleIndex {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![wgpu::VertexAttribute {
            shader_location: offset,
            format: wgpu::VertexFormat::Uint32x3,
            offset: 0,
        }]
    }
}
