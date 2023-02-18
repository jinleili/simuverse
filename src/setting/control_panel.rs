use crate::{FieldAnimationType, ParticleColorType, SettingObj, SimuType};
use app_surface::AppSurface;
use egui::{
    emath::{Pos2, Rect},
    Context,
};

pub struct ControlPanel {
    pub setting: SettingObj,
    panel_frame: egui::Frame,
    pos_rect: Rect,
    window_size: egui::emath::Vec2,
    pub particles_count: i32,
    pub particle_size: i32,
    pub particle_color: u32,
    pub lifetime: i32,
    pub wgsl_code: String,
    last_selected_code_snippet: i32,
    selected_code_snippet: Option<i32>,
    is_code_snippet_changed: bool,
    selected_simu_type: SimuType,
}

impl ControlPanel {
    pub fn new(app: &AppSurface, egui_ctx: &Context) -> Self {
        let lifetime = 90;
        let particles_count = 10000;
        let particle_size = if app.scale_factor.ceil() > 1.0 { 3 } else { 2 };
        let selected_simu_type = SimuType::Field;

        let mut setting = SettingObj::new(
            selected_simu_type,
            match selected_simu_type {
                SimuType::Fluid => FieldAnimationType::Poiseuille,
                _ => FieldAnimationType::Basic,
            },
            ParticleColorType::MovementAngle,
            particles_count,
            lifetime as f32,
            particle_size,
        );
        setting.update_canvas_size(&app, (&app.config).into());

        let margin = 8.0;
        let panel_width = 360.0;
        let panel_height = app.config.height as f32 / app.scale_factor - margin * 2.0;
        // let x = app.config.width as f32 / app.scale_factor - panel_width - margin;
        let x = margin;
        let pos_rect = Rect {
            min: Pos2 { x, y: margin },
            max: Pos2 {
                x: panel_width + x,
                y: panel_height + margin,
            },
        };

        // 实测出来的数值，避免圆角被裁剪
        let window_size: egui::emath::Vec2 = [panel_width - 26.0, panel_height - 12.].into();

        let mut bg = egui_ctx.style().visuals.window_fill();
        bg = egui::Color32::from_rgba_premultiplied(bg.r(), bg.g(), bg.b(), 210);
        let panel_frame = egui::Frame {
            fill: bg,
            rounding: 10.0.into(),
            stroke: egui_ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            inner_margin: 12.0.into(),
            ..Default::default()
        };

        Self {
            setting,
            panel_frame,
            pos_rect,
            window_size,
            particles_count,
            particle_size,
            particle_color: 0,
            lifetime,
            wgsl_code: crate::get_velocity_code_snippet(crate::FieldAnimationType::from_u32(0)),
            last_selected_code_snippet: 0,
            selected_code_snippet: Some(0),
            is_code_snippet_changed: false,
            selected_simu_type,
        }
    }

    pub fn is_code_snippet_changed(&mut self) -> bool {
        let is_changed = self.is_code_snippet_changed.clone();
        self.is_code_snippet_changed = false;
        is_changed
    }

    pub fn update_setting(&mut self, app: &AppSurface) -> (Option<(u32, u32, u32)>, bool) {
        let mut workgroup_count_changed = None;
        if self.particle_color != self.setting.color_ty as u32 {
            let color_ty = ParticleColorType::from_u32(self.particle_color);
            self.setting.update_particle_color(app, color_ty);
        }
        if self
            .setting
            .update_particles_count(app, self.particles_count)
        {
            // 更新了粒子数后，还须更新 workgroup count
            workgroup_count_changed = Some(self.setting.particles_workgroup_count);
        }
        self.setting
            .update_particle_point_size(&app, self.particle_size);
        self.setting
            .update_particle_life(&app, self.lifetime as f32);

        let mut simu_ty_changed = false;
        if self.selected_simu_type != self.setting.simu_type {
            let mut setting = SettingObj::new(
                self.selected_simu_type,
                match self.selected_simu_type {
                    SimuType::Fluid => FieldAnimationType::Poiseuille,
                    _ => FieldAnimationType::Basic,
                },
                self.setting.color_ty,
                self.setting.particles_count,
                self.lifetime as f32,
                self.particle_size,
            );
            setting.update_canvas_size(&app, (&app.config).into());
            self.setting = setting;

            simu_ty_changed = true;
        }

        (workgroup_count_changed, simu_ty_changed)
    }

    pub fn ui_contents(&mut self, ctx: &Context) {
        match self.selected_code_snippet {
            Some(code_index) if code_index != self.last_selected_code_snippet => {
                self.last_selected_code_snippet = code_index;
                self.wgsl_code = crate::get_velocity_code_snippet(
                    crate::FieldAnimationType::from_u32(code_index as u32),
                )
                .into();
                self.is_code_snippet_changed = true;
            }
            _ => {}
        }

        self.top_bar_ui(ctx);

        let window = egui::Window::new("参数设置")
            .id(egui::Id::new("particles_window_options")) // required since we change the title
            .resizable(false)
            .collapsible(true)
            .title_bar(true)
            .scroll2([false, true])
            .movable(false)
            .fixed_size(self.window_size)
            .frame(self.panel_frame)
            .enabled(true);

        window.show(ctx, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 12.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("粒子数：");
                    ui.add(egui::Slider::new(&mut self.particles_count, 2000..=40000));
                    ui.end_row();

                    ui.label("粒子大小：");
                    ui.add(egui::Slider::new(&mut self.particle_size, 1..=8).text("像素"));
                    ui.end_row();

                    ui.label("粒子生存时长：");
                    ui.add(egui::Slider::new(&mut self.lifetime, 40..=240).text("帧"));
                    ui.end_row();

                    ui.label("着色方案：");
                    egui::ComboBox::from_label("")
                        .selected_text(get_color_ty_name(self.particle_color))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            ui.selectable_value(&mut self.particle_color, 0, get_color_ty_name(0));
                            ui.selectable_value(&mut self.particle_color, 1, get_color_ty_name(1));
                            ui.selectable_value(&mut self.particle_color, 2, get_color_ty_name(2));
                        });
                    ui.end_row();
                });
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("速度矢量场计算  ");
                ui.add_enabled(true, egui::Label::new("目前还不支持实时编辑"));
            });

            ui.horizontal(|ui| {
                ui.label("预设实现：");
                ui.selectable_value(&mut self.selected_code_snippet, Some(0), "简单");
                ui.selectable_value(&mut self.selected_code_snippet, Some(1), "Julia Set");
                ui.selectable_value(&mut self.selected_code_snippet, Some(2), "螺旋");
                ui.selectable_value(&mut self.selected_code_snippet, Some(3), "黑洞");
            });

            let theme = crate::syntax_highlighting::CodeTheme::from_memory(ui.ctx());

            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = crate::syntax_highlighting::highlight(
                    ui.ctx(),
                    &theme,
                    &crate::remove_leading_indentation(string),
                    "rs".into(),
                );
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            crate::syntax_highlighting::code_view_ui(
                ui,
                "fn get_velocity(p: vec2<i32>) -> vec2<f32> {",
            );
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.wgsl_code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(6)
                        .lock_focus(true)
                        .desired_width(500.)
                        .layouter(&mut layouter),
                );
            });
            crate::syntax_highlighting::code_view_ui(ui, "}");

            ui.collapsing("矢量场计算着色器源码", |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    crate::show_code(
                        ui,
                        r#"
struct FieldUniform {
  // 矢量场格子数
  lattice_size: vec2<i32>,
  // 格子所占像素数
  lattice_pixel_size: vec2<f32>,
  // 画布物理像素数
  canvas_size: vec2<i32>,
  // 投影屏幕宽高比
  proj_ratio: vec2<f32>,
  // 单个像素在 NDC 空间中的大小
  ndc_pixel: vec2<f32>,
  speed_ty: i32,
};
@group(0) @binding(0) var<uniform> field: FieldUniform;
@group(0) @binding(1) var<storage, read_write> field_buf: array<vec4<f32>>;

fn field_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * field.lattice_size.x);
}

fn get_velocity(p: vec2<i32>) -> vec2<f32> {
    #insert_code_snippet
}

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let uv = vec2<i32>(gid.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
        return;
    }
    let index = field_index(uv);
    field_buf[index] = vec4<f32>(get_velocity(uv), 0.0, 0.0);
}
  
    "#,
                    );
                });
            });
        });
    }

    fn top_bar_ui(&mut self, ctx: &Context) {
        let menu_items = vec![
            ("🌾 矢量场", SimuType::Field),
            ("💦 流体场", SimuType::Fluid),
            ("🔏 隐形墨水", SimuType::Ink),
        ];
        egui::TopBottomPanel::top("simuverse_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                ui.label("🌌 Wgpu Simuverse");
                ui.separator();
                for (name, anchor) in menu_items.into_iter() {
                    if ui
                        .selectable_label(self.selected_simu_type == anchor, name)
                        .clicked()
                    {
                        self.selected_simu_type = anchor;
                    }
                }
            });
        });
    }
}

const ZH_TINY: &'static str = "zh";

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        ZH_TINY.to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/PingFangTiny.ttf")),
    );
    // Some good looking emojis.
    fonts.font_data.insert(
        "NotoEmoji-Regular".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoEmoji-Regular.ttf"))
            .tweak(egui::FontTweak {
                scale: 0.91,            // make it smaller
                y_offset_factor: -0.15, // move it up
                y_offset: 0.0,
            }),
    );

    // Bigger emojis, and more. <http://jslegers.github.io/emoji-icon-font/>:
    fonts.font_data.insert(
        "emoji-icon-font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/emoji-icon-font.ttf"))
            .tweak(egui::FontTweak {
                scale: 0.88,           // make it smaller
                y_offset_factor: 0.07, // move it down slightly
                y_offset: 0.0,
            }),
    );
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec![
            ZH_TINY.to_owned(),
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
        ],
    );

    ctx.set_fonts(fonts);
}

fn get_color_ty_name(index: u32) -> &'static str {
    match index {
        0 => "运动方向",
        1 => "运动速率",
        _ => "白色",
    }
}
