const OBSTACLE_RADIUS: f32 = 28.0;

mod lattice;
use lattice::*;

mod d2q9_node;
mod particle_render_node;

mod fluid_simulator;
pub use fluid_simulator::FluidSimulator;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LbmUniform {
    pub tau: f32,
    pub omega: f32,
    // fluid type, used fot storage buffer initialization
    // 0: poiseuille, 1: custom
    pub fluid_ty: i32,
    // structure of array (put the same direction of all lattice together ) lattice data offset
    pub soa_offset: i32,
    //  D2Q9 lattice direction coordinate:
    // 6 2 5
    // 3 0 1
    // 7 4 8
    // components xy: lattice direction, z: direction's weight, z: direction's max value
    pub e_w_max: [[f32; 4]; 9],
    pub inversed_direction: [[i32; 4]; 9],
}

impl LbmUniform {
    pub fn new(tau: f32, fluid_ty: i32, soa_offset: i32) -> Self {
        LbmUniform {
            tau,
            omega: 1.0 / tau,
            fluid_ty,
            soa_offset,
            // lattice direction's weight
            e_w_max: [
                [0.0, 0.0, 0.444444, 0.6],
                [1.0, 0.0, 0.111111, 0.2222],
                [0.0, -1.0, 0.111111, 0.2222],
                [-1.0, 0.0, 0.111111, 0.2222],
                [0.0, 1.0, 0.111111, 0.2222],
                [1.0, -1.0, 0.0277777, 0.1111],
                [-1.0, -1.0, 0.0277777, 0.1111],
                [-1.0, 1.0, 0.0277777, 0.1111],
                [1.0, 1.0, 0.0277777, 0.1111],
            ],
            inversed_direction: [
                [0; 4], [3; 4], [4; 4], [1; 4], [2; 4], [7; 4], [8; 4], [5; 4], [6; 4],
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TickTock {
    // A-A pattern lattice offset
    read_offset: i32,
    write_offset: i32,
    _pading0: i32,
    _pading1: i32,
}

fn is_sd_sphere(p: &app_surface::math::Position, r: f32) -> bool {
    p.length() <= r
}
