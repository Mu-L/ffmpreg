use crate::core::{Frame, Packet};
use crate::io::IoResult;

pub trait Decoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>>;
	fn flush(&mut self) -> IoResult<Option<Frame>>;
}
