use voice_activity_detector::{IteratorExt, LabeledAudio, VoiceActivityDetector};

const SAMPLE_RATE: i64 = 16000;
const CHUNK_SIZE: usize = 512; // 512 samples at 16kHz = 32ms
const SPEECH_THRESHOLD: f32 = 0.5;
const PADDING_CHUNKS: usize = 3; // Add 3 chunks (~96ms) before/after speech

pub struct VadProcessor {
    detector: VoiceActivityDetector,
}

impl VadProcessor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let detector = VoiceActivityDetector::builder()
            .sample_rate(SAMPLE_RATE)
            .chunk_size(CHUNK_SIZE)
            .build()
            .map_err(|e| format!("Failed to create VAD: {:?}", e))?;

        tracing::info!("VAD initialized: {}Hz, {} samples/chunk", SAMPLE_RATE, CHUNK_SIZE);

        Ok(Self { detector })
    }

    /// Extract speech segments from audio data
    /// Returns audio data with non-speech segments removed
    pub fn extract_speech(&mut self, audio_data: &[f32]) -> Vec<f32> {
        if audio_data.len() < CHUNK_SIZE {
            tracing::warn!("Audio too short for VAD processing, returning as-is");
            return audio_data.to_vec();
        }

        let mut speech_segments: Vec<f32> = Vec::new();
        let mut total_chunks = 0;
        let mut speech_chunks = 0;

        // Use the label iterator to classify audio chunks
        let labels = audio_data
            .iter()
            .copied()
            .label(&mut self.detector, SPEECH_THRESHOLD, PADDING_CHUNKS);

        for label in labels {
            total_chunks += 1;
            match label {
                LabeledAudio::Speech(samples) => {
                    speech_chunks += 1;
                    speech_segments.extend(samples);
                }
                LabeledAudio::NonSpeech(_) => {
                    // Skip non-speech segments
                }
            }
        }

        // If most of the audio is speech (>80%), return original to preserve quality
        if total_chunks > 0 && speech_chunks as f64 / total_chunks as f64 > 0.8 {
            tracing::info!(
                "VAD: {}/{} chunks contain speech (>80%), using original audio",
                speech_chunks,
                total_chunks
            );
            return audio_data.to_vec();
        }

        // If no speech detected, return empty (prevents hallucination on silence)
        if speech_segments.is_empty() {
            tracing::info!("VAD: No speech detected in {} chunks", total_chunks);
            return Vec::new();
        }

        tracing::info!(
            "VAD: {}/{} chunks contain speech ({:.1}%), output {} samples",
            speech_chunks,
            total_chunks,
            if total_chunks > 0 { speech_chunks as f64 / total_chunks as f64 * 100.0 } else { 0.0 },
            speech_segments.len()
        );

        speech_segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_creation() {
        let vad = VadProcessor::new();
        assert!(vad.is_ok());
    }

    #[test]
    fn test_silence_detection() {
        let mut vad = VadProcessor::new().unwrap();
        // Create silence (zeros)
        let silence: Vec<f32> = vec![0.0; CHUNK_SIZE * 10];
        let result = vad.extract_speech(&silence);
        // Should return empty or very little for silence
        assert!(result.len() < silence.len());
    }
}
