use crate::core::{Frame, Timebase, Transform};
use crate::io::IoResult;

pub struct Resample {
	target_rate: u32,
}

impl Resample {
	pub fn new(target_rate: u32) -> Self {
		Self { target_rate }
	}

	pub fn to_48k() -> Self {
		Self::new(48000)
	}

	pub fn to_96k() -> Self {
		Self::new(96000)
	}

	pub fn to_44k() -> Self {
		Self::new(44100)
	}

	fn linear_interpolate(samples: &[i16], src_rate: u32, dst_rate: u32) -> Vec<i16> {
		if src_rate == dst_rate {
			return samples.to_vec();
		}

		let ratio = src_rate as f64 / dst_rate as f64;
		let output_len = ((samples.len() as f64) / ratio).ceil() as usize;
		let mut output = Vec::with_capacity(output_len);

		for i in 0..output_len {
			let src_pos = i as f64 * ratio;
			let src_idx = src_pos as usize;
			let frac = src_pos - src_idx as f64;

			let sample = if src_idx + 1 < samples.len() {
				let s0 = samples[src_idx] as f64;
				let s1 = samples[src_idx + 1] as f64;
				(s0 * (1.0 - frac) + s1 * frac) as i16
			} else if src_idx < samples.len() {
				samples[src_idx]
			} else {
				0
			};

			output.push(sample);
		}

		output
	}
}

impl Transform for Resample {
	fn apply(&mut self, frame: Frame) -> IoResult<Frame> {
		let src_rate = frame.sample_rate;
		let channels = frame.channels as usize;

		if src_rate == self.target_rate {
			return Ok(frame);
		}

		let input_samples: Vec<i16> =
			frame.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();

		let _samples_per_channel = input_samples.len() / channels;
		let mut channel_data: Vec<Vec<i16>> = Vec::with_capacity(channels);

		for ch in 0..channels {
			let channel_samples: Vec<i16> =
				input_samples.iter().skip(ch).step_by(channels).copied().collect();
			let resampled = Self::linear_interpolate(&channel_samples, src_rate, self.target_rate);
			channel_data.push(resampled);
		}

		let output_samples_per_channel = channel_data.first().map(|c| c.len()).unwrap_or(0);
		let mut output_data = Vec::with_capacity(output_samples_per_channel * channels * 2);

		for i in 0..output_samples_per_channel {
			for ch in 0..channels {
				let sample = channel_data[ch].get(i).copied().unwrap_or(0);
				output_data.extend_from_slice(&sample.to_le_bytes());
			}
		}

		let new_timebase = Timebase::new(1, self.target_rate);
		let new_pts = (frame.pts as f64 * self.target_rate as f64 / src_rate as f64) as i64;

		Ok(Frame {
			data: output_data,
			pts: new_pts,
			timebase: new_timebase,
			sample_rate: self.target_rate,
			channels: frame.channels,
			nb_samples: output_samples_per_channel,
		})
	}

	fn name(&self) -> &'static str {
		"resample"
	}
}
