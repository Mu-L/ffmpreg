use crate::core::traits::Decoder;
use crate::core::{AudioFormat, Frame, FrameAudio, Packet, Time};
use crate::io::Result as IoResult;
use crate::io::{Error, ErrorKind};

use super::state::AdpcmState;

pub struct AdpcmDecoder {
	sample_rate: u32,
	channels: u8,
	states: Vec<AdpcmState>,
	#[allow(dead_code)]
	block_size: usize,
	samples_per_block: usize,
}

impl AdpcmDecoder {
	pub fn new(sample_rate: u32, channels: u8, block_size: usize) -> Self {
		let channels = std::cmp::max(1, channels);
		let states = vec![AdpcmState::new(); channels as usize];

		// IMA ADPCM: 4 bits per sample
		// block_align = 1 (sync nibbles) + (block_size - 1) encoded nibbles
		let samples_per_block = ((block_size - 4) * 2) + 1;

		Self { sample_rate, channels, states, block_size, samples_per_block }
	}

	fn decode_block(&mut self, block_data: &[u8]) -> IoResult<Vec<u8>> {
		if block_data.len() < 4 * self.channels as usize {
			return Err(Error::with_message(
				ErrorKind::InvalidData,
				"ADPCM block too small for sync header",
			));
		}

		let mut pcm_output = Vec::new();
		let mut offset = 0;

		for ch in 0..self.channels as usize {
			if offset + 4 > block_data.len() {
				break;
			}

			let predictor = i16::from_le_bytes([block_data[offset], block_data[offset + 1]]);
			let index = block_data[offset + 2];

			self.states[ch] = AdpcmState::with_initial_values(predictor, index);
			offset += 4;
		}

		let remaining = &block_data[offset..];
		let mut nibble_idx = 0;

		while nibble_idx < remaining.len() * 2
			&& pcm_output.len() < self.samples_per_block * 2 * self.channels as usize
		{
			for channel in 0..self.channels as usize {
				let byte_idx = nibble_idx / 2;

				if byte_idx >= remaining.len() {
					break;
				}

				let nibble = if nibble_idx % 2 == 0 {
					remaining[byte_idx] & 0x0F
				} else {
					(remaining[byte_idx] >> 4) & 0x0F
				};

				let sample = self.states[channel].decode_nibble(nibble);
				pcm_output.extend_from_slice(&sample.to_le_bytes());

				nibble_idx += 1;
			}
		}

		Ok(pcm_output)
	}
}

impl Decoder for AdpcmDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		if packet.is_empty() {
			return Ok(None);
		}

		let pcm_data = self.decode_block(&packet.data)?;

		if pcm_data.is_empty() {
			return Ok(None);
		}

		let nb_samples = pcm_data.len() / (self.channels as usize * 2);

		let audio = FrameAudio::new(pcm_data, self.sample_rate, self.channels, AudioFormat::PCM16);

		let time = Time::new(1, self.sample_rate);

		let frame = Frame::new_audio(audio.with_nb_samples(nb_samples), time, packet.stream_index, 0);

		Ok(Some(frame.with_pts(packet.pts)))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}
