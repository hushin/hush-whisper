use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub struct ShortcutHandler;

/// Parse a shortcut string like "Ctrl+Space" into a Shortcut
pub fn parse_shortcut(shortcut_str: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = shortcut_str.split('+').map(|s| s.trim()).collect();

    let mut modifiers: Option<Modifiers> = None;
    let mut code: Option<Code> = None;

    for part in parts {
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => {
                modifiers = Some(modifiers.map_or(Modifiers::CONTROL, |m| m | Modifiers::CONTROL));
            }
            "ALT" => {
                modifiers = Some(modifiers.map_or(Modifiers::ALT, |m| m | Modifiers::ALT));
            }
            "SHIFT" => {
                modifiers = Some(modifiers.map_or(Modifiers::SHIFT, |m| m | Modifiers::SHIFT));
            }
            "SUPER" | "WIN" | "CMD" | "META" => {
                modifiers = Some(modifiers.map_or(Modifiers::SUPER, |m| m | Modifiers::SUPER));
            }
            "SPACE" => code = Some(Code::Space),
            "ENTER" | "RETURN" => code = Some(Code::Enter),
            "TAB" => code = Some(Code::Tab),
            "ESCAPE" | "ESC" => code = Some(Code::Escape),
            "BACKSPACE" => code = Some(Code::Backspace),
            "DELETE" => code = Some(Code::Delete),
            "INSERT" => code = Some(Code::Insert),
            "HOME" => code = Some(Code::Home),
            "END" => code = Some(Code::End),
            "PAGEUP" => code = Some(Code::PageUp),
            "PAGEDOWN" => code = Some(Code::PageDown),
            "UP" => code = Some(Code::ArrowUp),
            "DOWN" => code = Some(Code::ArrowDown),
            "LEFT" => code = Some(Code::ArrowLeft),
            "RIGHT" => code = Some(Code::ArrowRight),
            "F1" => code = Some(Code::F1),
            "F2" => code = Some(Code::F2),
            "F3" => code = Some(Code::F3),
            "F4" => code = Some(Code::F4),
            "F5" => code = Some(Code::F5),
            "F6" => code = Some(Code::F6),
            "F7" => code = Some(Code::F7),
            "F8" => code = Some(Code::F8),
            "F9" => code = Some(Code::F9),
            "F10" => code = Some(Code::F10),
            "F11" => code = Some(Code::F11),
            "F12" => code = Some(Code::F12),
            // Symbol keys (from event.code)
            "EQUAL" => code = Some(Code::Equal),
            "MINUS" => code = Some(Code::Minus),
            "BRACKETLEFT" => code = Some(Code::BracketLeft),
            "BRACKETRIGHT" => code = Some(Code::BracketRight),
            "SEMICOLON" => code = Some(Code::Semicolon),
            "QUOTE" => code = Some(Code::Quote),
            "BACKQUOTE" => code = Some(Code::Backquote),
            "BACKSLASH" => code = Some(Code::Backslash),
            "COMMA" => code = Some(Code::Comma),
            "PERIOD" => code = Some(Code::Period),
            "SLASH" => code = Some(Code::Slash),
            // Numpad keys
            "NUMPAD0" => code = Some(Code::Numpad0),
            "NUMPAD1" => code = Some(Code::Numpad1),
            "NUMPAD2" => code = Some(Code::Numpad2),
            "NUMPAD3" => code = Some(Code::Numpad3),
            "NUMPAD4" => code = Some(Code::Numpad4),
            "NUMPAD5" => code = Some(Code::Numpad5),
            "NUMPAD6" => code = Some(Code::Numpad6),
            "NUMPAD7" => code = Some(Code::Numpad7),
            "NUMPAD8" => code = Some(Code::Numpad8),
            "NUMPAD9" => code = Some(Code::Numpad9),
            "NUMPADADD" => code = Some(Code::NumpadAdd),
            "NUMPADSUBTRACT" => code = Some(Code::NumpadSubtract),
            "NUMPADMULTIPLY" => code = Some(Code::NumpadMultiply),
            "NUMPADDIVIDE" => code = Some(Code::NumpadDivide),
            "NUMPADDECIMAL" => code = Some(Code::NumpadDecimal),
            "NUMPADENTER" => code = Some(Code::NumpadEnter),
            key if key.len() == 1 => {
                let c = key.chars().next().unwrap();
                code = Some(char_to_code(c)?);
            }
            _ => return Err(format!("Unknown key: {}", part)),
        }
    }

    Ok(Shortcut::new(modifiers, code.ok_or("No key specified")?))
}

fn char_to_code(c: char) -> Result<Code, String> {
    match c.to_ascii_uppercase() {
        'A' => Ok(Code::KeyA),
        'B' => Ok(Code::KeyB),
        'C' => Ok(Code::KeyC),
        'D' => Ok(Code::KeyD),
        'E' => Ok(Code::KeyE),
        'F' => Ok(Code::KeyF),
        'G' => Ok(Code::KeyG),
        'H' => Ok(Code::KeyH),
        'I' => Ok(Code::KeyI),
        'J' => Ok(Code::KeyJ),
        'K' => Ok(Code::KeyK),
        'L' => Ok(Code::KeyL),
        'M' => Ok(Code::KeyM),
        'N' => Ok(Code::KeyN),
        'O' => Ok(Code::KeyO),
        'P' => Ok(Code::KeyP),
        'Q' => Ok(Code::KeyQ),
        'R' => Ok(Code::KeyR),
        'S' => Ok(Code::KeyS),
        'T' => Ok(Code::KeyT),
        'U' => Ok(Code::KeyU),
        'V' => Ok(Code::KeyV),
        'W' => Ok(Code::KeyW),
        'X' => Ok(Code::KeyX),
        'Y' => Ok(Code::KeyY),
        'Z' => Ok(Code::KeyZ),
        '0' => Ok(Code::Digit0),
        '1' => Ok(Code::Digit1),
        '2' => Ok(Code::Digit2),
        '3' => Ok(Code::Digit3),
        '4' => Ok(Code::Digit4),
        '5' => Ok(Code::Digit5),
        '6' => Ok(Code::Digit6),
        '7' => Ok(Code::Digit7),
        '8' => Ok(Code::Digit8),
        '9' => Ok(Code::Digit9),
        _ => Err(format!("Unknown character: {}", c)),
    }
}

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

    pub fn register_with_shortcut<F>(
        app: &AppHandle,
        shortcut_str: &str,
        on_toggle: Arc<F>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&AppHandle) + Send + Sync + 'static,
    {
        let shortcut = parse_shortcut(shortcut_str)?;
        let callback = on_toggle.clone();
        let shortcut_display = shortcut_str.to_string();

        app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!("Global shortcut triggered: {}", shortcut_display);

                // Call the callback directly
                callback(app);

                // Also emit event for frontend/tray updates
                if let Err(e) = app.emit("recording-toggle", ()) {
                    tracing::error!("Failed to emit recording-toggle event: {}", e);
                }
            }
        })?;

        app.global_shortcut().register(shortcut)?;
        tracing::info!("Global shortcut registered: {}", shortcut_str);

        Ok(())
    }

    pub fn update_shortcut<F>(
        app: &AppHandle,
        _old_shortcut_str: &str,
        new_shortcut_str: &str,
        on_toggle: Arc<F>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&AppHandle) + Send + Sync + 'static,
    {
        // Parse new shortcut first to validate
        let new_shortcut = parse_shortcut(new_shortcut_str)?;

        // Unregister all shortcuts to ensure clean state
        let _ = app.global_shortcut().unregister_all();

        // Small delay to ensure unregistration is complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Register new shortcut handler
        let callback = on_toggle.clone();
        let shortcut_display = new_shortcut_str.to_string();

        app.global_shortcut().on_shortcut(new_shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!("Global shortcut triggered: {}", shortcut_display);

                // Call the callback directly
                callback(app);

                // Also emit event for frontend/tray updates
                if let Err(e) = app.emit("recording-toggle", ()) {
                    tracing::error!("Failed to emit recording-toggle event: {}", e);
                }
            }
        })?;

        // Try to register, but ignore "already registered" errors
        match app.global_shortcut().register(new_shortcut) {
            Ok(_) => {
                tracing::info!("Shortcut registered: {}", new_shortcut_str);
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("already registered") {
                    tracing::warn!("Shortcut already registered, continuing: {}", new_shortcut_str);
                } else {
                    return Err(e.into());
                }
            }
        }

        tracing::info!("Shortcut updated to: {}", new_shortcut_str);
        Ok(())
    }
}
