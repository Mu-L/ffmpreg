pub mod read;
pub mod write;

pub use read::AviReader;
pub use write::AviWriter;

pub const RIFF_SIGNATURE: &[u8; 4] = b"RIFF";
pub const AVI_SIGNATURE: &[u8; 4] = b"AVI ";
pub const LIST_SIGNATURE: &[u8; 4] = b"LIST";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
	Video,
	Audio,
	Midi,
	Text,
	Unknown,
}

impl StreamType {
	pub fn from_fourcc(fourcc: &[u8; 4]) -> Self {
		match fourcc {
			b"vids" => StreamType::Video,
			b"auds" => StreamType::Audio,
			b"mids" => StreamType::Midi,
			b"txts" => StreamType::Text,
			_ => StreamType::Unknown,
		}
	}

	pub fn as_fourcc(&self) -> [u8; 4] {
		match self {
			StreamType::Video => *b"vids",
			StreamType::Audio => *b"auds",
			StreamType::Midi => *b"mids",
			StreamType::Text => *b"txts",
			StreamType::Unknown => *b"    ",
		}
	}
}

#[derive(Debug, Clone)]
pub struct AviMainHeader {
	pub microseconds_per_frame: u32,
	pub max_bytes_per_sec: u32,
	pub padding_granularity: u32,
	pub flags: u32,
	pub total_frames: u32,
	pub initial_frames: u32,
	pub streams: u32,
	pub suggested_buffer_size: u32,
	pub width: u32,
	pub height: u32,
}

impl Default for AviMainHeader {
	fn default() -> Self {
		Self {
			microseconds_per_frame: 33333,
			max_bytes_per_sec: 0,
			padding_granularity: 0,
			flags: 0x10,
			total_frames: 0,
			initial_frames: 0,
			streams: 1,
			suggested_buffer_size: 0,
			width: 640,
			height: 480,
		}
	}
}

#[derive(Debug, Clone)]
pub struct AviStreamHeader {
	pub stream_type: StreamType,
	pub handler: [u8; 4],
	pub flags: u32,
	pub priority: u16,
	pub language: u16,
	pub initial_frames: u32,
	pub scale: u32,
	pub rate: u32,
	pub start: u32,
	pub length: u32,
	pub suggested_buffer_size: u32,
	pub quality: u32,
	pub sample_size: u32,
	pub rect: [u16; 4],
}

impl Default for AviStreamHeader {
	fn default() -> Self {
		Self {
			stream_type: StreamType::Video,
			handler: [0u8; 4],
			flags: 0,
			priority: 0,
			language: 0,
			initial_frames: 0,
			scale: 1,
			rate: 30,
			start: 0,
			length: 0,
			suggested_buffer_size: 0,
			quality: 0,
			sample_size: 0,
			rect: [0, 0, 640, 480],
		}
	}
}

#[derive(Debug, Clone)]
pub struct BitmapInfoHeader {
	pub size: u32,
	pub width: i32,
	pub height: i32,
	pub planes: u16,
	pub bit_count: u16,
	pub compression: [u8; 4],
	pub size_image: u32,
	pub x_pels_per_meter: i32,
	pub y_pels_per_meter: i32,
	pub clr_used: u32,
	pub clr_important: u32,
}

impl Default for BitmapInfoHeader {
	fn default() -> Self {
		Self {
			size: 40,
			width: 640,
			height: 480,
			planes: 1,
			bit_count: 24,
			compression: *b"DIB ",
			size_image: 0,
			x_pels_per_meter: 0,
			y_pels_per_meter: 0,
			clr_used: 0,
			clr_important: 0,
		}
	}
}

#[derive(Debug, Clone)]
pub struct WaveFormatEx {
	pub format_tag: u16,
	pub channels: u16,
	pub samples_per_sec: u32,
	pub avg_bytes_per_sec: u32,
	pub block_align: u16,
	pub bits_per_sample: u16,
}

impl Default for WaveFormatEx {
	fn default() -> Self {
		Self {
			format_tag: 1,
			channels: 2,
			samples_per_sec: 44100,
			avg_bytes_per_sec: 176400,
			block_align: 4,
			bits_per_sample: 16,
		}
	}
}

#[derive(Debug, Clone)]
pub struct AviFormat {
	pub main_header: AviMainHeader,
	pub streams: Vec<AviStream>,
}

#[derive(Debug, Clone)]
pub struct AviStream {
	pub header: AviStreamHeader,
	pub video_format: Option<BitmapInfoHeader>,
	pub audio_format: Option<WaveFormatEx>,
}

impl Default for AviFormat {
	fn default() -> Self {
		Self { main_header: AviMainHeader::default(), streams: Vec::new() }
	}
}
