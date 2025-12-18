pub mod read;
pub mod write;

pub use read::FlacReader;
pub use write::FlacWriter;

pub const FLAC_SIGNATURE: &[u8; 4] = b"fLaC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataBlockType {
	StreamInfo,
	Padding,
	Application,
	SeekTable,
	VorbisComment,
	CueSheet,
	Picture,
	Reserved,
	Invalid,
}

impl From<u8> for MetadataBlockType {
	fn from(value: u8) -> Self {
		match value & 0x7F {
			0 => MetadataBlockType::StreamInfo,
			1 => MetadataBlockType::Padding,
			2 => MetadataBlockType::Application,
			3 => MetadataBlockType::SeekTable,
			4 => MetadataBlockType::VorbisComment,
			5 => MetadataBlockType::CueSheet,
			6 => MetadataBlockType::Picture,
			127 => MetadataBlockType::Invalid,
			_ => MetadataBlockType::Reserved,
		}
	}
}

#[derive(Debug, Clone)]
pub struct FlacFormat {
	pub min_block_size: u16,
	pub max_block_size: u16,
	pub min_frame_size: u32,
	pub max_frame_size: u32,
	pub sample_rate: u32,
	pub channels: u8,
	pub bits_per_sample: u8,
	pub total_samples: u64,
	pub md5_signature: [u8; 16],
}

impl Default for FlacFormat {
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
			md5_signature: [0u8; 16],
		}
	}
}

impl FlacFormat {
	pub fn bytes_per_sample(&self) -> usize {
		((self.bits_per_sample + 7) / 8) as usize
	}

	pub fn bytes_per_frame(&self) -> usize {
		self.bytes_per_sample() * self.channels as usize
	}
}

pub fn parse_streaminfo(data: &[u8]) -> Option<FlacFormat> {
	if data.len() < 34 {
		return None;
	}

	let min_block_size = u16::from_be_bytes([data[0], data[1]]);
	let max_block_size = u16::from_be_bytes([data[2], data[3]]);

	let min_frame_size = u32::from_be_bytes([0, data[4], data[5], data[6]]);
	let max_frame_size = u32::from_be_bytes([0, data[7], data[8], data[9]]);

	let sample_rate_high =
		((data[10] as u32) << 12) | ((data[11] as u32) << 4) | ((data[12] as u32) >> 4);

	let channels = ((data[12] >> 1) & 0x07) + 1;
	let bits_per_sample = (((data[12] & 0x01) << 4) | ((data[13] >> 4) & 0x0F)) + 1;

	let total_samples = (((data[13] as u64) & 0x0F) << 32)
		| ((data[14] as u64) << 24)
		| ((data[15] as u64) << 16)
		| ((data[16] as u64) << 8)
		| (data[17] as u64);

	let mut md5_signature = [0u8; 16];
	md5_signature.copy_from_slice(&data[18..34]);

	Some(FlacFormat {
		min_block_size,
		max_block_size,
		min_frame_size,
		max_frame_size,
		sample_rate: sample_rate_high,
		channels,
		bits_per_sample,
		total_samples,
		md5_signature,
	})
}
