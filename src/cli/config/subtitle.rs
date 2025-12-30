use super::track::parse_track_id;
use crate::{
	cli::config::parse_flags,
	io::{Error, Result},
};

#[derive(Debug, Default)]
pub struct SubtitleConfig {
	pub track: Option<usize>,
	pub language: Option<String>,
	pub codec: Option<String>,
	pub default: Option<String>,
	pub shift: Option<String>,
	pub font_size: Option<String>,
	pub color: Option<String>,
	pub position: Option<String>,
	pub fps: Option<String>,
	pub encoding: Option<String>,
	pub translate: Option<String>,
}

impl SubtitleConfig {
	pub fn set_track(&mut self, track: Option<usize>) {
		self.track = track;
	}

	pub fn set_language(&mut self, language: Option<String>) {
		self.language = language;
	}

	pub fn is_empty(&self) -> bool {
		self.track.is_none()
			&& self.language.is_none()
			&& self.codec.is_none()
			&& self.default.is_none()
			&& self.shift.is_none()
			&& self.font_size.is_none()
			&& self.color.is_none()
			&& self.position.is_none()
			&& self.fps.is_none()
			&& self.encoding.is_none()
			&& self.translate.is_none()
	}
}

pub fn parse_subtitle(tokens: Vec<String>) -> Result<SubtitleConfig> {
	let map = parse_flags(tokens, false);

	let mut config = SubtitleConfig {
		track: None,
		language: None,
		codec: map.get("codec").cloned(),
		default: map.get("default").cloned(),
		shift: map.get("shift").cloned(),
		font_size: map.get("font_size").cloned(),
		color: map.get("color").cloned(),
		position: map.get("position").cloned(),
		fps: map.get("fps").cloned(),
		encoding: map.get("encoding").cloned(),
		translate: map.get("translate").cloned(),
	};

	let track = parse_track_id(&map)?;
	let language = map.get("language").cloned();

	if track.is_none() && language.is_none() && !config.is_empty() {
		return Err(Error::invalid_data("subtitle needs track or language"));
	}

	config.set_language(language);
	config.set_track(track);

	Ok(config)
}
