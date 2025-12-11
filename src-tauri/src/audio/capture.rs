use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl AudioCapture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;

        tracing::info!("Default input config: {:?}", config);

        Ok(Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate,
        })
    }

    pub fn start_recording(&self) -> Result<Box<Stream>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device.default_input_config()?;
        let stream_config: StreamConfig = config.config();

        let buffer = Arc::clone(&self.buffer);

        // Clear previous buffer
        buffer.lock().unwrap().clear();

        let err_fn = |err| {
            tracing::error!("Audio stream error: {}", err);
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_stream::<f32>(&device, &stream_config, buffer, err_fn)?,
            cpal::SampleFormat::I16 => self.build_stream::<i16>(&device, &stream_config, buffer, err_fn)?,
            cpal::SampleFormat::U16 => self.build_stream::<u16>(&device, &stream_config, buffer, err_fn)?,
            sample_format => {
                return Err(format!("Unsupported sample format: {}", sample_format).into())
            }
        };

        stream.play()?;
        tracing::info!("Recording started");

        Ok(Box::new(stream))
    }

    pub fn stop_recording(&self) -> Vec<f32> {
        let buffer = self.buffer.lock().unwrap();
        buffer.clone()
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn build_stream<T>(
        &self,
        device: &Device,
        config: &StreamConfig,
        buffer: Arc<Mutex<Vec<f32>>>,
        err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
    ) -> Result<Stream, Box<dyn std::error::Error>>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let channels = config.channels as usize;

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer.lock().unwrap();

                // Convert to mono f32
                for frame in data.chunks(channels) {
                    let mono_sample: f32 = frame.iter()
                        .map(|&s| f32::from_sample(s))
                        .sum::<f32>() / channels as f32;
                    buffer.push(mono_sample);
                }
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }
}
