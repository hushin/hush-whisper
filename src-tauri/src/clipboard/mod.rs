use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;

use crate::config::OutputMode;

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let clipboard = Clipboard::new()?;
        Ok(Self { clipboard })
    }

    pub fn set_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.clipboard.set_text(text)?;
        tracing::info!("Text copied to clipboard: {} chars", text.len());
        Ok(())
    }

    /// Get current clipboard text
    pub fn get_text(&mut self) -> Option<String> {
        self.clipboard.get_text().ok()
    }

    /// Simulate Shift+Insert to paste clipboard contents
    /// Using Shift+Insert instead of Ctrl+V to avoid AHK hook interception
    pub fn paste(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Wait a bit to ensure the shortcut key (Ctrl+Space) has been released
        thread::sleep(Duration::from_millis(50));

        let mut enigo = Enigo::new(&Settings::default())?;

        // Press Shift+Insert (alternative paste shortcut)
        enigo.key(Key::Shift, Direction::Press)?;
        enigo.key(Key::Insert, Direction::Click)?;
        enigo.key(Key::Shift, Direction::Release)?;

        tracing::info!("Paste simulated (Shift+Insert)");
        Ok(())
    }

    /// Copy text to clipboard and immediately paste it
    pub fn set_and_paste(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.set_text(text)?;
        self.paste()?;
        Ok(())
    }

    /// Paste with temporary clipboard (restore original after paste)
    pub fn paste_temporary(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Save current clipboard content
        let original = self.get_text();

        // Set new text
        self.set_text(text)?;

        // Paste
        self.paste()?;

        // Wait for paste to complete
        thread::sleep(Duration::from_millis(100));

        // Restore original clipboard content
        if let Some(original_text) = original {
            let _ = self.clipboard.set_text(original_text);
            tracing::info!("Clipboard restored");
        }

        Ok(())
    }

    /// Execute based on output mode
    pub fn output_text(&mut self, text: &str, mode: &OutputMode) -> Result<(), Box<dyn std::error::Error>> {
        match mode {
            OutputMode::ClipboardOnly => {
                self.set_text(text)?;
            }
            OutputMode::DirectInput => {
                self.paste_temporary(text)?;
            }
            OutputMode::Both => {
                self.set_and_paste(text)?;
            }
        }
        Ok(())
    }
}
