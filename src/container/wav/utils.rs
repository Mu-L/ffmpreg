use super::WavFormat;

pub struct WavInfo {
	pub format: WavFormat,
	pub duration_seconds: f64,
	pub total_samples: u64,
}

impl WavInfo {
	pub fn new(format: WavFormat, total_data_bytes: u64) -> Self {
		// let bytes_per_sample = format.bytes_per_sample();
		let bytes_per_frame = format.bytes_per_frame();
		let total_frames = total_data_bytes / bytes_per_frame as u64;
		let total_samples = total_frames;
		let duration_seconds = total_samples as f64 / format.sample_rate as f64;

		Self { format, duration_seconds, total_samples }
	}

	pub fn bit_rate_kbps(&self) -> u32 {
		let bit_rate =
			(self.format.sample_rate as u64 * self.format.channels as u64 * self.format.bit_depth as u64)
				/ 1000;
		bit_rate as u32
	}

	pub fn display_info(&self) -> String {
		format!(
			"WAV: {}Hz {} channels, {} bits, {:.2}s ({}s bitrate: {} kbps)",
			self.format.sample_rate,
			self.format.channels,
			self.format.bit_depth,
			self.duration_seconds,
			self.format.to_codec_string(),
			self.bit_rate_kbps()
		)
	}
}

pub fn normalize_pcm16(sample: i16) -> f32 {
	sample as f32 / 32768.0
}

pub fn denormalize_pcm16(normalized: f32) -> i16 {
	(normalized * 32768.0).clamp(-32768.0, 32767.0) as i16
}

pub fn normalize_pcm24(sample: i32) -> f32 {
	(sample >> 8) as f32 / 8388608.0
}

pub fn denormalize_pcm24(normalized: f32) -> i32 {
	((normalized * 8388608.0).clamp(-8388608.0, 8388607.0) as i32) << 8
}

pub fn normalize_pcm32(sample: f32) -> f32 {
	sample.clamp(-1.0, 1.0)
}

pub fn denormalize_pcm32(normalized: f32) -> f32 {
	normalized.clamp(-1.0, 1.0)
}

pub struct SampleConverter;

impl SampleConverter {
	pub fn to_f32(data: &[u8], format: &WavFormat) -> Result<Vec<f32>, String> {
		match format.bit_depth {
			16 => Self::from_pcm16(data, format.channels),
			24 => Self::from_pcm24(data, format.channels),
			32 => Self::from_pcm32(data, format.channels),
			_ => Err("unsupported bit depth".to_string()),
		}
	}

	pub fn from_f32(samples: &[f32], format: &WavFormat) -> Result<Vec<u8>, String> {
		match format.bit_depth {
			16 => Self::to_pcm16(samples, format.channels),
			24 => Self::to_pcm24(samples, format.channels),
			32 => Self::to_pcm32(samples, format.channels),
			_ => Err("unsupported bit depth".to_string()),
		}
	}

	fn from_pcm16(data: &[u8], _channels: u8) -> Result<Vec<f32>, String> {
		if data.len() % 2 != 0 {
			return Err("invalid pcm16 data length".to_string());
		}
		let samples = data.len() / 2;
		let mut result = Vec::with_capacity(samples);
		for i in 0..samples {
			let offset = i * 2;
			let sample = i16::from_le_bytes([data[offset], data[offset + 1]]);
			result.push(normalize_pcm16(sample));
		}
		Ok(result)
	}

	fn from_pcm24(data: &[u8], _channels: u8) -> Result<Vec<f32>, String> {
		if data.len() % 3 != 0 {
			return Err("invalid pcm24 data length".to_string());
		}
		let samples = data.len() / 3;
		let mut result = Vec::with_capacity(samples);
		for i in 0..samples {
			let offset = i * 3;
			let b0 = data[offset] as u32;
			let b1 = data[offset + 1] as u32;
			let b2 = data[offset + 2] as i32;
			let value = ((b0 | (b1 << 8) | ((b2 as u32) << 16)) as i32) << 8;
			result.push(normalize_pcm24(value));
		}
		Ok(result)
	}

	fn from_pcm32(data: &[u8], _channels: u8) -> Result<Vec<f32>, String> {
		if data.len() % 4 != 0 {
			return Err("invalid pcm32 data length".to_string());
		}
		let samples = data.len() / 4;
		let mut result = Vec::with_capacity(samples);
		for i in 0..samples {
			let offset = i * 4;
			let sample =
				f32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
			result.push(normalize_pcm32(sample));
		}
		Ok(result)
	}

	fn to_pcm16(samples: &[f32], _channels: u8) -> Result<Vec<u8>, String> {
		let mut result = Vec::with_capacity(samples.len() * 2);
		for sample in samples {
			let pcm = denormalize_pcm16(*sample);
			result.extend_from_slice(&pcm.to_le_bytes());
		}
		Ok(result)
	}

	fn to_pcm24(samples: &[f32], _channels: u8) -> Result<Vec<u8>, String> {
		let mut result = Vec::with_capacity(samples.len() * 3);
		for sample in samples {
			let pcm = denormalize_pcm24(*sample);
			result.push((pcm & 0xFF) as u8);
			result.push(((pcm >> 8) & 0xFF) as u8);
			result.push(((pcm >> 16) & 0xFF) as u8);
		}
		Ok(result)
	}

	fn to_pcm32(samples: &[f32], _channels: u8) -> Result<Vec<u8>, String> {
		let mut result = Vec::with_capacity(samples.len() * 4);
		for sample in samples {
			let pcm = denormalize_pcm32(*sample);
			result.extend_from_slice(&pcm.to_le_bytes());
		}
		Ok(result)
	}
}
