use crate::core::Packet;
use crate::core::stream::Streams;
use crate::io::Result;

pub trait Muxer {
	fn streams(&self) -> &Streams;
	fn write(&mut self, packet: Packet) -> Result<()>;
	fn finalize(&mut self) -> Result<()>;
}
