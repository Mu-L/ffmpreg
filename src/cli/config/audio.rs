use super::track::parse_track_id;
use crate::{cli::config::parse_flags, io::Result};

#[derive(Debug, Default)]
pub struct AudioConfig {
	pub track: Option<usize>,
	pub codec: Option<String>,
	pub channels: Option<String>,
	pub sample_rate: Option<String>,
	pub volume: Option<String>,
}

pub fn parse_audio(tokens: Vec<String>) -> Result<AudioConfig> {
	let map = parse_flags(tokens, false);
	let track = parse_track_id(&map)?;

	Ok(AudioConfig {
		track,
		codec: map.get("codec").cloned(),
		channels: map.get("channels").cloned(),
		sample_rate: map.get("sample_rate").cloned(),
		volume: map.get("volume").cloned(),
	})
}
