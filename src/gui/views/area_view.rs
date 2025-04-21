use egui::{Ui, ScrollArea, Color32, Stroke, Rect, Vec2, Pos2};
use std::sync::{Arc, Mutex};
use enigo::{Enigo, MouseControllable};

use crate::gui::app::AppState;
use crate::modules::config::{ClickArea, AreaSelectionMode};

pub struct AreaView {
    state: Arc<Mutex<AppState>>,
    selected_area_index: Option<usize>,
    new_area: ClickArea,
    is_adding_area: bool,
    screen_width: i32,
    screen_height: i32,
    drag_start: Option<Pos2>,
    current_drag: Option<Rect>,
}

impl AreaView {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        // Get screen dimensions from enigo
        let enigo = enigo::Enigo::new();
        let screen_size = enigo.main_display_size();
        let screen_width = screen_size.0 as i32;
        let screen_height = screen_size.1 as i32;

        Self {
            state,
            selected_area_index: None,
            new_area: ClickArea::default(),
            is_adding_area: false,
            screen_width,
            screen_height,
            drag_start: None,
            current_drag: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Click Area Configuration");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Areas");

                let config = {
                    let state = self.state.lock().unwrap();
                    state.current_config.clone()
                };

                // Primary area
                ui.collapsing("Primary Click Area", |ui| {
                    let mut area = config.click_area.clone();
                    let mut changed = false;

                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        if ui.add(egui::DragValue::new(&mut area.width).speed(1.0).range(10..=2000)).changed() {
                            changed = true;
                        }

                        ui.label("Height:");
                        if ui.add(egui::DragValue::new(&mut area.height).speed(1.0).range(10..=2000)).changed() {
                            changed = true;
                        }
                    });

                    let mut centered = area.centered;
                    if ui.checkbox(&mut centered, "Centered").changed() {
                        area.centered = centered;
                        changed = true;
                    }

                    if !area.centered {
                        ui.horizontal(|ui| {
                            ui.label("X Offset:");
                            if ui.add(egui::DragValue::new(&mut area.x_offset).speed(1.0).range(0..=self.screen_width)).changed() {
                                changed = true;
                            }

                            ui.label("Y Offset:");
                            if ui.add(egui::DragValue::new(&mut area.y_offset).speed(1.0).range(0..=self.screen_height)).changed() {
                                changed = true;
                            }
                        });
                    }

                    if changed {
                        let mut state = self.state.lock().unwrap();
                        state.current_config.click_area = area;
                    }
                });

                ui.add_space(10.0);

                // Multiple areas
                ui.collapsing("Multiple Click Areas", |ui| {
                    let mut multi_enabled = config.multi_area.enabled;
                    if ui.checkbox(&mut multi_enabled, "Enable Multiple Areas").changed() {
                        let mut state = self.state.lock().unwrap();
                        state.current_config.multi_area.enabled = multi_enabled;
                    }

                    if multi_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Selection Mode:");
                            egui::ComboBox::new("area_selection_mode", "Selection Mode")
                                .selected_text(format!("{:?}", config.multi_area.selection_mode))
                                .show_ui(ui, |ui| {
                                    let mut changed = false;
                                    let mut mode = config.multi_area.selection_mode;

                                    if ui.selectable_value(&mut mode, AreaSelectionMode::Sequential, "Sequential").changed() {
                                        changed = true;
                                    }

                                    if ui.selectable_value(&mut mode, AreaSelectionMode::Random, "Random").changed() {
                                        changed = true;
                                    }

                                    if ui.selectable_value(&mut mode, AreaSelectionMode::Weighted, "Weighted").changed() {
                                        changed = true;
                                    }

                                    if changed {
                                        let mut state = self.state.lock().unwrap();
                                        state.current_config.multi_area.selection_mode = mode;
                                    }
                                });
                        });

                        ui.add_space(10.0);

                        // Area list
                        ui.heading("Defined Areas");

                        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for (i, (area, weight)) in config.multi_area.areas.iter().enumerate() {
                                let is_selected = self.selected_area_index == Some(i);
                                let area_text = if area.centered {
                                    format!("Area {}: {}x{} (centered) - Weight: {:.2}", i + 1, area.width, area.height, weight)
                                } else {
                                    format!("Area {}: {}x{} at ({}, {}) - Weight: {:.2}",
                                        i + 1, area.width, area.height, area.x_offset, area.y_offset, weight)
                                };

                                if ui.selectable_label(is_selected, area_text).clicked() {
                                    self.selected_area_index = Some(i);
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Add Area").clicked() {
                                self.is_adding_area = true;
                                self.new_area = ClickArea::default();
                            }

                            let delete_enabled = self.selected_area_index.is_some();
                            if ui.add_enabled(delete_enabled, egui::Button::new("Remove")).clicked() {
                                if let Some(index) = self.selected_area_index {
                                    let mut state = self.state.lock().unwrap();
                                    if state.current_config.remove_click_area(index) {
                                        self.selected_area_index = None;
                                    }
                                }
                            }

                            if ui.button("Clear All").clicked() {
                                let mut state = self.state.lock().unwrap();
                                state.current_config.clear_click_areas();
                                self.selected_area_index = None;
                            }
                        });

                        // Add new area dialog
                        if self.is_adding_area {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.heading("Add New Area");

                            ui.horizontal(|ui| {
                                ui.label("Width:");
                                ui.add(egui::DragValue::new(&mut self.new_area.width).speed(1.0).range(10..=2000));

                                ui.label("Height:");
                                ui.add(egui::DragValue::new(&mut self.new_area.height).speed(1.0).range(10..=2000));
                            });

                            ui.checkbox(&mut self.new_area.centered, "Centered");

                            if !self.new_area.centered {
                                ui.horizontal(|ui| {
                                    ui.label("X Offset:");
                                    ui.add(egui::DragValue::new(&mut self.new_area.x_offset).speed(1.0).range(0..=self.screen_width));

                                    ui.label("Y Offset:");
                                    ui.add(egui::DragValue::new(&mut self.new_area.y_offset).speed(1.0).range(0..=self.screen_height));
                                });
                            }

                            let mut weight = 1.0;
                            if config.multi_area.selection_mode == AreaSelectionMode::Weighted {
                                ui.horizontal(|ui| {
                                    ui.label("Weight:");
                                    ui.add(egui::Slider::new(&mut weight, 0.01..=1.0).text("Weight"));
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button("Add").clicked() {
                                    let mut state = self.state.lock().unwrap();
                                    state.current_config.add_click_area(self.new_area.clone(), weight);
                                    self.is_adding_area = false;
                                }

                                if ui.button("Cancel").clicked() {
                                    self.is_adding_area = false;
                                }
                            });
                        }
                    }
                });
            });

            ui.separator();

            // Screen preview
            ui.vertical(|ui| {
                ui.heading("Screen Preview");

                let config = {
                    let state = self.state.lock().unwrap();
                    state.current_config.clone()
                };

                // Calculate the preview size
                let available_size = ui.available_size();
                let aspect_ratio = self.screen_width as f32 / self.screen_height as f32;
                let preview_width = available_size.x.min(400.0);
                let preview_height = preview_width / aspect_ratio;

                // Draw the screen preview
                let (response, painter) = ui.allocate_painter(Vec2::new(preview_width, preview_height), egui::Sense::drag());

                // Draw screen background
                painter.rect_filled(
                    Rect::from_min_size(response.rect.min, Vec2::new(preview_width, preview_height)),
                    0.0,
                    Color32::from_rgb(30, 30, 30)
                );

                // Scale factor for converting between screen and preview coordinates
                let scale_x = preview_width / self.screen_width as f32;
                let scale_y = preview_height / self.screen_height as f32;

                // Draw primary area
                let primary_area = &config.click_area;
                let (x, y) = if primary_area.centered {
                    let center_x = self.screen_width / 2;
                    let center_y = self.screen_height / 2;
                    (center_x - primary_area.width / 2, center_y - primary_area.height / 2)
                } else {
                    (primary_area.x_offset, primary_area.y_offset)
                };

                let rect = Rect::from_min_size(
                    Pos2::new(
                        response.rect.min.x + x as f32 * scale_x,
                        response.rect.min.y + y as f32 * scale_y
                    ),
                    Vec2::new(
                        primary_area.width as f32 * scale_x,
                        primary_area.height as f32 * scale_y
                    )
                );
                // Draw the rectangle outline
                painter.rect_filled(rect, 0.0, Color32::TRANSPARENT);
                painter.rect_stroke(rect, 0.0, Stroke::new(2.0, Color32::GREEN), egui::epaint::StrokeKind::Middle);

                // Draw multiple areas if enabled
                if config.multi_area.enabled {
                    for (i, (area, _)) in config.multi_area.areas.iter().enumerate() {
                        let is_selected = self.selected_area_index == Some(i);
                        let color = if is_selected { Color32::YELLOW } else { Color32::BLUE };

                        let (x, y) = if area.centered {
                            let center_x = self.screen_width / 2;
                            let center_y = self.screen_height / 2;
                            (center_x - area.width / 2, center_y - area.height / 2)
                        } else {
                            (area.x_offset, area.y_offset)
                        };

                        let rect = Rect::from_min_size(
                            Pos2::new(
                                response.rect.min.x + x as f32 * scale_x,
                                response.rect.min.y + y as f32 * scale_y
                            ),
                            Vec2::new(
                                area.width as f32 * scale_x,
                                area.height as f32 * scale_y
                            )
                        );
                        // Draw the rectangle outline
                        painter.rect_filled(rect, 0.0, Color32::TRANSPARENT);
                        painter.rect_stroke(rect, 0.0, Stroke::new(2.0, color), egui::epaint::StrokeKind::Middle);
                    }
                }

                // Handle drag to create new area
                if response.dragged() {
                    if self.drag_start.is_none() {
                        self.drag_start = Some(response.interact_pointer_pos().unwrap());
                    }

                    let current_pos = response.interact_pointer_pos().unwrap();
                    let start_pos = self.drag_start.unwrap();

                    let rect = Rect::from_two_pos(start_pos, current_pos);
                    self.current_drag = Some(rect);

                    // Draw the current drag rectangle
                    // Draw the rectangle outline
                    painter.rect_filled(rect, 0.0, Color32::TRANSPARENT);
                    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::RED), egui::epaint::StrokeKind::Middle);
                } else if response.drag_stopped() && self.drag_start.is_some() {
                    // Convert the drag rectangle to screen coordinates
                    if let Some(rect) = self.current_drag {
                        let min_x = ((rect.min.x - response.rect.min.x) / scale_x) as i32;
                        let min_y = ((rect.min.y - response.rect.min.y) / scale_y) as i32;
                        let width = (rect.width() / scale_x) as i32;
                        let height = (rect.height() / scale_y) as i32;

                        // Create a new area from the drag
                        let new_area = ClickArea {
                            width: width.max(10),
                            height: height.max(10),
                            centered: false,
                            x_offset: min_x,
                            y_offset: min_y,
                        };

                        // Add the area
                        let mut state = self.state.lock().unwrap();
                        if state.current_config.multi_area.enabled {
                            state.current_config.add_click_area(new_area, 1.0);
                        } else {
                            state.current_config.click_area = new_area;
                        }
                    }

                    self.drag_start = None;
                    self.current_drag = None;
                }

                ui.add_space(10.0);
                ui.label("Drag on the preview to create a new area");
                ui.label(format!("Screen size: {}x{}", self.screen_width, self.screen_height));

                if ui.button("Update Screen Size").clicked() {
                    // Get screen size from enigo
                    let enigo = enigo::Enigo::new();
                    let screen_size = enigo.main_display_size();
                    self.screen_width = screen_size.0 as i32;
                    self.screen_height = screen_size.1 as i32;
                }
            });
        });
    }
}
