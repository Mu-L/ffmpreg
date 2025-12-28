use crate::core::traits::Encoder;
use crate::core::{Frame, Packet, Time};
use crate::io::Result as IoResult;

pub struct PcmEncoder {
	sample_rate: u32,
}

impl PcmEncoder {
	pub fn new(sample_rate: u32) -> Self {
		Self { sample_rate }
	}
}

impl Encoder for PcmEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		if let Some(audio) = frame.audio() {
			let time = Time::new(1, self.sample_rate);
			let packet = Packet::new(audio.data.clone(), frame.stream_index, time).with_pts(frame.pts);
			Ok(Some(packet))
		} else {
			Ok(None)
		}
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pcm_encoder_creation() {
		let encoder = PcmEncoder::new(44100);
		assert_eq!(encoder.sample_rate, 44100);
	}
}
