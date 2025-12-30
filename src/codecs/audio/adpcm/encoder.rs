use crate::container::wav::WavFormat;
use crate::core::traits::Encoder;
use crate::core::{Frame, Packet, Time};
use crate::io::Result as IoResult;
use crate::io::{Error, ErrorKind};

use super::state::AdpcmState;

pub struct AdpcmEncoder {
	sample_rate: u32,
	channels: u8,
	states: Vec<AdpcmState>,
	block_size: usize,
	samples_per_block: usize,
	buffer: Vec<i16>,
}

impl AdpcmEncoder {
	pub fn new(sample_rate: u32, channels: u8, block_size: usize) -> Self {
		let channels = std::cmp::max(1, channels);
		let states = vec![AdpcmState::new(); channels as usize];

		// IMA ADPCM: samples_per_block = ((block_size - 4*channels) * 2) + 1
		let samples_per_block = ((block_size.saturating_sub(4 * channels as usize)) * 2) + 1;

		Self { sample_rate, channels, states, block_size, samples_per_block, buffer: Vec::new() }
	}

	pub fn new_from_metadata(metadata: &WavFormat) -> Self {
		Self::new(metadata.sample_rate, metadata.channels, metadata.bytes_per_sample())
	}

	fn encode_block(&mut self, samples: &[i16]) -> IoResult<Vec<u8>> {
		let mut output = Vec::new();

		// Write sync header (predictor + index) for each channel
		for ch in 0..self.channels as usize {
			output.extend_from_slice(&self.states[ch].predictor.to_le_bytes());
			output.push(self.states[ch].index);
			output.push(0); // reserved
		}

		// Encode samples as nibbles
		let mut nibbles = Vec::new();
		let mut sample_idx = 0;

		while sample_idx < samples.len() {
			for channel in 0..self.channels as usize {
				if sample_idx + channel >= samples.len() {
					break;
				}

				let sample = samples[sample_idx + channel];
				let code = self.states[channel].encode_sample(sample);
				nibbles.push(code);
			}

			sample_idx += self.channels as usize;
		}

		// Pack nibbles into bytes (2 nibbles per byte)
		for i in (0..nibbles.len()).step_by(2) {
			let low = nibbles[i];
			let high = if i + 1 < nibbles.len() { nibbles[i + 1] } else { 0 };
			output.push((high << 4) | (low & 0x0F));
		}

		// Pad to block_size if needed
		while output.len() < self.block_size {
			output.push(0);
		}

		if output.len() > self.block_size {
			output.truncate(self.block_size);
		}

		Ok(output)
	}
}

impl Encoder for AdpcmEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		let audio = frame
			.audio()
			.ok_or_else(|| Error::with_message(ErrorKind::InvalidData, "expected audio frame"))?;

		// Convert frame data to i16 samples
		if audio.data.len() % 2 != 0 {
			return Err(Error::with_message(ErrorKind::InvalidData, "audio data length must be even"));
		}

		let samples: Vec<i16> =
			audio.data.chunks_exact(2).map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]])).collect();

		// Accumulate samples in buffer
		self.buffer.extend_from_slice(&samples);

		// Encode when we have enough samples for a block
		if self.buffer.len() >= self.samples_per_block {
			let block_samples: Vec<i16> = self.buffer.drain(0..self.samples_per_block).collect();
			let encoded = self.encode_block(&block_samples)?;

			let time = Time::new(1, self.sample_rate);
			let packet = Packet::new(encoded, frame.stream_index, time).with_pts(frame.pts);

			Ok(Some(packet))
		} else {
			Ok(None)
		}
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		if self.buffer.is_empty() {
			return Ok(None);
		}

		// Pad remaining samples with zeros
		while self.buffer.len() < self.samples_per_block {
			self.buffer.push(0);
		}

		let samples_to_encode: Vec<i16> = self.buffer.drain(..).collect();
		let encoded = self.encode_block(&samples_to_encode)?;

		let time = Time::new(1, self.sample_rate);
		let packet = Packet::new(encoded, 0, time);

		Ok(Some(packet))
	}
}
