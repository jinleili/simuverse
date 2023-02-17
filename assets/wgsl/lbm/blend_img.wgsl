#include "bufferless.vs.wgsl"

@group(0) @binding(0) var macro_info: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

#include "func/color_space_convert.wgsl"

let PI: f32 = 3.1415926;
let PI_2: f32 = 1.570796;

@fragment
fn fs_main(in : VertexOutput) -> @location(0) vec4<f32> {
  let macro_data: vec4<f32> = textureSample(macro_info, tex_sampler, in.uv);
  // 角度
  let angle = (atan2(macro_data.x, macro_data.y) + PI) / (2.0 * PI);
  return vec4<f32>(hsv2rgb(angle, 0.75, 1.0), 1.0);
  // 速度
  // let velocity = abs(macro.x) + abs(macro.y);
  // let speed = clamp(velocity / 0.025, 0.1, 1.2);
  // return vec4<f32>(hsv2rgb(0.05 + speed * 0.75, 0.9, 1.0), macro.z);
  // 密度
  // let rho = (macro.z - 0.98) * 50.0;
  // return vec4<f32>(hsv2rgb(0.05 + rho * 0.75, 0.75, 1.0), 1.0);

  // return vec4<f32>(macro.rgb, macro.b);

}
