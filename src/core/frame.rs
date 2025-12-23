use crate::core::Timebase;

#[derive(Debug, Clone)]
pub enum FrameKind {
	Audio,
	Video,
	// Caption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
	RGB24,
	RGBA32,
	YUV420,
	YUV422,
	YUV444,
	GRAY8,
}

#[derive(Debug, Clone)]
pub struct FrameAudio {
	pub data: Vec<u8>,
	pub sample_rate: u32,
	pub channels: u8,
	pub nb_samples: usize,
}
impl FrameAudio {
	pub fn new(data: Vec<u8>, sample_rate: u32, channels: u8) -> Self {
		let nb_samples = data.len() / (channels as usize);
		Self { data, sample_rate, channels, nb_samples }
	}
	pub fn with_nb_samples(mut self, nb_samples: usize) -> Self {
		self.nb_samples = nb_samples;
		self
	}
}
#[derive(Debug, Clone)]
pub struct FrameVideo {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub format: VideoFormat,
}
impl FrameVideo {
	pub fn new(data: Vec<u8>, width: u32, height: u32, format: VideoFormat) -> Self {
		Self { data, width, height, format }
	}
}

#[derive(Debug, Clone)]
pub enum FrameData {
	Audio(FrameAudio),
	Video(FrameVideo),
	// Caption(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Frame {
	pub pts: i64,
	pub timebase: Timebase,
	pub stream_index: usize,
	pub data: FrameData,
}

impl Frame {
	pub fn new_audio(audio: FrameAudio, timebase: Timebase, stream_index: usize) -> Self {
		Self { pts: 0, timebase, stream_index, data: FrameData::Audio(audio) }
	}

	pub fn new_video(video: FrameVideo, timebase: Timebase, stream_index: usize) -> Self {
		Self { pts: 0, timebase, stream_index, data: FrameData::Video(video) }
	}

	pub fn with_pts(mut self, pts: i64) -> Self {
		self.pts = pts;
		self
	}

	pub fn size(&self) -> usize {
		match &self.data {
			FrameData::Audio(a) => a.data.len(),
			FrameData::Video(v) => v.data.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.size() == 0
	}

	pub fn kind(&self) -> FrameKind {
		match &self.data {
			FrameData::Audio(_) => FrameKind::Audio,
			FrameData::Video(_) => FrameKind::Video,
		}
	}

	pub fn with_data(mut self, data: FrameData) -> Self {
		self.data = data;
		self
	}

	pub fn audio(&self) -> Option<&FrameAudio> {
		if let FrameData::Audio(a) = &self.data { Some(a) } else { None }
	}

	pub fn audio_mut(&mut self) -> Option<&mut FrameAudio> {
		if let FrameData::Audio(a) = &mut self.data { Some(a) } else { None }
	}

	pub fn video(&self) -> Option<&FrameVideo> {
		if let FrameData::Video(v) = &self.data { Some(v) } else { None }
	}

	pub fn video_mut(&mut self) -> Option<&mut FrameVideo> {
		if let FrameData::Video(v) = &mut self.data { Some(v) } else { None }
	}

	pub fn is_audio_frame(&self) -> bool {
		matches!(&self.data, FrameData::Audio(_))
	}

	pub fn is_video_frame(&self) -> bool {
		matches!(&self.data, FrameData::Video(_))
	}
}
