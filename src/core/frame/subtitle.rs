#[derive(Debug, Clone)]
pub struct FrameSubtitle {
	pub data: Vec<u8>,
	pub format: SubtitleFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleFormat {
	SRT,
	ASS,
	VTT,
}

impl FrameSubtitle {
	pub fn new(data: Vec<u8>, format: SubtitleFormat) -> Self {
		Self { data, format }
	}
}
