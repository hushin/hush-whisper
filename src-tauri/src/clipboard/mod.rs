use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;

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

    pub fn get_text(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let text = self.clipboard.get_text()?;
        Ok(text)
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
}
