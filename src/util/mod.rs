pub mod matrix_helper;

mod buffer;
pub use buffer::BufferObj;

pub mod load_texture;
pub use load_texture::AnyTexture;

pub mod shader;
pub mod vertex;

mod plane;
pub use plane::Plane;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MVPUniform {
    pub mvp_matrix: [[f32; 4]; 4],
}
