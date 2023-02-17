
struct LbmUniform {
  // τ represents the viscosity of the fluid, given by τ = 0.5 * (1.0 + 6niu )
    tau: f32,
    omega: f32,
    fluid_ty: i32,
    // structure of array (put the same direction of all lattice together ) lattice data offset
    soa_offset: i32,
    // lattice direction + direction weight + max value
    e_w_max: array<vec4<f32>, 9>,
    inversed_direction: array<vec4<i32>, 9>,
};
