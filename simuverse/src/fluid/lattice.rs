use super::{OBSTACLE_RADIUS, is_sd_sphere};
use crate::FieldAnimationType;
use alloc::{vec, vec::Vec};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LatticeInfo {
    pub material: i32,
    //  dynamic iter value, change material ultimately
    pub block_iter: i32,
    pub vx: f32,
    pub vy: f32,
}

pub enum LatticeType {
    Bulk = 1,
    Boundary = 2,
    Inlet = 3,
    Obstacle = 4,
    Outlet = 5,
    // external force
    ExternalForce = 6,
    Ghost = 7,
}

pub fn init_lattice_material(
    lattice_size: wgpu::Extent3d,
    ty: FieldAnimationType,
) -> Vec<LatticeInfo> {
    let mut info: Vec<LatticeInfo> = vec![];
    let (nx, ny, nz) = (
        lattice_size.width,
        lattice_size.height,
        lattice_size.depth_or_array_layers,
    );
    let s0 = glam::Vec2::new(nx as f32 / 7.0 - OBSTACLE_RADIUS, ny as f32 / 2.0);
    let s1 = glam::Vec2::new(nx as f32 / 5.0, ny as f32 / 4.0);
    let s2 = glam::Vec2::new(nx as f32 / 5.0, ny as f32 * 0.75);
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let mut material = LatticeType::Bulk as i32;
                let mut vx = 0.0;

                // need boundary cell to avoid NAN
                match ty {
                    FieldAnimationType::Custom => {
                        if x == 0 || x == nx - 1 || y == 0 || y == ny - 1 {
                            material = LatticeType::Boundary as i32;
                        }
                    }
                    FieldAnimationType::LidDrivenCavity => {
                        if x == 0 || x == nx - 1 || y == ny - 1 {
                            material = LatticeType::Boundary as i32;
                        } else if y == 0 {
                            material = LatticeType::Ghost as i32;
                        } else if y == 1 {
                            material = LatticeType::ExternalForce as i32;
                            vx = 0.13;
                        }
                    }
                    FieldAnimationType::Poiseuille => {
                        // poiseuille
                        if y == 0 || y == ny - 1 || (nz > 1 && (z == 0 || z == nz - 1)) {
                            material = LatticeType::Boundary as i32;
                        } else if x == 0 || x == nx - 1 {
                            material = LatticeType::Ghost as i32;
                        } else if x == 1 {
                            material = LatticeType::Inlet as i32;
                            vx = 0.12;
                        } else if x == nx - 2 {
                            material = LatticeType::Outlet as i32;
                        } else {
                            // obstacle
                            let p = glam::Vec2::new(x as f32, y as f32);
                            if is_sd_sphere(&(p - s0), OBSTACLE_RADIUS)
                                || is_sd_sphere(&(p - s1), OBSTACLE_RADIUS)
                                || is_sd_sphere(&(p - s2), OBSTACLE_RADIUS)
                            {
                                material = LatticeType::Obstacle as i32;
                            }
                        }
                    }
                    _ => {}
                }

                info.push(LatticeInfo {
                    material,
                    block_iter: -1,
                    vx,
                    vy: 0.0,
                });
            }
        }
    }

    info
}
