#include "struct/mvp_mat_uniform.wgsl"

@group(0) @binding(0) var<uniform> mvp_mat: MVPMatUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) ec_pos: vec3<f32>,
    @location(2) mc_pos: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.position = mvp_mat.mvp * vec4<f32>(pos, 1.0);
    result.uv = uv;
    result.ec_pos = (mvp_mat.mv * vec4<f32>(pos, 1.0)).xyz;
    result.mc_pos = pos;
    return result;
}


@group(0) @binding(1) var noise: texture_3d<f32>;
@group(0) @binding(2) var noise_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {

    let nv: vec4<f32> = textureSample(noise, noise_sampler, vertex.ec_pos);
    
    return vec4<f32>(nv.rgb, 1.0);
}