#include "lbm/layout_and_fn.wgsl"


fn diffuse_feq(velocity: vec2<f32>, rho: f32, direction: i32) -> f32 {
  // return rho * w(direction) * (1.0 + 3.0 * dot(e(direction), velocity));
  return rho * w(direction);
}

fn diffuse_feq2(velocity: vec2<f32>, rho: f32, direction: i32, usqr: f32) -> f32 {
  let e_dot_u = dot(e(direction), velocity);
  // 公式： w * (rho + 𝛙*rho(3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr))
  // psi 放在 loop
  // 外计算后传进来并不能提高性能，似乎纯数据运算的速度是极快的，减小几步运算并没有优化效果
  let psi = smoothstep(0.01, 0.2, rho) * rho;
  return w(direction) * (rho + psi * (3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr));
}

fn equilibrium(velocity: vec2<f32>, rho: f32, direction: i32, usqr: f32) -> f32 {
  let e_dot_u = dot(e(direction), velocity);
  // internal fn pow(x, y) requires x cannot be negative
  return rho * w(direction) * (1.0 + 3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr);
}


@compute @workgroup_size(64, 4)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let uv = vec2<i32>(gid.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
      return;
    }
    var field_index : i32 = fieldIndex(uv);
    var info: LatticeInfo = lattice_info[field_index];
    // streaming out on boundary cell will cause crash
    if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
      // macro_info.data[field_index] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
      textureStore(macro_info, vec2<i32>(uv), vec4<f32>(0.0, 0.0, 0.0, 0.0));
      return;
    }
    
    var f_i : array<f32, 9>;
    var velocity : vec2<f32> = vec2<f32>(0.0, 0.0);
    var rho : f32 = 0.0;
    for (var i : i32 = 0; i < 9; i = i + 1) {
      f_i[i] = collide_cell.data[streaming_in(uv, i)];
      // f_i[i] = collide_cell.data[field_index + soaOffset(i)];
      rho = rho + f_i[i];
      velocity = velocity + e(i) * f_i[i];
    }
    rho = clamp(rho, 0.8, 1.2);

    velocity = velocity / rho;
    // external forcing
    var F : array<f32, 9> = array<f32, 9>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    // let force_x: f32 = 8.0 * 0.35 * 0.1 / (200.0 * 200.0);
    if (isAccelerateCell(info.material)) {
      if (info.block_iter > 0) {
        info.block_iter = info.block_iter - 1;
        if (info.block_iter == 0) {
          info.material = 1;
        }
      }
      lattice_info[field_index] = info;

      let force = vec2<f32>(info.vx, info.vy);
      // velocity.x = velocity.x + force_x * 0.5 / rho;
      velocity = force * 0.5 / rho;

      for (var i : i32 = 0; i < 9; i = i + 1) {
        F[i] = w(i) * 3.0 * dot(e(i), force);
      }
    }
   
    // macro_info.data[field_index] = vec4<f32>(velocity.x, velocity.y, rho, 0.0);
    textureStore(macro_info, vec2<i32>(uv), vec4<f32>(velocity.x, velocity.y, rho, 1.0));

    let usqr = 1.5 * dot(velocity, velocity);
    for (var i : i32 = 0; i < 9; i = i + 1) {
      var temp_val: f32 = f_i[i] - fluid.omega * (f_i[i] - equilibrium(velocity, rho, i, usqr)) + F[i];
      if (temp_val > max_value(i) || (temp_val == 1.0 / 0.0)) {
        temp_val = max_value(i);
      } else if (temp_val < 0.0) {
        temp_val = 0.0;
      }
      // stream_cell.data[streaming_out(uv, i)] = temp_val;
      stream_cell.data[field_index + soaOffset(i)] = temp_val;
      // stream_cell.data[field_index + soaOffset(fluid.inversed_direction[i].x)] = temp_val;
    }
}
