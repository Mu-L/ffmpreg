use crate::container;
use crate::core::Demuxer;
use crate::io::{Error, File, Result};

#[derive(Debug, Clone)]
pub enum Metadata {
	Wav(container::wav::WavFormat),
	Aac(container::aac::AACFormat),
	Mov,
}

impl Metadata {
	pub fn name(&self) -> &str {
		match self {
			Metadata::Wav(_) => "wav",
			Metadata::Aac(_) => "aac",
			Metadata::Mov => "mov",
		}
	}
}

pub struct Source {
	pub path: String,
	pub demuxer: Box<dyn Demuxer>,
	pub metadata: Metadata,
}

impl Source {
	pub fn new(demuxer: Box<dyn Demuxer>, metadata: Metadata) -> Self {
		Self { path: Default::default(), demuxer, metadata }
	}

	pub fn with_path(mut self, path: String) -> Self {
		self.path = path;
		self
	}
}

pub fn open_source_demuxer(path: &str, extension: &str) -> Result<Source> {
	let file = File::open(path)?;
	let result = match extension {
		container::WAV => open_source_wav_demuxer(file),
		container::AAC => open_source_aac_demuxer(file),
		container::MOV => open_source_mov_demuxer(file),
		_ => return Err(Error::invalid_data(format!("unsupported '{}' format", extension))),
	};
	result.map(|source| source.with_path(path.to_string()))
}

fn open_source_wav_demuxer(file: File) -> Result<Source> {
	let demuxer = container::wav::WavDemuxer::new(file)?;
	let format = demuxer.format();
	Ok(Source::new(Box::new(demuxer), Metadata::Wav(format)))
}

fn open_source_aac_demuxer(file: File) -> Result<Source> {
	let demuxer = container::aac::AACDemuxer::new(file)?;
	let format = container::aac::AACFormat::from_demuxer(&demuxer);
	Ok(Source::new(Box::new(demuxer), Metadata::Aac(format)))
}

fn open_source_mov_demuxer(file: File) -> Result<Source> {
	let demuxer = container::mov::MovDemuxer::new(file)?;
	Ok(Source::new(Box::new(demuxer), Metadata::Mov))
}
