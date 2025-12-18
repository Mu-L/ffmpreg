use super::{ulaw_decode, ulaw_encode};
use crate::container::WavFormat;
use crate::core::{Decoder, Encoder, Frame, Packet, Timebase};
use crate::io::IoResult;

pub struct UlawDecoder {
	format: WavFormat,
}

impl UlawDecoder {
	pub fn new(format: WavFormat) -> Self {
		Self { format }
	}
}

impl Decoder for UlawDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		let mut output = Vec::with_capacity(packet.data.len() * 2);

		for &encoded in &packet.data {
			let sample = ulaw_decode(encoded);
			output.extend_from_slice(&sample.to_le_bytes());
		}

		let nb_samples = output.len() / 2 / self.format.channels as usize;
		let frame = Frame::new(
			output,
			packet.timebase,
			self.format.sample_rate,
			self.format.channels,
			nb_samples,
		)
		.with_pts(packet.pts);

		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}

pub struct UlawEncoder {
	timebase: Timebase,
}

impl UlawEncoder {
	pub fn new(timebase: Timebase, _channels: u8) -> Self {
		Self { timebase }
	}
}

impl Encoder for UlawEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		let samples: Vec<i16> =
			frame.data.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();

		let mut output = Vec::with_capacity(samples.len());

		for sample in samples {
			output.push(ulaw_encode(sample));
		}

		let packet = Packet::new(output, 0, self.timebase).with_pts(frame.pts);
		Ok(Some(packet))
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}
