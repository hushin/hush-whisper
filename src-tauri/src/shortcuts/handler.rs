use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub struct ShortcutHandler;

impl ShortcutHandler {
    pub fn register(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);

        app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!("Global shortcut triggered: Ctrl+Space");

                // Emit event to frontend
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
