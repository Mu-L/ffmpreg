use std::collections::{HashMap, HashSet};

use crate::{
	codecs, container,
	io::{self, Error},
};

#[derive(Default, Debug, Clone)]
pub struct ContainerCompatible {
	pub name: String,
	pub video_codecs: HashSet<String>,
	pub audio_codecs: HashSet<String>,
	pub subtitle_formats: HashSet<String>,
}

impl ContainerCompatible {
	pub fn new(name: &str) -> Self {
		Self { name: name.into(), ..Default::default() }
	}

	pub fn supports_video<I: IntoIterator<Item = impl Into<String>>>(&mut self, codecs: I) {
		self.video_codecs.extend(codecs.into_iter().map(|s| s.into()));
	}

	pub fn supports_audio<I: IntoIterator<Item = impl Into<String>>>(&mut self, codecs: I) {
		self.audio_codecs.extend(codecs.into_iter().map(|s| s.into()));
	}

	pub fn supports_subtitles<I: IntoIterator<Item = impl Into<String>>>(&mut self, formats: I) {
		self.subtitle_formats.extend(formats.into_iter().map(|s| s.into()));
	}

	pub fn assert_video_codec(&self, codec: &str) -> io::Result<()> {
		if self.video_codecs.contains(codec) {
			return Ok(());
		}
		let message = format!("codec '{}' not supported in '{}'", codec, self.name);
		Err(Error::invalid_data(message))
	}

	pub fn assert_audio_codec(&self, codec: &str) -> io::Result<()> {
		if self.audio_codecs.contains(codec) {
			return Ok(());
		}
		let message = format!("codec '{}' not supported in '{}'", codec, self.name);
		Err(Error::invalid_data(message))
	}

	pub fn assert_subtitle_format(&self, fmt: &str) -> io::Result<()> {
		if self.subtitle_formats.contains(fmt) {
			return Ok(());
		}

		let message = format!("format '{}' not supported in '{}'", fmt, self.name);
		Err(Error::invalid_data(message))
	}
}

#[derive(Debug, Clone)]
pub struct Compatible {
	pub graph: HashMap<&'static str, ContainerCompatible>,
}

impl Default for Compatible {
	fn default() -> Self {
		Self::new()
	}
}

impl Compatible {
	pub fn new() -> Self {
		use crate::core::compatible::*;

		let mut graph = HashMap::new();

		// video container
		let mut mp4 = ContainerCompatible::new(container::MP4);
		mp4.supports_video([codecs::video::H264, codecs::video::H265]);
		mp4.supports_audio([codecs::audio::AAC, codecs::audio::MP3]);
		mp4.supports_subtitles([codecs::subtitle::MOV_TEXT]);
		graph.insert(container::MP4, mp4);

		let mut mkv = ContainerCompatible::new(container::MKV);
		mkv.supports_video([
			codecs::video::H264,
			codecs::video::H265,
			codecs::video::VP9,
			codecs::video::AV1,
		]);
		mkv.supports_audio([
			codecs::audio::AAC,
			codecs::audio::VORBIS,
			codecs::audio::OPUS,
			codecs::audio::MP3,
		]);

		mkv.supports_subtitles([codecs::subtitle::SRT, codecs::subtitle::ASS, codecs::subtitle::VTT]);
		graph.insert(container::MKV, mkv);

		let mut mov = ContainerCompatible::new(container::MOV);
		mov.supports_video([codecs::video::H264, codecs::video::H265]);
		mov.supports_audio([codecs::audio::AAC, codecs::audio::MP3]);
		mov.supports_subtitles([codecs::subtitle::MOV_TEXT]);
		graph.insert(container::MOV, mov);

		let mut webm = ContainerCompatible::new(container::WEBM);
		webm.supports_video([codecs::video::VP8, codecs::video::VP9, codecs::video::AV1]);
		webm.supports_audio([codecs::audio::VORBIS, codecs::audio::OPUS]);
		webm.supports_subtitles([codecs::subtitle::VTT]);
		graph.insert(container::WEBM, webm);

		let mut avi = ContainerCompatible::new(container::AVI);
		avi.supports_video([codecs::video::MPEG4, codecs::video::H264]);
		avi.supports_audio([codecs::audio::MP3, codecs::audio::AAC]);
		graph.insert(container::AVI, avi);

		let mut ogv = ContainerCompatible::new(container::OGV);
		ogv.supports_video([codecs::video::THEORA]);
		ogv.supports_audio([codecs::audio::VORBIS]);
		graph.insert(container::OGV, ogv);

		let mut flv = ContainerCompatible::new(container::FLV);
		flv.supports_video([codecs::video::H264, codecs::video::VP6]);
		flv.supports_audio([codecs::audio::MP3]);
		graph.insert(container::FLV, flv);

		let mut mxf = ContainerCompatible::new(container::MXF);
		mxf.supports_video([codecs::video::MPEG2, codecs::video::H264, codecs::video::H265]);
		mxf.supports_audio([codecs::audio::PCM_S16LE, codecs::audio::AAC]);
		graph.insert(container::MXF, mxf);

		let mut ts = ContainerCompatible::new(container::TS);
		ts.supports_video([codecs::video::H264, codecs::video::H265, codecs::video::MPEG2]);
		ts.supports_audio([codecs::audio::AAC, codecs::audio::MP2]);
		graph.insert(container::TS, ts);

		// audio container
		let mut mp3 = ContainerCompatible::new(container::MP3);
		mp3.supports_audio([codecs::audio::MP3]);
		graph.insert(container::MP3, mp3);

		let mut aac = ContainerCompatible::new(container::AAC);
		aac.supports_audio([codecs::audio::AAC]);
		graph.insert(container::AAC, aac);

		let mut opus = ContainerCompatible::new(container::OPUS);
		opus.supports_audio([codecs::audio::OPUS]);
		graph.insert(container::OPUS, opus);

		let mut flac = ContainerCompatible::new(container::FLAC);
		flac.supports_audio([codecs::audio::FLAC]);
		graph.insert(container::FLAC, flac);

		let mut wav = ContainerCompatible::new(container::WAV);
		wav.supports_audio([
			codecs::audio::PCM_S16LE,
			codecs::audio::PCM_S24LE,
			codecs::audio::PCM_F32LE,
		]);
		graph.insert(container::WAV, wav);

		let mut m4a = ContainerCompatible::new(container::M4A);
		m4a.supports_audio([codecs::audio::AAC, codecs::audio::ALAC]);
		graph.insert(container::M4A, m4a);

		let mut alac = ContainerCompatible::new(container::ALAC);
		alac.supports_audio([codecs::audio::ALAC]);
		graph.insert(container::ALAC, alac);

		let mut ogg_audio = ContainerCompatible::new(container::OGG);
		ogg_audio.supports_audio([codecs::audio::VORBIS, codecs::audio::OPUS]);
		graph.insert(container::OGG, ogg_audio);

		Self { graph }
	}

	pub fn container(&self, extension: &str) -> Option<&ContainerCompatible> {
		self.graph.get(extension)
	}

	pub fn assert_container_supported(&self, extension: &str) -> io::Result<()> {
		if self.graph.contains_key(extension) {
			return Ok(());
		}
		Err(Error::invalid_data(format!("'{}' is not supported", extension)))
	}

	pub fn assert_video_supported(&self, container: &str, codec: &str) -> io::Result<()> {
		match self.graph.get(container) {
			Some(container) => container.assert_video_codec(codec),
			None => Err(Error::invalid_data(format!("'{}' is not supported", container))),
		}
	}

	pub fn assert_audio_supported(&self, container: &str, codec: &str) -> io::Result<()> {
		match self.graph.get(container) {
			Some(container) => container.assert_audio_codec(codec),
			None => Err(Error::invalid_data(format!("'{}' is not supported", container))),
		}
	}

	pub fn assert_subtitle_supported(&self, container: &str, fmt: &str) -> io::Result<()> {
		match self.graph.get(container) {
			Some(container) => container.assert_subtitle_format(fmt),
			None => Err(Error::invalid_data(format!("'{}' is not supported", container))),
		}
	}

	pub fn assert_subtitle_format_supported(&self, container: &str, fmt: &str) -> io::Result<()> {
		match self.graph.get(container) {
			Some(container) => container.assert_subtitle_format(fmt),
			None => Err(Error::invalid_data(format!("'{}' is not supported", container))),
		}
	}
}
