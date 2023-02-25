use app_surface::{math::Size, AppSurface, SurfaceFrame};
use simuverse::framework::{run, Action};
use simuverse::util::BufferObj;
use simuverse::EguiLayer;
use simuverse::{
    noise::TextureSimulator, ControlPanel, FieldSimulator, FluidSimulator,
    SettingObj, SimuType, Simulator, DEPTH_FORMAT,
};
use winit::dpi::PhysicalSize;
use std::iter;
use wgpu::TextureView;
use winit::{event_loop::EventLoop, window::WindowId};

struct SimuverseApp {
    app: AppSurface,
    egui_layer: EguiLayer,
    ctrl_panel: ControlPanel,
    canvas_size: Size<u32>,
    canvas_buf: BufferObj,
    simulator: Box<dyn Simulator>,
    depth_view: TextureView,
}

impl Action for SimuverseApp {
    fn new(app: AppSurface, event_loop: &EventLoop<()>) -> Self {
        let mut app = app;
        let format = app.config.format.remove_srgb_suffix();
        app.sdq.update_config_format(format);

        // egui
        let egui_layer = EguiLayer::new(&app, event_loop, format);
        let ctrl_panel = ControlPanel::new(&app, &egui_layer.ctx);

        let canvas_size: Size<u32> = (&app.config).into();
        let canvas_buf = simuverse::util::BufferObj::create_empty_storage_buffer(
            &app.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );
        let simulator = Self::create_simulator(&app, canvas_size, &canvas_buf, &ctrl_panel.setting);

        let depth_view = Self::create_depth_tex(&app);

        Self {
            app,
            egui_layer,
            ctrl_panel,
            canvas_buf,
            canvas_size,
            simulator,
            depth_view,
        }
    }

    fn get_adapter_info(&self) -> wgpu::AdapterInfo {
        self.app.adapter.get_info()
    }

    fn current_window_id(&self) -> WindowId {
        self.app.view.id()
    }

    fn resize(&mut self , size: &PhysicalSize<u32>) {
        if self.app.config.width == size.width && self.app.config.height == size.height {
            return;
        }
        self.app.resize_surface();
        self.depth_view = Self::create_depth_tex(&self.app);
        self.egui_layer.resize(&self.app);

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
        self.simulator = Self::create_simulator(
            &self.app,
            canvas_size,
            &self.canvas_buf,
            &self.ctrl_panel.setting,
        );
    }

    fn on_ui_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        self.egui_layer.on_ui_event(event);
    }

    fn on_click(&mut self, pos: app_surface::math::Position) {
        self.simulator.on_click(&self.app, pos);
    }

    fn touch_move(&mut self, pos: app_surface::math::Position) {
        self.simulator.touch_move(&self.app, pos);
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
        let egui_app = &mut self.ctrl_panel;
        let egui_cmd_buffers = self
            .egui_layer
            .refresh_ui(&self.app, egui_app, &mut encoder);

        self.simulator.compute(&mut encoder);

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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
                label: None,
            });
            self.simulator
                .draw_by_rpass(&self.app, &mut rpass, &mut self.ctrl_panel.setting);

            self.egui_layer.compose_by_pass(&mut rpass);
        }

        if let Some(egui_cmd_bufs) = egui_cmd_buffers {
            self.app.queue.submit(
                egui_cmd_bufs
                    .into_iter()
                    .chain(iter::once(encoder.finish())),
            );
        } else {
            self.app.queue.submit(iter::once(encoder.finish()));
        }
        output.present();

        self.update_setting();

        Ok(())
    }
}

impl SimuverseApp {
    fn create_simulator(
        app: &AppSurface,
        canvas_size: Size<u32>,
        canvas_buf: &BufferObj,
        setting: &SettingObj,
    ) -> Box<dyn Simulator> {
        match setting.simu_type {
            SimuType::Fluid => Box::new(FluidSimulator::new(app, canvas_size, canvas_buf, setting)),
            SimuType::Noise => Box::new(TextureSimulator::new(app)),
            _ => Box::new(FieldSimulator::new(
                app,
                app.config.format,
                canvas_size,
                canvas_buf,
                setting,
            )),
        }
    }

    fn update_setting(&mut self) {
        let res = self.ctrl_panel.update_setting(&self.app);
        if res.1 {
            // 改变了模拟类型
            self.simulator = Self::create_simulator(
                &self.app,
                (&self.app.config).into(),
                &self.canvas_buf,
                &self.ctrl_panel.setting,
            );
        } else if self.ctrl_panel.selected_simu_type == SimuType::Noise {
            self.simulator.update_by(&self.app, &mut self.ctrl_panel);
        } else {
            if let Some(workgroup_count) = res.0 {
                // 更新了粒子数后，还须更新 workgroup count
                self.simulator
                    .update_workgroup_count(&self.app, workgroup_count);
            }
            self.simulator.update_by(&self.app, &mut self.ctrl_panel);
        }
    }

    fn create_depth_tex(app: &AppSurface) -> wgpu::TextureView {
        let depth_texture = app.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: app.config.width,
                height: app.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

pub fn main() {
    run::<SimuverseApp>(Some(1.6));
}
