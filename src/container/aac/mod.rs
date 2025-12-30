pub mod demuxer;
pub mod muxer;

use crate::io::MediaRead;
pub use demuxer::AACDemuxer;
pub use muxer::AACMuxer;

#[derive(Debug, Clone, Copy)]
pub struct AACFormat {
	pub sample_rate: u32,
	pub channels: u8,
	pub bit_depth: u16,
}

impl AACFormat {
	pub fn new(sample_rate: u32, channels: u8, bit_depth: u16) -> Self {
		Self { sample_rate, channels, bit_depth }
	}
	pub fn from_demuxer<R: MediaRead>(demuxer: &AACDemuxer<R>) -> Self {
		let (sample_rate, channels) = demuxer.get_format_info();
		Self { sample_rate, channels, bit_depth: 16 }
	}
}

impl Default for AACFormat {
	fn default() -> Self {
		Self { sample_rate: 44100, channels: 2, bit_depth: 16 }
	}
}
