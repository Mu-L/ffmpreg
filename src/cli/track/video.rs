use crate::{
	cli::track::{self, utils::Kv},
	io,
};

#[derive(Debug, Default)]
pub struct VideoOptions {
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

#[derive(Debug, Default)]
pub struct VideoTrackOptions {
	pub track: track::Track,
	pub options: VideoOptions,
}

fn parse_video_options(kv: &Kv) -> VideoOptions {
	let mut options = VideoOptions::default();

	options.codec = kv.get("codec").cloned();
	options.scale = kv.get("scale").cloned();
	options.width = kv.get("width").cloned();
	options.height = kv.get("height").cloned();
	options.fps = kv.get("fps").cloned();
	options.bitrate = kv.get("bitrate").cloned();
	options.aspect_ratio = kv.get("aspect_ratio").cloned();
	options.rotate = kv.get("rotate").cloned();
	options.brightness = kv.get("brightness").cloned();
	options.contrast = kv.get("contrast").cloned();

	options
}

pub fn parse_video_track_options(tokens: Vec<String>) -> io::Result<VideoTrackOptions> {
	let kv = track::utils::parse_kv(&tokens);
	let track = track::parse_track(&kv)?;
	let options = parse_video_options(&kv);
	Ok(VideoTrackOptions { track, options })
}
