#include "struct/mvp_mat_uniform.wgsl"
#include "struct/noise_params.wgsl"

@group(0) @binding(0) var<uniform> mvp_mat: MVPMatUniform;
@group(0) @binding(1) var<uniform> params: NoiseParams;

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
    result.uv = uv * f32(params.octave);
    result.ec_pos = (mvp_mat.mv * vec4<f32>(pos, 1.0)).xyz;
    result.mc_pos = (mvp_mat.mv * vec4<f32>(pos + vec3(2.0), 1.0)).xyz * f32(params.octave);
    return result;
}

@group(0) @binding(2) var<storage, read> permutation: array<vec4<i32>>;
@group(0) @binding(3) var<storage, read> gradient: array<vec4<f32>>;

#include "noise/fn_perlin_noise.wgsl"

fn turbulence(octaves: i32, P: vec3<f32>, lacunarity: f32, gain: f32) -> f32 {	
  var sum: f32 = 0.0;
  var scale: f32 = 1.0;
  var totalgain: f32 = 1.0;
  for(var i = 0; i < octaves; i = i + 1){
    sum = sum + totalgain * noise(P * scale);
    scale = scale * lacunarity;
    totalgain = totalgain * gain;
  }
  return abs(sum);
}

@fragment 
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // let p = vec3<f32>(vertex.position.xy / 105.0, 0.5) ; 
    

    var front_color = params.front_color.rgb;
    // if (abs(vertex.position.y % 20.0) > 4.0) {
    //   front_color = params.bg_color.rgb;
    // }
    // marble
    // let marble = lerp3(params.bg_color.rgb, front_color, cos(p.z * 0.1 + 6.0 * turbulence(params.octave, p, params.lacunarity, params.gain)));
    // let marble = vec3<f32>(cos(p.z * 0.1 + 6.0 * turbulence(params.octave, p, params.lacunarity, params.gain)));

    let g = noise(vertex.mc_pos) * 30.0;
    let grain = fract(g);
    let marble = lerp3(params.bg_color.rgb, front_color, cos(vertex.mc_pos.z * 0.1 + 6.0 * turbulence(params.octave, vertex.mc_pos, params.lacunarity, grain)));
    return vec4<f32>(marble, 1.0); 

    // noise self
    // let val = noise(vertex.mc_pos) * 0.5 + 0.5 ; 
    // return vec4<f32>(vec3<f32>(val), 1.0);
}

