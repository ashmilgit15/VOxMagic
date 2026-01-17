use serde::{Deserialize, Serialize};
use std::fs;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub groq_api_key: String,
    pub auto_paste: bool,
    pub always_on_top: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            groq_api_key: String::new(),
            auto_paste: true,
            always_on_top: true,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        if let Some(proj_dirs) = ProjectDirs::from("com", "ashmil", "speech_to_text") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("settings.json");

            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "ashmil", "speech_to_text") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;

            let config_path = config_dir.join("settings.json");
            let json = serde_json::to_string_pretty(self)?;
            fs::write(config_path, json)?;
        }
        Ok(())
    }
}
