#include "pbd/struct/particle.wgsl"
#include "pbd/struct/cloth_uniform.wgsl"

struct BendingConstraint {
    v: i32,
    b0: i32,
    b1: i32,
    // h0 is the rest length (rest radius of curvature)
    h0: f32,
};

@group(0) @binding(0) var<uniform> cloth: ClothUniform;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> constraints: array<BendingConstraint>;

struct DynamicUniform {
    offset: i32,
    max_num_x: i32,
    // 当前分组的数据长度
    group_len: i32,
    // 迭代計數的倒數
    invert_iter: f32,
};
@group(1) @binding(0) var<uniform> dy_uniform: DynamicUniform;

fn is_movable_particle(particle: Particle) -> bool {
    if (particle.uv_mass.z < 0.001) {
        return false;
    }
    return true;
}


@compute @workgroup_size(32, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {  
    var field_index = i32(gid.x);
    if (field_index >= dy_uniform.group_len) {
        return;
    }
    field_index = field_index + dy_uniform.offset;
    
    let bending: BendingConstraint = constraints[field_index];
    // 弯曲约束 C = acos(d) -ϕ0, d = n1.n2
    var v: Particle = particles[bending.v];
    var b0: Particle = particles[bending.b0];
    var b1: Particle = particles[bending.b1];

    // eq. 3
    let c: vec3<f32> = (b0.pos.xyz + b1.pos.xyz + v.pos.xyz) * 0.33333333;
    // eq. 8
    let w = b0.uv_mass.z + b1.uv_mass.z + 2.0 * v.uv_mass.z;
    let v_minus_c = v.pos.xyz - c;
    let v_minus_c_len = length(v_minus_c);
    // eq. 6
    let k = 1.0 - pow(1.0 - cloth.stiffness, dy_uniform.invert_iter);
    // float k = 0.0;
    // eq. 5
    // 弯曲度大于静态值才执行位置修正（论文里的表述反了）
    let c_triangle = v_minus_c_len - (k + bending.h0);
    if (c_triangle <= 0.0) {
        return;
    }
    // eq. 9a, 9b, 9c
    let f = v_minus_c * (1.0 - (k + bending.h0) / v_minus_c_len);

    if (is_movable_particle(v)) {
        v.pos = vec4<f32>(v.pos.xyz + (-4.0 * v.uv_mass.z) / w * f, 0.0);
        particles[bending.v] = v;
    }
    if (is_movable_particle(b0)) {
        b0.pos = vec4<f32>(b0.pos.xyz + (2.0 * b0.uv_mass.z) / w * f, 0.0);
        particles[bending.b0] = b0;
    }
    if (is_movable_particle(b1)) {
        b1.pos = vec4<f32>(b1.pos.xyz + (2.0 * b1.uv_mass.z) / w * f, 0.0);
        particles[bending.b1] = b1;
    }
}
