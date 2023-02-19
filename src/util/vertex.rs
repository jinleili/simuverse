#[allow(dead_code)]

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

impl PosOnly {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3]) -> PosOnly {
        PosOnly { pos }
    }
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

impl PosTangent {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3], tangent: [f32; 3]) -> PosTangent {
        PosTangent { pos, tangent }
    }
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

impl PosColor {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3], color: [f32; 4]) -> PosColor {
        PosColor { pos, color }
    }

    pub fn color_offset() -> wgpu::BufferAddress {
        4 * 3
    }
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
                offset: PosColor::color_offset(),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosTex {
    pub pos: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl PosTex {
    #[allow(dead_code)]
    pub fn vertex_i(pos: [i8; 3], tc: [i8; 2]) -> PosTex {
        PosTex {
            pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
            tex_coord: [tc[0] as f32, tc[1] as f32],
        }
    }

    pub fn vertex_f32(pos: [f32; 3], tex_coord: [f32; 2]) -> PosTex {
        PosTex { pos, tex_coord }
    }

    pub fn tex_offset() -> wgpu::BufferAddress {
        4 * 3
    }

    // 移动顶点位置到
    // step_rate: step_index / step_count
    pub fn move_to(&self, to: &[f32; 3], step_rate: f32) -> PosTex {
        PosTex {
            pos: [
                self.pos[0] + (to[0] - self.pos[0]) * step_rate,
                self.pos[1] + (to[1] - self.pos[1]) * step_rate,
                self.pos[2] + (to[2] - self.pos[2]) * step_rate,
            ],
            tex_coord: self.tex_coord,
        }
    }
}

impl Vertex for PosTex {
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
                offset: PosTex::tex_offset(),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosTex2 {
    pos: [f32; 3],
    tex_coord0: [f32; 2],
    tex_coord1: [f32; 2],
}

impl PosTex2 {
    #[allow(dead_code)]
    pub fn vertex_i(pos: [i8; 3], tc0: [i8; 2], tc1: [i8; 2]) -> PosTex2 {
        PosTex2 {
            pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
            tex_coord0: [tc0[0] as f32, tc0[1] as f32],
            tex_coord1: [tc1[0] as f32, tc1[1] as f32],
        }
    }

    pub fn vertex_f32(pos: [f32; 3], tex_coord0: [f32; 2], tex_coord1: [f32; 2]) -> PosTex2 {
        PosTex2 {
            pos,
            tex_coord0,
            tex_coord1,
        }
    }

    pub fn tex_offset() -> wgpu::BufferAddress {
        4 * 3
    }
}

impl Vertex for PosTex2 {
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
                offset: PosTex2::tex_offset(),
            },
            wgpu::VertexAttribute {
                shader_location: offset + 2,
                format: wgpu::VertexFormat::Float32x2,
                offset: PosTex2::tex_offset() + (4 * 2),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosWeight {
    pub pos: [f32; 3],
    // 离数学中心位置的权重
    pub weight: f32,
}

#[allow(dead_code)]
impl PosWeight {
    pub fn new(pos: [f32; 3], weight: f32) -> Self {
        PosWeight { pos, weight }
    }

    pub fn slope_ridian(&self, last: &PosWeight) -> f32 {
        (self.pos[1] - last.pos[1]).atan2(self.pos[0] - last.pos[0])
    }
}

impl Vertex for PosWeight {
    fn vertex_attributes(offset: u32) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                shader_location: offset,
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
            },
            wgpu::VertexAttribute {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float32,
                offset: 4 * 3,
            },
        ]
    }
}
