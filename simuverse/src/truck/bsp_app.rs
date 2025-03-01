use crate::CADSetting;

use super::CADApp;
use super::platform::*;
use super::rendimpl::*;
use alloc::{sync::Arc, vec::Vec};
use app_surface::AppSurface;
use core::{
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
};
use std::{sync::Mutex, thread::*};

use truck_modeling::*;

pub struct BSplineApp {
    object: Arc<Mutex<PolygonInstance>>,
    closed: Arc<AtomicBool>,
    updated: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl BSplineApp {
    pub fn new(_app: &AppSurface, scene: &mut Scene) -> Self {
        let creator = scene.instance_creator();
        let surface = Self::init_surface(3, 4);
        let object = creator.create_instance(
            &StructuredMesh::from_surface(&surface, surface.range_tuple(), 0.01),
            &Default::default(),
        );
        scene.add_object(&object);

        let object = Arc::new(Mutex::new(object));
        let closed = Arc::new(AtomicBool::new(false));
        let updated = Arc::new(AtomicBool::new(false));
        let thread = Some(Self::init_thread(
            creator,
            Arc::clone(&object),
            Arc::clone(&closed),
            Arc::clone(&updated),
            Arc::new(Mutex::new(surface)),
        ));
        Self {
            object,
            closed,
            updated,
            thread,
        }
    }

    fn init_surface(degree: usize, division: usize) -> BSplineSurface<Point3> {
        let range = degree + division - 1;
        let knot_vec = KnotVec::uniform_knot(degree, division);
        let mut ctrl_pts = Vec::new();
        for i in 0..=range {
            let u = (i as f64) / (range as f64);
            let mut vec = Vec::new();
            for j in 0..=range {
                let v = (j as f64) / (range as f64);
                vec.push(Point3::new(v, 0.0, u));
            }
            ctrl_pts.push(vec);
        }
        BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts)
    }
    fn init_thread(
        creator: InstanceCreator,
        object: Arc<Mutex<PolygonInstance>>,
        closed: Arc<AtomicBool>,
        updated: Arc<AtomicBool>,
        surface: Arc<Mutex<BSplineSurface<Point3>>>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let mut time: f64 = 0.0;
            let mut count = 0;
            // let mut instant = std::time::Instant::now();
            loop {
                std::thread::sleep(core::time::Duration::from_millis(1));
                if closed.load(Ordering::SeqCst) {
                    break;
                }
                if updated.load(Ordering::SeqCst) {
                    continue;
                }
                updated.store(true, Ordering::SeqCst);
                count += 1;
                time += 0.1;
                if count == 100 {
                    // let fps_inv = instant.elapsed().as_secs_f64();
                    // println!("{}", 100.0 / fps_inv);
                    // instant = std::time::Instant::now();
                    count = 0;
                }
                let mut mesh = None;
                if let Ok(mut surface) = surface.lock() {
                    surface.control_point_mut(3, 3)[1] = time.sin();
                    let surface0 = surface.clone();
                    drop(surface);
                    mesh = Some(StructuredMesh::from_surface(
                        &surface0,
                        surface0.range_tuple(),
                        0.01,
                    ));
                }
                let mut another_object =
                    creator.create_instance(&mesh.unwrap(), &Default::default());
                let mut object = object.lock().unwrap();
                object.swap_vertex(&mut another_object);
            }
        })
    }
}

impl CADApp for BSplineApp {
    fn get_camera(&self) -> Camera {
        let mut vec0 = Vector4::new(1.5, 0.0, -1.5, 0.0);
        vec0 /= vec0.magnitude();
        let mut vec1 = Vector4::new(-0.5, 1.0, -0.5, 0.0);
        vec1 /= vec1.magnitude();
        let mut vec2 = Vector4::new(1.0, 1.0, 1.0, 0.0);
        vec2 /= vec2.magnitude();
        let vec3 = Vector4::new(1.5, 0.8, 1.5, 1.0);
        let matrix = Matrix4::from_cols(vec0, vec1, vec2, vec3);
        let mut camera = Camera::default();
        camera.matrix = matrix;
        camera
    }

    fn update(&mut self, scene: &mut Scene, _setting: &CADSetting) {
        if self.updated.load(Ordering::SeqCst) {
            let object = self.object.lock().unwrap();
            scene.update_vertex_buffer(&*object);
            self.updated.store(false, Ordering::SeqCst);
        }
    }

    fn remove_from_scene(&mut self, scene: &mut Scene) {
        self.closed.store(true, Ordering::SeqCst);
        self.thread.take().unwrap().join().unwrap();
        let obj = self.object.lock().unwrap();
        scene.remove_object(obj.deref());
    }
}
