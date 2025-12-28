use crate::core::{Decoder, Encoder, Packet};
use crate::io;

pub struct Transcoder {
	decoder: Box<dyn Decoder>,
	encoder: Box<dyn Encoder>,
}

impl Transcoder {
	pub fn new(decoder: Box<dyn Decoder>, encoder: Box<dyn Encoder>) -> Self {
		Self { decoder, encoder }
	}

	pub fn transcode(&mut self, packet: Packet) -> io::Result<Vec<Packet>> {
		let mut packets = Vec::new();
		if let Some(frame) = self.decoder.decode(packet)? {
			if let Some(encoded_packet) = self.encoder.encode(frame)? {
				packets.push(encoded_packet);
			}
		}
		Ok(packets)
	}

	pub fn flush(&mut self) -> io::Result<Vec<Packet>> {
		let mut packets = Vec::new();

		while let Some(frame) = self.decoder.flush()? {
			if let Some(encoded_packet) = self.encoder.encode(frame)? {
				packets.push(encoded_packet);
			}
		}

		while let Some(packet) = self.encoder.flush()? {
			packets.push(packet);
		}

		Ok(packets)
	}
}
