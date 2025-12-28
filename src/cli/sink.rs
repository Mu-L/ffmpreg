use crate::cli::source;
use crate::container;
use crate::container::aac::AACFormat;
use crate::container::wav::WavFormat;
use crate::core::Muxer;
use crate::io::{File, Result};

#[derive(Debug, Clone)]
pub enum Metadata {
	Wav(WavFormat),
	WavAdpcm(WavFormat),
	Aac(AACFormat),
	Mov,
}

impl Metadata {
	pub fn from_extension(extension: &str) -> Self {
		match extension {
			container::WAV => Metadata::Wav(WavFormat::default()),
			container::AAC => Metadata::Aac(AACFormat::default()),
			container::MOV => Metadata::Mov,
			_ => Metadata::Wav(WavFormat::default()),
		}
	}

	pub fn with_input_format(extension: &str, metadata: Option<&source::Metadata>) -> Self {
		match (extension, metadata) {
			(container::WAV, Some(source::Metadata::Wav(format))) => Metadata::Wav(*format),
			(container::AAC, Some(source::Metadata::Aac(format))) => Metadata::Aac(*format),
			(container::MOV | container::MP4, Some(source::Metadata::Mov)) => Metadata::Mov,
			(container::WAV, _) => Metadata::Wav(WavFormat::default()),
			(container::AAC, _) => Metadata::Aac(AACFormat::default()),
			(container::MOV | container::MP4, _) => Metadata::Mov,
			_ => Metadata::Wav(WavFormat::default()),
		}
	}
}

pub struct Sink {
	pub muxer: Box<dyn Muxer>,
	pub path: String,
}

impl Sink {
	pub fn new(muxer: Box<dyn Muxer>) -> Self {
		Self { muxer, path: Default::default() }
	}

	pub fn with_path(mut self, path: String) -> Self {
		self.path = path;
		self
	}
}

pub fn open_sink_muxer(path: &str, metadata: Metadata) -> Result<Sink> {
	let file = File::create(path)?;
	let result = match metadata {
		Metadata::Wav(format) => wav_sink_muxer(file, format),
		Metadata::WavAdpcm(format) => wav_adpcm_sink_muxer(file, format),
		Metadata::Aac(format) => aac_sink_muxer(file, format),
		Metadata::Mov => mov_sink_muxer(file),
	};
	result.map(|sink| sink.with_path(path.to_string()))
}

fn wav_sink_muxer(file: File, format: WavFormat) -> Result<Sink> {
	let muxer = container::wav::WavMuxer::new(file, format)?;
	Ok(Sink::new(Box::new(muxer)))
}

fn wav_adpcm_sink_muxer(file: File, mut format: WavFormat) -> Result<Sink> {
	format.format_code = 0x11; // IMA ADPCM
	format.bit_depth = 4;
	let muxer = container::wav::WavMuxer::new(file, format)?;
	Ok(Sink::new(Box::new(muxer)))
}

fn aac_sink_muxer(file: File, format: AACFormat) -> Result<Sink> {
	let muxer = container::aac::AACMuxer::new(file, format.sample_rate, format.channels)?;
	Ok(Sink::new(Box::new(muxer)))
}

fn mov_sink_muxer(file: File) -> Result<Sink> {
	let muxer = container::mov::MovMuxer::new(file)?;
	Ok(Sink::new(Box::new(muxer)))
}
