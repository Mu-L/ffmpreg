use crate::core::{Encoder, Frame, Packet, Timebase};
use std::io::Result;

pub struct RawVideoEncoder {
	timebase: Timebase,
}

impl RawVideoEncoder {
	pub fn new(timebase: Timebase) -> Self {
		Self { timebase }
	}
}

impl Encoder for RawVideoEncoder {
	fn encode(&mut self, frame: Frame) -> Result<Option<Packet>> {
		let packet = Packet::new(frame.data, 0, self.timebase).with_pts(frame.pts);
		Ok(Some(packet))
	}

	fn flush(&mut self) -> Result<Option<Packet>> {
		Ok(None)
	}
}
