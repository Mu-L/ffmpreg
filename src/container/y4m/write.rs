use super::Y4mFormat;
use crate::core::{Muxer, Packet};
use crate::io::{IoResult, MediaWrite, WritePrimitives};

pub struct Y4mWriter<W: MediaWrite> {
	writer: W,
	header_written: bool,
	format: Y4mFormat,
}

impl<W: MediaWrite> Y4mWriter<W> {
	pub fn new(writer: W, format: Y4mFormat) -> IoResult<Self> {
		Ok(Self { writer, header_written: false, format })
	}

	fn write_header(&mut self) -> IoResult<()> {
		if self.header_written {
			return Ok(());
		}

		let mut header = format!(
			"YUV4MPEG2 W{} H{} F{}:{} I{}",
			self.format.width,
			self.format.height,
			self.format.framerate_num,
			self.format.framerate_den,
			self.format.interlacing.as_char(),
		);

		if let Some(ref aspect) = self.format.aspect_ratio {
			header.push_str(&format!(" A{}", aspect.to_string()));
		}

		if let Some(ref colorspace) = self.format.colorspace {
			header.push_str(&format!(" {}", colorspace.as_str()));
		}

		header.push('\n');
		self.writer.write_all(header.as_bytes())?;
		self.header_written = true;
		Ok(())
	}
}

impl<W: MediaWrite> Muxer for Y4mWriter<W> {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
		self.write_header()?;
		self.writer.write_all(b"FRAME\n")?;
		self.writer.write_all(&packet.data)?;
		Ok(())
	}

	fn finalize(&mut self) -> IoResult<()> {
		self.writer.flush()?;
		Ok(())
	}
}
