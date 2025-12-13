use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub whisper: WhisperSettings,
    /// Whether settings were loaded from a saved file (not defaults)
    #[serde(skip_deserializing, default)]
    pub is_saved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperSettings {
    pub model_name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            whisper: WhisperSettings {
                model_name: "large-v3-turbo".to_string(),
            },
            is_saved: false,
        }
    }
}

/// Get the path to the config file (%APPDATA%/voice-input/config.json)
fn get_config_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(appdata)
        .join("voice-input")
        .join("config.json")
}

/// Load settings from the config file
pub fn load_settings() -> Settings {
    let config_path = get_config_path();

    if !config_path.exists() {
        tracing::info!("Config file not found, using defaults");
        return Settings::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<Settings>(&content) {
            Ok(mut settings) => {
                tracing::info!("Loaded settings from {:?}", config_path);
                settings.is_saved = true;
                settings
            }
            Err(e) => {
                tracing::warn!("Failed to parse config file: {}, using defaults", e);
                Settings::default()
            }
        },
        Err(e) => {
            tracing::warn!("Failed to read config file: {}, using defaults", e);
            Settings::default()
        }
    }
}

/// Save settings to the config file
pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let config_path = get_config_path();

    // Create parent directory if needed
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content =
        serde_json::to_string_pretty(settings).map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

    tracing::info!("Saved settings to {:?}", config_path);
    Ok(())
}
