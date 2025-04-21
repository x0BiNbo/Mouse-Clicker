use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::modules::error::{AppError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ClickType {
    Single,
    Double,
    Right,
    Middle,
}

impl Default for ClickType {
    fn default() -> Self {
        ClickType::Single
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickArea {
    pub width: i32,
    pub height: i32,
    pub centered: bool,
    pub x_offset: i32,
    pub y_offset: i32,
}

impl Default for ClickArea {
    fn default() -> Self {
        Self {
            width: 200,
            height: 200,
            centered: true,
            x_offset: 0,
            y_offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickTiming {
    pub min_delay: f32,
    pub max_delay: f32,
    pub click_duration_mean: f64,
    pub click_duration_std_dev: f64,
    pub double_click_gap: u64,  // Time between clicks in a double-click (ms)
}

impl Default for ClickTiming {
    fn default() -> Self {
        Self {
            min_delay: 12.0,
            max_delay: 38.0,
            click_duration_mean: 80.0,
            click_duration_std_dev: 20.0,
            double_click_gap: 200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickOptions {
    pub click_type: ClickType,
    pub randomize_click_type: bool,
    pub click_type_weights: Vec<(ClickType, f32)>,  // (click_type, weight)
}

impl Default for ClickOptions {
    fn default() -> Self {
        Self {
            click_type: ClickType::Single,
            randomize_click_type: false,
            click_type_weights: vec![
                (ClickType::Single, 0.7),
                (ClickType::Double, 0.1),
                (ClickType::Right, 0.1),
                (ClickType::Middle, 0.1),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AreaSelectionMode {
    Sequential,  // Go through areas in order
    Random,      // Pick a random area each time
    Weighted,    // Pick a random area based on weights
}

impl Default for AreaSelectionMode {
    fn default() -> Self {
        AreaSelectionMode::Sequential
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAreaConfig {
    pub areas: Vec<(ClickArea, f32)>,  // (area, weight)
    pub selection_mode: AreaSelectionMode,
    pub enabled: bool,
}

impl Default for MultiAreaConfig {
    fn default() -> Self {
        Self {
            areas: Vec::new(),
            selection_mode: AreaSelectionMode::default(),
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub profile_name: String,
    pub click_area: ClickArea,         // Primary click area (for backward compatibility)
    pub click_timing: ClickTiming,
    pub click_options: ClickOptions,
    pub multi_area: MultiAreaConfig,   // Multiple click areas
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile_name: "Default".to_string(),
            click_area: ClickArea::default(),
            click_timing: ClickTiming::default(),
            click_options: ClickOptions::default(),
            multi_area: MultiAreaConfig::default(),
        }
    }
}

impl Config {
    pub fn new(profile_name: &str) -> Self {
        Self {
            profile_name: profile_name.to_string(),
            ..Default::default()
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::ParseError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, json)
            .map_err(|e| AppError::IoError(e))?;

        Ok(())
    }

    pub fn load(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            return Ok(Config::default());
        }

        let json = fs::read_to_string(path)
            .map_err(|e| AppError::IoError(e))?;

        let config = serde_json::from_str(&json)
            .map_err(|e| AppError::ParseError(format!("Failed to deserialize config: {}", e)))?;

        Ok(config)
    }

    // Add a new click area to the multi-area configuration
    pub fn add_click_area(&mut self, area: ClickArea, weight: f32) {
        self.multi_area.areas.push((area, weight));
        if !self.multi_area.areas.is_empty() {
            self.multi_area.enabled = true;
        }
    }

    // Remove a click area from the multi-area configuration
    pub fn remove_click_area(&mut self, index: usize) -> bool {
        if index < self.multi_area.areas.len() {
            self.multi_area.areas.remove(index);
            if self.multi_area.areas.is_empty() {
                self.multi_area.enabled = false;
            }
            true
        } else {
            false
        }
    }

    // Clear all click areas
    pub fn clear_click_areas(&mut self) {
        self.multi_area.areas.clear();
        self.multi_area.enabled = false;
    }
}