struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>, 
}

struct Camera {
    camera_matrix: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(0)@binding(0) var<uniform> camera: Camera;

struct Lights {
    lights: array<Light, 255>,
}

@group(0) @binding(1) var<uniform> lights: Lights;

struct SceneInfo {
    bk_color: vec4<f32>,
    time: f32,
    nlights: u32,
}

@group(0) @binding(2) var<uniform> info: SceneInfo;

struct ModelMatrix {
    model_matrix: mat4x4<f32>,
}

@group(1) @binding(0) var<uniform> model_matrix: ModelMatrix;

struct ModelMaterial {
    material: Material,
}

@group(1) @binding(1) var<uniform> material: ModelMaterial;
@group(1) @binding(2) var r_color: texture_2d<f32>;
@group(1) @binding(3) var r_sampler: sampler;

// @group(2) @binding(0) var<storage, read> permutation: array<vec4<i32>>;
// @group(2) @binding(1) var<storage, read> gradient: array<vec4<f32>>;

struct VertexOutput {
    @builtin(position) gl_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>, 
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_position = model_matrix.model_matrix * vec4<f32>(in.position, 1.0);
    let world_normal = model_matrix.model_matrix * vec4<f32>(in.normal, 0.0);
    return VertexOutput(
        camera.projection * world_position,
        world_position.xyz,
        in.uv,
        normalize(world_normal.xyz)
    );
}

const e: vec2<f32> = vec2<f32>(1.0, 0.0);

@fragment
fn nontex_main(in: VertexInput) -> @location(0) vec4<f32> {
    let camera_dir = normalize((camera.camera_matrix * e.yyyx).xyz - in.position);
    let normal = normalize(in.normal);
    var pre_color: vec3<f32> = vec3<f32>(0.0);

    // 这个循环性能极差，在 M1 Max 上耗时 >32ms 每帧
    // for (var i: u32 = 0u; i < info.nlights; i = i + 1u) {
        pre_color = pre_color + microfacet_color(
            in.position,
            normal,
            lights.lights[0],
            camera_dir,
            material.material,
        );
    // }
    pre_color = clamp(pre_color, vec3<f32>(0.0), vec3<f32>(1.0));
    pre_color = background_correction(pre_color, info.bk_color.xyz, material.material);
    pre_color = ambient_correction(pre_color, material.material);
    return vec4<f32>(pow(pre_color, vec3<f32>(0.4545)), material.material.albedo.a);
}

@fragment
fn tex_main(in: VertexInput) -> @location(0) vec4<f32> {
    var matr: Material = material.material;
    matr.albedo = textureSample(r_color, r_sampler, in.uv);
    matr.albedo = vec4<f32>(pow(matr.albedo.rgb, vec3<f32>(2.2)), matr.albedo.a);
    let camera_dir = normalize((camera.camera_matrix * e.yyyx).xyz - in.position);
    let normal = normalize(in.normal);
    var pre_color: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < info.nlights; i = i + 1u) {
        pre_color = pre_color + microfacet_color(
            in.position,
            normal,
            lights.lights[i],
            camera_dir,
            matr,
        );
    }
    pre_color = clamp(pre_color, vec3<f32>(0.0), vec3<f32>(1.0));
    pre_color = background_correction(pre_color, info.bk_color.xyz, material.material);
    pre_color = ambient_correction(pre_color, matr);

    return vec4<f32>(pow(pre_color, vec3<f32>(0.4545)), matr.albedo.a);
}

@fragment
fn procedural_main(in: VertexInput) -> @location(0) vec4<f32> {
    // Marble
    // let n = cos(in.mc_pos.z * 0.1 + 6.0 * turbulence(in.mc_pos, 6.0, 2.5, 0.6));
    // let color = lerp3(vec3<f32>(0.98), vec3<f32>(0.16, 0.22, 0.5), n);

    // var matr: Material = material.material;
    // matr.albedo = vec4<f32>(pow(color, vec3<f32>(2.2)), 1.0);
    // let camera_dir = normalize((camera.camera_matrix * e.yyyx).xyz - in.position);
    // let normal = normalize(in.normal);
    // var pre_color: vec3<f32> = vec3<f32>(0.0);
    // for (var i: u32 = 0u; i < info.nlights; i = i + 1u) {
    //     pre_color = pre_color + microfacet_color(
    //         in.position,
    //         normal,
    //         lights.lights[i],
    //         camera_dir,
    //         matr,
    //     );
    // }
    // pre_color = clamp(pre_color, vec3<f32>(0.0), vec3<f32>(1.0));
    // pre_color = background_correction(pre_color, info.bk_color.xyz, material.material);
    // pre_color = ambient_correction(pre_color, matr);

    // return vec4<f32>(pow(pre_color, vec3<f32>(0.4545)), matr.albedo.a);
    return vec4<f32>(0.0);
}
