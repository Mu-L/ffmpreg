#[derive(Debug, Clone)]
pub struct ShowOptions {
	pub json: bool,
	pub stream_filter: Option<usize>,
	pub frame_limit: usize,
	pub hex_limit: usize,
}

impl Default for ShowOptions {
	fn default() -> Self {
		Self { json: false, stream_filter: None, frame_limit: 10, hex_limit: 8 }
	}
}

#[derive(Debug, Clone)]
pub struct FileInfo {
	pub path: String,
	pub duration: f64,
	pub size: u64,
}

#[derive(Debug, Clone)]
pub enum StreamInfo {
	Video(VideoStreamInfo),
	Audio(AudioStreamInfo),
}

impl StreamInfo {
	pub fn index(&self) -> usize {
		match self {
			StreamInfo::Video(v) => v.index,
			StreamInfo::Audio(a) => a.index,
		}
	}
}

#[derive(Debug, Clone)]
pub struct VideoStreamInfo {
	pub index: usize,
	pub codec: String,
	pub pix_fmt: String,
	pub width: u32,
	pub height: u32,
	pub frame_rate: String,
	pub aspect_ratio: Option<String>,
	pub display_aspect: Option<String>,
	pub field_order: String,
}

#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
	pub index: usize,
	pub codec: String,
	pub sample_rate: u32,
	pub channels: u8,
	pub bit_depth: u16,
}

#[derive(Debug, Clone)]
pub struct FrameInfo {
	pub index: u64,
	pub pts: i64,
	pub keyframe: bool,
	pub size: usize,
	pub hex: String,
}

#[derive(Debug, Clone)]
pub struct MediaInfo {
	pub file: FileInfo,
	pub streams: Vec<StreamInfo>,
	pub frames: Vec<FrameInfo>,
}
