use crate::core::{Frame, Packet};
use crate::io::Result;

pub trait Encoder {
	fn encode(&mut self, frame: Frame) -> Result<Option<Packet>>;
	fn flush(&mut self) -> Result<Option<Packet>>;
}
