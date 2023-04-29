use super::{Cloth, ClothFabric};
use crate::Simulator;
use app_surface::math::Size;
use app_surface::AppSurface;
#[cfg(not(target_arch = "wasm32"))]
use std::{thread, sync::mpsc};

pub struct PBDSimulator {
    pbd_obj: Option<Cloth>,
    #[cfg(not(target_arch = "wasm32"))]
    rx: mpsc::Receiver<ClothFabric>,
}

impl PBDSimulator {
    pub fn new(app: &AppSurface) -> Self {
        let viewport_size: Size<f32> = (&app.config).into();

        #[cfg(target_arch = "wasm32")]
        {
            let cloth_fabric = create_cloth_fabric(viewport_size);
            return Self {
                pbd_obj: Some(Cloth::new(app, cloth_fabric)),
            };
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let cloth_fabric = create_cloth_fabric(viewport_size);
                let _ = tx.send(cloth_fabric);
            });

            Self { rx, pbd_obj: None }
        }
    }
}

impl Simulator for PBDSimulator {
    fn update_by(
        &mut self,
        app: &app_surface::AppSurface,
        control_panel: &mut crate::ControlPanel,
    ) {
        if self.pbd_obj.is_none() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(data) = self.rx.try_recv() {
                    self.pbd_obj = Some(Cloth::new(app, data));
                } else {
                    // Waiting for cloth data
                }
            }
        }

        if let Some(pbd) = self.pbd_obj.as_mut() {
            pbd.update_by(app, control_panel);
        }
    }

    fn update_workgroup_count(
        &mut self,
        _app: &app_surface::AppSurface,
        _workgroup_count: (u32, u32, u32),
    ) {
    }

    fn resize(&mut self, app: &app_surface::AppSurface) -> bool {
        if let Some(pbd) = self.pbd_obj.as_mut() {
            return pbd.resize(app);
        }
        false
    }

    fn compute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(pbd) = self.pbd_obj.as_mut() {
            pbd.compute(encoder);
        }
    }

    fn draw_by_rpass<'b, 'a: 'b>(
        &'a mut self,
        app: &app_surface::AppSurface,
        rpass: &mut wgpu::RenderPass<'b>,
        setting: &mut crate::SettingObj,
    ) {
        if let Some(pbd) = self.pbd_obj.as_mut() {
            pbd.draw_by_rpass(app, rpass, setting);
        }
    }
}

fn create_cloth_fabric(viewport_size: Size<f32>) -> ClothFabric {
    let horizontal_pixel = viewport_size.width as f32;
    let vertical_pixel = horizontal_pixel;

    let fovy: f32 = 75.0 / 180.0 * std::f32::consts::PI;
    let factor = crate::util::matrix_helper::fullscreen_factor(viewport_size, fovy);
    let a_pixel_on_ndc = factor.1 / viewport_size.width;

    let particle_x_num = 50;
    let particle_y_num = 50;

    ClothFabric::gen_fabric(
        particle_x_num,
        particle_y_num,
        horizontal_pixel,
        vertical_pixel,
        a_pixel_on_ndc,
    )
}
