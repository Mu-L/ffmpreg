use crate::core::Packet;
use crate::io::IoResult;

pub trait Demuxer {
	fn read_packet(&mut self) -> IoResult<Option<Packet>>;
	fn stream_count(&self) -> usize;
}
