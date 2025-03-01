use crate::FieldAnimationType;
use alloc::string::String;

pub fn get_velocity_code_snippet(ty: FieldAnimationType) -> String {
    match ty {
        FieldAnimationType::Basic => r#"
    let new_y = f32(p.y) - f32(field.lattice_size.y) / 2.0;
    // 将像素位移速度转换到 NDC 坐标空间
    let v = vec2<f32>(4.0, -8.0) * field.ndc_pixel;
    return v * new_y;
    "#
        .into(),
        FieldAnimationType::JuliaSet => r#"
    // 将场坐标转换到 [-1.5, 1.5] 坐标范围
    var c = vec2<f32>(p) / vec2<f32>(field.lattice_size);
    c = c * 3.0 - vec2<f32>(1.5);
    c *= field.proj_ratio;
    let z = vec2<f32>(0.4, 0.5);
    for (var i: i32 = 0; i < 8; i = i + 1) {
        c = vec2<f32>(c.x * c.x - c.y * c.y, c.y * c.x + c.x * c.y);
        c = c + z;
    }
    return c * 0.6;
    "#
        .into(),
        FieldAnimationType::BlackHole => r#"
    // 将场坐标转换到 [-3.5, 3.5] 坐标范围
    var c = vec2<f32>(p) / vec2<f32>(field.lattice_size);
    c = c * 7.0 - vec2<f32>(3.5);
    c *= field.proj_ratio;
    let r = dot(c, c);
    let v = vec2<f32>(c.y, -c.x);
    return v / r - c * 0.2;
    "#
        .into(),
        FieldAnimationType::Spirl => r#"
    // 将场坐标转换到 [-25, 25] 坐标范围
    var c = vec2<f32>(p) / vec2<f32>(field.lattice_size);
    c = c * 50.0 - vec2<f32>(25.0);
    c *= field.proj_ratio;
    let r = length(c);
    let theta = atan2(c.y, c.x);
    var v = vec2<f32>(c.y, -c.x) / r;
    let t = sqrt(r * 15.0) + theta;
    v *= sin(t) * length(v) * 50.0;
    return (v + c) * field.ndc_pixel * 15.0;
    "#
        .into(),
        _ => "".into(),
    }
}
