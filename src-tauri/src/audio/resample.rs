use rubato::{Resampler as RubatoResampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

pub struct Resampler {
    target_sample_rate: u32,
}

impl Resampler {
    pub fn new(target_sample_rate: u32) -> Self {
        Self {
            target_sample_rate,
        }
    }

    pub fn resample(
        &self,
        input: &[f32],
        input_sample_rate: u32,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        if input_sample_rate == self.target_sample_rate {
            // No resampling needed
            return Ok(input.to_vec());
        }

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let chunk_size = input.len();
        let mut resampler = SincFixedIn::<f32>::new(
            self.target_sample_rate as f64 / input_sample_rate as f64,
            2.0,
            params,
            chunk_size,
            1, // mono
        )?;

        let waves_in = vec![input.to_vec()];
        let waves_out = resampler.process(&waves_in, None)?;

        Ok(waves_out[0].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_48k_to_16k() {
        let resampler = Resampler::new(16000);
        let input: Vec<f32> = (0..48000).map(|i| (i as f32 * 0.001).sin()).collect();

        let output = resampler.resample(&input, 48000).unwrap();

        // Output should be approximately 1/3 the length
        assert!((output.len() as f32 - 16000.0).abs() < 100.0);
    }
}
