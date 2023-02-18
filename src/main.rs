use app_surface::{AppSurface, SurfaceFrame};
use simuverse::framework::{run, Action};
use simuverse::util::{math::Size, BufferObj};
use simuverse::{
    setup_custom_fonts, ControlPanel, FieldAnimationType, FieldPlayer, FluidPlayer,
    ParticleColorType, Player, SettingObj, SimuType,
};
use std::iter;
use winit::{event_loop::EventLoop, window::WindowId};

struct InteractiveApp {
    app: AppSurface,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    ctrl_panel: ControlPanel,
    canvas_size: Size<u32>,
    canvas_buf: BufferObj,
    player: Box<dyn Player>,
}

impl Action for InteractiveApp {
    fn new(app: AppSurface, event_loop: &EventLoop<()>) -> Self {
        let mut app = app;
        let format = app.config.format.remove_srgb_suffix();
        app.sdq.update_config_format(format);

        // egui
        let egui_ctx = egui::Context::default();
        setup_custom_fonts(&egui_ctx);
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_pixels_per_point(app.scale_factor);
        let egui_renderer = egui_wgpu::Renderer::new(&app.device, format, None, 1);
        let ctrl_panel = ControlPanel::new(&app, &egui_ctx);

        let canvas_size: Size<u32> = (&app.config).into();

        let canvas_buf = simuverse::util::BufferObj::create_empty_storage_buffer(
            &app.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );
        let player = Self::create_player(&app, canvas_size, &canvas_buf, &ctrl_panel.setting);

        Self {
            app,
            egui_ctx,
            egui_state,
            egui_renderer,
            ctrl_panel,
            canvas_buf,
            canvas_size,
            player,
        }
    }

    fn get_adapter_info(&self) -> wgpu::AdapterInfo {
        self.app.adapter.get_info()
    }

    fn current_window_id(&self) -> WindowId {
        self.app.view.id()
    }

    fn resize(&mut self) {
        self.app.resize_surface();

        let canvas_size: Size<u32> = (&self.app.config).into();
        self.ctrl_panel
            .setting
            .update_canvas_size(&self.app, canvas_size);
        self.canvas_size = canvas_size;
        self.canvas_buf = simuverse::util::BufferObj::create_empty_storage_buffer(
            &self.app.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );
        self.player = Self::create_player(
            &self.app,
            canvas_size,
            &self.canvas_buf,
            &self.ctrl_panel.setting,
        );
    }

    fn on_ui_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        let _ = self.egui_state.on_event(&self.egui_ctx, event);
    }

    fn request_redraw(&mut self) {
        self.app.view.request_redraw();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // egui ui 更新
        let mut raw_input = self.egui_state.take_egui_input(&self.app.view);
        // raw_input.screen_rect = Some(self.pos_rect);
        let egui_app = &mut self.ctrl_panel;
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            egui_app.ui_contents(ctx);
        });
        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes);
        let textures_delta = full_output.textures_delta;
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.app.config.width, self.app.config.height],
            pixels_per_point: self.app.scale_factor,
        };
        let egui_cmd_bufs = {
            for (id, image_delta) in &textures_delta.set {
                self.egui_renderer.update_texture(
                    &self.app.device,
                    &self.app.queue,
                    *id,
                    image_delta,
                );
            }
            self.egui_renderer.update_buffers(
                &self.app.device,
                &self.app.queue,
                &mut encoder,
                &clipped_primitives,
                &screen_descriptor,
            )
        };

        self.player.compute(&mut encoder);

        let (output, frame_view) = self.app.get_current_frame_view();
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.15,
                            b: 0.17,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
                label: None,
            });
            self.player
                .draw_by_rpass(&self.app, &mut rpass, &mut self.ctrl_panel.setting);

            // egui ui 渲染
            self.egui_renderer
                .render(&mut rpass, &clipped_primitives, &screen_descriptor);
        }

        self.app.queue.submit(
            egui_cmd_bufs
                .into_iter()
                .chain(iter::once(encoder.finish())),
        );
        output.present();

        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.update_setting();

        Ok(())
    }
}

impl InteractiveApp {
    fn create_player(
        app: &AppSurface,
        canvas_size: Size<u32>,
        canvas_buf: &BufferObj,
        setting: &SettingObj,
    ) -> Box<dyn Player> {
        return match setting.simu_type {
            SimuType::Fluid => Box::new(FluidPlayer::new(app, canvas_size, canvas_buf, setting)),
            _ => Box::new(FieldPlayer::new(
                app,
                app.config.format,
                canvas_size,
                canvas_buf,
                setting,
            )),
        };
    }

    fn update_setting(&mut self) {
        if let Some(workgroup_count) = self.ctrl_panel.update_setting(&self.app) {
            // 更新了粒子数后，还须更新 workgroup count
            self.player
                .update_workgroup_count(&self.app, workgroup_count);
        }

        self.player.update_by(&self.app, &mut self.ctrl_panel);
    }
}

pub fn main() {
    run::<InteractiveApp>(Some(1.6));
}
