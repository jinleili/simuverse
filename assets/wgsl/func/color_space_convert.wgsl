// https://github.com/hughsk/glsl-hsv2rgb
fn hsv2rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(vec3<f32>(h, h, h) + K.xyz) * 6.0 - vec3<f32>(K.w, K.w, K.w));
    let kx = vec3<f32>(K.x, K.x, K.x);
    let c = clamp(p - kx, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0));
    return v * mix(kx, c, vec3<f32>(s, s, s));
}
