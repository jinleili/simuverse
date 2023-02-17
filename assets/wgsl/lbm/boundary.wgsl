#include "lbm/layout_and_fn.wgsl"

@compute @workgroup_size(64, 4)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let uv = vec2<i32>(global_invocation_id.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
      return;
    }
    var field_index : i32 = fieldIndex(uv);
    let info: LatticeInfo = lattice_info[field_index];
    if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
        
        // find lattice that direction quantities flowed in
        // push scheme: bounce back the direction quantities to that lattice
        // pull scheme: copy lattice reversed direction quantities to boundary cell
        for (var i : i32 = 0; i < 9; i = i + 1) {
            // lattice coords that will bounce back to
            let new_uv : vec2<i32> = uv - vec2<i32>(e(i));
            if (new_uv.x <= 0 || new_uv.y <= 0 || new_uv.x >= (field.lattice_size.x - 1) || new_uv.y >= (field.lattice_size.y - 1)) {
                continue;
            } else {
                // push scheme:
                // let val = stream_cell.data[field_index + soaOffset(i)];
                // let new_index = latticeIndex(new_uv, inversed_direction[i]);
                // stream_cell.data[new_index] = val;

                // pull scheme:
                let val = stream_cell.data[latticeIndex(new_uv, i)];
                let lattice_index = field_index + soaOffset(fluid.inversed_direction[i].x);
                stream_cell.data[lattice_index] = val;
                stream_cell.data[latticeIndex(new_uv, i)] = 0.0;
            }
        }
    }
}