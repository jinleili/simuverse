struct Particle {
   pos: vec4<f32>,
   old_pos: vec4<f32>,
   accelerate: vec4<f32>,
  // uv_mass.z = invert_mass
  // struct 里包含不同的 vec3，vec2, float 时，实际会按 vec4 来对齐，导致数据访问异常
   uv_mass: vec4<f32>,
  // 与之相连的4个粒子的索引，用于计算法线
   connect: vec4<i32>,
};
