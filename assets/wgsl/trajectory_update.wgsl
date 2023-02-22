#include "struct/field.wgsl"
#include "struct/particle.wgsl"
#include "struct/pixel.wgsl"

@group(0) @binding(0) var<uniform> field: FieldUniform;
@group(0) @binding(1) var<uniform> particle_uniform: ParticleUniform;
@group(0) @binding(2) var<storage, read_write> field_buf: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> particle_buf: array<TrajectoryParticle>;
@group(0) @binding(4) var<storage, read_write> canvas: array<Pixel>;

fn src_2f(u: i32, v: i32) -> vec2<f32> {
  let new_u = clamp(u, 0, field.lattice_size.x - 1);
  let new_v = clamp(v, 0, field.lattice_size.y - 1);
  let index = new_v * field.lattice_size.x + new_u;

  return field_buf[index].xy;
}
#include "func/bilinear_interpolate_2f.wgsl"

fn field_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * field.lattice_size.x);
}

fn particle_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * particle_uniform.num.x);
}

fn update_canvas(particle: TrajectoryParticle, velocity: vec2<f32>) {
    // 计算出粒子所在的像素坐标
    let pixel_coords = vec2<i32>(particle.pos);
    let px = pixel_coords.x - particle_uniform.point_size / 2;
    let py = pixel_coords.y - particle_uniform.point_size / 2;
    // 根据粒子大小填充到画布
    let info = Pixel(particle.fade, velocity.x, velocity.y);
    for (var x: i32 = 0; x < particle_uniform.point_size; x = x + 1) {
        for (var y: i32 = 0; y < particle_uniform.point_size; y = y + 1) {
            let coords = vec2<i32>(px + x, py + y);
            if (coords.x >= 0 && coords.x < field.canvas_size.x 
                && coords.y >= 0 && coords.y < field.canvas_size.y) {
                canvas[coords.x + field.canvas_size.x * coords.y] = info;
            }
        }
    }
}

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let uv = vec2<i32>(gid.xy);
  if (uv.x >= particle_uniform.num.x || uv.y >= particle_uniform.num.y) {
    return;
  }
  let p_index: i32 = particle_index(uv);
  var particle: TrajectoryParticle = particle_buf[p_index];
  if (particle.life_time <= 0.1) {
    particle.fade = 0.0;
    particle.pos = particle.pos_initial;
    particle.life_time = particle_uniform.life_time;
  } else {
    particle.life_time = particle.life_time - 1.0;
    // 淡入效果
    if (particle.fade < 0.9) {
      particle.fade = particle.fade + 0.1;
    } else {
      particle.fade = 1.0;
    }

    // 计算出粒子所在的矢量场格子索引
    let ij = (particle.pos / field.lattice_pixel_size) - 0.5;
    let velocity = bilinear_interpolate_2f(ij);
    particle.pos += (velocity * particle_uniform.speed_factor);
    
    update_canvas(particle, velocity);
  }
   
  particle_buf[p_index] = particle;
}