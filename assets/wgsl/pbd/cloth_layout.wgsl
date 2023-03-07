#include "pbd/struct/particle.wgsl"
#include "pbd/struct/cloth_uniform.wgsl"

struct Constraint {
   rest_length: f32,
   lambda: f32,
   particle0: i32,
   particle1: i32,
};

@group(0) @binding(0) var<uniform> cloth: ClothUniform;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> constraints: array<Constraint>;

const EPSILON: f32 = 0.0000001;

fn is_movable_particle(particle: Particle) -> bool {
  if (particle.uv_mass.z < 0.001) {
    return false;
  }
  return true;
}
