#include "struct/field.wgsl"

@group(0) @binding(0) var<uniform> field: FieldUniform;
@group(0) @binding(1) var<storage, read_write> field_buf: array<vec4<f32>>;

fn field_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * field.lattice_size.x);
}

fn get_velocity(p: vec2<i32>) -> vec2<f32> {
    #insert_code_snippet
}

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let uv = vec2<i32>(gid.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
        return;
    }
    let index = field_index(uv);
    field_buf[index] = vec4<f32>(get_velocity(uv), 0.0, 0.0);
}