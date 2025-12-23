use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub whisper: WhisperSettings,
    #[serde(default)]
    pub llm: LlmSettings,
    /// Output mode for transcription results
    #[serde(default)]
    pub output_mode: OutputMode,
    /// Shortcut key settings
    #[serde(default)]
    pub shortcut: ShortcutSettings,
    /// Whether settings were loaded from a saved file (not defaults)
    #[serde(skip_deserializing, default)]
    pub is_saved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperSettings {
    pub model_name: String,
}

/// Shortcut settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutSettings {
    /// The shortcut key combination (e.g., "Ctrl+Space", "Alt+R")
    pub recording_toggle: String,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            recording_toggle: "Ctrl+Space".to_string(),
        }
    }
}

/// Output mode for transcription results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputMode {
    /// Copy to clipboard only (no paste)
    ClipboardOnly,
    /// Paste directly (temporary clipboard, restore after)
    DirectInput,
    /// Copy to clipboard and paste (current behavior)
    Both,
}

impl Default for OutputMode {
    fn default() -> Self {
        Self::DirectInput
    }
}

/// Available prompt presets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptPreset {
    /// Default: Natural Japanese text formatting
    Default,
    /// Meeting notes: Format as meeting minutes
    Meeting,
    /// Memo: Short, concise notes
    Memo,
    /// Chat: Casual conversation style
    Chat,
    /// Custom: User-defined prompt
    Custom,
}

impl Default for PromptPreset {
    fn default() -> Self {
        Self::Default
    }
}

/// Get the prompt template for a preset
pub fn get_preset_prompt(preset: &PromptPreset) -> &'static str {
    match preset {
        PromptPreset::Default => {
            r#"以下の音声認識結果を自然な日本語に整形してください。
誤字脱字の修正、句読点の追加、文法の修正を行ってください。
整形後のテキストのみを出力してください。余計な説明は不要です。

入力: {input}

出力:"#
        }
        PromptPreset::Meeting => {
            r#"以下の音声認識結果を議事録形式で整形してください。
- 発言内容を箇条書きで整理
- 重要なポイントや決定事項を明確に
- 誤字脱字を修正
整形後のテキストのみを出力してください。

入力: {input}

出力:"#
        }
        PromptPreset::Memo => {
            r#"以下の音声認識結果を簡潔なメモに整形してください。
- 要点を短くまとめる
- 不要な言葉を省く
- 誤字脱字を修正
整形後のテキストのみを出力してください。

入力: {input}

出力:"#
        }
        PromptPreset::Chat => {
            r#"以下の音声認識結果をカジュアルなチャット文に整形してください。
- 口語的な表現を維持
- 適度な絵文字や句読点を追加
- 誤字脱字のみ修正
整形後のテキストのみを出力してください。

入力: {input}

出力:"#
        }
        PromptPreset::Custom => "",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    /// Whether LLM refinement is enabled
    pub enabled: bool,
    /// Ollama API base URL
    pub ollama_url: String,
    /// Model name to use
    pub model_name: String,
    /// Selected prompt preset
    #[serde(default)]
    pub preset: PromptPreset,
    /// Custom prompt template (used when preset is Custom)
    #[serde(default)]
    pub custom_prompt: String,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "gpt-oss:20b".to_string(),
            preset: PromptPreset::Default,
            custom_prompt: String::new(),
        }
    }
}

impl LlmSettings {
    /// Get the effective prompt template based on preset or custom prompt
    pub fn get_prompt_template(&self) -> String {
        if self.preset == PromptPreset::Custom {
            if self.custom_prompt.is_empty() {
                get_preset_prompt(&PromptPreset::Default).to_string()
            } else {
                self.custom_prompt.clone()
            }
        } else {
            get_preset_prompt(&self.preset).to_string()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            whisper: WhisperSettings {
                model_name: "large-v3-turbo".to_string(),
            },
            llm: LlmSettings::default(),
            output_mode: OutputMode::default(),
            shortcut: ShortcutSettings::default(),
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
