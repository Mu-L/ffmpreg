use crate::core::Packet;
use crate::io::IoResult;

pub trait Muxer {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()>;
	fn finalize(&mut self) -> IoResult<()>;
}
