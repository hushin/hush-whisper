# 音声処理パイプライン

## 概要

```
┌─────────────┐    ┌──────────────┐    ┌───────────┐    ┌─────────────┐
│ Microphone  │───▶│  Resample    │───▶│  Silero   │───▶│  whisper-rs │
│   (cpal)    │    │ 48k→16k mono │    │    VAD    │    │   (CUDA)    │
└─────────────┘    └──────────────┘    └───────────┘    └─────────────┘
     48kHz              16kHz            speech           transcription
     stereo             mono             chunks
```

## 1. 音声キャプチャ (cpal)

### デバイス選択

```rust
use cpal::traits::{DeviceTrait, HostTrait};

pub fn get_default_input_device() -> Result<cpal::Device, AudioError> {
    let host = cpal::default_host();
    host.default_input_device()
        .ok_or(AudioError::NoInputDevice)
}

pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.name().ok())
                .collect()
        })
        .unwrap_or_default()
}
```

### ストリーム設定

```rust
use cpal::{SampleFormat, StreamConfig};

pub fn create_input_stream(
    device: &cpal::Device,
    tx: std::sync::mpsc::Sender<Vec<f32>>,
) -> Result<cpal::Stream, AudioError> {
    let config = device.default_input_config()?;
    
    // 多くのデバイスは48kHz stereo
    let stream_config = StreamConfig {
        channels: config.channels(),
        sample_rate: config.sample_rate(),
        buffer_size: cpal::BufferSize::Fixed(1024),
    };

    let stream = match config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                tx.send(data.to_vec()).ok();
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| {
                let floats: Vec<f32> = data.iter()
                    .map(|&s| s as f32 / i16::MAX as f32)
                    .collect();
                tx.send(floats).ok();
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )?,
        _ => return Err(AudioError::UnsupportedFormat),
    };

    Ok(stream)
}
```

## 2. リサンプリング (rubato)

Whisperは**16kHz mono f32**を要求。一般的なマイクは48kHz stereoなので変換が必要。

### Stereo→Mono変換

```rust
pub fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
    stereo
        .chunks_exact(2)
        .map(|pair| (pair[0] + pair[1]) / 2.0)
        .collect()
}
```

### リサンプリング (48kHz→16kHz)

```rust
use rubato::{FftFixedIn, Resampler};

pub struct AudioResampler {
    resampler: FftFixedIn<f32>,
    input_buffer: Vec<Vec<f32>>,
}

impl AudioResampler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 48000 → 16000 = 1/3
        let resampler = FftFixedIn::<f32>::new(
            48000,  // input sample rate
            16000,  // output sample rate
            1024,   // chunk size
            1,      // sub chunks
            1,      // channels (mono)
        )?;
        
        Ok(Self {
            resampler,
            input_buffer: vec![Vec::new()],
        })
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        self.input_buffer[0] = input.to_vec();
        
        let output = self.resampler
            .process(&self.input_buffer, None)
            .unwrap_or_default();
        
        output.into_iter().next().unwrap_or_default()
    }
}
```

## 3. Voice Activity Detection (Silero VAD)

無音区間を除去してWhisperへの入力を最適化。ハルシネーション防止にも効果的。

### VADラッパー

```rust
use voice_activity_detector::{VoiceActivityDetector, LabeledAudio};

pub struct VadProcessor {
    vad: VoiceActivityDetector,
    threshold: f32,
    speech_buffer: Vec<f32>,
    silence_frames: usize,
    min_silence_frames: usize,  // 音声終了判定用
}

impl VadProcessor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let vad = VoiceActivityDetector::builder()
            .sample_rate(16000)
            .chunk_size(512)  // 32ms @ 16kHz
            .build()?;

        Ok(Self {
            vad,
            threshold: 0.5,
            speech_buffer: Vec::new(),
            silence_frames: 0,
            min_silence_frames: 10,  // ~320ms of silence to end
        })
    }

    /// チャンクを処理し、音声終了時にバッファを返す
    pub fn process(&mut self, chunk: &[f32]) -> Option<Vec<f32>> {
        let probability = self.vad.predict(chunk.iter().copied());

        if probability > self.threshold {
            // 音声検出
            self.speech_buffer.extend_from_slice(chunk);
            self.silence_frames = 0;
            None
        } else if !self.speech_buffer.is_empty() {
            // 無音だがバッファあり
            self.silence_frames += 1;
            
            if self.silence_frames >= self.min_silence_frames {
                // 十分な無音 → 音声終了
                let speech = std::mem::take(&mut self.speech_buffer);
                self.silence_frames = 0;
                Some(speech)
            } else {
                // まだ継続中（短い無音は含める）
                self.speech_buffer.extend_from_slice(chunk);
                None
            }
        } else {
            // 無音でバッファなし → 何もしない
            None
        }
    }

    /// 強制的にバッファを取得（録音終了時）
    pub fn flush(&mut self) -> Option<Vec<f32>> {
        if self.speech_buffer.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.speech_buffer))
        }
    }
}
```

### VADパラメータ調整

| パラメータ | 推奨値 | 説明 |
|-----------|--------|------|
| threshold | 0.5 | 音声判定閾値（0.3-0.7で調整） |
| chunk_size | 512 | 32ms @ 16kHz |
| min_silence_frames | 10 | 音声終了判定の無音フレーム数 |

## 4. 統合パイプライン

```rust
use tokio::sync::mpsc;

pub struct AudioPipeline {
    resampler: AudioResampler,
    vad: VadProcessor,
}

impl AudioPipeline {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            resampler: AudioResampler::new()?,
            vad: VadProcessor::new()?,
        })
    }

    /// 生オーディオを処理し、音声チャンクを生成
    pub fn process_raw_audio(
        &mut self,
        raw: &[f32],
        is_stereo: bool,
    ) -> Option<Vec<f32>> {
        // 1. Stereo→Mono
        let mono = if is_stereo {
            stereo_to_mono(raw)
        } else {
            raw.to_vec()
        };

        // 2. Resample to 16kHz
        let resampled = self.resampler.process(&mono);

        // 3. VAD processing
        // 512サンプルずつ処理
        for chunk in resampled.chunks(512) {
            if chunk.len() == 512 {
                if let Some(speech) = self.vad.process(chunk) {
                    return Some(speech);
                }
            }
        }
        None
    }

    pub fn flush(&mut self) -> Option<Vec<f32>> {
        self.vad.flush()
    }
}
```

## 5. 最小音声長チェック

短すぎる音声はWhisperの精度が低下するためスキップ。

```rust
const MIN_SPEECH_DURATION_MS: u64 = 300;  // 最小300ms
const SAMPLE_RATE: u32 = 16000;

pub fn is_valid_speech(samples: &[f32]) -> bool {
    let duration_ms = (samples.len() as u64 * 1000) / SAMPLE_RATE as u64;
    duration_ms >= MIN_SPEECH_DURATION_MS
}
```

## 6. エラーハンドリング

```rust
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("No input device found")]
    NoInputDevice,
    
    #[error("Unsupported audio format")]
    UnsupportedFormat,
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("VAD initialization failed: {0}")]
    VadError(String),
    
    #[error("Resampler error: {0}")]
    ResamplerError(String),
}
```

## 7. テスト用ユーティリティ

```rust
/// WAVファイルからf32サンプルを読み込み（テスト用）
pub fn load_wav_as_f32(path: &str) -> Result<(Vec<f32>, u32), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().filter_map(|s| s.ok()).collect()
        }
        hound::SampleFormat::Int => {
            let max = (1 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max)
                .collect()
        }
    };
    
    Ok((samples, spec.sample_rate))
}
```

## パフォーマンス考慮事項

1. **バッファサイズ**: cpalの`BufferSize::Fixed(1024)`で安定性とレイテンシーのバランス
2. **チャンネル変換**: 可能なら入力デバイスをmonoに設定してオーバーヘッド削減
3. **VADチャンク**: 512サンプル（32ms）が精度と速度のバランス良好
4. **メモリ**: speech_bufferは最大30秒分（480,000サンプル≈1.9MB）に制限推奨
