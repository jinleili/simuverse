use app_surface::{AppSurface, SurfaceFrame};
use simuverse::framework::{run, Action};
use simuverse::util::{math::Size, BufferObj};
use simuverse::{
    FieldAnimationType, FieldPlayer, FieldType, ParticleColorType, Player, SettingObj,
};
use std::iter;
use winit::{event_loop::EventLoop, window::WindowId};

mod control_panel;
use control_panel::ControlPanel;

struct InteractiveApp {
    app: AppSurface,
    panel: ControlPanel,
    canvas_size: Size<u32>,
    canvas_buf: BufferObj,
    setting: SettingObj,
    player: Box<dyn Player>,
    frame_count: u64,
}

impl Action for InteractiveApp {
    fn new(app: AppSurface, event_loop: &EventLoop<()>) -> Self {
        let mut app = app;
        let format = app.config.format.remove_srgb_suffix();
        app.sdq.update_config_format(format);
        let panel = ControlPanel::new(&app, format, event_loop);

        let canvas_size: Size<u32> = (&app.config).into();
        let mut setting = SettingObj::new(
            FieldType::Fluid,
            FieldAnimationType::Spirl,
            ParticleColorType::MovementAngle,
            panel.particles_count,
            panel.lifetime as f32,
        );
        setting.update_canvas_size(&app, canvas_size);
        let canvas_buf = simuverse::util::BufferObj::create_empty_storage_buffer(
            &app.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );
        let player = Self::create_player(&app, canvas_size, &canvas_buf, &setting);

        Self {
            app,
            panel,
            canvas_buf,
            canvas_size,
            setting,
            player,
            frame_count: 0,
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
        self.setting.update_canvas_size(&self.app, canvas_size);
        self.canvas_size = canvas_size;
        self.canvas_buf = simuverse::util::BufferObj::create_empty_storage_buffer(
            &self.app.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );
        self.player = Self::create_player(&self.app, canvas_size, &self.canvas_buf, &self.setting);
    }

    fn on_ui_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        self.panel.on_event(event);
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

        self.player.compute(&mut encoder);

        let (output, frame_view) = self.app.get_current_frame_view();
        let (egui_cmd_bufs, textures_delta) = {
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
                .draw_by_rpass(&self.app, &mut rpass, &mut self.setting);

            {
                self.panel.begin_pass(&self.app, &mut rpass)
            }
        };

        self.app.queue.submit(
            egui_cmd_bufs
                .into_iter()
                .chain(iter::once(encoder.finish())),
        );
        output.present();

        self.panel.end_pass(&textures_delta);

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
        return match setting.field_type {
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
        if self.panel.particle_color != self.setting.color_ty as u32 {
            let color_ty = ParticleColorType::from_u32(self.panel.particle_color);
            self.setting.update_particle_color(&self.app, color_ty);
        }
        self.setting
            .update_particles_count(&self.app, self.panel.particles_count);
        self.setting
            .update_particle_point_size(&self.app, self.panel.particle_size);
    }
}

pub fn main() {
    run::<InteractiveApp>(Some(1.6));
}
