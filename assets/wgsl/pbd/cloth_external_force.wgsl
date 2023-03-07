#include "pbd/struct/particle.wgsl"
#include "pbd/struct/cloth_uniform.wgsl"

@group(0) @binding(0) var<uniform> cloth: ClothUniform;
@group(0) @binding(1) var<storage, read_write> velocity: vec4<f32>;
@group(0) @binding(2) var<storage, read_write> particles: array<Particle>;

fn is_movable_particle(particle: Particle) -> bool {
  if (particle.uv_mass.z < 0.001) {
    return false;
  }
  return true;
}

const offset = -0.015;

@compute @workgroup_size(64, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {  
    let index = i32(gid.x);
    var particle = particles[index];

    // if (!is_movable_particle(particle)) {
    if (index < cloth.num_x) {
      if (velocity.x == 0.0) {
        velocity = vec4<f32>(-1.0, 0.0, 0.07, 0.0);
      }

      particle.pos.x += velocity.x * offset;
      particle.pos.z += velocity.z * offset;

      particle.old_pos = particle.pos;
      particles[index] = particle;

      if (index == cloth.num_x - 1) {
        if (particle.pos.x < - 0.1 ) {
          velocity = vec4<f32>(-1.0, 0.0, 0.07, 0.0);
        } else if (particle.pos.x > 2.17) {
          velocity *= -1.0;
        }
      }
    }
}