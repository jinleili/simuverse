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
    out.mc_pos = (mvp_mat.mv_no_rotation * vec4<f32>(pos + vec3(3.5), 1.0)).xyz * params.noise_scale;
    out.normal = (mvp_mat.normal * vec4<f32>(normal, 1.0)).xyz;
    return out;
}

@group(0) @binding(2) var<storage, read> permutation: array<vec4<i32>>;
@group(0) @binding(3) var<storage, read> gradient: array<vec4<f32>>;

#include "noise/fn_perlin_noise.wgsl"
#include "func/color_space_convert.wgsl"

// Fractal brownian motion
fn fbm(pos: vec3<f32>) -> f32 {
	var freq = 1.0;
    var amp = 0.5;
	var sum = 0.0;	
	for (var i: i32 = 0; i < params.octave; i++) {
		sum += noise(pos * freq) * amp;
		freq *= params.lacunarity;
		amp *= params.gain;
	}
	return sum;
}

const m3 = mat3x3<f32>(vec3<f32>(0.10,  0.80,  0.60),
                      vec3<f32>(-0.80,  0.36, -0.48),
                      vec3<f32>(-0.60, -0.48,  0.64) );

fn fbm2(pos: vec3<f32>) -> f32 {
	var x = pos;
    var amp = 0.5;
	var sum = 0.0;	
	for (var i: i32 = 0; i < params.octave; i++) {
		sum += noise(x) * amp;
        // Rotate to reduce axial bias
		x = params.lacunarity * m3 * x;
		amp *= params.gain;
	}
	return sum;
}

fn bias(t: f32, b: f32) -> f32 {
	return pow(t, log(b)/log(0.5));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var n: f32;
    var simu_color: vec3<f32>;
    if (params.ty == 0) {
        // Marble
        n = cos(in.mc_pos.z * 0.1 + 6.0 * turbulence(in.mc_pos, params.octave, params.lacunarity, params.gain));
        simu_color = lerp3(params.bg_color.rgb, params.front_color.rgb, n);
    } else if (params.ty == 1) {
        // Wood
        let g = noise(in.mc_pos) * 30.0;
        let grain = fract(g);
        n = cos(in.mc_pos.z * 0.1 + 6.0 * turbulence(in.mc_pos, params.octave, params.lacunarity, grain));
        simu_color = lerp3(params.bg_color.rgb, params.front_color.rgb, n);
    } else if (params.ty == 2) {
        // Grim world
        let q = vec3<f32>(n, fbm(in.mc_pos + vec3<f32>(5.2, 1.3, 0.4)), fbm(in.mc_pos + vec3<f32>(9.2, 2.3, 13.6)));
        let r = vec3<f32>(fbm(in.mc_pos + 4.0*q + vec3<f32>(1.7,9.2, 12.7)),
                   fbm(in.mc_pos + 4.0*q + vec3<f32>(8.3,2.8, 0.3)), fbm(in.mc_pos + 4.0*q));
        let f = fbm(in.mc_pos + 4.0 * r);
        simu_color =  vec3<f32>(0.176, 0.204, 0.216);
        simu_color =  mix(simu_color, params.bg_color.rgb, f);
        simu_color =  mix(simu_color, params.front_color.rgb, r*0.9);
    } else {
        var q = vec3<f32>(fbm2(in.mc_pos), fbm2(in.mc_pos + 1.0), fbm2(in.mc_pos + 2.0));
        let r = vec3<f32>(fbm2(in.mc_pos + q + vec3<f32>(1.7,9.2, 3.3)+ 0.15 * mvp_mat.u_time), 
                            fbm2(in.mc_pos + q + vec3<f32>(8.3,2.8, 1.1)+ 0.126 * mvp_mat.u_time), 
                            fbm2(in.mc_pos + q + vec3<f32>(1.3,5.1, 9.7)+ 0.09 * mvp_mat.u_time));
        let f = fbm2(in.mc_pos + r);

        simu_color = mix(vec3<f32>(0.101961,0.619608,0.666667),
                    vec3<f32>(0.666667,0.666667,0.498039), min(f*3.2, 1.0));
        simu_color = mix(simu_color,
                    params.bg_color.rgb, min(length(q), 1.0));
        simu_color = mix(simu_color, 
                    params.front_color.rgb, min(length(r.x), 1.0));
    }

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