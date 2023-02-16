#include "struct/field.wgsl"

@group(0) @binding(0) var<uniform> field: FieldUniform;
@group(0) @binding(1) var<storage, read_write> field_buf: array<vec4<f32>>;

fn field_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * field.lattice_size.x);
}

fn get_velocity00(p: vec2<i32>) -> vec2<f32> {
    #insert_code_segment
}

fn get_velocity(p: vec2<i32>) -> vec2<f32> {
    // 将场坐标转换到 [-25, 25] 坐标范围
    var c = vec2<f32>(p) / vec2<f32>(field.lattice_size);
    c = c * 50.0 - vec2<f32>(25.0);
    c *= field.proj_ratio;
    let r = length(c);
    let theta = atan2(c.y, c.x);
    var v = vec2<f32>(c.y, -c.x) / r;
    let t = sqrt(r * 20.0) + theta;
    v *= sin(t) * length(v) * 20.0;
    return (v + c * 0.2) * field.ndc_pixel * 5.0;
}

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let uv = vec2<i32>(global_invocation_id.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
        return;
    }
    let index = field_index(uv);
    field_buf[index] = vec4<f32>(get_velocity(uv), 0.0, 0.0);
}