use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub struct ShortcutHandler;

impl ShortcutHandler {
    pub fn register<F>(app: &AppHandle, on_toggle: Arc<F>) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&AppHandle) + Send + Sync + 'static,
    {
        let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
        let callback = on_toggle.clone();

        app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!("Global shortcut triggered: Ctrl+Space");

                // Call the callback directly
                callback(app);

                // Also emit event for frontend/tray updates
                if let Err(e) = app.emit("recording-toggle", ()) {
                    tracing::error!("Failed to emit recording-toggle event: {}", e);
                }
            }
        })?;

        app.global_shortcut().register(shortcut)?;
        tracing::info!("Global shortcut registered: Ctrl+Space");

        Ok(())
    }
}
