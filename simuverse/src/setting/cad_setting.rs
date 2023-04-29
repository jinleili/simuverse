use egui::Color32;

pub struct CADSetting {
    pub simu_ty: u32,
    pub render_mode: u32,
    pub texture: u32,
}

impl Default for CADSetting {
    fn default() -> Self {
        Self::new()
    }
}

impl CADSetting {
    pub fn new() -> Self {
        Self {
            simu_ty: 0,
            render_mode: 3,
            texture: 0,
        }
    }

    pub fn ui_contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Type:");
            ui.selectable_value(&mut self.simu_ty, 0, "B-Spline");
            ui.selectable_value(&mut self.simu_ty, 1, "Obj Load");
        });
        if self.simu_ty == 0 {
            return;
        }

        // ui.separator();
        // ui.horizontal(|ui| {
        //     ui.heading("Procedural texture：");
        //     egui::ComboBox::from_label("")
        //         .selected_text(get_texture_name(self.texture))
        //         .show_ui(ui, |ui| {
        //             ui.style_mut().wrap = Some(false);
        //             ui.set_min_width(60.0);
        //             ui.selectable_value(&mut self.texture, 0, get_texture_name(0));
        //             ui.selectable_value(&mut self.texture, 1, get_texture_name(1));
        //             ui.selectable_value(&mut self.texture, 2, get_texture_name(2));
        //         });
        // });

        ui.separator();
        ui.heading("Render Mode");
        egui::Grid::new("my_grid")
            .num_columns(1)
            .spacing([10.0, 12.0])
            .striped(true)
            .show(ui, |ui| {
                if ui.radio(self.render_mode == 0, "Show surface").clicked() {
                    self.render_mode = 0;
                }
                ui.end_row();

                if ui.radio(self.render_mode == 1, "Show wire frame").clicked() {
                    self.render_mode = 1;
                }
                ui.end_row();

                if ui
                    .radio(self.render_mode == 2, "Hidden line eliminate")
                    .clicked()
                {
                    self.render_mode = 2;
                }
                ui.end_row();

                if ui
                    .radio(self.render_mode == 3, "Show surface and wire frame")
                    .clicked()
                {
                    self.render_mode = 3;
                }
                ui.end_row();
            });

        ui.separator();
        ui.heading("Operations");
        ui.horizontal_wrapped(|ui| {
            ui.label("0. Drag the mouse to");
            ui.colored_label(Color32::from_rgb(110, 235, 110), "rotate the model");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("1. Right-click to");
            ui.colored_label(Color32::from_rgb(110, 255, 110), "move the light");
            ui.label("to the camera's position");
        });
    }
}

#[allow(dead_code)]
fn get_texture_name(index: u32) -> &'static str {
    match index {
        0 => "None",
        1 => "Wood",
        _ => "Marble",
    }
}
