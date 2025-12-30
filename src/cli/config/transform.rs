use super::track::parse_track_id;
use crate::{cli::config::parse_flags, io::Result};

#[derive(Debug, Default)]
pub struct TransformConfig {
	pub track: Option<usize>,
	pub normalize: Option<String>,
	pub trim: Option<String>,
	pub fade: Option<String>,
	pub reverse: Option<String>,
	pub speed: Option<String>,
	pub rotate: Option<String>,
	pub filter_chain: Option<String>,
}

pub fn parse_transform(tokens: Vec<String>) -> Result<TransformConfig> {
	let map = parse_flags(tokens, true);
	let track = parse_track_id(&map)?;

	Ok(TransformConfig {
		track,
		normalize: map.get("normalize").cloned(),
		trim: map.get("trim").cloned(),
		fade: map.get("fade").cloned(),
		reverse: map.get("reverse").cloned(),
		speed: map.get("speed").cloned(),
		rotate: map.get("rotate").cloned(),
		filter_chain: map.get("filter_chain").cloned(),
	})
}
