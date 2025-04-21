use egui::{Ui, ScrollArea};
use std::sync::{Arc, Mutex};

use crate::gui::app::AppState;
use crate::modules::config::Config;

pub struct ProfileView {
    state: Arc<Mutex<AppState>>,
    new_profile_name: String,
    selected_profile_index: Option<usize>,
}

impl ProfileView {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
            new_profile_name: String::new(),
            selected_profile_index: None,
        }
    }
    
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Profile Management");
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("New Profile:");
            ui.text_edit_singleline(&mut self.new_profile_name);
            
            if ui.button("Create").clicked() && !self.new_profile_name.is_empty() {
                let mut state = self.state.lock().unwrap();
                let mut config = Config::default();
                config.profile_name = self.new_profile_name.clone();
                
                let profile_path = state.profile_manager.get_profile_path(&config.profile_name);
                if let Err(e) = config.save(profile_path.to_str().unwrap()) {
                    eprintln!("Failed to save profile: {}", e);
                } else {
                    state.current_config = config;
                    self.new_profile_name.clear();
                }
            }
        });
        
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Available Profiles");
                
                let profiles = {
                    let state = self.state.lock().unwrap();
                    state.profile_manager.list_profiles()
                };
                
                ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for (i, profile_name) in profiles.iter().enumerate() {
                        let is_selected = self.selected_profile_index == Some(i);
                        if ui.selectable_label(is_selected, profile_name).clicked() {
                            self.selected_profile_index = Some(i);
                            
                            // Load the selected profile
                            let mut state = self.state.lock().unwrap();
                            if let Ok(config) = state.profile_manager.load_profile(profile_name) {
                                state.current_config = config;
                            }
                        }
                    }
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    let delete_enabled = self.selected_profile_index.is_some();
                    if ui.add_enabled(delete_enabled, egui::Button::new("Delete")).clicked() {
                        if let Some(index) = self.selected_profile_index {
                            let state = self.state.lock().unwrap();
                            let profiles = state.profile_manager.list_profiles();
                            if index < profiles.len() {
                                let profile_name = &profiles[index];
                                if let Err(e) = state.profile_manager.delete_profile(profile_name) {
                                    eprintln!("Failed to delete profile: {}", e);
                                } else {
                                    self.selected_profile_index = None;
                                }
                            }
                        }
                    }
                    
                    if ui.button("Refresh").clicked() {
                        // Just refresh the view
                    }
                });
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.heading("Profile Details");
                
                let config = {
                    let state = self.state.lock().unwrap();
                    state.current_config.clone()
                };
                
                ui.label(format!("Name: {}", config.profile_name));
                ui.label(format!("Click Area: {}x{}", config.click_area.width, config.click_area.height));
                
                let click_type = if config.click_options.randomize_click_type {
                    "Random (weighted)".to_string()
                } else {
                    format!("{:?}", config.click_options.click_type)
                };
                
                ui.label(format!("Click Type: {}", click_type));
                
                ui.label(format!("Multiple Areas: {}", if config.multi_area.enabled { "Yes" } else { "No" }));
                
                if config.multi_area.enabled {
                    ui.label(format!("Area Count: {}", config.multi_area.areas.len()));
                    ui.label(format!("Selection Mode: {:?}", config.multi_area.selection_mode));
                }
                
                ui.add_space(20.0);
                
                if ui.button("Edit Profile").clicked() {
                    let mut state = self.state.lock().unwrap();
                    state.current_view = crate::gui::app::AppView::Areas;
                }
            });
        });
    }
}
