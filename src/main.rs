mod modules;
mod gui;

use eframe::egui;
use std::path::Path;

use modules::error::Result;
use gui::MouseClickerApp;

fn main() -> Result<()> {
    // Create profiles directory if it doesn't exist
    let profiles_dir = "profiles";
    if !Path::new(profiles_dir).exists() {
        std::fs::create_dir_all(profiles_dir)?;
    }

    // Set up the native options
    let mut native_options = eframe::NativeOptions::default();

    // Configure the viewport
    let mut viewport = egui::ViewportBuilder::default();
    viewport = viewport.with_inner_size([1024.0, 768.0]);
    viewport = viewport.with_min_inner_size([800.0, 600.0]);
    // Center the window on the screen
    viewport = viewport.with_position(egui::Pos2::new(300.0, 200.0));

    native_options.viewport = viewport;

    // Run the application
    eframe::run_native(
        "Mouse Clicker",
        native_options,
        Box::new(|cc| Ok(Box::new(MouseClickerApp::new(cc))))
    ).map_err(|e| modules::error::AppError::ParseError(format!("Failed to start GUI: {}", e)))?;

    Ok(())
}