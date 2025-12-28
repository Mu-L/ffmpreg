use crate::{
	cli::track::{self, utils::Kv},
	io,
};

#[derive(Debug, Default)]
pub struct SubtitleOptions {
	pub codec: Option<String>,
	pub default: Option<String>,
	pub shift: Option<String>,
	pub font_size: Option<String>,
	pub color: Option<String>,
	pub position: Option<String>,
	pub fps: Option<String>,
	pub encoding: Option<String>,

	// experimental
	pub translate: Option<String>,
}

impl SubtitleOptions {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_empty(&self) -> bool {
		self.codec.is_none()
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

#[derive(Debug, Default)]
pub struct SubtitleTrackOptions {
	pub track: Option<track::Track>,
	pub lang: Option<String>,
	pub options: SubtitleOptions,
}

fn parse_subtitle_options(kv: &Kv) -> SubtitleOptions {
	let mut options = SubtitleOptions::default();

	options.codec = kv.get("codec").cloned();
	options.default = kv.get("default").cloned();
	options.shift = kv.get("shift").cloned();
	options.font_size = kv.get("font_size").cloned();
	options.color = kv.get("color").cloned();
	options.position = kv.get("position").cloned();
	options.fps = kv.get("fps").cloned();
	options.encoding = kv.get("encoding").cloned();
	options.translate = kv.get("translate").cloned();

	options
}

pub fn parse_subtitle_track_options(tokens: Vec<String>) -> io::Result<SubtitleTrackOptions> {
	let kv = track::utils::parse_kv(&tokens);

	let track = if kv.contains_key("track") { Some(track::parse_track(&kv)?) } else { None };

	let lang = kv.get("lang").cloned();

	let options = parse_subtitle_options(&kv);

	if !options.is_empty() && track.is_none() && lang.is_none() {
		return Err(io::Error::invalid_data("subtitle missing track or lang"));
	}
	Ok(SubtitleTrackOptions { track, lang, options })
}
