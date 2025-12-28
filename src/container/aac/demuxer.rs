use crate::codecs::audio::aac::ADTSParser;
use crate::core::{Demuxer, Packet, Stream, StreamKind, Time, stream};
use crate::io::{Error, MediaRead, Result};

pub struct AACDemuxer<R: MediaRead> {
	reader: R,
	parser: ADTSParser,
	streams: stream::Streams,
	sample_rate: u32,
	channels: u8,
	sample_position: u64,
	first_read_done: bool,
}

impl<R: MediaRead> AACDemuxer<R> {
	pub fn new(mut reader: R) -> Result<Self> {
		// Read first chunk to peek at header for format info
		let mut peek_buffer = vec![0u8; 1024];
		let peek_size = reader.read(&mut peek_buffer)?;

		if peek_size < 7 {
			return Err(Error::invalid_data("File too small to contain AAC header"));
		}

		peek_buffer.truncate(peek_size);

		// Parse first header to get format
		let header = crate::codecs::audio::aac::ADTSHeader::parse(&peek_buffer)?;
		let sample_rate = header.get_sample_rate()?;
		let channels = header.get_channels();

		// Reinitialize reader to start from beginning
		// Since we can't seek, we need to buffer the peeked data
		let mut parser = ADTSParser::new();
		parser.feed(&peek_buffer);

		let codec_name = "aac";
		let time = Time::new(1, sample_rate);
		let stream = Stream::new(0, 0, StreamKind::Audio, codec_name.to_string(), time);
		let streams = stream::Streams::new(vec![stream]);

		Ok(Self {
			reader,
			parser,
			streams,
			sample_rate,
			channels,
			sample_position: 0,
			first_read_done: true,
		})
	}

	pub fn get_format_info(&self) -> (u32, u8) {
		(self.sample_rate, self.channels)
	}

	pub fn read_packet(&mut self) -> Result<Option<Packet>> {
		loop {
			// Try to extract frame from buffer
			match self.parser.extract_frame()? {
				Some((header, frame_data)) => {
					let time = Time::new(1, self.sample_rate);
					let packet = Packet::new(frame_data, 0, time).with_pts(self.sample_position as i64);
					self.sample_position += 1024; // AAC frames are typically 1024 samples
					return Ok(Some(packet));
				}
				None => {
					// Need more data
					let mut buffer = vec![0u8; 8192];
					let bytes_read = self.reader.read(&mut buffer)?;

					if bytes_read == 0 {
						return Ok(None);
					}

					buffer.truncate(bytes_read);
					self.parser.feed(&buffer);
				}
			}
		}
	}
}

impl<R: MediaRead> Demuxer for AACDemuxer<R> {
	fn streams(&self) -> &stream::Streams {
		&self.streams
	}

	fn read_packet(&mut self) -> Result<Option<Packet>> {
		self.read_packet()
	}
}
