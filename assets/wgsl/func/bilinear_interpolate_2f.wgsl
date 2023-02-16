fn bilinear_interpolate_2f(uv: vec2<f32>) -> vec2<f32> {
  let minX: i32 = i32(floor(uv.x));
  let minY: i32 = i32(floor(uv.y));

  let fx: f32 = uv.x - f32(minX);
  let fy: f32 = uv.y - f32(minY);
  // formula： f(i+u,j+v) = (1-u)(1-v)f(i,j) + (1-u)vf(i,j+1) +
  // u(1-v)f(i+1,j) + uvf(i+1,j+1)
  return src_2f(minX, minY) * ((1.0 - fx) * (1.0 - fy)) +
         src_2f(minX, minY + 1) * ((1.0 - fx) * fy) +
         src_2f(minX + 1, minY) * (fx * (1.0 - fy)) +
         src_2f(minX + 1, minY + 1) * (fx * fy);
}