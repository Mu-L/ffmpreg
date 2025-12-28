mod audio;
mod subtitle;
mod video;

pub use audio::*;
pub use subtitle::*;
pub use video::*;

use crate::core::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
	Audio,
	Video,
	Subtitle,
}

#[derive(Debug, Clone)]
pub enum FrameData {
	Audio(audio::FrameAudio),
	Video(video::FrameVideo),
	Subtitle(subtitle::FrameSubtitle),
}

#[derive(Debug, Clone)]
pub struct Frame {
	pub pts: i64,
	pub dts: Option<i64>,
	pub time: Time,
	pub stream_index: usize,
	pub stream_id: u32,
	pub data: FrameData,
}

impl Frame {
	pub fn new_audio(
		audio: audio::FrameAudio,
		time: Time,
		stream_index: usize,
		stream_id: u32,
	) -> Self {
		Self { pts: 0, dts: None, time, stream_index, data: FrameData::Audio(audio), stream_id }
	}

	pub fn new_video(
		video: video::FrameVideo,
		time: Time,
		stream_index: usize,
		stream_id: u32,
	) -> Self {
		Self { pts: 0, dts: None, time, stream_index, data: FrameData::Video(video), stream_id }
	}

	pub fn new_subtitle(
		subtitle: subtitle::FrameSubtitle,
		time: Time,
		stream_index: usize,
		stream_id: u32,
	) -> Self {
		Self { pts: 0, dts: None, time, stream_index, data: FrameData::Subtitle(subtitle), stream_id }
	}

	pub fn with_pts(mut self, pts: i64) -> Self {
		self.pts = pts;
		self
	}

	pub fn with_dts(mut self, dts: i64) -> Self {
		self.dts = Some(dts);
		self
	}

	pub fn size(&self) -> usize {
		match &self.data {
			FrameData::Audio(a) => a.data.len(),
			FrameData::Video(v) => v.data.len(),
			FrameData::Subtitle(c) => c.data.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.size() == 0
	}

	pub fn kind(&self) -> FrameKind {
		match &self.data {
			FrameData::Audio(_) => FrameKind::Audio,
			FrameData::Video(_) => FrameKind::Video,
			FrameData::Subtitle(_) => FrameKind::Subtitle,
		}
	}

	pub fn is_keyframe(&self) -> bool {
		matches!(&self.data, FrameData::Video(v) if v.keyframe)
	}

	pub fn duration_seconds(&self) -> f64 {
		self.time.to_seconds(self.pts)
	}

	pub fn audio(&self) -> Option<&audio::FrameAudio> {
		if let FrameData::Audio(audio) = &self.data {
			return Some(audio);
		}
		None
	}

	pub fn audio_mut(&mut self) -> Option<&mut audio::FrameAudio> {
		if let FrameData::Audio(audio) = &mut self.data {
			return Some(audio);
		}
		None
	}

	pub fn video(&self) -> Option<&video::FrameVideo> {
		if let FrameData::Video(video) = &self.data {
			return Some(video);
		}
		None
	}

	pub fn video_mut(&mut self) -> Option<&mut video::FrameVideo> {
		if let FrameData::Video(video) = &mut self.data {
			return Some(video);
		}
		None
	}

	pub fn subtitle(&self) -> Option<&subtitle::FrameSubtitle> {
		if let FrameData::Subtitle(subtitle) = &self.data {
			return Some(subtitle);
		}
		None
	}

	pub fn subtitle_mut(&mut self) -> Option<&mut subtitle::FrameSubtitle> {
		if let FrameData::Subtitle(subtitle) = &mut self.data {
			return Some(subtitle);
		}
		None
	}
}
