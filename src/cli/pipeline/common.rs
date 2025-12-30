use crate::cli::config;

#[derive(Debug, Default)]
pub struct Pipeline {
	pub input: String,
	pub output: String,
	pub audio: config::AudioConfig,
	pub video: config::VideoConfig,
	pub subtitle: config::SubtitleConfig,
	pub transform: config::TransformConfig,
}

impl Pipeline {
	pub fn new(input: &str, output: &str) -> Self {
		Self { input: input.to_string(), output: output.to_string(), ..Default::default() }
	}

	pub fn with_audio(&mut self, audio: config::AudioConfig) {
		self.audio = audio;
	}

	pub fn with_video(&mut self, video: config::VideoConfig) {
		self.video = video;
	}

	pub fn with_subtitle(&mut self, subtitle: config::SubtitleConfig) {
		self.subtitle = subtitle;
	}

	pub fn with_transform(&mut self, transform: config::TransformConfig) {
		self.transform = transform;
	}
}
