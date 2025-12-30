// use super::WavFormat;
// pub struct WavInfo {
// 	pub format: WavFormat,
// 	pub duration_seconds: f64,
// 	pub total_samples: u64,
// }

// impl WavInfo {
// 	pub fn new(format: WavFormat, total_data_bytes: u64) -> Self {
// 		// let bytes_per_sample = format.bytes_per_sample();
// 		let bytes_per_frame = format.bytes_per_frame();
// 		let total_frames = total_data_bytes / bytes_per_frame as u64;
// 		let total_samples = total_frames;
// 		let duration_seconds = total_samples as f64 / format.sample_rate as f64;

// 		Self { format, duration_seconds, total_samples }
// 	}

// 	pub fn bit_rate_kbps(&self) -> u32 {
// 		let bit_rate =
// 			(self.format.sample_rate as u64 * self.format.channels as u64 * self.format.bit_depth as u64)
// 				/ 1000;
// 		bit_rate as u32
// 	}

// 	pub fn display_info(&self) -> String {
// 		format!(
// 			"WAV: {}Hz {} channels, {} bits, {:.2}s ({}s bitrate: {} kbps)",
// 			self.format.sample_rate,
// 			self.format.channels,
// 			self.format.bit_depth,
// 			self.duration_seconds,
// 			self.format.to_codec_string(),
// 			self.bit_rate_kbps()
// 		)
// 	}
// }

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
