use crate::container::wav::WavFormat;
use crate::core::{Demuxer, Packet, Stream, StreamKind, Time, stream};
use crate::io::{Error, MediaRead, ReadPrimitives, Result};

#[derive(Debug)]
struct WavHeader {
	channels: u8,
	sample_rate: u32,
	byte_rate: u32,
	block_align: u16,
	bits_per_sample: u16,
	format_code: u16,
}

impl WavHeader {
	fn validate(&self) -> Result<()> {
		if self.channels == 0 {
			return Err(Error::invalid_data("channels must be non-zero"));
		}
		if self.sample_rate == 0 {
			return Err(Error::invalid_data("sample rate must be non-zero"));
		}
		match self.format_code {
			1 => {
				// PCM format
				if self.bits_per_sample == 0 {
					return Err(Error::invalid_data("bits per sample must be non-zero"));
				}
				if self.bits_per_sample % 8 != 0 {
					return Err(Error::invalid_data("bits per sample must be multiple of 8"));
				}
			}
			0x11 => {
				// IMA ADPCM format
				if self.bits_per_sample != 4 {
					return Err(Error::invalid_data("IMA ADPCM must have 4 bits per sample"));
				}
			}
			_ => {
				return Err(Error::invalid_data(&format!(
					"audio format code {} is not supported",
					self.format_code
				)));
			}
		}
		Ok(())
	}
}

pub struct WavDemuxer<R: MediaRead> {
	reader: R,
	format: WavFormat,
	streams: stream::Streams,
	data_remaining: u64,
	packet_count: u64,
	sample_position: u64,
}

impl<R: MediaRead> WavDemuxer<R> {
	pub fn new(mut reader: R) -> Result<Self> {
		let (header, data_size) = Self::read_wav_and_find_data(&mut reader)?;
		header.validate()?;

		let format = WavFormat {
			channels: header.channels,
			sample_rate: header.sample_rate,
			bit_depth: header.bits_per_sample,
			format_code: header.format_code,
		};

		let codec_name = format.to_codec_string().to_string();
		let time = Time::new(1, header.sample_rate);
		let stream = Stream::new(0, 0, StreamKind::Audio, codec_name, time);
		let streams = stream::Streams::new(vec![stream]);

		Ok(Self {
			reader,
			format,
			streams,
			data_remaining: data_size,
			packet_count: 0,
			sample_position: 0,
		})
	}

	fn read_wav_and_find_data(reader: &mut R) -> Result<(WavHeader, u64)> {
		let riff_header = Self::read_fourcc(reader)?;
		if riff_header != "RIFF" {
			return Err(Error::invalid_data("invalid RIFF header"));
		}

		let _file_size = reader.read_u32_le()?;

		let wave_header = Self::read_fourcc(reader)?;
		if wave_header != "WAVE" {
			return Err(Error::invalid_data("invalid WAVE header"));
		}

		let mut header = WavHeader {
			channels: 0,
			sample_rate: 0,
			byte_rate: 0,
			block_align: 0,
			bits_per_sample: 0,
			format_code: 0,
		};

		loop {
			let chunk_id = Self::read_fourcc(reader)?;
			let chunk_size = reader.read_u32_le()? as u64;

			match chunk_id.as_str() {
				"fmt " => {
					if chunk_size < 16 {
						return Err(Error::invalid_data("fmt chunk too small"));
					}
					header.format_code = reader.read_u16_le()?;
					let channels_u16 = reader.read_u16_le()?;
					header.channels = channels_u16 as u8;
					header.sample_rate = reader.read_u32_le()?;
					header.byte_rate = reader.read_u32_le()?;
					header.block_align = reader.read_u16_le()?;
					header.bits_per_sample = reader.read_u16_le()?;

					let remaining = chunk_size - 16;
					if remaining > 0 {
						let mut skip_buf = vec![0u8; remaining as usize];
						reader.read_exact(&mut skip_buf)?;
					}
				}
				"data" => {
					return Ok((header, chunk_size));
				}
				_ => {
					let mut skip_buf = vec![0u8; chunk_size as usize];
					reader.read_exact(&mut skip_buf)?;
				}
			}
		}
	}

	fn read_fourcc(reader: &mut R) -> Result<String> {
		let mut buf = [0u8; 4];
		reader.read_exact(&mut buf)?;
		Ok(String::from_utf8_lossy(&buf).to_string())
	}

	pub fn read_packet(&mut self) -> Result<Option<Packet>> {
		if self.data_remaining == 0 {
			return Ok(None);
		}

		let chunk_size = std::cmp::min(self.data_remaining, 65536) as usize;
		let mut data = vec![0u8; chunk_size];
		let bytes_read = self.reader.read(&mut data)?;

		if bytes_read == 0 {
			return Ok(None);
		}

		data.truncate(bytes_read);
		self.data_remaining -= bytes_read as u64;

		let time = Time::new(1, self.format.sample_rate);
		let packet = Packet::new(data, 0, time).with_pts(self.sample_position as i64);

		let samples_read = bytes_read / self.format.bytes_per_frame();
		self.sample_position += samples_read as u64;
		self.packet_count += 1;

		Ok(Some(packet))
	}

	pub fn read_audio_packet(&mut self) -> Result<Option<Packet>> {
		self.read_packet()
	}

	pub fn format(&self) -> WavFormat {
		self.format
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
