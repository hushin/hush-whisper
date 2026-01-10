use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Listener, Manager,
};

const TRAY_ID: &str = "main-tray";

// Embedded icon data
const ICON_NORMAL: &[u8] = include_bytes!("../../icons/32x32.png");

/// Create a status indicator icon by adding a colored dot to the bottom-right of the default icon
fn create_status_icon(color: [u8; 3]) -> Vec<u8> {
    use image::{ImageReader, Rgba, RgbaImage};
    use std::io::Cursor;

    // Load the default icon
    let img = ImageReader::new(Cursor::new(ICON_NORMAL))
        .with_guessed_format()
        .expect("Failed to guess format")
        .decode()
        .expect("Failed to decode icon");

    let mut rgba_img: RgbaImage = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();

    // Dot parameters - positioned at bottom-right
    let dot_radius = 5.0f32;
    let dot_center_x = width as f32 - dot_radius - 2.0;
    let dot_center_y = height as f32 - dot_radius - 2.0;

    // Draw colored dot with anti-aliasing
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - dot_center_x;
            let dy = y as f32 - dot_center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= dot_radius {
                // Inside the dot - solid color
                rgba_img.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
            } else if distance <= dot_radius + 1.0 {
                // Edge - blend with existing pixel for anti-aliasing
                let blend = (dot_radius + 1.0 - distance).clamp(0.0, 1.0);
                let existing = rgba_img.get_pixel(x, y);
                let r = ((color[0] as f32 * blend) + (existing[0] as f32 * (1.0 - blend))) as u8;
                let g = ((color[1] as f32 * blend) + (existing[1] as f32 * (1.0 - blend))) as u8;
                let b = ((color[2] as f32 * blend) + (existing[2] as f32 * (1.0 - blend))) as u8;
                let a = ((255.0 * blend) + (existing[3] as f32 * (1.0 - blend))) as u8;
                rgba_img.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }
    }

    rgba_img.into_raw()
}

/// Create a recording indicator icon (red dot)
fn create_recording_icon() -> Vec<u8> {
    create_status_icon([220, 53, 69]) // Red
}

/// Create an LLM processing indicator icon (purple/blue dot)
fn create_llm_processing_icon() -> Vec<u8> {
    create_status_icon([102, 126, 234]) // Purple/Blue (#667eea)
}

pub struct TrayManager {
    is_recording: Arc<AtomicBool>,
    is_llm_processing: Arc<AtomicBool>,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            is_llm_processing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn setup(&self, app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let is_recording = self.is_recording.clone();
        let is_llm_processing = self.is_llm_processing.clone();

        // Create menu items
        let show_item = MenuItemBuilder::with_id("show", "ウィンドウを表示").build(app)?;
        let quit_item = MenuItemBuilder::with_id("quit", "終了").build(app)?;

        // Build menu
        let menu = MenuBuilder::new(app)
            .items(&[&show_item, &quit_item])
            .build()?;

        // Load normal icon
        let icon = Image::from_bytes(ICON_NORMAL)?;

        // Build tray icon
        let _tray = TrayIconBuilder::with_id(TRAY_ID)
            .icon(icon)
            .tooltip("VoiceInput - 待機中")
            .menu(&menu)
            .on_menu_event(move |app, event| {
                match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            })
            .on_tray_icon_event(|tray, event| {
                if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                    if button == tauri::tray::MouseButton::Left {
                        // Show window on left click
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            })
            .build(app)?;

        // Listen for recording state changes
        let app_handle = app.clone();
        let is_recording_clone = is_recording.clone();
        let is_llm_processing_clone = is_llm_processing.clone();
        app.listen("recording-started", move |_| {
            is_recording_clone.store(true, Ordering::SeqCst);
            update_tray_icon(
                &app_handle,
                is_recording_clone.load(Ordering::SeqCst),
                is_llm_processing_clone.load(Ordering::SeqCst),
            );
        });

        let app_handle = app.clone();
        let is_recording_clone = is_recording.clone();
        let is_llm_processing_clone = is_llm_processing.clone();
        app.listen("recording-stopped", move |_| {
            is_recording_clone.store(false, Ordering::SeqCst);
            update_tray_icon(
                &app_handle,
                is_recording_clone.load(Ordering::SeqCst),
                is_llm_processing_clone.load(Ordering::SeqCst),
            );
        });

        // Listen for LLM processing state changes
        let app_handle = app.clone();
        let is_recording_clone = is_recording.clone();
        let is_llm_processing_clone = is_llm_processing.clone();
        app.listen("llm-refinement-started", move |_| {
            is_llm_processing_clone.store(true, Ordering::SeqCst);
            update_tray_icon(
                &app_handle,
                is_recording_clone.load(Ordering::SeqCst),
                is_llm_processing_clone.load(Ordering::SeqCst),
            );
        });

        let app_handle = app.clone();
        let is_recording_clone = is_recording.clone();
        let is_llm_processing_clone = is_llm_processing.clone();
        app.listen("llm-refinement-complete", move |_| {
            is_llm_processing_clone.store(false, Ordering::SeqCst);
            update_tray_icon(
                &app_handle,
                is_recording_clone.load(Ordering::SeqCst),
                is_llm_processing_clone.load(Ordering::SeqCst),
            );
        });

        let app_handle = app.clone();
        let is_recording_clone = is_recording.clone();
        let is_llm_processing_clone = is_llm_processing.clone();
        app.listen("llm-refinement-failed", move |_| {
            is_llm_processing_clone.store(false, Ordering::SeqCst);
            update_tray_icon(
                &app_handle,
                is_recording_clone.load(Ordering::SeqCst),
                is_llm_processing_clone.load(Ordering::SeqCst),
            );
        });

        tracing::info!("System tray initialized");
        Ok(())
    }
}

fn update_tray_icon(app: &AppHandle, is_recording: bool, is_llm_processing: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let result = if is_recording {
            // Recording: red circle icon (highest priority)
            let icon_data = create_recording_icon();
            let icon = Image::new_owned(icon_data, 32, 32);
            let _ = tray.set_icon(Some(icon));
            tray.set_tooltip(Some("VoiceInput - 録音中..."))
        } else if is_llm_processing {
            // LLM Processing: purple/blue circle icon
            let icon_data = create_llm_processing_icon();
            let icon = Image::new_owned(icon_data, 32, 32);
            let _ = tray.set_icon(Some(icon));
            tray.set_tooltip(Some("VoiceInput - LLM処理中..."))
        } else {
            // Normal: default icon
            match Image::from_bytes(ICON_NORMAL) {
                Ok(icon) => {
                    let _ = tray.set_icon(Some(icon));
                    tray.set_tooltip(Some("VoiceInput - 待機中"))
                }
                Err(e) => {
                    tracing::error!("Failed to load normal icon: {}", e);
                    Ok(())
                }
            }
        };

        if let Err(e) = result {
            tracing::error!("Failed to update tray: {}", e);
        }
    }
}
