use app_surface::AppSurface;
use egui::emath::{Pos2, Rect};
use winit::event_loop::EventLoop;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Enum {
    First,
    Second,
    Third,
}

pub struct ControlPanel {
    egui_ctx: egui::Context,
    state: egui_winit::State,
    panel_frame: egui::Frame,
    egui_renderer: egui_wgpu::Renderer,
    pub particle_num: i32,
    pub particle_size: u32,
    particle_color: Enum,
    pub lifetime: i32,
}

impl ControlPanel {
    pub fn new(app: &AppSurface, format: wgpu::TextureFormat, event_loop: &EventLoop<()>) -> Self {
        let egui_ctx = egui::Context::default();
        setup_custom_fonts(&egui_ctx);

        let mut state = egui_winit::State::new(event_loop);
        state.set_pixels_per_point(app.scale_factor);

        // let mut raw_input: egui::RawInput = egui::RawInput::default();
        // raw_input.screen_rect = Some(Rect {
        //     min: Pos2 { x: 20., y: 20. },
        //     max: Pos2 { x: 400., y: 400. },
        // });

        let panel_frame = egui::Frame {
            fill: egui_ctx.style().visuals.window_fill(),
            rounding: 10.0.into(),
            stroke: egui_ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            inner_margin: 12.0.into(),
            ..Default::default()
        };

        let egui_renderer = egui_wgpu::Renderer::new(&app.device, format, None, 1);
        let particle_size = app.scale_factor.ceil() as u32 * 2;
        Self {
            egui_ctx,
            state,
            panel_frame,
            egui_renderer,
            particle_num: 20000,
            particle_size,
            particle_color: Enum::First,
            lifetime: 120,
        }
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        self.state.on_event(&self.egui_ctx, event);
    }

    pub fn begin_pass<'b, 'a: 'b>(
        &'a mut self,
        app: &AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
    ) -> (Vec<wgpu::CommandBuffer>, egui::TexturesDelta) {
        let full_output = self.egui_ctx.run(self.state.egui_input().clone(), |ctx| {
            egui::CentralPanel::default()
                .frame(self.panel_frame)
                .show(&ctx, |ui| {
                    ui.heading("参数调整");
                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([40.0, 12.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("粒子数：");
                            ui.add(egui::Slider::new(&mut self.particle_num, 0..=100000));
                            ui.end_row();

                            ui.label("粒子大小：");
                            ui.add(egui::Slider::new(&mut self.particle_size, 1..=8).text("像素"));
                            ui.end_row();

                            ui.label("粒子生存时长：");
                            ui.add(egui::Slider::new(&mut self.lifetime, 15..=240).text("帧"));
                            ui.end_row();

                            ui.label("着色方案：");
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", &self.particle_color))
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);
                                    ui.selectable_value(
                                        &mut self.particle_color,
                                        Enum::First,
                                        "运动方向",
                                    );
                                    ui.selectable_value(
                                        &mut self.particle_color,
                                        Enum::Second,
                                        "Second",
                                    );
                                    ui.selectable_value(
                                        &mut self.particle_color,
                                        Enum::Third,
                                        "Third",
                                    );
                                });
                            ui.end_row();
                        });
                });
        });

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
