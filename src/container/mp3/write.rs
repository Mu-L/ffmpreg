use crate::core::{Muxer, Packet};
use crate::io::{IoResult, MediaWrite, WritePrimitives};

pub struct Mp3Writer<W: MediaWrite> {
	writer: W,
}

impl<W: MediaWrite> Mp3Writer<W> {
	pub fn new(writer: W) -> IoResult<Self> {
		Ok(Self { writer })
	}
}

impl<W: MediaWrite> Muxer for Mp3Writer<W> {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
		self.writer.write_all(&packet.data)?;
		Ok(())
	}

	fn finalize(&mut self) -> IoResult<()> {
		self.writer.flush()?;
		Ok(())
	}
}
