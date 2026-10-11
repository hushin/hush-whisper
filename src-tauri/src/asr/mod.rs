use crate::qwen3::Qwen3Transcriber;
use crate::whisper::WhisperTranscriber;

/// ASR backend selected at model-load time.
pub enum AsrEngine {
    Whisper(WhisperTranscriber),
    // Boxed because `Qwen3Transcriber` is much larger than `WhisperTranscriber`.
    Qwen3(Box<Qwen3Transcriber>),
}

impl AsrEngine {
    /// Transcribe 16 kHz mono f32 samples into text.
    pub fn transcribe(&self, audio_data: &[f32], insert_newline: bool) -> Result<String, String> {
        match self {
            AsrEngine::Whisper(t) => t
                .transcribe_with_options(audio_data, insert_newline)
                .map_err(|e| e.to_string()),
            AsrEngine::Qwen3(t) => t.transcribe_with_options(audio_data, insert_newline),
        }
    }

    /// Whether this engine supports low-latency live (partial) transcription.
    pub fn supports_live(&self) -> bool {
        matches!(self, AsrEngine::Qwen3(_))
    }
}
