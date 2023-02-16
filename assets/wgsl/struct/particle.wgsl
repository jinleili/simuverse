
struct ParticleUniform {
    // particle uniform color
    color: vec4<f32>,
    num: vec2<i32>,
    point_size: i32,
    life_time: f32,
    fade_out_factor: f32,
    // pixels moved per unit speed 
    speed_factor: f32,
    // particle color type 1: use velocity as particle color, 2: angle as color
    color_ty: i32,
    // 0: draw on the canvas;
    // 1: not draw on the canvas
    is_only_update_pos: i32,
};

struct TrajectoryParticle {
    pos: vec2<f32>,
    // initial position, use to reset particle position
    pos_initial: vec2<f32>,
    life_time: f32,
    // alpha value:[1.0, 0.0]
    fade: f32,
};
