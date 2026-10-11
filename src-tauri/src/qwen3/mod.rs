use std::path::Path;

use qwen3_asr::{best_device, AsrInference, TranscribeOptions};

/// Qwen3-ASR transcriber (pure-Rust, candle backend).
///
/// Wraps [`AsrInference`] from the `qwen3-asr` crate. Models are downloaded
/// from HuggingFace Hub on first use and cached on disk by the crate.
pub struct Qwen3Transcriber {
    engine: AsrInference,
}

impl Qwen3Transcriber {
    /// Download (if needed) and load a Qwen3-ASR model from HuggingFace Hub.
    pub fn from_pretrained(model_id: &str, cache_dir: &Path) -> Result<Self, String> {
        tracing::info!(
            "Loading Qwen3-ASR model '{}' (cache: {:?})",
            model_id,
            cache_dir
        );

        let device = best_device();
        let engine = AsrInference::from_pretrained(model_id, cache_dir, device)
            .map_err(|e| format!("Failed to load Qwen3-ASR model '{}': {}", model_id, e))?;

        tracing::info!("Qwen3-ASR model loaded successfully");
        Ok(Self { engine })
    }

    /// Transcribe 16 kHz mono f32 samples. Language is forced to Japanese.
    pub fn transcribe_with_options(
        &self,
        audio_data: &[f32],
        _insert_newline: bool,
    ) -> Result<String, String> {
        let options = TranscribeOptions::default().with_language("japanese");

        let result = self
            .engine
            .transcribe_samples(audio_data, options)
            .map_err(|e| format!("Qwen3-ASR transcription failed: {}", e))?;

        tracing::info!(
            "Qwen3-ASR result (lang={}): {}",
            result.language,
            result.text
        );

        Ok(Self::clean_transcript(&result.text))
    }

    /// Qwen3-ASR prepends an `<asr_text>` marker between the language tag and the
    /// transcription. The `qwen3-asr` crate only strips it when the language is
    /// auto-detected; since we force Japanese, remove it here.
    fn clean_transcript(raw: &str) -> String {
        const MARKER: &str = "<asr_text>";
        raw.rfind(MARKER)
            .map(|pos| &raw[pos + MARKER.len()..])
            .unwrap_or(raw)
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Qwen3Transcriber;

    #[test]
    fn strips_asr_text_marker() {
        assert_eq!(
            Qwen3Transcriber::clean_transcript("<asr_text>これどんな日本語入力になりますか。"),
            "これどんな日本語入力になりますか。"
        );
    }

    #[test]
    fn strips_language_prefix_and_marker() {
        assert_eq!(
            Qwen3Transcriber::clean_transcript("language Japanese<asr_text>こんにちは"),
            "こんにちは"
        );
    }

    #[test]
    fn keeps_text_without_marker() {
        assert_eq!(
            Qwen3Transcriber::clean_transcript("  こんにちは  "),
            "こんにちは"
        );
    }
}
