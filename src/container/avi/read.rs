use super::{
	AVI_SIGNATURE, AviFormat, AviMainHeader, AviStream, AviStreamHeader, BitmapInfoHeader,
	LIST_SIGNATURE, RIFF_SIGNATURE, StreamType, WaveFormatEx,
};
use crate::core::{Demuxer, Packet, Timebase};
use crate::io::{IoError, IoResult, MediaRead, ReadPrimitives};

pub struct AviReader<R: MediaRead> {
	reader: R,
	format: AviFormat,
	timebase: Timebase,
	#[allow(dead_code)]
	movi_start: u64,
	current_pos: u64,
	eof: bool,
}

impl<R: MediaRead> AviReader<R> {
	pub fn new(mut reader: R) -> IoResult<Self> {
		let (format, movi_start) = Self::read_header(&mut reader)?;
		let fps = if format.main_header.microseconds_per_frame > 0 {
			1_000_000 / format.main_header.microseconds_per_frame
		} else {
			30
		};
		let timebase = Timebase::new(1, fps);

		Ok(Self { reader, format, timebase, movi_start, current_pos: movi_start, eof: false })
	}

	pub fn format(&self) -> &AviFormat {
		&self.format
	}

	fn read_header(reader: &mut R) -> IoResult<(AviFormat, u64)> {
		let mut fourcc = [0u8; 4];
		reader.read_exact(&mut fourcc)?;

		if &fourcc != RIFF_SIGNATURE {
			return Err(IoError::invalid_data("not a RIFF file"));
		}

		let _file_size = reader.read_u32_le()?;

		let mut form_type = [0u8; 4];
		reader.read_exact(&mut form_type)?;

		if &form_type != AVI_SIGNATURE {
			return Err(IoError::invalid_data("not an AVI file"));
		}

		let mut format = AviFormat::default();
		let mut pos: u64 = 12;
		let mut movi_start: u64 = 0;

		loop {
			let mut chunk_id = [0u8; 4];
			match reader.read_exact(&mut chunk_id) {
				Ok(()) => {}
				Err(_) => break,
			}

			let chunk_size = reader.read_u32_le()? as u64;
			pos += 8;

			if &chunk_id == LIST_SIGNATURE {
				let mut list_type = [0u8; 4];
				reader.read_exact(&mut list_type)?;
				pos += 4;

				if &list_type == b"hdrl" {
					Self::parse_hdrl(reader, &mut format, chunk_size - 4)?;
					pos += chunk_size - 4;
				} else if &list_type == b"movi" {
					movi_start = pos;
					break;
				} else {
					Self::skip_bytes(reader, chunk_size - 4)?;
					pos += chunk_size - 4;
				}
			} else {
				Self::skip_bytes(reader, chunk_size)?;
				pos += chunk_size;
			}

			if chunk_size % 2 == 1 {
				Self::skip_bytes(reader, 1)?;
				pos += 1;
			}
		}

		Ok((format, movi_start))
	}

	fn parse_hdrl(reader: &mut R, format: &mut AviFormat, size: u64) -> IoResult<()> {
		let mut remaining = size;

		while remaining >= 8 {
			let mut chunk_id = [0u8; 4];
			reader.read_exact(&mut chunk_id)?;
			let chunk_size = reader.read_u32_le()? as u64;
			remaining -= 8;

			if &chunk_id == b"avih" {
				format.main_header = Self::parse_avih(reader)?;
				remaining -= chunk_size;
			} else if &chunk_id == LIST_SIGNATURE {
				let mut list_type = [0u8; 4];
				reader.read_exact(&mut list_type)?;
				remaining -= 4;

				if &list_type == b"strl" {
					let stream = Self::parse_strl(reader, chunk_size - 4)?;
					format.streams.push(stream);
					remaining -= chunk_size - 4;
				} else {
					Self::skip_bytes(reader, chunk_size - 4)?;
					remaining -= chunk_size - 4;
				}
			} else {
				Self::skip_bytes(reader, chunk_size)?;
				remaining -= chunk_size;
			}

			if chunk_size % 2 == 1 && remaining > 0 {
				Self::skip_bytes(reader, 1)?;
				remaining -= 1;
			}
		}

		Ok(())
	}

	fn parse_avih(reader: &mut R) -> IoResult<AviMainHeader> {
		Ok(AviMainHeader {
			microseconds_per_frame: reader.read_u32_le()?,
			max_bytes_per_sec: reader.read_u32_le()?,
			padding_granularity: reader.read_u32_le()?,
			flags: reader.read_u32_le()?,
			total_frames: reader.read_u32_le()?,
			initial_frames: reader.read_u32_le()?,
			streams: reader.read_u32_le()?,
			suggested_buffer_size: reader.read_u32_le()?,
			width: reader.read_u32_le()?,
			height: reader.read_u32_le()?,
		})
	}

	fn parse_strl(reader: &mut R, size: u64) -> IoResult<AviStream> {
		let mut stream =
			AviStream { header: AviStreamHeader::default(), video_format: None, audio_format: None };

		let mut remaining = size;

		while remaining >= 8 {
			let mut chunk_id = [0u8; 4];
			reader.read_exact(&mut chunk_id)?;
			let chunk_size = reader.read_u32_le()? as u64;
			remaining -= 8;

			if &chunk_id == b"strh" {
				stream.header = Self::parse_strh(reader)?;
				remaining -= 56.min(chunk_size);
				if chunk_size > 56 {
					Self::skip_bytes(reader, chunk_size - 56)?;
					remaining -= chunk_size - 56;
				}
			} else if &chunk_id == b"strf" {
				match stream.header.stream_type {
					StreamType::Video => {
						stream.video_format = Some(Self::parse_bitmapinfo(reader)?);
						remaining -= 40.min(chunk_size);
						if chunk_size > 40 {
							Self::skip_bytes(reader, chunk_size - 40)?;
							remaining -= chunk_size - 40;
						}
					}
					StreamType::Audio => {
						stream.audio_format = Some(Self::parse_waveformat(reader)?);
						remaining -= 16.min(chunk_size);
						if chunk_size > 16 {
							Self::skip_bytes(reader, chunk_size - 16)?;
							remaining -= chunk_size - 16;
						}
					}
					_ => {
						Self::skip_bytes(reader, chunk_size)?;
						remaining -= chunk_size;
					}
				}
			} else {
				Self::skip_bytes(reader, chunk_size)?;
				remaining -= chunk_size;
			}

			if chunk_size % 2 == 1 && remaining > 0 {
				Self::skip_bytes(reader, 1)?;
				remaining -= 1;
			}
		}

		Ok(stream)
	}

	fn parse_strh(reader: &mut R) -> IoResult<AviStreamHeader> {
		let mut fcc_type = [0u8; 4];
		reader.read_exact(&mut fcc_type)?;

		let mut handler = [0u8; 4];
		reader.read_exact(&mut handler)?;

		Ok(AviStreamHeader {
			stream_type: StreamType::from_fourcc(&fcc_type),
			handler,
			flags: reader.read_u32_le()?,
			priority: reader.read_u16_le()?,
			language: reader.read_u16_le()?,
			initial_frames: reader.read_u32_le()?,
			scale: reader.read_u32_le()?,
			rate: reader.read_u32_le()?,
			start: reader.read_u32_le()?,
			length: reader.read_u32_le()?,
			suggested_buffer_size: reader.read_u32_le()?,
			quality: reader.read_u32_le()?,
			sample_size: reader.read_u32_le()?,
			rect: [
				reader.read_u16_le()?,
				reader.read_u16_le()?,
				reader.read_u16_le()?,
				reader.read_u16_le()?,
			],
		})
	}

	fn parse_bitmapinfo(reader: &mut R) -> IoResult<BitmapInfoHeader> {
		let size = reader.read_u32_le()?;
		let mut compression = [0u8; 4];

		Ok(BitmapInfoHeader {
			size,
			width: reader.read_i32_le()?,
			height: reader.read_i32_le()?,
			planes: reader.read_u16_le()?,
			bit_count: reader.read_u16_le()?,
			compression: {
				reader.read_exact(&mut compression)?;
				compression
			},
			size_image: reader.read_u32_le()?,
			x_pels_per_meter: reader.read_i32_le()?,
			y_pels_per_meter: reader.read_i32_le()?,
			clr_used: reader.read_u32_le()?,
			clr_important: reader.read_u32_le()?,
		})
	}

	fn parse_waveformat(reader: &mut R) -> IoResult<WaveFormatEx> {
		Ok(WaveFormatEx {
			format_tag: reader.read_u16_le()?,
			channels: reader.read_u16_le()?,
			samples_per_sec: reader.read_u32_le()?,
			avg_bytes_per_sec: reader.read_u32_le()?,
			block_align: reader.read_u16_le()?,
			bits_per_sample: reader.read_u16_le()?,
		})
	}

	fn skip_bytes(reader: &mut R, count: u64) -> IoResult<()> {
		let mut buf = [0u8; 1024];
		let mut remaining = count as usize;
		while remaining > 0 {
			let to_read = remaining.min(buf.len());
			reader.read_exact(&mut buf[..to_read])?;
			remaining -= to_read;
		}
		Ok(())
	}
}

impl<R: MediaRead> Demuxer for AviReader<R> {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		if self.eof {
			return Ok(None);
		}

		let mut chunk_id = [0u8; 4];
		match self.reader.read_exact(&mut chunk_id) {
			Ok(()) => {}
			Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
				self.eof = true;
				return Ok(None);
			}
			Err(e) => return Err(e),
		}

		if &chunk_id == LIST_SIGNATURE {
			let list_size = self.reader.read_u32_le()? as u64;
			let mut list_type = [0u8; 4];
			self.reader.read_exact(&mut list_type)?;

			if &list_type == b"rec " {
				return self.read_packet();
			} else {
				Self::skip_bytes(&mut self.reader, list_size - 4)?;
				return self.read_packet();
			}
		}

		let chunk_size = self.reader.read_u32_le()? as usize;

		let stream_index = if chunk_id[0].is_ascii_digit() && chunk_id[1].is_ascii_digit() {
			((chunk_id[0] - b'0') * 10 + (chunk_id[1] - b'0')) as usize
		} else {
			0
		};

		let mut data = vec![0u8; chunk_size];
		self.reader.read_exact(&mut data)?;

		if chunk_size % 2 == 1 {
			let mut pad = [0u8; 1];
			let _ = self.reader.read_exact(&mut pad);
		}

		let pts = self.current_pos as i64;
		self.current_pos += (8 + chunk_size + chunk_size % 2) as u64;

		Ok(Some(Packet::new(data, stream_index, self.timebase).with_pts(pts)))
	}

	fn stream_count(&self) -> usize {
		self.format.streams.len()
	}
}
