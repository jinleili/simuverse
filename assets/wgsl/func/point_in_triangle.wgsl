// 判断与三角形在同一个平面的点是否处在三角形内

// 此方法的具体解释：
// 第一条回答
// https://math.stackexchange.com/questions/51326/determining-if-an-arbitrary-point-lies-inside-a-triangle-defined-by-three-points
// https://gdbooks.gitbooks.io/3dcollisions/content/Chapter4/point_in_triangle.html
fn is_point_in_triangle( p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> bool {
  // 以 p 为原点
  let na = a - p;
  let nb = b - p;
  let nc = c - p;

  // Compute the normal vectors for triangles:
  // u = normal of PBC
  // v = normal of PCA
  // w = normal of PAB
  let u: vec3<f32> = cross(nb, nc);
  let v: vec3<f32> = cross(nc, na);
  let w: vec3<f32> = cross(na, nb);

  // 检测法线是否都在同一个方向
  // 点积 dot(u, v)== 0，可能是因为要碰撞的点与三角形的静态位置已经在同一个平面
  if (dot(u, v) <= 0.0) {
    return false;
  }
  if (dot(u, w) <= 0.0) {
    return false;
  }

  return true;
}