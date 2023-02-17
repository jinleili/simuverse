#include "lbm/struct/lbm_uniform.wgsl"
#include "lbm/struct/lattice_info.wgsl"
#include "struct/field.wgsl"


struct StoreFloat {
    data: array<f32>,
};

@group(0) @binding(0) var<uniform> fluid: LbmUniform;
@group(0) @binding(1) var<uniform> field: FieldUniform;
@group(0) @binding(2) var<storage, read_write> aa_cell: StoreFloat;
@group(0) @binding(3) var<storage, read_write> lattice_info: array<LatticeInfo>;
@group(0) @binding(4) var macro_info: texture_storage_2d<rgba16float, write>;


fn isPoiseuilleFlow() -> bool { return fluid.fluid_ty == 0; }

// direction's coordinate
fn e(direction: i32) -> vec2<f32> { return fluid.e_w_max[direction].xy; }
// direction's weight
fn w(direction: i32) -> f32 { return fluid.e_w_max[direction].z; }
fn max_value(direction: i32) -> f32 { return fluid.e_w_max[direction].w; }

fn fieldIndex(uv: vec2<i32>) -> i32 { return uv.x + (uv.y * field.lattice_size.x); }
fn soaOffset(direction: i32) -> i32 { return direction * fluid.soa_offset; }
fn latticeIndex(uv: vec2<i32>, direction: i32) -> i32 {
  return fieldIndex(uv) + soaOffset(direction);
}

fn isBoundaryCell(material: i32) -> bool { return material == 2; }
fn isNotBoundaryCell(material: i32) -> bool { return material != 2; }
fn isNotNeedCollide(material: i32) -> bool { return material == 2 || material == 4 || material == 7; }
fn isInletCell(material: i32) -> bool { return material == 3; }
fn isObstacleCell(material: i32) -> bool { return material == 4; }
fn isOutletCell(material: i32) -> bool { return material == 5; }
fn isAccelerateCell(material: i32) -> bool { return material == 3 || material == 6; }

fn isBulkFluidCell(material: i32) -> bool { return material == 1 || material == 3 || material == 5; }

// push scheme
fn streaming_out(uv : vec2<i32>, direction : i32)->i32 {
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
