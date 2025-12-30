use super::header::WavHeader;
use super::{WavFormat, WavMetadata};
use crate::core::{Demuxer, Packet, Stream, StreamKind, Time, stream};
use crate::io::{Error, MediaRead, ReadPrimitives, Result};

pub struct WavDemuxer<R: MediaRead> {
	reader: R,
	format: WavFormat,
	streams: stream::Streams,
	metadata: WavMetadata,
	data_remaining: u64,
	packet_count: u64,
	sample_position: u64,
}

impl<R: MediaRead> WavDemuxer<R> {
	const CHUNK_SIZE_LIMIT: usize = 65536;

	pub fn new(mut reader: R) -> Result<Self> {
		let (header, metadata, data_size) = Self::read_wav_and_find_data(&mut reader)?;
		header.validate()?;

		let format = header.to_format();

		let codec_name = format.to_codec_string().to_string();
		let time = Time::new(1, header.sample_rate);
		let stream = Stream::new(0, 0, StreamKind::Audio, codec_name, time);
		let streams = stream::Streams::new(vec![stream]);

		Ok(Self {
			reader,
			format,
			streams,
			metadata,
			data_remaining: data_size,
			packet_count: 0,
			sample_position: 0,
		})
	}

	fn read_wav_and_find_data(reader: &mut R) -> Result<(WavHeader, WavMetadata, u64)> {
		Self::check_fourcc(reader, "RIFF")?;
		let _file_size = reader.read_u32_le()?;
		Self::check_fourcc(reader, "WAVE")?;

		let mut header = WavHeader {
			channels: 0,
			sample_rate: 0,
			byte_rate: 0,
			block_align: 0,
			bits_per_sample: 0,
			format_code: 0,
		};
		let mut metadata = WavMetadata::new();

		loop {
			let chunk_id = Self::read_fourcc(reader)?;
			let chunk_size = reader.read_u32_le()? as u64;

			match chunk_id.as_str() {
				"fmt " => Self::read_fmt_chunk(reader, chunk_size, &mut header)?,
				"LIST" => Self::read_list_chunk(reader, chunk_size, &mut metadata)?,
				"data" => return Ok((header, metadata, chunk_size)),
				_ => Self::skip_bytes(reader, chunk_size)?,
			}
		}
	}

	fn read_fmt_chunk(reader: &mut R, chunk_size: u64, header: &mut WavHeader) -> Result<()> {
		if chunk_size < 16 {
			return Err(Error::invalid_data("fmt chunk too small"));
		}

		header.format_code = reader.read_u16_le()?;
		header.channels = reader.read_u16_le()? as u8;
		header.sample_rate = reader.read_u32_le()?;
		header.byte_rate = reader.read_u32_le()?;
		header.block_align = reader.read_u16_le()?;
		header.bits_per_sample = reader.read_u16_le()?;

		let remaining = chunk_size - 16;
		if remaining > 0 {
			Self::skip_bytes(reader, remaining)?;
		}
		Ok(())
	}

	fn read_list_chunk(reader: &mut R, chunk_size: u64, metadata: &mut WavMetadata) -> Result<()> {
		if chunk_size < 4 {
			return Ok(());
		}

		let form_type = Self::read_fourcc(reader)?;
		if form_type != "INFO" {
			return Self::skip_bytes(reader, chunk_size - 4);
		}

		let mut position = 4u64;
		while position + 8 <= chunk_size {
			let id = Self::read_fourcc(reader)?;
			let size = reader.read_u32_le()? as u64;
			position += 8;

			let data = Self::read_bytes(reader, size)?;
			position += size;

			let value = String::from_utf8_lossy(&data).trim_end_matches('\0').to_string();

			match id.as_str() {
				"IART" => metadata.set("artist", value),
				"INAM" => metadata.set("title", value),
				"ICOM" => metadata.set("comment", value),
				"ICOP" => metadata.set("copyright", value),
				"ISFT" => metadata.set("software", value),
				"IGNR" => metadata.set("genre", value),
				"ITRK" => metadata.set("track", value),
				_ => {}
			}

			if size % 2 == 1 {
				reader.read_u8()?;
				position += 1;
			}
		}
		Ok(())
	}

	fn read_fourcc(reader: &mut R) -> Result<String> {
		let mut buf = [0u8; 4];
		reader.read_exact(&mut buf)?;
		Ok(String::from_utf8_lossy(&buf).to_string())
	}

	fn check_fourcc(reader: &mut R, expected: &str) -> Result<()> {
		let actual = Self::read_fourcc(reader)?;
		if actual != expected {
			return Err(Error::invalid_data(&format!("expected {}, found {}", expected, actual)));
		}
		Ok(())
	}

	fn read_bytes(reader: &mut R, size: u64) -> Result<Vec<u8>> {
		let mut buf = vec![0u8; size as usize];
		reader.read_exact(&mut buf)?;
		Ok(buf)
	}

	fn skip_bytes(reader: &mut R, size: u64) -> Result<()> {
		let mut buf = vec![0u8; size as usize];
		reader.read_exact(&mut buf)?;
		Ok(())
	}

	pub fn read_packet(&mut self) -> Result<Option<Packet>> {
		if self.data_remaining == 0 {
			return Ok(None);
		}

		let block_align = self.format.block_align() as u64;
		let max_chunk = (Self::CHUNK_SIZE_LIMIT as u64 / block_align) * block_align;
		let chunk_size = std::cmp::min(self.data_remaining, max_chunk) as usize;
		let mut data = vec![0u8; chunk_size];
		let bytes_read = self.reader.read(&mut data)?;

		if bytes_read == 0 {
			return Ok(None);
		}

		data.truncate(bytes_read);
		self.data_remaining -= bytes_read as u64;

		let time = Time::new(1, self.format.sample_rate);
		let packet = Packet::new(data, 0, time).with_pts(self.sample_position as i64);

		self.sample_position += (bytes_read / self.format.bytes_per_frame()) as u64;
		self.packet_count += 1;

		Ok(Some(packet))
	}

	pub fn read_audio_packet(&mut self) -> Result<Option<Packet>> {
		self.read_packet()
	}

	pub fn format(&self) -> WavFormat {
		self.format
	}
	pub fn metadata(&self) -> &WavMetadata {
		&self.metadata
	}
}

impl<R: MediaRead> Demuxer for WavDemuxer<R> {
	fn streams(&self) -> &stream::Streams {
		&self.streams
	}
	fn read_packet(&mut self) -> Result<Option<Packet>> {
		self.read_packet()
	}
}
