use crate::container::wav::WavFormat;
use crate::core::traits::Decoder;
use crate::core::{AudioFormat, Frame, FrameAudio, Packet, Time};
use crate::io::Result as IoResult;

pub struct PcmDecoder {
	sample_rate: u32,
	channels: u8,
	bytes_per_sample: usize,
}

impl PcmDecoder {
	pub fn new(sample_rate: u32, channels: u8, bytes_per_sample: usize) -> Self {
		Self { sample_rate, channels, bytes_per_sample }
	}

	pub fn new_from_metadata(metadata: &WavFormat) -> Self {
		Self::new(metadata.sample_rate, metadata.channels, metadata.bytes_per_sample())
	}
}

impl Decoder for PcmDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		if packet.is_empty() {
			return Ok(None);
		}

		let nb_samples = packet.data.len() / (self.channels as usize * self.bytes_per_sample);

		let audio_format = match self.bytes_per_sample {
			2 => AudioFormat::PCM16,
			3 => AudioFormat::PCM24,
			4 => AudioFormat::PCM32,
			_ => AudioFormat::PCM16,
		};

		let audio = FrameAudio::new(packet.data, self.sample_rate, self.channels, audio_format)
			.with_nb_samples(nb_samples);

		let time = Time::new(1, self.sample_rate);
		let frame = Frame::new_audio(audio, time, packet.stream_index, 0).with_pts(packet.pts);

		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pcm_decoder_creation() {
		let decoder = PcmDecoder::new(44100, 2, 2);
		assert_eq!(decoder.sample_rate, 44100);
		assert_eq!(decoder.channels, 2);
	}
}
