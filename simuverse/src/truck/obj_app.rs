use crate::CADSetting;

use super::{CADApp, RenderMode, platform::*, rendimpl::*};
use app_surface::AppSurface;
use std::io::Read;
use truck_meshalgo::prelude::*;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase},
};

const TEAPOT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/obj/skull-with-texcoord.obj",
));
// const TEAPOT_BYTES: &[u8] = include_bytes!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "/assets/obj/skull-modified.obj",
// ));
// const TEAPOT_BYTES: &[u8] = include_bytes!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "/assets/obj/teapot.obj",
// ));

pub struct ObjApp {
    _creator: InstanceCreator,
    rotate_flag: bool,
    prev_cursor: Vector2,
    instance: PolygonInstance,
    wireframe: WireFrameInstance,
    render_mode: RenderMode,
}

impl ObjApp {
    pub fn new(_app: &AppSurface, scene: &mut Scene) -> Self {
        let creator = scene.instance_creator();
        let (instance, wireframe) = Self::load_obj(&creator, TEAPOT_BYTES);
        scene.add_object(&instance);
        scene.add_object(&wireframe);
        let mut app = Self {
            _creator: creator,
            rotate_flag: false,
            prev_cursor: Vector2::zero(),
            instance,
            wireframe,
            render_mode: RenderMode::NaiveSurface,
        };
        app.update_render_mode(scene);
        app
    }

    fn update_render_mode(&mut self, scene: &mut Scene) {
        let visible = match self.render_mode {
            RenderMode::NaiveSurface => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    reflectance: 0.5,
                    roughness: 0.1,
                    ambient_ratio: 0.02,
                    background_ratio: 0.0,
                    alpha_blend: false,
                };
                scene.update_bind_group(&self.instance);
                (true, false)
            }
            RenderMode::NaiveWireFrame => {
                self.wireframe.instance_state_mut().color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                scene.update_bind_group(&self.wireframe);
                (false, true)
            }
            RenderMode::HiddenLineEliminate => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(0.0, 0.0, 0.0, 1.0),
                    reflectance: 0.0,
                    roughness: 0.0,
                    ambient_ratio: 1.0,
                    background_ratio: 0.0,
                    alpha_blend: false,
                };
                self.wireframe.instance_state_mut().color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                scene.update_bind_group(&self.instance);
                scene.update_bind_group(&self.wireframe);
                (true, true)
            }
            RenderMode::SurfaceAndWireFrame => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    reflectance: 0.5,
                    roughness: 0.1,
                    ambient_ratio: 0.02,
                    background_ratio: 0.0,
                    alpha_blend: false,
                };
                self.wireframe.instance_state_mut().color = Vector4::new(0.0, 0.0, 0.0, 1.0);
                scene.update_bind_group(&self.instance);
                scene.update_bind_group(&self.wireframe);
                (true, true)
            }
        };
        scene.set_visibility(&self.instance, visible.0);
        scene.set_visibility(&self.wireframe, visible.1);
    }

    fn load_obj<R: Read>(
        creator: &InstanceCreator,
        reader: R,
    ) -> (PolygonInstance, WireFrameInstance) {
        let mut mesh = obj::read(reader).unwrap();
        mesh.put_together_same_attrs(TOLERANCE * 2.0)
            .add_smooth_normals(0.5, false);
        let bdd_box = mesh.bounding_box();
        let (size, center) = (bdd_box.size(), bdd_box.center());
        let mat = Matrix4::from_translation(center.to_vec()) * Matrix4::from_scale(size);
        let polygon_state = PolygonState {
            matrix: mat.invert().unwrap(),
            ..Default::default()
        };
        let wire_state = WireFrameState {
            matrix: mat.invert().unwrap(),
            ..Default::default()
        };
        (
            creator.create_instance(&mesh, &polygon_state),
            creator.create_instance(&mesh, &wire_state),
        )
    }
}

impl CADApp for ObjApp {
    fn mouse_input(&mut self, scene: &mut Scene, state: &ElementState, button: &MouseButton) {
        match button {
            MouseButton::Left => {
                self.rotate_flag = *state == ElementState::Pressed;
            }
            MouseButton::Right => {
                let (light, camera) = {
                    let desc = scene.studio_config_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                match light.light_type {
                    LightType::Point => {
                        light.position = camera.position();
                    }
                    LightType::Uniform => {
                        light.position = camera.position();
                        let strength = light.position.to_vec().magnitude();
                        light.position /= strength;
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_wheel(&mut self, scene: &mut Scene, delta: &MouseScrollDelta, _: &TouchPhase) {
        match *delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let camera = &mut scene.studio_config_mut().camera;
                let trans_vec = camera.eye_direction() * 0.2 * y as f64;
                camera.matrix = Matrix4::from_translation(trans_vec) * camera.matrix;
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
    }

    fn cursor_moved(&mut self, scene: &mut Scene, position: PhysicalPosition<f64>) {
        let position = Vector2::new(position.x, position.y);
        if self.rotate_flag {
            let matrix = &mut scene.studio_config_mut().camera.matrix;
            let position = Vector2::new(position.x, position.y);
            let dir2d = position - self.prev_cursor;
            if dir2d.so_small() {
                return;
            }
            let mut axis = dir2d[1] * matrix[0].truncate();
            axis += dir2d[0] * matrix[1].truncate();
            axis /= axis.magnitude();
            let angle = dir2d.magnitude() * 0.01;
            let mat = Matrix4::from_axis_angle(axis, Rad(angle));
            *matrix = mat.invert().unwrap() * *matrix;
        }
        self.prev_cursor = position;
    }

    fn get_camera(&self) -> Camera {
        let matrix = Matrix4::look_at_rh(
            Point3::new(1.0, 1.0, 1.0),
            Point3::origin(),
            Vector3::unit_y(),
        );
        Camera::perspective_camera(
            matrix.invert().unwrap(),
            Rad(core::f64::consts::PI / 4.0),
            0.1,
            40.0,
        )
    }

    fn update(&mut self, scene: &mut Scene, setting: &CADSetting) {
        let mode = RenderMode::from_u32(setting.render_mode);
        if self.render_mode != mode {
            self.render_mode = mode;
            self.update_render_mode(scene);
        }
    }

    fn remove_from_scene(&mut self, scene: &mut Scene) {
        scene.remove_object(&self.instance);
        scene.remove_object(&self.wireframe);
    }
}
