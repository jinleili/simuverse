use app_surface::math::Size;

pub fn fullscreen_mvp(viewport_size: Size<f32>) -> [[f32; 4]; 4] {
    let (p_matrix, mv_matrix) = perspective_fullscreen_mvp(viewport_size);
    (p_matrix * mv_matrix).to_cols_array_2d()
}

// 将[-1, 1]的矩形空间映射到刚好填充整个视口
pub fn perspective_fullscreen_mvp(viewport_size: Size<f32>) -> (glam::Mat4, glam::Mat4) {
    let fovy: f32 = 75.0 / 180.0 * std::f32::consts::PI;
    let p_matrix =
        glam::Mat4::perspective_rh(fovy, viewport_size.width / viewport_size.height, 0.1, 100.0);

    let factor = fullscreen_factor(viewport_size);
    let vm_matrix = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, factor.0));
    let scale_matrix = glam::Mat4::from_scale(glam::Vec3::new(factor.1, factor.2, 1.0));

    (p_matrix, vm_matrix * scale_matrix)
}

// 外部调用者使用返回的 fullscreen_factor 参数缩放顶点坐标可实现填充整个视口
pub fn perspective_mvp(viewport_size: Size<f32>) -> (glam::Mat4, glam::Mat4, (f32, f32, f32)) {
    let fovy: f32 = 75.0 / 180.0 * std::f32::consts::PI;
    let p_matrix =
        glam::Mat4::perspective_rh(fovy, viewport_size.width / viewport_size.height, 0.1, 100.0);

    let factor = fullscreen_factor(viewport_size);
    let vm_matrix = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, factor.0));

    (p_matrix, vm_matrix, (factor.1, factor.2, factor.0))
}

pub fn fullscreen_factor(viewport_size: Size<f32>) -> (f32, f32, f32) {
    // 缩放到贴合屏幕
    //
    // 移动近裁剪平面,屏幕上的投影并不会缩放,
    // 因为虽然模型对象在裁剪平面上看起来投影随之缩放,但裁剪平面本身也在随之缩放
    // 相当于是 裁剪平面与其上的投影在整体缩放, 而裁剪平面始终是等于屏幕空间平面的, 所以映射到屏幕上就是没有缩放
    // 满屏效果: 默认 camera 在原点，利用 fovy 计算 tan (近裁剪平面 x | y 与 camera 原点的距离之比) 得出 z 轴平移距离
    // 屏幕 h > w 时，才需要计算 ratio, w > h 时， ration = 1
    let ratio = if viewport_size.height > viewport_size.width {
        viewport_size.height / viewport_size.width
    } else {
        1.0
    };
    let fovy: f32 = 75.0 / 180.0 * std::f32::consts::PI;
    let factor: f32 = (fovy / 2.0).tan();

    let mut sx = 1.0;
    let mut sy = 1.0;
    if viewport_size.height > viewport_size.width {
        sy = viewport_size.height / viewport_size.width;
    } else {
        sx = viewport_size.width / viewport_size.height;
    };
    let translate_z = -(ratio / factor);
    (translate_z, sx, sy)
}

#[allow(dead_code)]
pub fn ortho_mvp(viewport_size: Size<f32>) -> [[f32; 4]; 4] {
    let factor = fullscreen_factor(viewport_size);
    let p_matrix = glam::Mat4::orthographic_rh(
        -1.0 * factor.1,
        1.0 * factor.1,
        -1.0 * factor.2,
        1.0 * factor.2,
        -100.0,
        100.0,
    );
    (p_matrix * glam::Mat4::IDENTITY).to_cols_array_2d()
}
