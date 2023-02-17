
// sound speed
const Cs2: f32 = 0.333333;

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
fn isInletCell(material: i32) -> bool { return material == 3; }
fn isObstacleCell(material: i32) -> bool { return material == 4; }
fn isOutletCell(material: i32) -> bool { return material == 5; }
fn isAccelerateCell(material: i32) -> bool { return material == 3 || material == 6; }

fn isBulkFluidCell(material: i32) -> bool { return material == 1 || material == 3 || material == 5; }
