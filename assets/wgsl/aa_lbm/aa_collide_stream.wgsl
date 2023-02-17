#include "aa_lbm/aa_layout_and_fn.wgsl"

struct TickTock {
  // A-A pattern lattice offset
  read_offset: i32,
  write_offset: i32,
  _pading0: i32,
  _pading1: i32,
}

@group(1) @binding(0) var<uniform> params: array<TickTock, 9>;

struct EqResults {
  val0: f32,
  val1: f32,
};
fn equilibrium(velocity: vec2<f32>, rho: f32, direction: i32, usqr: f32) -> EqResults {
  let e_dot_u = dot(e(direction), velocity);
  // internal fn pow(x, y) requires x cannot be negative
  var res: EqResults;
  res.val0 = rho * w(direction) * (1.0 + 3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr);
  res.val1 = res.val0 - 6.0 * rho * w(direction) * e_dot_u;
  return res;
}
fn equilibrium2(velocity: vec2<f32>, rho: f32, direction: i32, usqr: f32) -> f32 {
  let e_dot_u = dot(e(direction), velocity);
  // internal fn pow(x, y) requires x cannot be negative
  return rho * w(direction) * (1.0 + 3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr);
}

@compute @workgroup_size(64, 4)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let uv = vec2<i32>(global_invocation_id.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
      return;
    }
    var field_index : i32 = fieldIndex(uv);
    var info: LatticeInfo = lattice_info[field_index];
    if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
      textureStore(macro_info, vec2<i32>(uv), vec4<f32>(0.0, 0.0, 0.0, 0.0));
      return;
    }
    
    var f_i: array<f32, 9>;
    f_i[0] = aa_cell.data[field_index];
    for (var i: i32 = 1; i < 9; i = i + 1) {
      f_i[i] = aa_cell.data[field_index + params[i].read_offset ];
    }
    var rho: f32 = f_i[0] + f_i[1] + f_i[2] + f_i[3] + f_i[4] + f_i[5] + f_i[6] + f_i[7] + f_i[8];
    rho = clamp(rho, 0.8, 1.2);

    var velocity : vec2<f32> = vec2<f32>(0.0);
    // external forcing
    var F : array<f32, 9> = array<f32, 9>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    if (isAccelerateCell(info.material)) {
      if (info.block_iter > 0) {
        info.block_iter = info.block_iter - 1;
        if (info.block_iter == 0) {
          info.material = 1;
        }
        lattice_info[field_index] = info;
      }

      // A-A pattern external force need to inverse
      let force = vec2<f32>(info.vx, info.vy) * vec2<f32>(-1.0);
      // velocity.x = velocity.x + force_x * 0.5 / rho;
      velocity = force * 0.5 / rho;

      for (var i : i32 = 1; i < 9; i = i + 1) {
        F[i] = w(i) * 3.0 * dot(e(i), force);
      }
    } else {
      velocity = e(1) * f_i[1] + e(2) * f_i[2] + e(3) * f_i[3] + e(4) * f_i[4] + e(5) * f_i[5] + e(6) * f_i[6] + e(7) * f_i[7] + e(8) * f_i[8];
      velocity = velocity / rho;
      // Avoid numerical simulation errors
      velocity = clamp(velocity, vec2<f32>(-0.26), vec2<f32>(0.26));
    }
    // A-A pattern macro velocity need to inverse
    textureStore(macro_info, vec2<i32>(uv), vec4<f32>(velocity * vec2<f32>(-1.0), rho, 1.0));

    let usqr = 1.5 * dot(velocity, velocity);
    aa_cell.data[field_index] = f_i[0] - fluid.omega * (f_i[0] - rho * w(0) * (1.0 - usqr));
    var eq: EqResults = equilibrium(velocity, rho, 1, usqr);
    aa_cell.data[field_index + params[1].write_offset ] = f_i[1] - fluid.omega * (f_i[1] - eq.val0) + F[1];
    aa_cell.data[field_index + params[3].write_offset ] = f_i[3] - fluid.omega * (f_i[3] - eq.val1) + F[3];
    eq = equilibrium(velocity, rho, 2, usqr);
    aa_cell.data[field_index + params[2].write_offset ] = f_i[2] - fluid.omega * (f_i[2] - eq.val0) + F[2];
    aa_cell.data[field_index + params[4].write_offset ] = f_i[4] - fluid.omega * (f_i[4] - eq.val1) + F[4];
    eq = equilibrium(velocity, rho, 5, usqr);
    aa_cell.data[field_index + params[5].write_offset ] = f_i[5] - fluid.omega * (f_i[5] - eq.val0) + F[5];
    aa_cell.data[field_index + params[7].write_offset ] = f_i[7] - fluid.omega * (f_i[7] - eq.val1) + F[7];
    eq = equilibrium(velocity, rho, 6, usqr);
    aa_cell.data[field_index + params[6].write_offset ] = f_i[6] - fluid.omega * (f_i[6] - eq.val0) + F[6];
    aa_cell.data[field_index + params[8].write_offset ] = f_i[8] - fluid.omega * (f_i[8] - eq.val1) + F[8];
}
