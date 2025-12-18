use super::{OGG_SIGNATURE, OggFormat, OggPage};
use crate::core::{Demuxer, Packet, Timebase};
use crate::io::{IoError, IoResult, MediaRead, ReadPrimitives};

pub struct OggReader<R: MediaRead> {
	reader: R,
	format: OggFormat,
	timebase: Timebase,
	packet_buffer: Vec<u8>,
	eof: bool,
}

impl<R: MediaRead> OggReader<R> {
	pub fn new(mut reader: R) -> IoResult<Self> {
		let format = Self::read_header(&mut reader)?;
		let timebase = Timebase::new(1, format.sample_rate);

		Ok(Self { reader, format, timebase, packet_buffer: Vec::new(), eof: false })
	}

	pub fn format(&self) -> &OggFormat {
		&self.format
	}

	fn read_header(reader: &mut R) -> IoResult<OggFormat> {
		let page = Self::read_page_from(reader)?;

		if !page.is_bos() {
			return Err(IoError::invalid_data("expected BOS page"));
		}

		let format =
			OggFormat { sample_rate: 44100, channels: 2, bitstream_serial: page.bitstream_serial };

		Ok(format)
	}

	fn read_page_from(reader: &mut R) -> IoResult<OggPage> {
		let mut signature = [0u8; 4];
		reader.read_exact(&mut signature)?;

		if &signature != OGG_SIGNATURE {
			return Err(IoError::invalid_data("not an OGG page"));
		}

		let version = reader.read_u8()?;
		let header_type = reader.read_u8()?;
		let granule_position = reader.read_i64_le()?;
		let bitstream_serial = reader.read_u32_le()?;
		let page_sequence = reader.read_u32_le()?;
		let checksum = reader.read_u32_le()?;
		let segment_count = reader.read_u8()?;

		let mut segment_table = vec![0u8; segment_count as usize];
		reader.read_exact(&mut segment_table)?;

		let data_size: usize = segment_table.iter().map(|&s| s as usize).sum();
		let mut data = vec![0u8; data_size];
		reader.read_exact(&mut data)?;

		Ok(OggPage {
			version,
			header_type,
			granule_position,
			bitstream_serial,
			page_sequence,
			checksum,
			segment_count,
			segment_table,
			data,
		})
	}

	fn read_page(&mut self) -> IoResult<Option<OggPage>> {
		if self.eof {
			return Ok(None);
		}

		match Self::read_page_from(&mut self.reader) {
			Ok(page) => {
				if page.is_eos() {
					self.eof = true;
				}
				Ok(Some(page))
			}
			Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
				self.eof = true;
				Ok(None)
			}
			Err(e) => Err(e),
		}
	}
}

impl<R: MediaRead> Demuxer for OggReader<R> {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		let page = match self.read_page()? {
			Some(p) => p,
			None => {
				if !self.packet_buffer.is_empty() {
					let data = std::mem::take(&mut self.packet_buffer);
					return Ok(Some(Packet::new(data, 0, self.timebase).with_pts(0)));
				}
				return Ok(None);
			}
		};

		let mut packets = Vec::new();
		let mut packet_data = Vec::new();
		let mut offset = 0;

		for &segment_size in &page.segment_table {
			packet_data.extend_from_slice(&page.data[offset..offset + segment_size as usize]);
			offset += segment_size as usize;

			if segment_size < 255 {
				if page.is_continued() && packets.is_empty() && !self.packet_buffer.is_empty() {
					self.packet_buffer.extend_from_slice(&packet_data);
					packets.push(std::mem::take(&mut self.packet_buffer));
				} else {
					packets.push(std::mem::take(&mut packet_data));
				}
			}
		}

		if !packet_data.is_empty() {
			self.packet_buffer = packet_data;
		}

		if let Some(data) = packets.into_iter().next() {
			let pts = page.granule_position;
			Ok(Some(Packet::new(data, 0, self.timebase).with_pts(pts)))
		} else {
			self.read_packet()
		}
	}

	fn stream_count(&self) -> usize {
		1
	}
}
