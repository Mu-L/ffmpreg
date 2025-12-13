use crate::core::{Frame, Packet};
use crate::io::IoResult;

pub trait Encoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>>;
	fn flush(&mut self) -> IoResult<Option<Packet>>;
}
