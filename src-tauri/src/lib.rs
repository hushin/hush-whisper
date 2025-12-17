mod audio;
mod clipboard;
mod config;
mod llm;
mod log;
mod shortcuts;
mod tray;
mod whisper;

use audio::{AudioCapture, Resampler, VadProcessor};
use clipboard::ClipboardManager;
use llm::OllamaClient;
use shortcuts::ShortcutHandler;
use tray::TrayManager;
use whisper::WhisperTranscriber;

use futures_util::StreamExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tokio::io::AsyncWriteExt;

/// Available Whisper models with their URLs and filenames
const MODELS: &[(&str, &str, &str)] = &[
    ("large-v3-turbo-q8_0", "ggml-large-v3-turbo-q8_0.bin", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin"),
    ("large-v3-turbo", "ggml-large-v3-turbo.bin", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"),
    ("medium", "ggml-medium.bin", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"),
    ("small", "ggml-small.bin", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"),
    ("base", "ggml-base.bin", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"),
    ("tiny", "ggml-tiny.bin", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"),
];

fn get_model_url(model_name: &str) -> Option<&'static str> {
    MODELS.iter().find(|(name, _, _)| *name == model_name).map(|(_, _, url)| *url)
}

fn get_model_filename(model_name: &str) -> Option<&'static str> {
    MODELS.iter().find(|(name, _, _)| *name == model_name).map(|(_, filename, _)| *filename)
}

/// Expand Windows environment variables like %APPDATA%
fn expand_env_vars(path: &str) -> String {
    let mut result = path.to_string();

    // Find and replace all %VAR% patterns
    while let Some(start) = result.find('%') {
        if let Some(end) = result[start + 1..].find('%') {
            let var_name = &result[start + 1..start + 1 + end];
            if let Ok(value) = std::env::var(var_name) {
                result = result.replace(&format!("%{}%", var_name), &value);
            } else {
                // If variable not found, skip this one
                break;
            }
        } else {
            break;
        }
    }

    result
}

// Application state
pub struct AppState {
    audio_capture: Mutex<Option<AudioCapture>>,
    // Stream is not stored here because it's not Send
    active_stream: Mutex<Option<Box<cpal::Stream>>>,
    whisper: Mutex<Option<WhisperTranscriber>>,
    clipboard: Mutex<Option<ClipboardManager>>,
    vad: Mutex<Option<VadProcessor>>,
    is_recording: Mutex<bool>,
}

// Manual Send/Sync implementation
// SAFETY: We ensure proper synchronization using Mutex
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    fn new() -> Self {
        Self {
            audio_capture: Mutex::new(None),
            active_stream: Mutex::new(None),
            whisper: Mutex::new(None),
            clipboard: Mutex::new(None),
            vad: Mutex::new(None),
            is_recording: Mutex::new(false),
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    downloaded: u64,
    total: u64,
    percentage: f64,
}

#[derive(Clone, serde::Serialize)]
struct ModelInfo {
    name: String,
    filename: String,
    size_hint: String,
}

#[tauri::command]
fn get_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo { name: "large-v3-turbo-q8_0".into(), filename: "ggml-large-v3-turbo-q8_0.bin".into(), size_hint: "~820MB (推奨)".into() },
        ModelInfo { name: "large-v3-turbo".into(), filename: "ggml-large-v3-turbo.bin".into(), size_hint: "~1.5GB".into() },
        ModelInfo { name: "medium".into(), filename: "ggml-medium.bin".into(), size_hint: "~1.5GB".into() },
        ModelInfo { name: "small".into(), filename: "ggml-small.bin".into(), size_hint: "~500MB".into() },
        ModelInfo { name: "base".into(), filename: "ggml-base.bin".into(), size_hint: "~150MB".into() },
        ModelInfo { name: "tiny".into(), filename: "ggml-tiny.bin".into(), size_hint: "~77MB".into() },
    ]
}

async fn download_model_internal(app: &AppHandle, model_url: &str, target_path: &PathBuf) -> Result<(), String> {
    // Create parent directory if needed
    if let Some(parent) = target_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    tracing::info!("Downloading model from {} to {:?}", model_url, target_path);

    // Start download
    let client = reqwest::Client::new();
    let response = client
        .get(model_url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    tracing::info!("Total size: {} bytes", total_size);

    // Create temp file
    let temp_path = target_path.with_extension("bin.tmp");
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        downloaded += chunk.len() as u64;

        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        // Emit progress event
        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                downloaded,
                total: total_size,
                percentage,
            },
        );
    }

    // Rename temp file to final path
    tokio::fs::rename(&temp_path, target_path)
        .await
        .map_err(|e| format!("Failed to finalize download: {}", e))?;

    tracing::info!("Download complete: {:?}", target_path);
    Ok(())
}

#[tauri::command]
async fn initialize_whisper(
    state: State<'_, AppState>,
    app: AppHandle,
    model_name: String,
) -> Result<String, String> {
    tracing::info!("Initializing Whisper with model: {}", model_name);

    // Get model info
    let model_url = get_model_url(&model_name)
        .ok_or_else(|| format!("Unknown model: {}", model_name))?;
    let model_filename = get_model_filename(&model_name)
        .ok_or_else(|| format!("Unknown model: {}", model_name))?;

    // Construct model path: %APPDATA%/voice-input/models/<filename>
    let base_path = expand_env_vars("%APPDATA%\\voice-input\\models");
    let path = PathBuf::from(&base_path).join(model_filename);

    tracing::info!("Model path: {:?}", path);

    // If model doesn't exist, download it
    if !path.exists() {
        tracing::info!("Model not found, starting download...");
        app.emit("download-started", ())
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        download_model_internal(&app, model_url, &path).await?;

        app.emit("download-complete", ())
            .map_err(|e| format!("Failed to emit event: {}", e))?;
    }

    let transcriber = WhisperTranscriber::new(path)
        .map_err(|e| format!("Failed to initialize Whisper: {}", e))?;

    *state.whisper.lock().unwrap() = Some(transcriber);

    tracing::info!("Whisper initialized successfully");
    Ok("Whisper initialized successfully".to_string())
}

#[tauri::command]
async fn start_recording(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock().unwrap();

    if *is_recording {
        return Err("Already recording".to_string());
    }

    let mut audio_capture_guard = state.audio_capture.lock().unwrap();

    if audio_capture_guard.is_none() {
        let capture = AudioCapture::new()
            .map_err(|e| format!("Failed to create audio capture: {}", e))?;
        *audio_capture_guard = Some(capture);
    }

    let audio_capture = audio_capture_guard.as_ref().unwrap();
    let stream = audio_capture
        .start_recording()
        .map_err(|e| format!("Failed to start recording: {}", e))?;

    // Store the stream
    *state.active_stream.lock().unwrap() = Some(stream);

    *is_recording = true;

    // Notify frontend
    app.emit("recording-started", ())
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok("Recording started".to_string())
}

#[tauri::command]
async fn stop_recording(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    // Phase 1: Gather all data while holding locks, then release them before any await
    let text = {
        let mut is_recording = state.is_recording.lock().unwrap();

        if !*is_recording {
            return Err("Not recording".to_string());
        }

        // Drop the stream to stop recording
        *state.active_stream.lock().unwrap() = None;

        let audio_capture_guard = state.audio_capture.lock().unwrap();
        let audio_capture = audio_capture_guard
            .as_ref()
            .ok_or("Audio capture not initialized")?;

        let audio_data = audio_capture.stop_recording();
        let sample_rate = audio_capture.get_sample_rate();

        *is_recording = false;
        drop(is_recording);
        drop(audio_capture_guard);

        // Notify frontend
        app.emit("recording-stopped", ())
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        tracing::info!(
            "Recording stopped. Captured {} samples at {} Hz",
            audio_data.len(),
            sample_rate
        );

        // Check if we have any audio data
        if audio_data.is_empty() {
            return Err("No audio data captured".to_string());
        }

        // Resample to 16kHz for Whisper
        let resampler = Resampler::new(16000);
        let resampled_data = resampler
            .resample(&audio_data, sample_rate)
            .map_err(|e| format!("Failed to resample audio: {}", e))?;

        tracing::info!("Resampled to {} samples", resampled_data.len());

        // Apply VAD to extract speech segments
        let speech_data = {
            let mut vad_guard = state.vad.lock().unwrap();
            if vad_guard.is_none() {
                match VadProcessor::new() {
                    Ok(vad) => *vad_guard = Some(vad),
                    Err(e) => tracing::warn!("Failed to create VAD, skipping: {}", e),
                }
            }

            if let Some(vad) = vad_guard.as_mut() {
                let extracted = vad.extract_speech(&resampled_data);
                if extracted.is_empty() {
                    tracing::info!("VAD detected no speech, using original audio");
                    resampled_data
                } else {
                    extracted
                }
            } else {
                resampled_data
            }
        };

        tracing::info!("After VAD: {} samples", speech_data.len());

        // Transcribe
        app.emit("transcription-started", ())
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        let whisper_guard = state.whisper.lock().unwrap();
        let transcribed = if let Some(whisper) = whisper_guard.as_ref() {
            // Use Whisper for transcription
            whisper
                .transcribe(&speech_data)
                .map_err(|e| format!("Failed to transcribe: {}", e))?
        } else {
            // Fallback to dummy mode if Whisper not initialized
            tracing::warn!("Whisper not initialized, using dummy mode");
            format!(
                "[デモモード] {}サンプルの音声を録音しました。モデルを読み込んでください。",
                speech_data.len()
            )
        };
        drop(whisper_guard);

        tracing::info!("Transcription result: {}", transcribed);
        transcribed
    };
    // All MutexGuards are now dropped

    // Phase 2: LLM refinement (async, no locks held)
    let settings = config::load_settings();
    let final_text = if settings.llm.enabled {
        tracing::info!("LLM refinement enabled, sending to Ollama...");
        app.emit("llm-refinement-started", ())
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        let ollama = OllamaClient::new(&settings.llm.ollama_url, &settings.llm.model_name);
        let prompt_template = settings.llm.get_prompt_template();
        tracing::info!("Using prompt preset: {:?}", settings.llm.preset);
        match ollama.refine_text_with_prompt(&text, &prompt_template).await {
            Ok(refined) => {
                tracing::info!("LLM refined: {} -> {}", text, refined);
                app.emit("llm-refinement-complete", refined.clone())
                    .map_err(|e| format!("Failed to emit event: {}", e))?;
                refined
            }
            Err(e) => {
                tracing::warn!("LLM refinement failed, using original text: {}", e);
                app.emit("llm-refinement-failed", e.clone())
                    .map_err(|e| format!("Failed to emit event: {}", e))?;
                text.clone()
            }
        }
    } else {
        text.clone()
    };

    // Phase 3: Save log entry
    {
        if let Ok(log_manager) = log::LogManager::new() {
            let refined = if settings.llm.enabled {
                Some(final_text.clone())
            } else {
                None
            };
            let preset_name = if settings.llm.enabled {
                Some(format!("{:?}", settings.llm.preset))
            } else {
                None
            };
            if let Err(e) = log_manager.add_entry(
                text.clone(),
                refined,
                None, // audio_duration_secs - could be calculated if needed
                settings.llm.enabled,
                preset_name,
            ) {
                tracing::warn!("Failed to save log entry: {}", e);
            }
        }
    }

    // Phase 4: Copy to clipboard (sync, brief lock)
    {
        let mut clipboard_guard = state.clipboard.lock().unwrap();
        if clipboard_guard.is_none() {
            let clipboard = ClipboardManager::new()
                .map_err(|e| format!("Failed to create clipboard manager: {}", e))?;
            *clipboard_guard = Some(clipboard);
        }

        let clipboard = clipboard_guard.as_mut().unwrap();
        clipboard
            .set_and_paste(&final_text)
            .map_err(|e| format!("Failed to paste text: {}", e))?;
    }

    // Notify frontend with result
    app.emit("transcription-complete", final_text.clone())
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(final_text)
}

#[tauri::command]
async fn toggle_recording(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let is_recording = *state.is_recording.lock().unwrap();

    if is_recording {
        stop_recording(state, app).await
    } else {
        start_recording(state, app).await
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_settings() -> config::Settings {
    config::load_settings()
}

#[tauri::command]
fn save_model_selection(model_name: String) -> Result<(), String> {
    let mut settings = config::load_settings();
    settings.whisper.model_name = model_name;
    config::save_settings(&settings)
}

#[tauri::command]
fn save_llm_settings(enabled: bool, ollama_url: String, model_name: String) -> Result<(), String> {
    let mut settings = config::load_settings();
    settings.llm.enabled = enabled;
    settings.llm.ollama_url = ollama_url;
    settings.llm.model_name = model_name;
    config::save_settings(&settings)
}

#[tauri::command]
fn save_prompt_settings(preset: String, custom_prompt: String) -> Result<(), String> {
    let mut settings = config::load_settings();
    settings.llm.preset = match preset.as_str() {
        "Default" => config::PromptPreset::Default,
        "Meeting" => config::PromptPreset::Meeting,
        "Memo" => config::PromptPreset::Memo,
        "Chat" => config::PromptPreset::Chat,
        "Custom" => config::PromptPreset::Custom,
        _ => config::PromptPreset::Default,
    };
    settings.llm.custom_prompt = custom_prompt;
    config::save_settings(&settings)
}

#[tauri::command]
fn get_preset_prompts() -> Vec<(String, String)> {
    vec![
        ("Default".to_string(), config::get_preset_prompt(&config::PromptPreset::Default).to_string()),
        ("Meeting".to_string(), config::get_preset_prompt(&config::PromptPreset::Meeting).to_string()),
        ("Memo".to_string(), config::get_preset_prompt(&config::PromptPreset::Memo).to_string()),
        ("Chat".to_string(), config::get_preset_prompt(&config::PromptPreset::Chat).to_string()),
    ]
}

#[tauri::command]
async fn check_ollama_status(ollama_url: String) -> Result<bool, String> {
    let client = OllamaClient::new(&ollama_url, "");
    Ok(client.is_available().await)
}

// Log management commands
#[tauri::command]
fn get_recent_logs(limit: Option<usize>) -> Result<Vec<log::LogEntry>, String> {
    let log_manager = log::LogManager::new()?;
    Ok(log_manager.get_recent_logs(limit.unwrap_or(50)))
}

#[tauri::command]
fn get_logs_for_date(year: i32, month: u32, day: u32) -> Result<Vec<log::LogEntry>, String> {
    let log_manager = log::LogManager::new()?;
    Ok(log_manager.get_logs_for_date(year, month, day))
}

#[tauri::command]
fn get_available_log_dates() -> Result<Vec<String>, String> {
    let log_manager = log::LogManager::new()?;
    Ok(log_manager.get_available_dates())
}

#[tauri::command]
fn delete_log_entry(id: String) -> Result<bool, String> {
    let log_manager = log::LogManager::new()?;
    log_manager.delete_entry(&id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Create toggle callback
            let toggle_callback = Arc::new(|app_handle: &AppHandle| {
                let app_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let state: State<AppState> = app_clone.state();
                    if let Err(e) = toggle_recording(state, app_clone.clone()).await {
                        tracing::error!("Failed to toggle recording: {}", e);
                    }
                });
            });

            // Register global shortcut with callback
            if let Err(e) = ShortcutHandler::register(app.handle(), toggle_callback) {
                tracing::error!("Failed to register global shortcut: {}", e);
            }

            // Setup system tray
            let tray_manager = TrayManager::new();
            if let Err(e) = tray_manager.setup(app.handle()) {
                tracing::error!("Failed to setup system tray: {}", e);
            }
            tracing::info!("Setup complete - tray and shortcuts registered");

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Prevent closing the window, hide it instead
                api.prevent_close();
                let _ = window.hide();
                tracing::info!("Window hidden to system tray");
            }
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_available_models,
            initialize_whisper,
            start_recording,
            stop_recording,
            toggle_recording,
            get_settings,
            save_model_selection,
            save_llm_settings,
            save_prompt_settings,
            get_preset_prompts,
            check_ollama_status,
            get_recent_logs,
            get_logs_for_date,
            get_available_log_dates,
            delete_log_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
