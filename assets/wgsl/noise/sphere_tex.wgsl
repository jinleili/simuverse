#include "struct/mvp_mat_uniform.wgsl"
#include "struct/noise_params.wgsl"

@group(0) @binding(0) var<uniform> mvp_mat: MVPMatUniform;
@group(0) @binding(1) var<uniform> params: NoiseParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) mc_pos: vec3<f32>,
    @location(3) normal: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = mvp_mat.mvp * vec4<f32>(pos, 1.0);
    out.uv = uv;
    out.world_pos = (mvp_mat.mv * vec4<f32>(pos, 1.0)).xyz;
    out.mc_pos = (mvp_mat.mv_no_rotation * vec4<f32>(pos + vec3(4.0), 1.0)).xyz * params.noise_scale;
    out.normal = (mvp_mat.normal * vec4<f32>(normal, 1.0)).xyz;
    return out;
}

@group(0) @binding(2) var<storage, read> permutation: array<vec4<i32>>;
@group(0) @binding(3) var<storage, read> gradient: array<vec4<f32>>;

#include "noise/fn_perlin_noise.wgsl"

fn turbulence(pos: vec3<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {	
  var sum: f32 = 0.0;
  var scale: f32 = 1.0;
  var totalgain: f32 = 1.0;
  for(var i = 0; i < octaves; i = i + 1){
    sum = sum + totalgain * noise(pos * scale);
    scale = scale * lacunarity;
    totalgain = totalgain * gain;
  }
  return abs(sum);
}

// fractal sum
fn fBm(pos: vec3<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
	var freq = 1.0;
    var amp = 0.5;
	var sum = 0.0;	
	for (var i: i32 = 0; i < octaves; i++) {
		sum += noise(pos*freq)*amp;
		freq *= lacunarity;
		amp *= gain;
	}
	return sum;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var n: f32;

    if (params.ty == 0) {
        // Marble
        n = cos(in.mc_pos.z * 0.1 + 6.0 * turbulence(in.mc_pos, params.octave, params.lacunarity, params.gain));
    } else if (params.ty == 1) {
        // Wood
        let g = noise(in.mc_pos) * 30.0;
        let grain = fract(g);
        n = cos(in.mc_pos.z * 0.1 + 6.0 * turbulence(in.mc_pos, params.octave, params.lacunarity, grain));
    } else {
        // Earth
        n = fBm(in.mc_pos, 4, params.lacunarity, params.gain) * 0.6 + 0.4;
    }
    let simu_color = lerp3(params.bg_color.rgb, params.front_color.rgb, n);

    // Light
    let light_color = vec3<f32>(1.0);
    let light_pos = vec3<f32>(2.0, 3.5, 4.0);
    let view_pos = vec3<f32>(0.0, 0., 3.0);
    let ambient_strength = 0.5;
    let ambient_color = light_color * ambient_strength;

    // Create the lighting vectors
    let light_dir = normalize(light_pos - in.world_pos);
    let view_dir = normalize(view_pos - in.world_pos);
    let half_dir = normalize(view_dir + light_dir);

    let new_normal = normalize(in.normal);
    let diffuse_strength = max(dot(new_normal, light_dir), 0.0);
    let diffuse_color = light_color * diffuse_strength;

    let specular_strength = pow(max(dot(new_normal, half_dir), 0.0), 16.0) * 0.5;
    let specular_color = light_color * specular_strength;

    let res_color = (ambient_color + diffuse_color + specular_color) * simu_color;

    return vec4<f32>(res_color, 1.);
}