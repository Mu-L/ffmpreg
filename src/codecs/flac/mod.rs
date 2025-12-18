pub mod decode;
pub mod encode;
pub mod frame;
pub mod lpc;
pub mod rice;

pub use decode::FlacDecoder;
pub use encode::FlacEncoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubframeType {
	Constant,
	Verbatim,
	Fixed(u8),
	Lpc(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelAssignment {
	Independent,
	LeftSide,
	RightSide,
	MidSide,
}

impl ChannelAssignment {
	pub fn from_raw(_channels: u8, assignment: u8) -> Self {
		match assignment {
			0..=7 => Self::Independent,
			8 => Self::LeftSide,
			9 => Self::RightSide,
			10 => Self::MidSide,
			_ => Self::Independent,
		}
	}
}

pub struct FlacStreamInfo {
	pub min_block_size: u16,
	pub max_block_size: u16,
	pub min_frame_size: u32,
	pub max_frame_size: u32,
	pub sample_rate: u32,
	pub channels: u8,
	pub bits_per_sample: u8,
	pub total_samples: u64,
}

impl Default for FlacStreamInfo {
	fn default() -> Self {
		Self {
			min_block_size: 4096,
			max_block_size: 4096,
			min_frame_size: 0,
			max_frame_size: 0,
			sample_rate: 44100,
			channels: 2,
			bits_per_sample: 16,
			total_samples: 0,
		}
	}
}
