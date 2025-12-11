mod audio;
mod clipboard;
mod shortcuts;
mod whisper;

use audio::{AudioCapture, Resampler};
use clipboard::ClipboardManager;
use shortcuts::ShortcutHandler;
use whisper::WhisperTranscriber;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Listener, Manager, State};

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
            is_recording: Mutex::new(false),
        }
    }
}

#[tauri::command]
async fn initialize_whisper(
    state: State<'_, AppState>,
    model_path: String,
) -> Result<String, String> {
    tracing::info!("Initializing Whisper with model: {}", model_path);

    // Expand environment variables like %APPDATA%
    let expanded_path = expand_env_vars(&model_path);
    tracing::info!("Expanded path: {}", expanded_path);

    let path = PathBuf::from(&expanded_path);
    if !path.exists() {
        return Err(format!("Model file not found: {}", expanded_path));
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

    // Transcribe
    app.emit("transcription-started", ())
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    let whisper_guard = state.whisper.lock().unwrap();
    let text = if let Some(whisper) = whisper_guard.as_ref() {
        // Use Whisper for transcription
        whisper
            .transcribe(&resampled_data)
            .map_err(|e| format!("Failed to transcribe: {}", e))?
    } else {
        // Fallback to dummy mode if Whisper not initialized
        tracing::warn!("Whisper not initialized, using dummy mode");
        format!(
            "[デモモード] {}サンプルの音声を録音しました。モデルを読み込んでください。",
            resampled_data.len()
        )
    };

    tracing::info!("Transcription result: {}", text);

    // Copy to clipboard
    let mut clipboard_guard = state.clipboard.lock().unwrap();
    if clipboard_guard.is_none() {
        let clipboard = ClipboardManager::new()
            .map_err(|e| format!("Failed to create clipboard manager: {}", e))?;
        *clipboard_guard = Some(clipboard);
    }

    let clipboard = clipboard_guard.as_mut().unwrap();
    clipboard
        .set_text(&text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    // Notify frontend with result
    app.emit("transcription-complete", text.clone())
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(text)
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
            // Register global shortcut
            if let Err(e) = ShortcutHandler::register(app.handle()) {
                tracing::error!("Failed to register global shortcut: {}", e);
            }

            // Listen to recording-toggle event from shortcut
            let app_handle = app.handle().clone();
            let _ = app.listen("recording-toggle", move |_event| {
                let app_clone = app_handle.clone();

                tauri::async_runtime::block_on(async move {
                    let state: State<AppState> = app_clone.state();
                    if let Err(e) = toggle_recording(state, app_clone.clone()).await {
                        tracing::error!("Failed to toggle recording: {}", e);
                    }
                });
            });

            Ok(())
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            initialize_whisper,
            start_recording,
            stop_recording,
            toggle_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
