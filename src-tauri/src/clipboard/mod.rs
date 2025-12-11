use arboard::Clipboard;

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
}
