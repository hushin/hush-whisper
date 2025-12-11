mod audio;
mod clipboard;
mod shortcuts;
// mod whisper; // Temporarily disabled

use audio::{AudioCapture, Resampler};
use clipboard::ClipboardManager;
use shortcuts::ShortcutHandler;
// use whisper::WhisperTranscriber; // Temporarily disabled

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Listener, Manager, State};

// Application state
pub struct AppState {
    audio_capture: Mutex<Option<AudioCapture>>,
    // Stream is not stored here because it's not Send
    active_stream: Mutex<Option<Box<cpal::Stream>>>,
    // whisper: Mutex<Option<WhisperTranscriber>>, // Temporarily disabled
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
            // whisper: Mutex::new(None), // Temporarily disabled
            clipboard: Mutex::new(None),
            is_recording: Mutex::new(false),
        }
    }
}

#[tauri::command]
async fn initialize_whisper(
    _state: State<'_, AppState>,
    model_path: String,
) -> Result<String, String> {
    tracing::info!("Whisper initialization requested with model: {}", model_path);

    // Temporarily disabled - will enable once cmake issues are resolved
    tracing::warn!("Whisper is temporarily disabled. Using dummy mode for testing.");

    Ok("Whisper initialization skipped (dummy mode)".to_string())
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

    // Resample to 16kHz for Whisper
    let resampler = Resampler::new(16000);
    let resampled_data = resampler
        .resample(&audio_data, sample_rate)
        .map_err(|e| format!("Failed to resample audio: {}", e))?;

    tracing::info!("Resampled to {} samples", resampled_data.len());

    // Transcribe (temporarily using dummy text)
    app.emit("transcription-started", ())
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    // Dummy transcription for testing without Whisper
    let text = format!(
        "[デモモード] {}サンプルの音声を録音しました。Whisperモデルを統合すると、ここに認識結果が表示されます。",
        resampled_data.len()
    );

    tracing::info!("Dummy transcription result: {}", text);

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
