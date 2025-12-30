#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
	PCM16,
	PCM24,
	PCM32,
	FLAC,
	AAC,
	Opus,
	ADPCM,
}

impl AudioFormat {
	pub fn bytes_per_sample(&self) -> usize {
		match self {
			AudioFormat::PCM16 => 2,
			AudioFormat::PCM24 => 3,
			AudioFormat::PCM32 => 4,
			AudioFormat::FLAC | AudioFormat::AAC | AudioFormat::Opus | AudioFormat::ADPCM => 1,
		}
	}
}

#[derive(Debug, Clone)]
pub struct FrameAudio {
	pub data: Vec<u8>,
	pub sample_rate: u32,
	pub channels: u8,
	pub nb_samples: usize,
	pub format: AudioFormat,
}

impl FrameAudio {
	pub fn new(data: Vec<u8>, sample_rate: u32, channels: u8, format: AudioFormat) -> Self {
		let nb_samples = data.len() / (channels as usize * format.bytes_per_sample());
		Self { data, sample_rate, channels, nb_samples, format }
	}

	pub fn with_nb_samples(mut self, nb_samples: usize) -> Self {
		self.nb_samples = nb_samples;
		self
	}

	pub fn bytes_per_sample(&self) -> usize {
		self.format.bytes_per_sample()
	}
}
