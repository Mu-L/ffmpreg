pub mod demuxer;
pub mod metadata;
pub mod muxer;
pub mod utils;
pub use crate::core;
pub use demuxer::WavDemuxer;
pub use metadata::WavMetadata;
pub use muxer::WavMuxer;
pub use utils::{SampleConverter, WavInfo};

#[derive(Debug, Clone, Copy)]
pub struct WavFormat {
	pub channels: u8,
	pub sample_rate: u32,
	pub bit_depth: u16,
	pub format_code: u16,
}

impl Default for WavFormat {
	fn default() -> Self {
		WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16, format_code: 1 }
	}
}

impl WavFormat {
	pub fn bytes_per_sample(&self) -> usize {
		if self.format_code == 0x11 {
			// ADPCM: 4 bits per sample, but we work with blocks
			1 // Return 1 to avoid division by zero
		} else {
			(self.bit_depth / 8) as usize
		}
	}

	pub fn bytes_per_frame(&self) -> usize {
		if self.format_code == 0x11 {
			// ADPCM block: header (4 bytes per channel) + encoded data
			// Use block_align as provided by WAV header
			self.block_align() as usize
		} else {
			self.bytes_per_sample() * self.channels as usize
		}
	}

	pub fn byte_rate(&self) -> u32 {
		self
			.sample_rate
			.saturating_mul(self.channels as u32)
			.saturating_mul(self.bytes_per_sample() as u32)
	}

	pub fn block_align(&self) -> u16 {
		(self.channels as u16).saturating_mul(self.bytes_per_sample() as u16)
	}

	pub fn to_codec_string(&self) -> &'static str {
		match self.format_code {
			0x11 => "adpcm_ima_wav",
			_ => match self.bit_depth {
				16 => "pcm_s16le",
				24 => "pcm_s24le",
				32 => "pcm_f32",
				_ => "pcm",
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmFormat {
	S16LE,
	S24LE,
	F32LE,
}

impl PcmFormat {
	pub fn from_bit_depth(depth: u16) -> Option<Self> {
		match depth {
			16 => Some(PcmFormat::S16LE),
			24 => Some(PcmFormat::S24LE),
			32 => Some(PcmFormat::F32LE),
			_ => None,
		}
	}

	pub fn to_audio_format(&self) -> core::AudioFormat {
		match self {
			PcmFormat::S16LE => core::AudioFormat::PCM16,
			PcmFormat::S24LE => core::AudioFormat::PCM24,
			PcmFormat::F32LE => core::AudioFormat::PCM32,
		}
	}

	pub fn codec_name(&self) -> &'static str {
		match self {
			PcmFormat::S16LE => "pcm_s16le",
			PcmFormat::S24LE => "pcm_s24le",
			PcmFormat::F32LE => "pcm_f32",
		}
	}

	pub fn bit_depth(&self) -> u16 {
		match self {
			PcmFormat::S16LE => 16,
			PcmFormat::S24LE => 24,
			PcmFormat::F32LE => 32,
		}
	}
}

pub fn check_codec(codec: &str) -> Result<PcmFormat, String> {
	match codec {
		"pcm_s16le" => Ok(PcmFormat::S16LE),
		"pcm_s24le" => Ok(PcmFormat::S24LE),
		"pcm_f32le" => Ok(PcmFormat::F32LE),
		_ => Err(format!("wav codec '{}' is not supported", codec)),
	}
}
