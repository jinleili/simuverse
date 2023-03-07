pub struct PBDSetting {
    pub simu_ty: Option<i32>,
    pub damping: f32,
    pub gravity: f32,
}

impl Default for PBDSetting {
    fn default() -> Self {
        Self::new()
    }
}

impl PBDSetting {
    pub fn new() -> Self {
        Self {
            simu_ty: Some(0),
            damping: 0.6,
            gravity: 0.7,
        }
    }

    fn ty_changed(&self) {}

    pub fn ui_contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Type:");
            if ui
                .selectable_value(&mut self.simu_ty, Some(0), "Cloth")
                .clicked()
            {
                self.ty_changed();
            };
        });
        ui.separator();

        egui::Grid::new("my_grid")
            .num_columns(2)
            .spacing([10.0, 12.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Damping:");
                ui.add(egui::Slider::new(&mut self.damping, 0.3..=1.0));
                ui.end_row();

                ui.label("Gravity:");
                ui.add(egui::Slider::new(&mut self.gravity, 0.1..=1.0));
                ui.end_row();
            });
    }
}
