use eframe::{egui, CreationContext};
use egui::{Context, RichText, Visuals};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use crate::gui::clicker::ClickerThread;
use std::time::Instant;

use crate::modules::config::Config;
use crate::modules::profiles::ProfileManager;
use crate::gui::views::{ProfileView, AreaView, SettingsView, StatsView, ImageView};
use crate::gui::theme::AppTheme;
use crate::gui::components;
use crate::gui::animations::Animation;

/// Enum representing the current view in the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    Profiles,
    Areas,
    Settings,
    Stats,
    Images,
    Running,
}

/// Enum representing the current status of the clicker
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickerStatus {
    Stopped,
    Running,
    Paused,
}

/// Main application state
pub struct AppState {
    pub current_view: AppView,
    pub clicker_status: ClickerStatus,
    pub profile_manager: ProfileManager,
    pub current_config: Config,
    pub profiles_dir: PathBuf,
    pub click_count: u32,
    pub start_time: Option<Instant>,
    pub is_dark_mode: bool,
    pub theme: AppTheme,
    pub view_transition: Animation,
}

impl Default for AppState {
    fn default() -> Self {
        let profiles_dir = PathBuf::from("profiles");
        if !profiles_dir.exists() {
            std::fs::create_dir_all(&profiles_dir).expect("Failed to create profiles directory");
        }

        // Make sure we don't start on the Images view
        Self {
            current_view: AppView::Profiles, // Default view
            clicker_status: ClickerStatus::Stopped,
            profile_manager: ProfileManager::new("profiles"),
            current_config: Config::default(),
            profiles_dir,
            click_count: 0,
            start_time: None,
            is_dark_mode: true,
            theme: AppTheme::dark(),
            view_transition: Animation::new(0.3),
        }
    }
}

/// Main application
pub struct MouseClickerApp {
    state: Arc<Mutex<AppState>>,
    profile_view: ProfileView,
    area_view: AreaView,
    settings_view: SettingsView,
    stats_view: StatsView,
    image_view: ImageView,
    clicker_thread: ClickerThread,
}



impl MouseClickerApp {
    /// Create a new instance of the application
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Set up the initial state
        let state = Arc::new(Mutex::new(AppState::default()));

        // Apply the theme
        {
            let app_state = state.lock().unwrap();
            app_state.theme.apply_to_ctx(&cc.egui_ctx);
        }

        // Create the views
        let profile_view = ProfileView::new(Arc::clone(&state));
        let area_view = AreaView::new(Arc::clone(&state));
        let settings_view = SettingsView::new(Arc::clone(&state));
        let stats_view = StatsView::new(Arc::clone(&state));
        let image_view = ImageView::new(Arc::clone(&state));

        Self {
            state,
            profile_view,
            area_view,
            settings_view,
            stats_view,
            image_view,
            clicker_thread: ClickerThread::new(),
        }
    }

    /// Toggle between light and dark mode
    fn toggle_theme(&self, ctx: &Context) {
        let mut state = self.state.lock().unwrap();
        state.is_dark_mode = !state.is_dark_mode;

        if state.is_dark_mode {
            state.theme = AppTheme::dark();
        } else {
            state.theme = AppTheme::light();
        }

        // Apply the updated theme
        state.theme.apply_to_ctx(ctx);
    }
}

impl eframe::App for MouseClickerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Check if we're on the Images view and redirect if needed
        {
            let mut state = self.state.lock().unwrap();
            if state.current_view == AppView::Images {
                // Uncomment the following line to re-enable redirection
                // state.current_view = AppView::Areas; // Redirect to Areas view
            }
        }
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Profile").clicked() {
                        let mut state = self.state.lock().unwrap();
                        state.current_view = AppView::Profiles;
                        state.view_transition.reset();
                        ui.close_menu();
                    }

                    if ui.button("Save Profile").clicked() {
                        let state = self.state.lock().unwrap();
                        let profile_path = state.profile_manager.get_profile_path(&state.current_config.profile_name);
                        if let Err(e) = state.current_config.save(profile_path.to_str().unwrap()) {
                            eprintln!("Failed to save profile: {}", e);
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        // TODO: Show about dialog
                        ui.close_menu();
                    }
                });

                // Right-aligned status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let state = self.state.lock().unwrap();
                    let theme = &state.theme;

                    // Status indicator with appropriate color
                    let status_text = match state.clicker_status {
                        ClickerStatus::Stopped => RichText::new("Stopped").color(theme.text),
                        ClickerStatus::Running => RichText::new("Running").color(theme.success),
                        ClickerStatus::Paused => RichText::new("Paused").color(theme.warning),
                    };
                    ui.label(status_text);

                    // Profile name
                    ui.label(RichText::new(format!("Profile: {}", state.current_config.profile_name)).strong());

                    // Dark mode toggle
                    let mut is_dark = state.is_dark_mode;
                    if ui.checkbox(&mut is_dark, "Dark Mode").changed() {
                        drop(state); // Drop the lock before calling toggle_theme
                        self.toggle_theme(ctx);
                    }
                });
            });
        });

        // Left sidebar for navigation
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Mouse Clicker");
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(16.0);

                let current_view = {
                    let state = self.state.lock().unwrap();
                    state.current_view
                };

                let theme = {
                    let state = self.state.lock().unwrap();
                    state.theme.clone()
                };

                // Navigation buttons
                if components::sidebar_button(ui, &theme, "Profiles", "📋", current_view == AppView::Profiles) {
                    let mut state = self.state.lock().unwrap();
                    state.current_view = AppView::Profiles;
                    state.view_transition.reset();
                }

                if components::sidebar_button(ui, &theme, "Click Areas", "🎯", current_view == AppView::Areas) {
                    let mut state = self.state.lock().unwrap();
                    state.current_view = AppView::Areas;
                    state.view_transition.reset();
                }

                // Image Recognition feature is temporarily disabled
                // Uncomment the following code to re-enable it
                /*
                if components::sidebar_button(ui, &theme, "Image Recognition", "🔍", current_view == AppView::Images) {
                    let mut state = self.state.lock().unwrap();
                    state.current_view = AppView::Images;
                    state.view_transition.reset();
                }
                */

                if components::sidebar_button(ui, &theme, "Settings", "⚙", current_view == AppView::Settings) {
                    let mut state = self.state.lock().unwrap();
                    state.current_view = AppView::Settings;
                    state.view_transition.reset();
                }

                if components::sidebar_button(ui, &theme, "Statistics", "📊", current_view == AppView::Stats) {
                    let mut state = self.state.lock().unwrap();
                    state.current_view = AppView::Stats;
                    state.view_transition.reset();
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                // Version information
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label(RichText::new("v1.0.0").color(theme.muted_text).small());
                    ui.label(RichText::new("Mouse Clicker").color(theme.muted_text).small());
                });
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            let current_view = {
                let state = self.state.lock().unwrap();
                state.current_view
            };

            let theme = {
                let state = self.state.lock().unwrap();
                state.theme.clone()
            };

            // Add a title for the current view
            let view_title = match current_view {
                AppView::Profiles => "Profile Management",
                AppView::Areas => "Click Areas Configuration",
                AppView::Settings => "Settings",
                AppView::Stats => "Statistics",
                AppView::Images => "Image Recognition",
                AppView::Running => "Running",
            };

            components::section_header(ui, &theme, view_title);

            // Main content
            theme.panel_frame().show(ui, |ui| {
                match current_view {
                    AppView::Profiles => self.profile_view.ui(ui),
                    AppView::Areas => self.area_view.ui(ui),
                    AppView::Settings => self.settings_view.ui(ui),
                    AppView::Stats => self.stats_view.ui(ui),
                    AppView::Images => {
                        // Image Recognition feature is temporarily disabled
                        // Redirect to the Areas view
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.heading("Image Recognition is temporarily disabled");
                            ui.add_space(10.0);
                            ui.label("This feature has been temporarily disabled.");
                            ui.add_space(20.0);

                            if ui.button("Go to Click Areas").clicked() {
                                let mut state = self.state.lock().unwrap();
                                state.current_view = AppView::Areas;
                                state.view_transition.reset();
                            }
                        });
                    },
                    AppView::Running => {
                        // TODO: Implement running view
                        ui.heading("Running");
                        ui.label("The clicker is currently running...");
                    },
                }
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            let theme = {
                let state = self.state.lock().unwrap();
                state.theme.clone()
            };

            theme.section_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let state = self.state.lock().unwrap();

                    if let Some(start_time) = state.start_time {
                        let elapsed = start_time.elapsed();
                        ui.label(RichText::new(format!("Running time: {:.1}s", elapsed.as_secs_f32())).strong());
                        ui.separator();
                    }

                    ui.label(RichText::new(format!("Clicks: {}", state.click_count)).strong());

                    // Right-aligned controls
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let status = state.clicker_status;
                        drop(state); // Drop the lock before UI interactions

                        match status {
                            ClickerStatus::Stopped => {
                                if components::secondary_button(ui, &theme, "Start") {
                                    println!("Start button clicked");
                                    let state_arc = Arc::clone(&self.state);
                                    self.clicker_thread.start(state_arc);
                                }
                            },
                            ClickerStatus::Running => {
                                if components::primary_button(ui, &theme, "Pause") {
                                    self.clicker_thread.pause();
                                    let mut state = self.state.lock().unwrap();
                                    state.clicker_status = ClickerStatus::Paused;
                                }

                                ui.add_space(8.0);

                                if components::accent_button(ui, &theme, "Stop") {
                                    self.clicker_thread.stop();
                                    let mut state = self.state.lock().unwrap();
                                    state.clicker_status = ClickerStatus::Stopped;
                                    state.start_time = None;
                                }
                            },
                            ClickerStatus::Paused => {
                                if components::primary_button(ui, &theme, "Resume") {
                                    self.clicker_thread.resume();
                                    let mut state = self.state.lock().unwrap();
                                    state.clicker_status = ClickerStatus::Running;
                                }

                                ui.add_space(8.0);

                                if components::accent_button(ui, &theme, "Stop") {
                                    self.clicker_thread.stop();
                                    let mut state = self.state.lock().unwrap();
                                    state.clicker_status = ClickerStatus::Stopped;
                                    state.start_time = None;
                                }
                            },
                        }
                    });
                });
            });
        });

        // Request a repaint for animations
        ctx.request_repaint();
    }
}
