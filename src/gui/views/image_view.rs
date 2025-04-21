use eframe::egui::{self, RichText, Ui, TextureHandle, TextureOptions, Stroke};
use eframe::epaint::CornerRadius;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::time::Instant;
use std::collections::HashMap;
use chrono;

use crate::gui::app::AppState;
use crate::modules::image_recognition::{ImageLibrary, base64_to_image};

pub struct ImageView {
    state: Arc<Mutex<AppState>>,
    image_library: Arc<Mutex<ImageLibrary>>,
    selected_target_id: Option<String>,
    screenshot_area: Option<(i32, i32, u32, u32)>,
    is_selecting_area: bool,
    selection_start: Option<egui::Pos2>,
    preview_texture: Option<TextureHandle>,
    screen_texture: Option<TextureHandle>,  // Added for screen capture
    target_textures: HashMap<String, TextureHandle>,  // Store textures for each target
    last_search_result: Option<(i32, i32)>,
    last_search_time: Option<Instant>,
}

impl ImageView {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        // Create the image library
        let targets_dir = PathBuf::from("targets");
        if !targets_dir.exists() {
            std::fs::create_dir_all(&targets_dir).expect("Failed to create targets directory");
        }

        let mut image_library = ImageLibrary::new("targets");
        let _ = image_library.load_targets(); // Ignore errors on initial load

        Self {
            state,
            image_library: Arc::new(Mutex::new(image_library)),
            selected_target_id: None,
            screenshot_area: None,
            is_selecting_area: false,
            selection_start: None,
            preview_texture: None,
            screen_texture: None,
            target_textures: HashMap::new(),
            last_search_result: None,
            last_search_time: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        // Handle area selection if active
        if self.is_selecting_area {
            let input = ui.ctx().input(|i| i.clone());

            // Check for ESC key to cancel selection
            if input.key_pressed(egui::Key::Escape) {
                self.is_selecting_area = false;
                self.selection_start = None;
                self.screen_texture = None; // Clear the screen texture

                println!("Selection cancelled with ESC key");
                println!("Selection cancelled");

                return; // Skip the rest of the selection logic
            }

            // Display the screen capture as a full-screen image
            if let Some(texture) = &self.screen_texture {
                // Create a fullscreen area for the screenshot
                let screen_rect = ui.max_rect();

                // Display the screenshot
                ui.put(screen_rect, egui::Image::new(texture));

                // Check for mouse press to start selection
                if input.pointer.any_pressed() && self.selection_start.is_none() {
                    if let Some(pos) = input.pointer.interact_pos() {
                        self.selection_start = Some(pos);
                        println!("Selection started at: {:?}", pos);
                    }
                }

                // Check for mouse release to end selection
                if input.pointer.any_released() && self.selection_start.is_some() {
                    if let Some(pos) = input.pointer.interact_pos() {
                        if let Some(start) = self.selection_start {
                            // Calculate the rectangle
                            let min_x = start.x.min(pos.x) as i32;
                            let min_y = start.y.min(pos.y) as i32;
                            let width = (start.x - pos.x).abs() as u32;
                            let height = (start.y - pos.y).abs() as u32;

                            // Only create a selection if it has some size
                            if width > 5 && height > 5 {
                                // Store the selection coordinates relative to the screen
                                self.screenshot_area = Some((min_x, min_y, width, height));
                                println!("Selection completed: {:?}", self.screenshot_area);

                                // Show message in console
                                println!("Selected area: {}x{} at ({}, {})", width, height, min_x, min_y);

                                // We'll create a preview when the form is submitted
                                // The screenshot_area coordinates will be used to capture the actual area
                            } else {
                                // Selection too small
                                println!("Selection too small - please try again");
                            }

                            self.selection_start = None;
                            self.is_selecting_area = false;
                            self.screen_texture = None; // Clear the screen texture

                            println!("Selection completed");
                        }
                    }
                }

                // Draw the current selection rectangle if we're in the process of selecting
                if let Some(start) = self.selection_start {
                    if let Some(current_pos) = input.pointer.hover_pos() {
                        let rect = egui::Rect::from_two_pos(start, current_pos);

                        // Paint the selection rectangle on screen
                        let painter = ui.painter();
                        painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0)), egui::StrokeKind::Middle);
                        painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_premultiplied(255, 165, 0, 30));

                        // Show the dimensions as text
                        let width = (current_pos.x - start.x).abs() as u32;
                        let height = (current_pos.y - start.y).abs() as u32;
                        let text = format!("{} x {}", width, height);

                        let text_pos = egui::pos2(
                            (start.x + current_pos.x) / 2.0,
                            (start.y + current_pos.y) / 2.0
                        );

                        painter.text(
                            text_pos,
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE
                        );
                    }
                }

                // Add instructions at the top of the screen
                let painter = ui.painter();
                painter.text(
                    egui::pos2(ui.max_rect().center().x, 30.0),
                    egui::Align2::CENTER_CENTER,
                    "Click and drag to select an area. Press ESC to cancel.",
                    egui::FontId::proportional(18.0),
                    egui::Color32::WHITE
                );

                // Skip the rest of the UI when in selection mode
                return;
            }
        }
        // Get the theme for consistent styling
        let theme = {
            let state = self.state.lock().unwrap();
            state.theme.clone()
        };

        // Main header
        ui.heading(RichText::new("Image Recognition").size(24.0));

        // Add a brief explanation with better styling
        ui.add_space(4.0);
        ui.label(RichText::new("This feature allows you to capture and recognize images on your screen.").italics());
        ui.add_space(12.0);

        // Use a split layout with proper spacing
        let panel_height = ui.available_height() - 200.0; // Reserve space for bottom section

        // Create a frame for the main content area
        theme.panel_frame().show(ui, |ui| {
            // Split the main area into two panels
            egui::SidePanel::left("image_targets_panel")
                .resizable(true)
                .min_width(200.0)
                .default_width(250.0)
                .max_width(350.0)
                .show_inside(ui, |ui| {
                    ui.heading(RichText::new("Target Images").strong());
                    ui.add_space(8.0);
                    self.target_list_ui(ui);
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                // Use a vertical layout for better organization
                egui::TopBottomPanel::top("image_details_panel")
                    .resizable(true)
                    .min_height(150.0)
                    .max_height(panel_height * 0.4)
                    .show_inside(ui, |ui| {
                        ui.heading(RichText::new("Target Details").strong());
                        ui.add_space(8.0);
                        self.target_details_ui(ui);
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading(RichText::new("Preview").strong());
                    ui.add_space(8.0);
                    self.preview_ui(ui);
                });
            });
        });

        ui.add_space(16.0);

        // Bottom section with capture area and search results
        theme.panel_frame().show(ui, |ui| {
            egui::Grid::new("image_bottom_grid")
                .num_columns(2)
                .spacing([20.0, 0.0])
                .show(ui, |ui| {
                    // Capture Area column
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("Capture Area").strong());
                        ui.add_space(8.0);
                        self.capture_area_ui(ui);
                    });

                    // Search Results column
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("Search Results").strong());
                        ui.add_space(8.0);

                        if let Some(time) = self.last_search_time {
                            ui.label(RichText::new(format!("Last search: {:.2}s ago", time.elapsed().as_secs_f32())).monospace());
                            ui.add_space(4.0);
                        }

                        if let Some((x, y)) = self.last_search_result {
                            ui.label(RichText::new(format!("Found at: ({}, {})", x, y)).strong());
                            ui.add_space(8.0);

                            // Make the button green to stand out
                            if theme.secondary_button(ui, "Click at this position") {
                                // Use the mouse module to click at this position
                                let mut enigo = enigo::Enigo::new();
                                if let Err(e) = crate::modules::mouse::simulate_human_movement(
                                    &mut enigo,
                                    x,
                                    y,
                                    &mut rand::thread_rng()
                                ) {
                                    eprintln!("Failed to move mouse: {}", e);
                                } else {
                                    // Perform a click
                                    if let Err(e) = crate::modules::mouse::human_like_click(
                                        &mut enigo,
                                        &mut rand::thread_rng(),
                                        &self.state.lock().unwrap().current_config
                                    ) {
                                        eprintln!("Failed to click: {}", e);
                                    }
                                }
                            }
                        } else if self.selected_target_id.is_some() {
                            ui.label(RichText::new("Target not found on screen").italics());
                        } else {
                            ui.label(RichText::new("Select a target and click 'Find on Screen'").italics());
                        }
                    });

                    ui.end_row();
                });
        });
    }

    fn target_list_ui(&mut self, ui: &mut Ui) {
        // Get the theme for consistent styling
        let theme = {
            let state = self.state.lock().unwrap();
            state.theme.clone()
        };

        let image_library = self.image_library.lock().unwrap();
        let targets = image_library.get_targets();

        // Show a message if no targets are available
        if targets.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No target images available.").strong());
                ui.add_space(4.0);
                ui.label(RichText::new("Click 'New Target' to create one.").italics());
                ui.add_space(20.0);
            });
        } else {
            // Create a scrollable area with fixed height
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for target in targets {
                        let is_selected = self.selected_target_id.as_ref().map_or(false, |id| id == &target.id);

                        // Create a frame for each target for better visual separation
                        let frame = egui::Frame::new()
                            .fill(if is_selected { theme.selected } else { theme.panel_background })
                            .stroke(Stroke::new(1.0, if is_selected { theme.primary } else { theme.border }))
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                            .outer_margin(4.0);

                        frame.show(ui, |ui| {
                            // Use a vertical layout for better organization
                            ui.vertical(|ui| {
                                // Target name as header
                                let text_color = if is_selected { theme.primary } else { theme.text };
                                ui.label(RichText::new(&target.name).color(text_color).strong().size(16.0));

                                ui.add_space(4.0);

                                // Show a preview for each target
                                ui.label(RichText::new("Image data length: ").italics().size(10.0).color(theme.muted_text));
                                ui.label(RichText::new(format!("{} bytes", target.image_data.len())).italics().size(10.0).color(theme.muted_text));

                                // Check if the image data is valid
                                if target.image_data.is_empty() {
                                    ui.label(RichText::new("No image data available").color(theme.warning));
                                } else {
                                    println!("Loading image for target {}, data length: {}", target.id, target.image_data.len());

                                    // Load the target image from base64
                                    match base64_to_image(&target.image_data) {
                                        Ok(img) => {
                                            println!("Successfully loaded image for target {}, dimensions: {}x{}",
                                                target.id, img.width(), img.height());

                                            let rgba_image = img.to_rgba8();
                                            let pixels = rgba_image.as_flat_samples();
                                            let size = [rgba_image.width() as usize, rgba_image.height() as usize];

                                            // Create a texture for this target if we don't have it already
                                            let target_texture = if !self.target_textures.contains_key(&target.id) {
                                                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                                let texture = ui.ctx().load_texture(
                                                    format!("target_{}", target.id),
                                                    color_image,
                                                    TextureOptions::default()
                                                );

                                                // Store the texture in our HashMap
                                                self.target_textures.insert(target.id.clone(), texture.clone());
                                                println!("Created and stored texture for target {}", target.id);
                                                texture
                                            } else {
                                                // Use the existing texture
                                                self.target_textures.get(&target.id).unwrap().clone()
                                            };

                                            // Display the image with automatic sizing
                                            ui.horizontal(|ui| {
                                                ui.add_space(8.0); // Indent the image
                                                ui.image(&target_texture);
                                            });

                                            // If this is the selected target, update the preview texture
                                            if is_selected {
                                                println!("Setting preview texture for selected target {}", target.id);
                                                self.preview_texture = Some(target_texture);
                                            }
                                        },
                                        Err(e) => {
                                            ui.label(RichText::new(format!("Failed to load image: {}", e)).color(theme.warning));
                                            eprintln!("Failed to load image for target {}: {}", target.id, e);
                                        }
                                    }
                                }

                                // Add a selectable area for the entire frame
                                if ui.selectable_label(is_selected, "").clicked() {
                                    self.selected_target_id = Some(target.id.clone());

                                    // The preview texture is already set above when we display the image
                                    // No need to reload it here
                                }
                            });
                        });
                    }
                });
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            // Use themed buttons for consistency
            if theme.secondary_button(ui, "New Target") {
                // Instead of minimizing, we'll take a screenshot first and then allow selection
                match crate::modules::image_recognition::capture_screen() {
                    Ok(screenshot) => {
                        // Convert to a texture we can display
                        // The screenshot is already in RGBA format
                        let pixels = screenshot.as_flat_samples();
                        let size = [screenshot.width() as usize, screenshot.height() as usize];

                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                        let screen_texture = ui.ctx().load_texture(
                            "screen_capture",
                            color_image,
                            TextureOptions::default()
                        );

                        // Store the texture and start selection mode
                        self.screen_texture = Some(screen_texture);
                        self.is_selecting_area = true;
                        self.selection_start = None;
                        self.screenshot_area = None;

                        println!("Starting area selection - use mouse to select an area");
                    },
                    Err(e) => {
                        eprintln!("Failed to capture screen: {}", e);
                    }
                }
            }

            ui.add_space(8.0);

            if let Some(target_id) = &self.selected_target_id {
                if theme.accent_button(ui, "Delete") {
                    let mut image_library = self.image_library.lock().unwrap();
                    if let Err(e) = image_library.delete_target(target_id) {
                        eprintln!("Failed to delete target: {}", e);
                    } else {
                        // Remove the texture from our HashMap
                        self.target_textures.remove(target_id);
                        println!("Removed texture for deleted target {}", target_id);

                        self.selected_target_id = None;
                        self.preview_texture = None;
                    }
                }
            }
        });
    }

    fn target_details_ui(&mut self, ui: &mut Ui) {
        // Get the theme for consistent styling
        let theme = {
            let state = self.state.lock().unwrap();
            state.theme.clone()
        };

        if let Some(target_id) = &self.selected_target_id {
            let image_library = self.image_library.lock().unwrap();
            if let Some(target) = image_library.get_targets().iter().find(|t| &t.id == target_id) {
                // Create a frame for the details
                theme.card_frame().show(ui, |ui| {
                    // Use a grid layout for better organization
                    egui::Grid::new("target_details_grid")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // ID row
                            ui.label(RichText::new("ID:").strong());
                            ui.label(RichText::new(&target.id).monospace().size(14.0));
                            ui.end_row();

                            // Name row
                            ui.label(RichText::new("Name:").strong());
                            ui.label(RichText::new(&target.name).size(14.0));
                            ui.end_row();

                            // Threshold row
                            ui.label(RichText::new("Threshold:").strong());
                            ui.label(RichText::new(format!("{:.2}", target.threshold)).size(14.0));
                            ui.end_row();

                            // Click offset row
                            ui.label(RichText::new("Click offset:").strong());
                            if let Some((x, y)) = target.click_offset {
                                ui.label(RichText::new(format!("({}, {})", x, y)).size(14.0));
                            } else {
                                ui.label(RichText::new("Center").size(14.0));
                            }
                            ui.end_row();
                        });
                });

                ui.add_space(12.0);

                // Add a search button with theme styling
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme.primary_button(ui, "Find on Screen") {
                            let image_library = self.image_library.lock().unwrap();
                            match image_library.find_on_screen(&target.id) {
                                Ok(Some((x, y))) => {
                                    self.last_search_result = Some((x, y));
                                    self.last_search_time = Some(Instant::now());
                                },
                                Ok(None) => {
                                    self.last_search_result = None;
                                    self.last_search_time = Some(Instant::now());
                                },
                                Err(e) => {
                                    eprintln!("Failed to search for target: {}", e);
                                    self.last_search_result = None;
                                    self.last_search_time = Some(Instant::now());
                                }
                            }
                        }
                    });
                });

                // Add a help text
                ui.add_space(8.0);
                ui.label(RichText::new("Click 'Find on Screen' to locate this image on your screen.").italics().size(12.0));
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("Target not found in library").color(theme.warning));
                    ui.add_space(20.0);
                });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No target selected").strong());
                ui.add_space(4.0);
                ui.label(RichText::new("Select a target from the list or create a new one.").italics());
                ui.add_space(20.0);
            });
        }
    }

    fn capture_area_ui(&mut self, ui: &mut Ui) {
        // Get the theme for consistent styling
        let theme = {
            let state = self.state.lock().unwrap();
            state.theme.clone()
        };

        if self.is_selecting_area {
            // Show instructions with a colored background
            theme.card_frame().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    let text = RichText::new("Click and drag to select an area on the screen")
                        .color(theme.warning)
                        .strong();
                    ui.label(text);
                    ui.add_space(8.0);
                });
            });

            ui.add_space(12.0);

            // Cancel button with theme styling
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme.accent_button(ui, "Cancel") {
                    self.is_selecting_area = false;
                    self.selection_start = None;
                    self.screenshot_area = None;
                    self.screen_texture = None; // Clear the screen texture

                    println!("Selection cancelled");
                }
            });
        } else if let Some((x, y, width, height)) = self.screenshot_area {
            // Create a preview of the selected area if we don't have one yet
            if self.preview_texture.is_none() {
                // Take a screenshot of the specified region
                match crate::modules::image_recognition::capture_screen_area(x, y, width, height) {
                    Ok(screenshot) => {
                        // Convert to a texture we can display
                        let pixels = screenshot.as_flat_samples();
                        let size = [screenshot.width() as usize, screenshot.height() as usize];

                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                        let texture = ui.ctx().load_texture(
                            "area_preview",
                            color_image,
                            TextureOptions::default()
                        );

                        // Store the preview texture
                        self.preview_texture = Some(texture);

                        println!("Created preview of selected area: {}x{}", width, height);
                    },
                    Err(e) => {
                        eprintln!("Failed to capture screen area: {}", e);
                    }
                }
            }

            // Show the selected area details in a themed frame
            theme.card_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    // Area details
                    ui.label(RichText::new("Selected Area").strong().size(16.0));
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("Size: {}x{} pixels", width, height)).monospace());
                    ui.label(RichText::new(format!("Position: ({}, {})", x, y)).monospace());
                    ui.add_space(12.0);

                    // Form for saving the target
                    let mut name = String::new();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Name:").strong());
                        ui.add_space(8.0);
                        ui.add(egui::TextEdit::singleline(&mut name).hint_text("Enter target name"));
                    });

                    ui.add_space(8.0);

                    // Add threshold slider with better styling
                    let mut threshold = 0.8; // Default threshold
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Threshold:").strong());
                        ui.add_space(8.0);
                        ui.add(egui::Slider::new(&mut threshold, 0.5..=0.95)
                            .text("Match Precision")
                            .trailing_fill(true));
                    });

                    ui.add_space(8.0);

                    // Add click offset options with better styling
                    let mut use_center = true;
                    let mut offset_x = 0;
                    let mut offset_y = 0;

                    ui.label(RichText::new("Click Position").strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut use_center, true, "Center of image");
                        ui.add_space(16.0);
                        ui.radio_value(&mut use_center, false, "Custom offset");
                    });

                    if !use_center {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0); // Indent
                            ui.label("X offset:");
                            ui.add(egui::DragValue::new(&mut offset_x).speed(1.0));
                            ui.add_space(16.0);
                            ui.label("Y offset:");
                            ui.add(egui::DragValue::new(&mut offset_y).speed(1.0));
                        });
                    }

                    ui.add_space(16.0);

                    // Save button with theme styling
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button_enabled = !name.is_empty();
                        let button_text = if button_enabled { "Save Target" } else { "Enter a name first" };

                        if button_enabled && theme.secondary_button(ui, button_text) {
                            let id = format!("target_{}", chrono::Utc::now().timestamp());

                            let click_offset = if use_center {
                                None
                            } else {
                                Some((offset_x, offset_y))
                            };

                            let mut image_library = self.image_library.lock().unwrap();
                            match image_library.create_target_from_screenshot(
                                &id,
                                &name,
                                x,
                                y,
                                width,
                                height,
                                threshold,
                                click_offset,
                            ) {
                                Ok(target) => {
                                    println!("Created target with ID: {}, name: {}, image data length: {}",
                                        target.id, target.name, target.image_data.len());

                                    if let Err(e) = image_library.save_target(&target) {
                                        eprintln!("Failed to save target: {}", e);
                                    } else {
                                        println!("Successfully saved target to disk");

                                        // Reload the targets
                                        if let Err(e) = image_library.load_targets() {
                                            eprintln!("Failed to reload targets: {}", e);
                                        } else {
                                            println!("Successfully reloaded targets, count: {}", image_library.get_targets().len());
                                        }

                                        // Store the preview texture in our target textures HashMap
                                        if let Some(texture) = &self.preview_texture {
                                            self.target_textures.insert(id.clone(), texture.clone());
                                            println!("Stored texture for new target {}", id);
                                        }

                                        self.selected_target_id = Some(id.clone());
                                        self.screenshot_area = None;
                                        // Keep the preview texture as it will be used to display the target

                                        println!("Set selected target ID to: {}", id);
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Failed to create target: {}", e);
                                }
                            }
                        } else if !button_enabled {
                            // Disabled button styling
                            let button = egui::Button::new(RichText::new(button_text).color(theme.muted_text))
                                .fill(theme.panel_background)
                                .stroke(egui::Stroke::new(1.0, theme.border));
                            ui.add_enabled(false, button);
                        }
                    });
                });
            });
        } else {
            // Select area button with theme styling
            theme.card_frame().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new("No area selected").strong());
                    ui.add_space(8.0);

                    if theme.primary_button(ui, "Select Area") {
                        // Instead of minimizing, we'll take a screenshot first and then allow selection
                        match crate::modules::image_recognition::capture_screen() {
                            Ok(screenshot) => {
                                // Convert to a texture we can display
                                // The screenshot is already in RGBA format
                                let pixels = screenshot.as_flat_samples();
                                let size = [screenshot.width() as usize, screenshot.height() as usize];

                                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                let screen_texture = ui.ctx().load_texture(
                                    "screen_capture",
                                    color_image,
                                    TextureOptions::default()
                                );

                                // Store the texture and start selection mode
                                self.screen_texture = Some(screen_texture);
                                self.is_selecting_area = true;
                                self.selection_start = None;

                                println!("Starting area selection - use mouse to select an area");
                            },
                            Err(e) => {
                                eprintln!("Failed to capture screen: {}", e);
                            }
                        }
                    }

                    ui.add_space(8.0);
                    ui.label(RichText::new("Click 'Select Area' to capture a region of the screen.").italics().size(12.0));
                    ui.add_space(8.0);
                });
            });
        }
    }

    fn preview_ui(&mut self, ui: &mut Ui) {
        // Get the theme for consistent styling
        let theme = {
            let state = self.state.lock().unwrap();
            state.theme.clone()
        };

        // Create a frame for the preview area
        theme.card_frame().show(ui, |ui| {
            // Try to get the texture from either the preview or the target textures
            let texture = if let Some(texture) = &self.preview_texture {
                // Use the preview texture if available
                Some(texture)
            } else if let Some(target_id) = &self.selected_target_id {
                // Try to get the texture from the target textures
                self.target_textures.get(target_id).map(|t| t)
            } else {
                None
            };

            if let Some(texture) = texture {
                // Center the image in the available space
                // No need to scale the image as egui handles it automatically
                // Just add some spacing for better layout

                // Center the image
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.image(texture);
                    ui.add_space(8.0);

                    // Add image dimensions with better styling
                    let size = texture.size();
                    ui.label(RichText::new(format!("Dimensions: {}x{} pixels", size[0], size[1]))
                        .monospace()
                        .size(14.0));
                });

                // If we're showing a target texture but don't have a preview texture,
                // set the preview texture to the target texture for consistency
                if self.preview_texture.is_none() && self.selected_target_id.is_some() {
                    println!("Setting preview texture from target texture");
                    self.preview_texture = Some(texture.clone());
                }
            } else {
                // Show a placeholder when no image is available
                let available_size = ui.available_size();
                let min_height = 200.0;

                // Create a placeholder rectangle
                let rect = egui::Rect::from_min_size(
                    ui.min_rect().min,
                    egui::vec2(available_size.x, available_size.y.min(min_height))
                );

                ui.allocate_rect(rect, egui::Sense::hover());

                // Center the text
                ui.vertical_centered(|ui| {
                    ui.add_space(rect.height() / 3.0); // Push down to center vertically
                    ui.label(RichText::new("No preview available").color(theme.muted_text).size(16.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new("Select a target from the list or create a new one.")
                        .color(theme.muted_text)
                        .italics()
                        .size(14.0));
                });
            }
        });
    }
}
