use crate::util::vertex::PosNormalUv;
use alloc::{vec, vec::Vec};

pub struct Sphere {
    radius: f32,
    h_segments: usize,
    v_segments: usize,
}

impl Sphere {
    pub fn new(radius: f32, h_segments: usize, v_segments: usize) -> Self {
        Sphere {
            radius,
            h_segments,
            v_segments,
        }
    }

    pub fn generate_vertices(&self) -> (Vec<PosNormalUv>, Vec<u32>) {
        let phi_len = core::f32::consts::PI * 2.0;
        let theta_len = core::f32::consts::PI;

        let mut index: u32 = 0;
        let mut grid: Vec<Vec<u32>> = vec![];

        let mut vertices = vec![];
        let mut indices: Vec<u32> = vec![];
        for iy in 0..=self.v_segments {
            let mut vertices_row: Vec<u32> = vec![];
            let v = iy as f32 / self.v_segments as f32;
            // poles
            let mut u_offset = 0.0;
            if iy == 0 {
                u_offset = 0.5 / self.h_segments as f32;
            } else if iy == self.h_segments {
                u_offset = -0.5 / self.h_segments as f32;
            }

            for ix in 0..=self.h_segments {
                let u = ix as f32 / self.h_segments as f32;

                // vertex
                let pos = [
                    -self.radius * (u * phi_len).cos() * (v * theta_len).sin(),
                    self.radius * (v * theta_len).cos(),
                    self.radius * (u * phi_len).sin() * (v * theta_len).sin(),
                ];
                // normal
                let normal = glam::Vec3::new(pos[0], pos[1], pos[2])
                    .normalize()
                    .to_array();
                // uv
                let uv = [u + u_offset, 1.0 - v];
                vertices.push(PosNormalUv { pos, normal, uv });

                vertices_row.push(index);
                index += 1;
            }
            grid.push(vertices_row);
        }

        // indecis
        for iy in 0..self.v_segments {
            for ix in 0..self.h_segments {
                let a = grid[iy][ix + 1];
                let b = grid[iy][ix];
                let c = grid[iy + 1][ix];
                let d = grid[iy + 1][ix + 1];

                if iy != 0 {
                    indices.push(a);
                    indices.push(b);
                    indices.push(d);
                }
                // pole
                if iy != self.v_segments - 1 {
                    indices.push(b);
                    indices.push(c);
                    indices.push(d);
                }
            }
        }
        (vertices, indices)
    }
}
