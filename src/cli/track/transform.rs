use crate::{
	cli::track::{self, utils::Kv},
	io,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TransformOptions {
	pub normalize: Option<String>,
	pub trim: Option<String>,
	pub fade: Option<String>,
	pub reverse: Option<String>,
	pub speed: Option<String>,
	pub rotate: Option<String>,
	pub filter_chain: Option<String>,
}

#[derive(Debug, Default)]
pub struct TransformTrackOptions {
	pub track: track::Track,
	pub options: TransformOptions,
}

fn parse_transform_options(kv: &Kv) -> io::Result<TransformOptions> {
	let mut options = TransformOptions::default();
	options.normalize = kv.get("normalize").cloned();
	options.trim = kv.get("trim").cloned();
	options.fade = kv.get("fade").cloned();
	options.reverse = kv.get("reverse").cloned();
	options.speed = kv.get("speed").cloned();
	options.rotate = kv.get("rotate").cloned();
	options.filter_chain = kv.get("filter_chain").cloned();

	Ok(options)
}

pub fn parse_transform_track_options(tokens: Vec<String>) -> io::Result<TransformTrackOptions> {
	let kv = track::utils::parse_kv(&tokens);
	let options = parse_transform_options(&kv)?;
	let track = track::parse_track(&kv)?;

	Ok(TransformTrackOptions { track, options })
}
