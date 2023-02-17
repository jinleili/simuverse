#include "aa_lbm/aa_layout_and_fn.wgsl"

@compute @workgroup_size(64, 4)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let uv = vec2<i32>(global_invocation_id.xy);
  if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
    return;
  }
  let field_index = fieldIndex(uv);
  
  var info: LatticeInfo = lattice_info[field_index];
  if (isObstacleCell(info.material)) {
    for (var i : i32 = 0; i < 9; i = i + 1) {
        // lattice coords that will bounce back to
        // A-A pattern not need warry about boundray cell
        aa_cell.data[field_index + soaOffset(i)] =  0.0;
    }
  } else if (isPoiseuilleFlow()) {
    for (var i: i32 = 0; i < 9; i = i + 1) {
      aa_cell.data[field_index + soaOffset(i)] =  w(i);
    }
    let temp = w(3) * 0.3;
    aa_cell.data[field_index + soaOffset(1)] = w(1) + temp;
    aa_cell.data[field_index + soaOffset(3)] = temp;
  } else {
    for (var i: i32 = 0; i < 9; i = i + 1) {
      aa_cell.data[field_index + soaOffset(i)] =  w(i);
    }
  }

  if (isAccelerateCell(info.material)) {
    if (info.block_iter > 0) {
        info.block_iter = 0;
        info.material = 1;
        info.vx = 0.0;
        info.vy = 0.0;
        lattice_info[field_index] = info;
      }
  }

  // macro_info.data[field_index] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
  textureStore(macro_info, vec2<i32>(uv), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}