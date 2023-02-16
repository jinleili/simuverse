
struct FieldUniform {
  // 场格子数
  lattice_size: vec2<i32>,
  // 格子所占像素数
  lattice_pixel_size: vec2<f32>,
  // 画布物理像素数
  canvas_size: vec2<i32>,
  // 投影屏幕宽高比
  proj_ratio: vec2<f32>,
  // 单个像素在 NDC 空间中的大小
  ndc_pixel: vec2<f32>,
  // 0: pixel speed, field player used 
  // 1: lbm lattice speed, fluid player used. Its value is usually no greater than 0.2
  speed_ty: i32,
};

