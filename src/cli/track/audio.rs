use crate::{
	cli::track::{self, utils::Kv},
	io,
};

#[derive(Debug, Default)]
pub struct AudioOptions {
	pub codec: Option<String>,
	pub channels: Option<String>,
	pub sample_rate: Option<String>,
	pub volume: Option<String>,
}

#[derive(Debug, Default)]
pub struct AudioTrackOptions {
	pub track: track::Track,
	pub options: AudioOptions,
}

fn parse_audio_options(kv: &Kv) -> AudioOptions {
	let mut options = AudioOptions::default();

	options.codec = kv.get("codec").cloned();
	options.channels = kv.get("channels").cloned();
	options.sample_rate = kv.get("sample_rate").cloned();
	options.volume = kv.get("volume").cloned();

	options
}

pub fn parse_audio_track_options(tokens: Vec<String>) -> io::Result<AudioTrackOptions> {
	let kv = track::utils::parse_kv(&tokens);
	let track = track::parse_track(&kv)?;

	let options = parse_audio_options(&kv);
	Ok(AudioTrackOptions { track, options })
}
