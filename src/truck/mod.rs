pub(crate) mod platform;
pub(crate) mod rendimpl;

mod cad_obj_viewer;
pub use cad_obj_viewer::CADObjViewer;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase},
};

mod bsp_app;
mod obj_app;

use crate::CADSetting;

use self::platform::{Camera, Scene};

#[derive(Clone, Copy, PartialEq)]
pub enum CADAppType {
    Bspline = 0,
    Obj,
}

impl CADAppType {
    pub fn from_u32(ty: u32) -> Self {
        match ty {
            0 => CADAppType::Bspline,
            _ => CADAppType::Obj,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    NaiveSurface = 0,
    NaiveWireFrame,
    HiddenLineEliminate,
    SurfaceAndWireFrame,
}

impl RenderMode {
    pub fn from_u32(ty: u32) -> Self {
        match ty {
            0 => RenderMode::NaiveSurface,
            1 => RenderMode::NaiveWireFrame,
            2 => RenderMode::HiddenLineEliminate,
            _ => RenderMode::SurfaceAndWireFrame,
        }
    }
}

trait CADApp {
    fn mouse_input(&mut self, _scene: &mut Scene, _state: &ElementState, _button: &MouseButton) {}
    fn mouse_wheel(
        &mut self,
        _scene: &mut Scene,
        _delta: &MouseScrollDelta,
        _touch_phase: &TouchPhase,
    ) {
    }
    fn cursor_moved(&mut self, _scene: &mut Scene, _position: PhysicalPosition<f64>) {}

    fn get_camera(&self) -> Camera;
    fn update(&mut self, scene: &mut Scene, setting: &CADSetting);
    fn remove_from_scene(&mut self, scene: &mut Scene);
}
