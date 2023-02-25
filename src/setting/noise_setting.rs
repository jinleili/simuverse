#[derive(Default)]
pub struct NoiseSetting {
    pub simu_ty: Option<i32>,
    pub back_color: [f32; 3],
    pub front_color: [f32; 3],
    pub noise_scale: f32,
    pub octave: i32,
    pub lacunarity: f32,
    pub gain: f32,
    hide_gain: bool,
}

impl NoiseSetting {
    pub fn new() -> Self {
        let mut instance = Self {
            simu_ty: Some(0),
            ..Default::default()
        };
        instance.ty_changed();

        instance
    }

    fn ty_changed(&mut self) {
        self.hide_gain = false;
        match self.simu_ty {
            Some(1) => {
                self.back_color = [0.69, 0.498, 0.361];
                self.front_color = [0.125, 0.094, 0.067];
                self.noise_scale = 2.0;
                self.octave = 3;
                self.lacunarity = 0.7;
                self.hide_gain = true;
            }
            Some(2) => {
                self.back_color = [1.0; 3];
                self.front_color = [0.353, 0.120, 0.106];
                self.noise_scale = 1.0;
                self.octave = 3;
                self.lacunarity = 2.7;
                self.gain = 0.49;
            }
            Some(3) => {
                self.back_color  = [0.000,0.000,0.165];
                self.front_color = [0.667,1.000,1.000];
                self.noise_scale = 1.5;
                self.octave = 3;
                self.lacunarity = 1.8;
                self.gain = 0.91;
            }
            _ => {
                self.back_color = [0.98; 3];
                self.front_color = [0.16, 0.22, 0.5];
                self.noise_scale = 2.2;
                self.octave = 6;
                self.lacunarity = 2.5;
                self.gain = 0.6;
            }
        }
    }

    pub fn ui_contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Type:");
            if ui
                .selectable_value(&mut self.simu_ty, Some(0), "Marble")
                .clicked()
            {
                self.ty_changed();
            };
            if ui
                .selectable_value(&mut self.simu_ty, Some(1), "Wood")
                .clicked()
            {
                self.ty_changed();
            };
            if ui
                .selectable_value(&mut self.simu_ty, Some(2), "Grim world")
                .clicked()
            {
                self.ty_changed();
            };
            if ui
                .selectable_value(&mut self.simu_ty, Some(3), "Mercury")
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
                ui.label("Color 0:");
                ui.color_edit_button_rgb(&mut self.back_color);
                ui.end_row();

                ui.label("Color 1:");
                ui.color_edit_button_rgb(&mut self.front_color);
                ui.end_row();

                ui.label("Noise scale:");
                ui.add(egui::Slider::new(&mut self.noise_scale, 0.2..=14.0));
                ui.end_row();

                if self.simu_ty == Some(0) || self.simu_ty == Some(1) {
                    ui.label("Octave:");
                    ui.add(egui::Slider::new(&mut self.octave, 1..=40));
                    ui.end_row();
                }

                ui.label("Lacunarity:");
                ui.add(egui::Slider::new(&mut self.lacunarity, 0.2..=8.4));
                ui.end_row();

                if self.simu_ty != Some(1) {
                    ui.label("Gain:");
                    ui.add(egui::Slider::new(&mut self.gain, 0.15..=1.0));
                    ui.end_row();
                }
            });
    }
}
