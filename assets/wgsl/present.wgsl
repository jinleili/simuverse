#include "bufferless.vs.wgsl"

#include "struct/field.wgsl"
#include "struct/particle.wgsl"
#include "struct/pixel.wgsl"

@group(0) @binding(0) var<uniform> field: FieldUniform;
@group(0) @binding(1) var<uniform> particle_uniform: ParticleUniform;
@group(0) @binding(2) var<storage, read_write> canvas: array<Pixel>;

#include "func/color_space_convert.wgsl"

const PI: f32 = 3.1415926535;

@fragment 
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coord = min(vec2<i32>(floor(in.position.xy)), field.canvas_size - 1);
    let p_index = pixel_coord.x + pixel_coord.y * field.canvas_size.x;
    var p: Pixel = canvas[p_index];

    var frag_color: vec4<f32>;
    if (p.alpha > 0.001) {
        if (particle_uniform.color_ty == 1) {
            // 将速率映射为 rgb
            // let velocity = abs(p.velocity_x) + abs(p.velocity_y);
            let velocity = length(vec2<f32>(p.velocity_x, p.velocity_y));
            var speed: f32;
            if (field.speed_ty == 0) {
                // speed =  velocity / max((f32(field.canvas_size.x) / particle_uniform.speed_factor), (f32(field.canvas_size.y) / particle_uniform.speed_factor));
                speed = velocity;
            } else {
                speed =  min(velocity / 0.25, 1.15);
            }
            frag_color = vec4<f32>(hsv2rgb(0.05 + speed * 0.75, 0.9, 1.0), p.alpha);
        } else if (particle_uniform.color_ty == 0) {
            // 将运动方向映射为 rgb
            let angle = atan2(p.velocity_y, p.velocity_x) / (2.0 * PI);
            frag_color = vec4<f32>(hsv2rgb(angle, 0.9, 1.0), p.alpha);
        } else {
            frag_color = vec4<f32>(particle_uniform.color.rgb, p.alpha);
        }

        // 淡出轨迹
        if (p.alpha >= 0.2) {
            p.alpha = p.alpha * particle_uniform.fade_out_factor;
        } else {
            p.alpha = p.alpha * 0.5;
        }
        canvas[p_index] = p;
    } else {
        frag_color = vec4<f32>(0.0);
    }
    return frag_color;
}

