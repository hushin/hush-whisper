use tauri::{App, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::error::AppError;

/// グローバルショートカットのセットアップ
pub fn setup_shortcuts(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Push-to-Talk shortcut: Ctrl+Shift+;
    let ptt_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Semicolon);

    let app_handle = app.handle().clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                if shortcut == &ptt_shortcut {
                    match event.state() {
                        ShortcutState::Pressed => {
                            log::info!("PTT pressed");
                            app_handle.emit("recording_start", ()).ok();
                        }
                        ShortcutState::Released => {
                            log::info!("PTT released");
                            app_handle.emit("recording_stop", ()).ok();
                        }
                    }
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(ptt_shortcut)?;

    log::info!("Global shortcut registered: Ctrl+Shift+;");
    Ok(())
}

/// ショートカットを再登録（設定変更時用）
pub fn update_shortcut(
    app_handle: &tauri::AppHandle,
    modifiers: Option<Modifiers>,
    code: Code,
) -> Result<(), AppError> {
    // Unregister all existing shortcuts
    app_handle
        .global_shortcut()
        .unregister_all()
        .map_err(|e| AppError::Other(e.to_string()))?;

    // Register new shortcut
    let shortcut = Shortcut::new(modifiers, code);
    app_handle
        .global_shortcut()
        .register(shortcut)
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(())
}
