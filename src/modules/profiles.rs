use std::fs;
use std::path::PathBuf;
use crate::modules::config::Config;
use crate::modules::error::{AppError, Result};

pub struct ProfileManager {
    profiles_dir: PathBuf,
}

impl ProfileManager {
    pub fn new(profiles_dir: &str) -> Self {
        let dir = PathBuf::from(profiles_dir);
        if !dir.exists() {
            fs::create_dir_all(&dir).expect("Failed to create profiles directory");
        }
        Self { profiles_dir: dir }
    }

    pub fn get_profile_path(&self, profile_name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", profile_name))
    }

    pub fn save_profile(&self, config: &Config) -> Result<()> {
        let path = self.get_profile_path(&config.profile_name);
        config.save(path.to_str().unwrap())?;
        Ok(())
    }

    pub fn load_profile(&self, profile_name: &str) -> Result<Config> {
        let path = self.get_profile_path(profile_name);
        if !path.exists() {
            return Err(AppError::ParseError(format!("Profile '{}' not found", profile_name)));
        }
        Config::load(path.to_str().unwrap())
    }

    pub fn list_profiles(&self) -> Vec<String> {
        let mut profiles = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.profiles_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".json") {
                                if let Some(profile_name) = file_name.strip_suffix(".json") {
                                    profiles.push(profile_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        profiles.sort();
        profiles
    }

    pub fn delete_profile(&self, profile_name: &str) -> Result<()> {
        let path = self.get_profile_path(profile_name);
        if !path.exists() {
            return Err(AppError::ParseError(format!("Profile '{}' not found", profile_name)));
        }
        fs::remove_file(path).map_err(|e| AppError::IoError(e))?;
        Ok(())
    }

    pub fn create_default_profile(&self) -> Result<Config> {
        let config = Config::default();
        self.save_profile(&config)?;
        Ok(config)
    }
}
