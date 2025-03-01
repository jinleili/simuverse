use crate::{
    ControlPanel, DEPTH_FORMAT, EguiLayer, FieldSimulator, FluidSimulator, SimuType, Simulator,
    noise::TextureSimulator, util::AnyTexture, util::BufferObj,
};
use alloc::{boxed::Box, sync::Arc};
use app_surface::{AppSurface, SurfaceFrame};
use wgpu::TextureView;
use winit::dpi::PhysicalSize;
use winit::window::{Window, WindowId};

pub struct SimuverseApp {
    pub(crate) app_surface: AppSurface,
    size: PhysicalSize<u32>,
    size_changed: bool,
    frame_count: u32,
    egui_layer: EguiLayer,
    ctrl_panel: ControlPanel,
    canvas_size: glam::UVec2,
    canvas_buf: BufferObj,
    simulator: Box<dyn Simulator>,
    depth_view: TextureView,
    cloth_texture: Option<AnyTexture>,
}

impl SimuverseApp {
    pub async fn new(window: Arc<Window>) -> Self {
        // 创建 wgpu 应用
        let mut app = AppSurface::new(window.clone()).await;
        let format = app.config.format.remove_srgb_suffix();
        // 设置一个最小 surface 大小，使得在 Web 环境，egui 面板能有合适的展示大小
        let size = app.get_view().inner_size();
        app.ctx.config.width = size.width.max(375);
        app.ctx.config.height = size.height.max(500);
        app.ctx.config.format = format;
        app.surface.configure(&app.ctx.device, &app.ctx.config);

        // egui
        let egui_layer = EguiLayer::new(&app, &window, format);
        let ctrl_panel = ControlPanel::new(&app, &egui_layer.ctx);

        let canvas_size = glam::UVec2::new(app.config.width, app.config.height);
        let canvas_buf = crate::util::BufferObj::create_empty_storage_buffer(
            &app.device,
            (canvas_size.x * canvas_size.y * 12) as u64,
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

        let size = PhysicalSize::new(app.config.width, app.config.height);

        Self {
            app_surface: app,
            size,
            size_changed: true,
            frame_count: 0,
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
        self.app_surface.adapter.get_info()
    }

    pub fn get_view(&mut self) -> &winit::window::Window {
        self.app_surface.get_view()
    }

    pub fn current_window_id(&self) -> WindowId {
        self.app_surface.get_view().id()
    }

    pub fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        if self.size == new_size {
            return;
        }
        self.size = new_size;
        self.size_changed = true;
    }

    /// 必要的时候调整 surface 大小
    ///
    /// resize 在缩放窗口时会高频触发，所以需要限制 resize 的频率
    fn resize_surface_if_needed(&mut self) {
        if self.size_changed && self.frame_count > 10 {
            //  需先 resize surface
            self.app_surface
                .resize_surface_by_size((self.size.width, self.size.height));

            self.depth_view = Self::create_depth_tex(&self.app_surface);
            self.egui_layer.resize(&self.app_surface);

            let canvas_size = glam::UVec2::new(
                self.app_surface.config.width,
                self.app_surface.config.height,
            );
            self.ctrl_panel
                .setting
                .update_canvas_size(&self.app_surface, canvas_size);
            self.canvas_size = canvas_size;
            self.canvas_buf = crate::util::BufferObj::create_empty_storage_buffer(
                &self.app_surface.device,
                (canvas_size.x * canvas_size.y * 12) as u64,
                false,
                Some("canvas_buf"),
            );

            if !self.simulator.resize(&self.app_surface) {
                self.create_simulator();
            }

            self.size_changed = false;
            self.frame_count = 0;
        }
    }

    pub fn on_ui_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_layer
            .on_ui_event(self.app_surface.get_view(), event);
    }

    pub fn on_click(&mut self, pos: glam::Vec2) {
        self.simulator.on_click(&self.app_surface, pos);
    }

    pub fn touch_move(&mut self, pos: glam::Vec2) {
        self.simulator.touch_move(&self.app_surface, pos);
    }

    pub fn cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.simulator.cursor_moved(&self.app_surface, position);
    }
    pub fn mouse_input(
        &mut self,
        state: &winit::event::ElementState,
        button: &winit::event::MouseButton,
    ) {
        self.simulator.mouse_input(&self.app_surface, state, button);
    }
    pub fn mouse_wheel(
        &mut self,
        delta: &winit::event::MouseScrollDelta,
        touch_phase: &winit::event::TouchPhase,
    ) {
        self.simulator
            .mouse_wheel(&self.app_surface, delta, touch_phase);
    }

    pub fn request_redraw(&mut self) {
        self.app_surface.get_view().request_redraw();
    }

    pub fn render(&mut self) {
        self.frame_count += 1;
        self.resize_surface_if_needed();

        let mut encoder =
            self.app_surface
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // egui ui 更新
        let egui_app = &mut self.ctrl_panel;
        let egui_cmd_buffers =
            self.egui_layer
                .refresh_ui(&self.app_surface, egui_app, &mut encoder);

        self.simulator.compute(&mut encoder);

        let (output, frame_view) = self
            .app_surface
            .get_current_frame_view(Some(self.app_surface.config.format.remove_srgb_suffix()));
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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            self.simulator.draw_by_rpass(
                &self.app_surface,
                &mut rpass,
                &mut self.ctrl_panel.setting,
            );

            self.egui_layer.compose_by_pass(&mut rpass);
        }

        if let Some(egui_cmd_bufs) = egui_cmd_buffers {
            self.app_surface
                .queue
                .submit(egui_cmd_bufs.into_iter().chain(Some(encoder.finish())));
        } else {
            self.app_surface.queue.submit(Some(encoder.finish()));
        }
        output.present();

        self.update_setting();
    }

    fn create_simulator<'a, 'b: 'a>(&'b mut self) {
        let app = &self.app_surface;
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
            #[cfg(not(target_arch = "wasm32"))]
            SimuType::CAD => Box::new(crate::CADObjViewer::new(app, ctrl_panel)),
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
        let res = self.ctrl_panel.update_setting(&self.app_surface);
        if res.1 {
            // 改变了模拟类型
            self.create_simulator();
        } else if self.ctrl_panel.selected_simu_type == SimuType::Noise {
            self.simulator
                .update_by(&self.app_surface, &mut self.ctrl_panel);
        } else {
            if let Some(workgroup_count) = res.0 {
                // 更新了粒子数后，还须更新 workgroup count
                self.simulator
                    .update_workgroup_count(&self.app_surface, workgroup_count);
            }
            self.simulator
                .update_by(&self.app_surface, &mut self.ctrl_panel);
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
