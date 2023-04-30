use crate::{
    noise::TextureSimulator, util::AnyTexture, util::BufferObj, CADObjViewer, ControlPanel,
    EguiLayer, FieldSimulator, FluidSimulator, SimuType, Simulator, DEPTH_FORMAT,
};
use app_surface::{math::Size, AppSurface, SurfaceFrame};
use raw_window_handle::HasRawDisplayHandle;
use std::iter;
use wgpu::TextureView;
use winit::dpi::PhysicalSize;
use winit::window::WindowId;

pub struct SimuverseApp {
    app: AppSurface,
    egui_layer: EguiLayer,
    ctrl_panel: ControlPanel,
    canvas_size: Size<u32>,
    canvas_buf: BufferObj,
    simulator: Box<dyn Simulator>,
    depth_view: TextureView,
    cloth_texture: Option<AnyTexture>,
}

impl SimuverseApp {
    pub async fn new(app: AppSurface, event_loop: &dyn HasRawDisplayHandle) -> Self {
        let mut app = app;
        let format = app.config.format.remove_srgb_suffix();
        app.sdq.update_config_format(format);

        // egui
        let egui_layer = EguiLayer::new(&app, event_loop, format);
        let ctrl_panel = ControlPanel::new(&app, &egui_layer.ctx);

        let canvas_size: Size<u32> = (&app.config).into();
        let canvas_buf = crate::util::BufferObj::create_empty_storage_buffer(
            &app.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );

        #[cfg(target_arch = "wasm32")]
        let cloth_texture = {
            let (cloth_texture, _) = crate::util::load_texture::from_path(
                "cloth_500x500.png",
                &app,
                wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                false,
            )
            .await;
            Some(cloth_texture)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let cloth_texture = None;

        let simulator = Box::new(FieldSimulator::new(
            &app,
            app.config.format.remove_srgb_suffix(),
            canvas_size,
            &canvas_buf,
            &ctrl_panel.setting,
        ));
        let depth_view = Self::create_depth_tex(&app);

        Self {
            app,
            egui_layer,
            ctrl_panel,
            canvas_buf,
            canvas_size,
            simulator,
            depth_view,
            cloth_texture,
        }
    }

    pub fn get_adapter_info(&self) -> wgpu::AdapterInfo {
        self.app.adapter.get_info()
    }

    pub fn get_view_mut(&mut self) -> &mut winit::window::Window {
        &mut self.app.view
    }

    pub fn current_window_id(&self) -> WindowId {
        self.app.view.id()
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
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
        self.canvas_buf = crate::util::BufferObj::create_empty_storage_buffer(
            &self.app.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );

        if !self.simulator.resize(&self.app) {
            self.create_simulator();
        }
    }

    pub fn on_ui_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        self.egui_layer.on_ui_event(event);
    }

    pub fn on_click(&mut self, pos: app_surface::math::Position) {
        self.simulator.on_click(&self.app, pos);
    }

    pub fn touch_move(&mut self, pos: app_surface::math::Position) {
        self.simulator.touch_move(&self.app, pos);
    }

    pub fn cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.simulator.cursor_moved(&self.app, position);
    }
    pub fn mouse_input(
        &mut self,
        state: &winit::event::ElementState,
        button: &winit::event::MouseButton,
    ) {
        self.simulator.mouse_input(&self.app, state, button);
    }
    pub fn mouse_wheel(
        &mut self,
        delta: &winit::event::MouseScrollDelta,
        touch_phase: &winit::event::TouchPhase,
    ) {
        self.simulator.mouse_wheel(&self.app, delta, touch_phase);
    }

    pub fn request_redraw(&mut self) {
        self.app.view.request_redraw();
    }

    pub fn render(&mut self) {
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

        let (output, frame_view) = self
            .app
            .get_current_frame_view(Some(self.app.config.format.remove_srgb_suffix()));
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
    }

    fn create_simulator<'a, 'b: 'a>(&'b mut self) {
        let app = &self.app;
        let canvas_size = self.canvas_size;
        let canvas_buf = &self.canvas_buf;
        let ctrl_panel = &self.ctrl_panel;
        let simulator: Box<dyn Simulator> = match ctrl_panel.setting.simu_type {
            SimuType::Fluid => Box::new(FluidSimulator::new(
                app,
                canvas_size,
                canvas_buf,
                &ctrl_panel.setting,
            )),
            SimuType::Noise => Box::new(TextureSimulator::new(app)),
            SimuType::PBDynamic => Box::new(crate::pbd::PBDSimulator::new(
                app,
                self.cloth_texture.as_ref(),
            )),
            SimuType::CAD => Box::new(CADObjViewer::new(app, ctrl_panel)),
            _ => Box::new(FieldSimulator::new(
                app,
                app.config.format.remove_srgb_suffix(),
                canvas_size,
                canvas_buf,
                &ctrl_panel.setting,
            )),
        };
        self.simulator = simulator;
    }

    fn update_setting(&mut self) {
        let res = self.ctrl_panel.update_setting(&self.app);
        if res.1 {
            // 改变了模拟类型
            self.create_simulator();
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
