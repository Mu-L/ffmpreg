use crate::core::Packet;
use crate::core::stream::Streams;
use crate::io::Result;

pub trait Demuxer {
	fn streams(&self) -> &Streams;
	fn read_packet(&mut self) -> Result<Option<Packet>>;

	fn read_audio_packet(&mut self) -> Result<Option<Packet>> {
		while let Some(packet) = self.read_packet()? {
			let stream = self.streams().get(packet.stream_index);
			if stream.map(|pkt| pkt.is_audio()).unwrap_or(false) {
				return Ok(Some(packet));
			}
		}

		Ok(None)
	}

	fn read_video_packet(&mut self) -> Result<Option<Packet>> {
		while let Some(packet) = self.read_packet()? {
			let stream = self.streams().get(packet.stream_index);
			if stream.map(|pkt| pkt.is_video()).unwrap_or(false) {
				return Ok(Some(packet));
			}
		}
		Ok(None)
	}

	fn read_subtitle_packet(&mut self) -> Result<Option<Packet>> {
		while let Some(packet) = self.read_packet()? {
			let stream = self.streams().get(packet.stream_index);
			if stream.map(|pkt| pkt.is_subtitle()).unwrap_or(false) {
				return Ok(Some(packet));
			}
		}
		Ok(None)
	}
}
