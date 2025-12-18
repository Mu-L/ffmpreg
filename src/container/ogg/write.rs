use super::{OGG_SIGNATURE, compute_crc32};
use crate::core::{Muxer, Packet};
use crate::io::{IoResult, MediaWrite, WritePrimitives};

pub struct OggWriter<W: MediaWrite> {
	writer: W,
	bitstream_serial: u32,
	page_sequence: u32,
	granule_position: i64,
	bos_written: bool,
}

impl<W: MediaWrite> OggWriter<W> {
	pub fn new(writer: W, bitstream_serial: u32) -> IoResult<Self> {
		Ok(Self { writer, bitstream_serial, page_sequence: 0, granule_position: 0, bos_written: false })
	}

	fn write_page(&mut self, data: &[u8], header_type: u8, granule_position: i64) -> IoResult<()> {
		let segment_table = Self::compute_segment_table(data.len());
		let segment_count = segment_table.len() as u8;

		let mut header = Vec::with_capacity(27 + segment_count as usize);
		header.extend_from_slice(OGG_SIGNATURE);
		header.push(0);
		header.push(header_type);
		header.extend_from_slice(&granule_position.to_le_bytes());
		header.extend_from_slice(&self.bitstream_serial.to_le_bytes());
		header.extend_from_slice(&self.page_sequence.to_le_bytes());
		header.extend_from_slice(&[0u8; 4]);
		header.push(segment_count);
		header.extend_from_slice(&segment_table);

		let mut page = header.clone();
		page.extend_from_slice(data);

		let crc = compute_crc32(&page);
		page[22..26].copy_from_slice(&crc.to_le_bytes());

		self.writer.write_all(&page)?;
		self.page_sequence += 1;

		Ok(())
	}

	fn compute_segment_table(data_len: usize) -> Vec<u8> {
		let mut table = Vec::new();
		let mut remaining = data_len;

		while remaining >= 255 {
			table.push(255);
			remaining -= 255;
		}
		table.push(remaining as u8);

		table
	}
}

impl<W: MediaWrite> Muxer for OggWriter<W> {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
		let header_type = if !self.bos_written {
			self.bos_written = true;
			0x02
		} else {
			0x00
		};

		self.granule_position = packet.pts;
		self.write_page(&packet.data, header_type, self.granule_position)?;

		Ok(())
	}

	fn finalize(&mut self) -> IoResult<()> {
		self.write_page(&[], 0x04, self.granule_position)?;
		self.writer.flush()?;
		Ok(())
	}
}
