use super::{FLAC_SIGNATURE, FlacFormat, MetadataBlockType, parse_streaminfo};
use crate::core::{Demuxer, Packet, Timebase};
use crate::io::{IoError, IoResult, MediaRead, ReadPrimitives};

pub struct FlacReader<R: MediaRead> {
	reader: R,
	format: FlacFormat,
	timebase: Timebase,
	frame_count: u64,
	eof: bool,
}

impl<R: MediaRead> FlacReader<R> {
	pub fn new(mut reader: R) -> IoResult<Self> {
		let format = Self::read_header(&mut reader)?;
		let timebase = Timebase::new(1, format.sample_rate);

		Ok(Self { reader, format, timebase, frame_count: 0, eof: false })
	}

	pub fn format(&self) -> &FlacFormat {
		&self.format
	}

	fn read_header(reader: &mut R) -> IoResult<FlacFormat> {
		let mut signature = [0u8; 4];
		reader.read_exact(&mut signature)?;

		if &signature != FLAC_SIGNATURE {
			return Err(IoError::invalid_data("not a FLAC file"));
		}

		let mut format = None;

		loop {
			let header_byte = reader.read_u8()?;
			let is_last = (header_byte & 0x80) != 0;
			let block_type = MetadataBlockType::from(header_byte);

			let mut size_buf = [0u8; 3];
			reader.read_exact(&mut size_buf)?;
			let block_size = u32::from_be_bytes([0, size_buf[0], size_buf[1], size_buf[2]]) as usize;

			let mut block_data = vec![0u8; block_size];
			reader.read_exact(&mut block_data)?;

			if block_type == MetadataBlockType::StreamInfo {
				format = parse_streaminfo(&block_data);
			}

			if is_last {
				break;
			}
		}

		format.ok_or_else(|| IoError::invalid_data("no STREAMINFO block found"))
	}

	fn read_frame(&mut self) -> IoResult<Option<Vec<u8>>> {
		if self.eof {
			return Ok(None);
		}

		let mut frame_data = Vec::with_capacity(self.format.max_block_size as usize * 4);
		let mut header = [0u8; 2];

		match self.reader.read_exact(&mut header) {
			Ok(()) => {}
			Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
				self.eof = true;
				return Ok(None);
			}
			Err(e) => return Err(e),
		}

		if header[0] != 0xFF || (header[1] & 0xFC) != 0xF8 {
			self.eof = true;
			return Ok(None);
		}

		frame_data.extend_from_slice(&header);

		let max_frame_size = if self.format.max_frame_size > 0 {
			self.format.max_frame_size as usize
		} else {
			self.format.max_block_size as usize * self.format.bytes_per_frame() * 2
		};

		let mut buf = vec![0u8; max_frame_size.min(65536)];
		match self.reader.read(&mut buf) {
			Ok(0) => {
				self.eof = true;
				if frame_data.is_empty() {
					return Ok(None);
				}
			}
			Ok(n) => {
				frame_data.extend_from_slice(&buf[..n]);
			}
			Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
				self.eof = true;
				if frame_data.is_empty() {
					return Ok(None);
				}
			}
			Err(e) => return Err(e),
		}

		Ok(Some(frame_data))
	}
}

impl<R: MediaRead> Demuxer for FlacReader<R> {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		match self.read_frame()? {
			Some(data) => {
				let block_size = self.format.max_block_size as i64;
				let pts = self.frame_count as i64 * block_size;
				self.frame_count += 1;

				Ok(Some(Packet::new(data, 0, self.timebase).with_pts(pts)))
			}
			None => Ok(None),
		}
	}

	fn stream_count(&self) -> usize {
		1
	}
}
