#include "lbm/struct/lbm_uniform.wgsl"
#include "lbm/struct/lattice_info.wgsl"
#include "struct/field.wgsl"


struct StoreFloat {
    data: array<f32>,
};

@group(0) @binding(0) var<uniform> fluid: LbmUniform;
@group(0) @binding(1) var<uniform> field: FieldUniform;
@group(0) @binding(2) var<storage, read> collide_cell: StoreFloat;
@group(0) @binding(3) var<storage, read_write> stream_cell: StoreFloat;
@group(0) @binding(4) var<storage, read_write> lattice_info: array<LatticeInfo>;
@group(0) @binding(5) var macro_info: texture_storage_2d<rgba16float, write>;

#include "lbm/d2q9_fn.wgsl"

// push scheme
fn streaming_out(uv: vec2<i32>, direction: i32) -> i32 {
  // https://pdfs.semanticscholar.org/e626/ca323a9a8a4ad82fb16ccbbbd93ba5aa98e0.pdf
  // along current direction streaming to neighbour lattice same direction
    var target_uv : vec2<i32> = uv + vec2<i32>(e(direction));
    if (target_uv.x < 0) {
      target_uv.x = field.lattice_size.x - 1;
    } else if (target_uv.x >= field.lattice_size.x) {
      target_uv.x = 0;
    }
    if (target_uv.y < 0) {
      target_uv.y = field.lattice_size.y - 1;
    } else if (target_uv.y >= field.lattice_size.y) {
      target_uv.y = 0;
    }
    return latticeIndex(target_uv, direction);
}

// pull scheme
fn streaming_in(uv: vec2<i32>, direction: i32) -> i32 {
    var target_uv : vec2<i32> = uv + vec2<i32>(e(fluid.inversed_direction[direction].x));  
    if (target_uv.x < 0) {
      target_uv.x = field.lattice_size.x - 1;
    } else if (target_uv.x >= field.lattice_size.x) {
      target_uv.x = 0;
    }
    if (target_uv.y < 0) {
      target_uv.y = field.lattice_size.y - 1;
    } else if (target_uv.y >= field.lattice_size.y) {
      target_uv.y = 0;
    } 
    return latticeIndex(target_uv, direction);
}