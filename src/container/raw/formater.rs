use crate::codecs;
use crate::core::frame::AudioFormat;

#[derive(Debug, Clone, Copy)]
pub struct RawPcmFormat {
	pub channels: u8,
	pub sample_rate: u32,
	pub bit_depth: u16,
}

impl Default for RawPcmFormat {
	fn default() -> Self {
		// default is pcm_16, stereo, 44.1kHz
		Self { channels: 2, sample_rate: 44100, bit_depth: 16 }
	}
}

impl RawPcmFormat {
	pub fn new_for_codec(codec: &str) -> Result<Self, String> {
		match codec {
			codecs::audio::PCM_S16LE => Ok(Self::default()),
			codecs::audio::PCM_S24LE => Ok(Self { bit_depth: 24, ..Self::default() }),
			codecs::audio::PCM_F32LE => Ok(Self { bit_depth: 32, ..Self::default() }),
			_ => Err(format!("raw codec '{}' is not supported", codec)),
		}
	}

	pub fn bytes_per_sample(&self) -> usize {
		(self.bit_depth / 8) as usize
	}

	pub fn bytes_per_frame(&self) -> usize {
		self.bytes_per_sample() * self.channels as usize
	}

	pub fn byte_rate(&self) -> u32 {
		self
			.sample_rate
			.saturating_mul(self.channels as u32)
			.saturating_mul(self.bytes_per_sample() as u32)
	}

	pub fn block_align(&self) -> u16 {
		self.channels as u16 * (self.bit_depth / 8)
	}

	pub fn audio_format(&self) -> AudioFormat {
		match self.bit_depth {
			16 => AudioFormat::PCM16,
			24 => AudioFormat::PCM24,
			32 => AudioFormat::PCM32,
			_ => AudioFormat::PCM16,
		}
	}

	pub fn to_codec_string(&self) -> &'static str {
		match self.bit_depth {
			16 => codecs::audio::PCM_S16LE,
			24 => codecs::audio::PCM_S24LE,
			32 => codecs::audio::PCM_F32LE,
			_ => codecs::audio::PCM_S16LE,
		}
	}

	pub fn apply_codec(&mut self, codec: &str) -> Result<(), String> {
		match codec {
			codecs::audio::PCM_S16LE => self.bit_depth = 16,
			codecs::audio::PCM_S24LE => self.bit_depth = 24,
			codecs::audio::PCM_F32LE => self.bit_depth = 32,
			_ => return Err(format!("raw codec '{}' is not supported", codec)),
		}
		Ok(())
	}
}
