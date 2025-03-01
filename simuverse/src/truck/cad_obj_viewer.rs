use crate::{ControlPanel, SettingObj, Simulator};
use alloc::{boxed::Box, vec};
use app_surface::AppSurface;

use super::{CADApp, CADAppType, bsp_app::BSplineApp, obj_app::ObjApp, platform::*, rendimpl::*};

pub struct CADObjViewer {
    scene: Scene,
    cad_obj: Box<dyn CADApp>,
    ty: CADAppType,
}

impl CADObjViewer {
    pub fn new(app: &AppSurface, control_panel: &ControlPanel) -> Self {
        let render_texture = RenderTextureConfig {
            canvas_size: app.get_view().inner_size().into(),
            format: app.config.format,
        };
        let desc = SceneDescriptor {
            studio: StudioConfig {
                camera: Camera::default(),
                lights: vec![Light {
                    position: Point3::new(0.5, 2.0, 0.5),
                    color: Vector3::new(1.0, 1.0, 1.0),
                    light_type: LightType::Point,
                }],
                ..Default::default()
            },
            backend_buffer: BackendBufferConfig {
                sample_count: 1,
                ..Default::default()
            },
            render_texture,
        };
        let mut scene = Scene::new(app, &desc);
        let ty = CADAppType::from_u32(control_panel.cad_setting.simu_ty);
        let cad_obj = Self::create_cad_app(app, &mut scene, ty);

        Self { scene, cad_obj, ty }
    }

    fn create_cad_app(app: &AppSurface, scene: &mut Scene, ty: CADAppType) -> Box<dyn CADApp> {
        let cad_obj: Box<dyn CADApp> = match ty {
            CADAppType::Bspline => Box::new(BSplineApp::new(app, scene)),
            CADAppType::Obj => Box::new(ObjApp::new(app, scene)),
        };
        scene.descriptor_mut().studio.camera = cad_obj.get_camera();

        cad_obj
    }
}

impl Simulator for CADObjViewer {
    fn cursor_moved(&mut self, _app: &AppSurface, position: winit::dpi::PhysicalPosition<f64>) {
        self.cad_obj.cursor_moved(&mut self.scene, position);
    }
    fn mouse_input(
        &mut self,
        _app: &AppSurface,
        state: &winit::event::ElementState,
        button: &winit::event::MouseButton,
    ) {
        self.cad_obj.mouse_input(&mut self.scene, state, button);
    }
    fn mouse_wheel(
        &mut self,
        _app: &AppSurface,
        delta: &winit::event::MouseScrollDelta,
        touch_phase: &winit::event::TouchPhase,
    ) {
        self.cad_obj
            .mouse_wheel(&mut self.scene, delta, touch_phase)
    }

    fn update_workgroup_count(&mut self, _app: &AppSurface, _workgroup_count: (u32, u32, u32)) {}
    fn compute(&mut self, _encoder: &mut wgpu::CommandEncoder) {}

    fn update_by(&mut self, app: &AppSurface, control_panel: &mut ControlPanel) {
        let ty = CADAppType::from_u32(control_panel.cad_setting.simu_ty);
        if self.ty != ty {
            // Remove object from scene
            self.cad_obj.remove_from_scene(&mut self.scene);
            self.cad_obj = Self::create_cad_app(app, &mut self.scene, ty);
            self.ty = ty;
        }
        self.cad_obj
            .update(&mut self.scene, &control_panel.cad_setting);
    }

    fn draw_by_rpass<'b, 'a: 'b>(
        &'a mut self,
        _app: &AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
        _setting: &mut SettingObj,
    ) {
        self.scene.render_by_rpass(rpass);
    }
}
