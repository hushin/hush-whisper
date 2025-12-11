use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperTranscriber {
    ctx: WhisperContext,
}

impl WhisperTranscriber {
    pub fn new(model_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Loading Whisper model from: {:?}", model_path);

        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(&model_path.to_string_lossy(), params)?;

        tracing::info!("Whisper model loaded successfully");

        Ok(Self { ctx })
    }

    pub fn transcribe(&self, audio_data: &[f32]) -> Result<String, Box<dyn std::error::Error>> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Japanese language setting
        params.set_language(Some("ja"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        tracing::info!("Starting transcription for {} samples", audio_data.len());

        let mut state = self.ctx.create_state()?;
        state.full(params, audio_data)?;

        let num_segments = state.full_n_segments()?;
        tracing::info!("Transcription complete. Segments: {}", num_segments);

        let mut result = String::new();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)?;
            result.push_str(&segment);
        }

        Ok(result.trim().to_string())
    }
}
