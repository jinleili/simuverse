use app_surface::AppSurface;
use egui::emath::{Pos2, Rect};
use winit::event_loop::EventLoop;

pub struct ControlPanel {
    egui_ctx: egui::Context,
    state: egui_winit::State,
    panel_frame: egui::Frame,
    pos_rect: Rect,
    egui_renderer: egui_wgpu::Renderer,
    pub particles_count: i32,
    pub particle_size: i32,
    pub particle_color: u32,
    pub lifetime: i32,
    pub wgsl_code: String,
    code_index: u32,
}

impl ControlPanel {
    pub fn new(app: &AppSurface, format: wgpu::TextureFormat, event_loop: &EventLoop<()>) -> Self {
        let egui_ctx = egui::Context::default();
        setup_custom_fonts(&egui_ctx);

        let mut state = egui_winit::State::new(event_loop);
        state.set_pixels_per_point(app.scale_factor);

        let margin = 8.0;
        let panel_width = 360.0;
        // let x = app.config.width as f32 / app.scale_factor - panel_width - margin;
        let x = margin;
        let pos_rect = Rect {
            min: Pos2 { x, y: margin },
            max: Pos2 {
                x: panel_width + x,
                y: app.config.height as f32 / app.scale_factor - margin,
            },
        };

        let mut bg = egui_ctx.style().visuals.window_fill();
        bg = egui::Color32::from_rgba_premultiplied(bg.r(), bg.g(), bg.b(), 220);
        let panel_frame = egui::Frame {
            fill: bg,
            rounding: 10.0.into(),
            stroke: egui_ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            inner_margin: 12.0.into(),
            ..Default::default()
        };

        let egui_renderer = egui_wgpu::Renderer::new(&app.device, format, None, 1);
        let particle_size = app.scale_factor.ceil() as i32 * 2;
        Self {
            egui_ctx,
            state,
            panel_frame,
            pos_rect,
            egui_renderer,
            particles_count: 20000,
            particle_size,
            particle_color: 0,
            lifetime: 120,
            wgsl_code: "".into(),
            code_index: 0,
        }
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        let _ = self.state.on_event(&self.egui_ctx, event);
    }

    pub fn begin_pass<'b, 'a: 'b>(
        &'a mut self,
        app: &AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
    ) -> (Vec<wgpu::CommandBuffer>, egui::TexturesDelta) {
        let full_output = self.run_egui(app);

        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes); // create triangles to paint
        let textures_delta = full_output.textures_delta;
        // Upload all resources for the GPU.
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [app.config.width, app.config.height],
            pixels_per_point: app.scale_factor,
        };

        let user_cmd_bufs = {
            for (id, image_delta) in &textures_delta.set {
                self.egui_renderer
                    .update_texture(&app.device, &app.queue, *id, image_delta);
            }
            let mut encoder = app
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui Encoder"),
                });
            self.egui_renderer.update_buffers(
                &app.device,
                &app.queue,
                &mut encoder,
                &clipped_primitives,
                &screen_descriptor,
            )
        };
        {
            self.egui_renderer
                .render(rpass, &clipped_primitives, &screen_descriptor);
        }

        (user_cmd_bufs, textures_delta)
    }

    pub fn end_pass(&mut self, textures_delta: &egui::TexturesDelta) {
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
    }

    fn run_egui(&mut self, app: &app_surface::AppSurface) -> egui::FullOutput {
        let mut raw_input = self.state.take_egui_input(&app.view);
        raw_input.screen_rect = Some(self.pos_rect);

        self.wgsl_code = simuverse::get_velocity_code_segment(
            simuverse::FieldAnimationType::from_u32(self.code_index),
        )
        .into();

        self.egui_ctx.run(raw_input, |ctx| {
            let window = egui::Window::new("参数调整")
                .id(egui::Id::new("particles_window_options")) // required since we change the title
                .resizable(false)
                .collapsible(true)
                .title_bar(true)
                .scroll2([true; 2])
                .enabled(true);

            window.show(ctx, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([40.0, 12.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("粒子数：");
                        ui.add(egui::Slider::new(&mut self.particles_count, 0..=40000));
                        ui.end_row();

                        ui.label("粒子大小：");
                        ui.add(egui::Slider::new(&mut self.particle_size, 1..=8).text("像素"));
                        ui.end_row();

                        ui.label("粒子生存时长：");
                        ui.add(egui::Slider::new(&mut self.lifetime, 15..=240).text("帧"));
                        ui.end_row();

                        ui.label("着色方案：");
                        egui::ComboBox::from_label("")
                            .selected_text(get_color_ty_name(self.particle_color))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                ui.selectable_value(
                                    &mut self.particle_color,
                                    0,
                                    get_color_ty_name(0),
                                );
                                ui.selectable_value(
                                    &mut self.particle_color,
                                    1,
                                    get_color_ty_name(1),
                                );
                                ui.selectable_value(
                                    &mut self.particle_color,
                                    2,
                                    get_color_ty_name(2),
                                );
                            });
                        ui.end_row();
                    });
                let mut theme =
                    simuverse::egui_lib::syntax_highlighting::CodeTheme::from_memory(ui.ctx());

                let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                    let mut layout_job = simuverse::egui_lib::syntax_highlighting::highlight(
                        ui.ctx(),
                        &theme,
                        string,
                        "rs".into(),
                    );
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.wgsl_code)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );
                });
            });
        })
    }
}

// const MONACO: &'static str = "monaco";
const ZH_TINY: &'static str = "zh";

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // fonts.font_data.insert(
    //     MONACO.to_owned(),
    //     egui::FontData::from_static(include_bytes!("../assets/Monaco.ttf")),
    // );
    fonts.font_data.insert(
        ZH_TINY.to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/PingFangTiny.ttf")),
    );
    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, ZH_TINY.to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(ZH_TINY.to_owned());

    ctx.set_fonts(fonts);
}

fn get_color_ty_name(index: u32) -> &'static str {
    match index {
        0 => "运动方向",
        1 => "运动速率",
        _ => "白色",
    }
}
