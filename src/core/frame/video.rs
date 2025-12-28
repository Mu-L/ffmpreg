#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
	RGB24,
	RGBA32,
	YUV420,
	YUV422,
	YUV444,
	GRAY8,
}

impl VideoFormat {
	pub fn bytes_per_pixel(&self) -> usize {
		match self {
			VideoFormat::RGB24 => 3,
			VideoFormat::RGBA32 => 4,
			// 1.5 bytes by pixel
			VideoFormat::YUV420 => 3 / 2,
			VideoFormat::YUV422 => 2,
			VideoFormat::YUV444 => 3,
			VideoFormat::GRAY8 => 1,
		}
	}
}

#[derive(Debug, Clone)]
pub struct FrameVideo {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub format: VideoFormat,
	pub keyframe: bool,
}

impl FrameVideo {
	pub fn new(data: Vec<u8>, width: u32, height: u32, format: VideoFormat, keyframe: bool) -> Self {
		Self { data, width, height, format, keyframe }
	}

	pub fn bytes_per_pixel(&self) -> usize {
		self.format.bytes_per_pixel()
	}

	pub fn expected_size(&self) -> usize {
		(self.width as usize) * (self.height as usize) * self.bytes_per_pixel()
	}
}
