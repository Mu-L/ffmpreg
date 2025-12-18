use super::{FLAC_SIGNATURE, FlacFormat};
use crate::core::{Muxer, Packet};
use crate::io::{IoResult, MediaWrite, WritePrimitives};

pub struct FlacWriter<W: MediaWrite> {
	writer: W,
	format: FlacFormat,
	header_written: bool,
}

impl<W: MediaWrite> FlacWriter<W> {
	pub fn new(writer: W, format: FlacFormat) -> IoResult<Self> {
		Ok(Self { writer, format, header_written: false })
	}

	fn write_header(&mut self) -> IoResult<()> {
		if self.header_written {
			return Ok(());
		}

		self.writer.write_all(FLAC_SIGNATURE)?;

		let header_byte = 0x80 | 0x00;
		self.writer.write_u8(header_byte)?;

		self.writer.write_all(&[0x00, 0x00, 0x22])?;

		self.write_streaminfo()?;

		self.header_written = true;
		Ok(())
	}

	fn write_streaminfo(&mut self) -> IoResult<()> {
		self.writer.write_u16_be(self.format.min_block_size)?;
		self.writer.write_u16_be(self.format.max_block_size)?;

		let min_frame = self.format.min_frame_size;
		self.writer.write_all(&[(min_frame >> 16) as u8, (min_frame >> 8) as u8, min_frame as u8])?;

		let max_frame = self.format.max_frame_size;
		self.writer.write_all(&[(max_frame >> 16) as u8, (max_frame >> 8) as u8, max_frame as u8])?;

		let sample_rate = self.format.sample_rate;
		let channels = (self.format.channels - 1) & 0x07;
		let bps = (self.format.bits_per_sample - 1) & 0x1F;
		let total_samples = self.format.total_samples;

		let byte10 = (sample_rate >> 12) as u8;
		let byte11 = (sample_rate >> 4) as u8;
		let byte12 = ((sample_rate << 4) as u8) | (channels << 1) | ((bps >> 4) & 0x01);
		let byte13 = ((bps << 4) & 0xF0) | ((total_samples >> 32) as u8 & 0x0F);

		self.writer.write_all(&[byte10, byte11, byte12, byte13])?;
		self.writer.write_u32_be(total_samples as u32)?;
		self.writer.write_all(&self.format.md5_signature)?;

		Ok(())
	}
}

impl<W: MediaWrite> Muxer for FlacWriter<W> {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
		self.write_header()?;
		self.writer.write_all(&packet.data)?;
		Ok(())
	}

	fn finalize(&mut self) -> IoResult<()> {
		self.writer.flush()?;
		Ok(())
	}
}
