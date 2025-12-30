use super::track::parse_track_id;
use crate::{cli::config::parse_flags, io::Result};

#[derive(Debug, Default)]
pub struct VideoConfig {
	pub track: Option<usize>,
	pub codec: Option<String>,
	pub scale: Option<String>,
	pub width: Option<String>,
	pub height: Option<String>,
	pub fps: Option<String>,
	pub bitrate: Option<String>,
	pub aspect_ratio: Option<String>,
	pub rotate: Option<String>,
	pub brightness: Option<String>,
	pub contrast: Option<String>,
}

pub fn parse_video(tokens: Vec<String>) -> Result<VideoConfig> {
	let map = parse_flags(tokens, false);
	let track = parse_track_id(&map)?;

	Ok(VideoConfig {
		track,
		codec: map.get("codec").cloned(),
		scale: map.get("scale").cloned(),
		width: map.get("width").cloned(),
		height: map.get("height").cloned(),
		fps: map.get("fps").cloned(),
		bitrate: map.get("bitrate").cloned(),
		aspect_ratio: map.get("aspect_ratio").cloned(),
		rotate: map.get("rotate").cloned(),
		brightness: map.get("brightness").cloned(),
		contrast: map.get("contrast").cloned(),
	})
}
