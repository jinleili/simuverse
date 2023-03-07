#include "pbd/cloth_layout.wgsl"
#include "pbd/struct/dynamic_uniform.wgsl"

@group(1) @binding(0) var<uniform> dy_uniform: DynamicUniform;

@compute @workgroup_size(32, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {  
    var field_index = i32(gid.x);
    if (field_index >= dy_uniform.group_len) {
        return;
    }
    field_index += dy_uniform.offset;

    // a~
    // new_compliance 直接在 uniform 里计算好
    // float new_compliance = compliance / (dt * dt);
    var constraint = constraints[field_index];
    let particle0_index = constraint.particle0;
    var particle = particles[particle0_index];
    let invert_mass0 = particle.uv_mass.z;

    var particle1 = particles[constraint.particle1];
    let invert_mass1 = particle1.uv_mass.z;
    let sum_mass = invert_mass0 + invert_mass1;
    if (sum_mass < 0.01) {
        return;
    }
    let p0_minus_p1 = particle.pos - particle1.pos;
    let dis = length(p0_minus_p1.xyz);
    // Cj(x)
    let distance = dis - constraint.rest_length;

    var correction_vector: vec4<f32>;
    // eq.18
    let dlambda = -distance / (sum_mass + cloth.compliance);
    // eq.17
    correction_vector = dlambda * p0_minus_p1 / (dis + EPSILON);

    // 更新位置
    if (is_movable_particle(particle)) {
        particle.pos = particle.pos + invert_mass0 * correction_vector;
        particles[particle0_index] = particle;
    }
    if (is_movable_particle(particle1)) {
        particle1.pos = particle1.pos + (-invert_mass1) * correction_vector;
        particles[constraint.particle1] = particle1;
    }
}