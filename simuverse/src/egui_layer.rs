use crate::{
    ControlPanel,
    node::{BindGroupData, BufferlessFullscreenNode},
    util::{AnyTexture, load_texture, shader::create_shader_module},
};
use alloc::{vec, vec::Vec};
use app_surface::AppSurface;
use raw_window_handle::HasDisplayHandle;
use winit::{event::WindowEvent, window::Window};

pub struct EguiLayer {
    format: wgpu::TextureFormat,
    canvas: AnyTexture,
    sampler: wgpu::Sampler,
    shader: wgpu::ShaderModule,
    pub ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    egui_repaint: i32,
    // compose egui ui layer to frame buffer
    composer: BufferlessFullscreenNode,
}

impl EguiLayer {
    pub fn new(
        app: &AppSurface,
        display_target: &dyn HasDisplayHandle,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = create_shader_module(&app.device, "egui_layer_compose", None);
        let sampler = app
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());

        let (canvas, composer) = Self::create_canvas_and_composer(app, format, &sampler, &shader);

        let ctx = egui::Context::default();
        crate::setup_custom_fonts(&ctx);

        let egui_state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::default(),
            display_target,
            Some(app.scale_factor),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&app.device, format, None, 1, false);

        Self {
            format,
            canvas,
            shader,
            sampler,
            ctx,
            egui_state,
            egui_renderer,
            egui_repaint: 2,
            composer,
        }
    }

    pub fn on_ui_event(&mut self, window: &Window, event: &WindowEvent) {
        let response = self.egui_state.on_window_event(window, event);
        self.egui_repaint = if response.consumed {
            20
        } else {
            self.egui_repaint.max(1)
        };
    }

    pub fn resize(&mut self, app: &AppSurface) {
        let (canvas, composer) =
            Self::create_canvas_and_composer(app, self.format, &self.sampler, &self.shader);
        self.canvas = canvas;
        self.composer = composer;
    }

    pub fn refresh_ui(
        &mut self,
        app: &AppSurface,
        egui_app: &mut ControlPanel,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<Vec<wgpu::CommandBuffer>> {
        if self.egui_repaint <= 0 {
            return None;
        }
        self.egui_repaint -= 1;

        let raw_input = self.egui_state.take_egui_input(app.get_view());
        let full_output = self.ctx.run(raw_input, |ctx| {
            egui_app.ui_contents(ctx);
        });
        let clipped_primitives = self.ctx.tessellate(full_output.shapes, app.scale_factor);
        let textures_delta = full_output.textures_delta;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [app.config.width, app.config.height],
            pixels_per_point: app.scale_factor,
        };
        let egui_cmd_bufs = {
            for (id, image_delta) in &textures_delta.set {
                self.egui_renderer
                    .update_texture(&app.device, &app.queue, *id, image_delta);
            }
            self.egui_renderer.update_buffers(
                &app.device,
                &app.queue,
                encoder,
                &clipped_primitives,
                &screen_descriptor,
            )
        };
        {
            let mut rpass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.canvas.tex_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                })
                .forget_lifetime();

            // egui ui 渲染
            self.egui_renderer
                .render(&mut rpass, &clipped_primitives, &screen_descriptor);
        }

        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        Some(egui_cmd_bufs)
    }

    pub fn compose_by_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'a>) {
        self.composer.draw_by_pass(rpass);
    }

    fn create_canvas_and_composer(
        app: &AppSurface,
        format: wgpu::TextureFormat,
        sampler: &wgpu::Sampler,
        shader: &wgpu::ShaderModule,
    ) -> (AnyTexture, BufferlessFullscreenNode) {
        let canvas = load_texture::empty(
            &app.device,
            format,
            wgpu::Extent3d {
                width: app.config.width,
                height: app.config.height,
                depth_or_array_layers: 1,
            },
            None,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some("egui canvas"),
        );

        let composer = BufferlessFullscreenNode::new(
            &app.device,
            format,
            &BindGroupData {
                inout_tv: vec![(&canvas, None)],
                samplers: vec![sampler],
                ..Default::default()
            },
            shader,
            Some(wgpu::BlendState::ALPHA_BLENDING),
        );

        (canvas, composer)
    }
}
