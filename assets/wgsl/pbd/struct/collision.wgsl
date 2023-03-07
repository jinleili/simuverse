struct CollisionObj {
  // 与之碰撞的三角面
  triangles: array<i32, 8>,
  normals: array<vec4<f32>, 8>,
  // 碰撞约束计数
  // storage buffer 对象的字段中，如果出现了 vec3, 所有字段都将按 vec4 来对齐
  count: i32,
  padding0: i32,
  padding1: i32,
  padding2: i32,
};
