use crate::core::ebml::types::{EbmlElement, EbmlType};

#[derive(Clone, Debug)]
pub struct MkvTrack {
	pub number: u64,
	pub uid: u64,
	pub track_type: u8,
	pub codec_id: String,
	pub codec_name: String,
	pub name: Option<String>,
	pub language: String,
	pub default_flag: bool,
	pub enabled: bool,
	pub video: Option<VideoTrackInfo>,
	pub audio: Option<AudioTrackInfo>,
}

#[derive(Clone, Debug)]
pub struct VideoTrackInfo {
	pub width: u64,
	pub height: u64,
	pub frame_rate: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct AudioTrackInfo {
	pub sample_rate: u64,
	pub channels: u64,
	pub bit_depth: u64,
}

impl MkvTrack {
	pub fn new(number: u64, track_type: u8) -> Self {
		MkvTrack {
			number,
			uid: 0,
			track_type,
			codec_id: String::new(),
			codec_name: String::new(),
			name: None,
			language: "und".to_string(),
			default_flag: true,
			enabled: true,
			video: None,
			audio: None,
		}
	}

	pub fn is_video(&self) -> bool {
		self.track_type == 1
	}

	pub fn is_audio(&self) -> bool {
		self.track_type == 2
	}

	pub fn is_subtitle(&self) -> bool {
		self.track_type == 3
	}

	pub fn from_element(elem: &EbmlElement) -> Option<Self> {
		if elem.id != 0xAE {
			return None;
		}

		let mut track = MkvTrack::new(0, 0);

		let children = match &elem.data {
			EbmlType::Master(c) => Some(c.clone()),
			EbmlType::Binary(data) => {
				use crate::core::ebml::parser::EbmlParser;
				EbmlParser::parse_document(data).ok().map(|(_, children)| children)
			}
			_ => None,
		};

		if let Some(children) = children {
			for child in children {
				match child.id {
					0x57 | 0xD7 => track.number = parse_uint(&child.data),
					0x33C5 | 0x73C5 => track.uid = parse_uint(&child.data),
					0x1C | 0x83 => track.track_type = parse_uint(&child.data) as u8,
					0x6 | 0x86 => track.codec_id = parse_str(&child.data),
					0x3E383 | 0x258688 => track.codec_name = parse_str(&child.data),
					0x60 | 0x4D80 => track.name = Some(parse_str(&child.data)),
					0x2B59C | 0x22B59C => track.language = parse_str(&child.data),
					0x3 | 0x88 => track.default_flag = parse_uint(&child.data) != 0,
					0x15EE | 0x9A => track.enabled = parse_uint(&child.data) != 0,
					0xE0 => {
						track.video = match &child.data {
							EbmlType::Master(video_children) => parse_video_info(video_children),
							EbmlType::Binary(data) => {
								use crate::core::ebml::parser::EbmlParser;
								match EbmlParser::parse_document(data) {
									Ok((_, parsed)) => parse_video_info(&parsed),
									Err(_) => None,
								}
							}
							_ => None,
						};
					}
					0xE1 => {
						track.audio = match &child.data {
							EbmlType::Master(audio_children) => parse_audio_info(audio_children),
							EbmlType::Binary(data) => {
								use crate::core::ebml::parser::EbmlParser;
								match EbmlParser::parse_document(data) {
									Ok((_, parsed)) => parse_audio_info(&parsed),
									Err(_) => None,
								}
							}
							_ => None,
						};
					}
					_ => {}
				}
			}
		}

		Some(track)
	}
}

fn parse_uint(data: &EbmlType) -> u64 {
	match data {
		EbmlType::Uinteger(v) => *v,
		EbmlType::Integer(v) => *v as u64,
		EbmlType::Binary(bytes) => {
			let mut value = 0u64;
			for byte in bytes {
				value = (value << 8) | (*byte as u64);
			}
			value
		}
		_ => 0,
	}
}

fn parse_str(data: &EbmlType) -> String {
	match data {
		EbmlType::String(s) => s.clone(),
		EbmlType::Binary(bytes) => String::from_utf8_lossy(bytes).to_string(),
		_ => String::new(),
	}
}

fn infer_track_type(codec_id: &str) -> &'static str {
	if codec_id.starts_with("V_") {
		"Video"
	} else if codec_id.starts_with("A_") {
		"Audio"
	} else if codec_id.starts_with("S_") {
		"Subtitles"
	} else {
		"Unknown"
	}
}

fn parse_video_info(children: &[EbmlElement]) -> Option<VideoTrackInfo> {
	let mut width = 0;
	let mut height = 0;
	let mut frame_rate = None;

	for child in children {
		match child.id {
			0xB0 => width = parse_uint(&child.data),
			0xBA => height = parse_uint(&child.data),
			0x2383E3 => {
				if let EbmlType::Float(f) = &child.data {
					frame_rate = Some(*f);
				}
			}
			_ => {}
		}
	}

	Some(VideoTrackInfo { width, height, frame_rate })
}

fn parse_audio_info(children: &[EbmlElement]) -> Option<AudioTrackInfo> {
	let mut sample_rate = 0;
	let mut channels = 0;
	let mut bit_depth = 0;

	for child in children {
		match child.id {
			0xB5 => {
				if let EbmlType::Float(f) = &child.data {
					sample_rate = *f as u64;
				}
			}
			0x9F => channels = parse_uint(&child.data),
			0x6264 => bit_depth = parse_uint(&child.data),
			_ => {}
		}
	}

	Some(AudioTrackInfo { sample_rate, channels, bit_depth })
}

#[derive(Clone, Debug, Default)]
pub struct MkvTracks {
	pub tracks: Vec<MkvTrack>,
}

impl MkvTracks {
	pub fn new() -> Self {
		MkvTracks { tracks: Vec::new() }
	}

	pub fn from_element(elem: &EbmlElement) -> Option<Self> {
		if elem.id != 0x1654AE6B {
			return None;
		}

		let mut tracks = MkvTracks::new();
		if let EbmlType::Master(children) = &elem.data {
			for child in children {
				if let Some(track) = MkvTrack::from_element(child) {
					tracks.tracks.push(track);
				}
			}
		}

		Some(tracks)
	}

	pub fn video_tracks(&self) -> Vec<&MkvTrack> {
		self.tracks.iter().filter(|t| t.is_video()).collect()
	}

	pub fn audio_tracks(&self) -> Vec<&MkvTrack> {
		self.tracks.iter().filter(|t| t.is_audio()).collect()
	}

	pub fn subtitle_tracks(&self) -> Vec<&MkvTrack> {
		self.tracks.iter().filter(|t| t.is_subtitle()).collect()
	}

	pub fn find_track(&self, number: u64) -> Option<&MkvTrack> {
		self.tracks.iter().find(|t| t.number == number)
	}

	pub fn print_summary(&self) {
		println!("  Tracks:");
		if self.tracks.is_empty() {
			println!("    (none)");
			return;
		}

		for track in &self.tracks {
			print_track(track);
		}
	}
}

fn print_track(track: &MkvTrack) {
	use super::codec;

	let inferred_type = infer_track_type(&track.codec_id);
	let track_type = match track.track_type {
		1 => "Video",
		2 => "Audio",
		3 => "Subtitles",
		0 => inferred_type,
		_ => "Unknown",
	};

	let mapped_codec = if track.codec_id.starts_with("V_") {
		codec::map_video_codec(&track.codec_id)
	} else if track.codec_id.starts_with("A_") {
		codec::map_audio_codec(&track.codec_id)
	} else if track.codec_id.starts_with("S_") {
		codec::map_subtitle_codec(&track.codec_id)
	} else {
		crate::codecs::UNKNOWN
	};

	let display_codec = if mapped_codec == crate::codecs::UNKNOWN {
		track.codec_id.clone()
	} else {
		mapped_codec.to_string()
	};

	println!("    Track #{}: {} - {} [{}]", track.number, track_type, display_codec, track.language);

	if let Some(video) = &track.video {
		if video.width > 0 && video.height > 0 {
			let fps = video.frame_rate.map(|f| format!(" @ {:.2}fps", f)).unwrap_or_default();
			println!("      {}x{}{}", video.width, video.height, fps);
		}
	}

	if let Some(audio) = &track.audio {
		if audio.sample_rate > 0 {
			println!(
				"      {}Hz, {} channels, {} bit",
				audio.sample_rate, audio.channels, audio.bit_depth
			);
		}
	}
}
