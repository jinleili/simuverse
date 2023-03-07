#include "pbd/cloth_layout.wgsl"

const ball_pos: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

@compute @workgroup_size(32, 1, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let total = arrayLength(&particles);
    let field_index = gid.x;
    if (field_index >= total) {
      return;
    }
    var particle: Particle = particles[field_index];
    if (is_movable_particle(particle)) {
      let temp_pos = particle.pos;

      // 预估新的位置
    //   particle.pos = particle.pos + (particle.pos - particle.old_pos) + (particle.accelerate + force) * cloth.dt * cloth.dt;
    // Xn+1 = Xn + dt * Vn + dt^2 * M^-1 * F(Xn)
      particle.pos += (particle.pos - particle.old_pos)*(1.0 - cloth.damping) + vec4<f32>(0.0, cloth.gravity, 0.0, 0.0) * particle.uv_mass.z * cloth.dt * cloth.dt ;
      particle.old_pos = temp_pos;
      particles[field_index] = particle;
    }
}