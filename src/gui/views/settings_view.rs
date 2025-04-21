use egui::{Ui, ComboBox};
use std::sync::{Arc, Mutex};

use crate::gui::app::AppState;
use crate::modules::config::ClickType;

pub struct SettingsView {
    state: Arc<Mutex<AppState>>,
}

impl SettingsView {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Settings");

        let mut config = {
            let state = self.state.lock().unwrap();
            state.current_config.clone()
        };

        ui.collapsing("Click Type", |ui| {
            let mut randomize = config.click_options.randomize_click_type;
            if ui.checkbox(&mut randomize, "Randomize Click Type").changed() {
                config.click_options.randomize_click_type = randomize;

                let mut state = self.state.lock().unwrap();
                state.current_config.click_options.randomize_click_type = randomize;
            }

            if !randomize {
                ui.horizontal(|ui| {
                    ui.label("Click Type:");
                    ComboBox::new("click_type", "Click Type")
                        .selected_text(format!("{:?}", config.click_options.click_type))
                        .show_ui(ui, |ui| {
                            let mut changed = false;
                            let mut click_type = config.click_options.click_type;

                            if ui.selectable_value(&mut click_type, ClickType::Single, "Single").changed() {
                                changed = true;
                            }

                            if ui.selectable_value(&mut click_type, ClickType::Double, "Double").changed() {
                                changed = true;
                            }

                            if ui.selectable_value(&mut click_type, ClickType::Right, "Right").changed() {
                                changed = true;
                            }

                            if ui.selectable_value(&mut click_type, ClickType::Middle, "Middle").changed() {
                                changed = true;
                            }

                            if changed {
                                config.click_options.click_type = click_type;

                                let mut state = self.state.lock().unwrap();
                                state.current_config.click_options.click_type = click_type;
                            }
                        });
                });
            } else {
                ui.heading("Click Type Weights");

                for i in 0..config.click_options.click_type_weights.len() {
                    let (click_type, weight) = &mut config.click_options.click_type_weights[i];
                    let label = format!("{:?}", click_type);

                    ui.horizontal(|ui| {
                        ui.label(format!("{} Weight:", label));
                        if ui.add(egui::Slider::new(weight, 0.0..=1.0).text("")).changed() {
                            let mut state = self.state.lock().unwrap();
                            state.current_config.click_options.click_type_weights[i].1 = *weight;
                        }
                    });
                }
            }
        });

        ui.collapsing("Timing Settings", |ui| {
            let mut timing = config.click_timing.clone();
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Min Delay (seconds):");
                if ui.add(egui::Slider::new(&mut timing.min_delay, 0.1..=60.0).text("s")).changed() {
                    changed = true;

                    // Ensure min_delay <= max_delay
                    if timing.min_delay > timing.max_delay {
                        timing.max_delay = timing.min_delay;
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Max Delay (seconds):");
                if ui.add(egui::Slider::new(&mut timing.max_delay, timing.min_delay..=60.0).text("s")).changed() {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Click Duration Mean (ms):");
                if ui.add(egui::Slider::new(&mut timing.click_duration_mean, 10.0..=200.0).text("ms")).changed() {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Click Duration Std Dev (ms):");
                if ui.add(egui::Slider::new(&mut timing.click_duration_std_dev, 1.0..=50.0).text("ms")).changed() {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Double Click Gap (ms):");
                if ui.add(egui::Slider::new(&mut timing.double_click_gap, 50..=500).text("ms")).changed() {
                    changed = true;
                }
            });

            if changed {
                let mut state = self.state.lock().unwrap();
                state.current_config.click_timing = timing;
            }
        });

        ui.collapsing("Application Settings", |ui| {
            let mut is_dark_mode = {
                let state = self.state.lock().unwrap();
                state.is_dark_mode
            };

            if ui.checkbox(&mut is_dark_mode, "Dark Mode").changed() {
                let mut state = self.state.lock().unwrap();
                state.is_dark_mode = is_dark_mode;
                // Note: The actual theme change is handled in the app.rs file
            }

            ui.add_space(10.0);

            if ui.button("Reset to Defaults").clicked() {
                let mut state = self.state.lock().unwrap();
                let profile_name = state.current_config.profile_name.clone();
                state.current_config = crate::modules::config::Config::default();
                state.current_config.profile_name = profile_name;
            }
        });
    }
}
