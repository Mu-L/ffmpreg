use crate::core::traits::Encoder;
use crate::core::{AudioFormat, Frame, FrameAudio, FrameData, Packet, Time};
use crate::io::Result as IoResult;
use crate::io::{Error, ErrorKind};

use super::parser::ADTSHeader;
use super::utils::{FRAME_SIZE_SAMPLES, get_sample_rate_index};

pub struct AACEncoder {
	sample_rate: u32,
	channels: u8,
	profile: u8,
	bit_rate: u32,
	frame_count: u64,
	stream_index: usize,
}

impl AACEncoder {
	pub fn new(sample_rate: u32, channels: u8) -> IoResult<Self> {
		Self::with_profile(sample_rate, channels, 1, 128000)
	}

	pub fn with_profile(
		sample_rate: u32,
		channels: u8,
		profile: u8,
		bit_rate: u32,
	) -> IoResult<Self> {
		if channels == 0 || channels > 8 {
			return Err(Error::with_message(ErrorKind::InvalidData, "AAC channels must be 1-8"));
		}

		if profile > 3 {
			return Err(Error::with_message(ErrorKind::InvalidData, "AAC profile must be 0-3"));
		}

		if get_sample_rate_index(sample_rate).is_none() {
			return Err(Error::with_message(ErrorKind::InvalidData, "Unsupported sample rate for AAC"));
		}

		Ok(Self { sample_rate, channels, profile, bit_rate, frame_count: 0, stream_index: 0 })
	}

	pub fn set_stream_index(&mut self, index: usize) {
		self.stream_index = index;
	}

	pub fn set_bit_rate(&mut self, bit_rate: u32) {
		self.bit_rate = bit_rate;
	}

	fn create_adts_header(&self, frame_size: u16, _sample_count: usize) -> IoResult<ADTSHeader> {
		let sample_rate_index = get_sample_rate_index(self.sample_rate)
			.ok_or_else(|| Error::with_message(ErrorKind::InvalidData, "Invalid sample rate for AAC"))?;

		let adts_header_size = 7u16;
		let total_frame_length = adts_header_size + frame_size;

		if total_frame_length > 16384 {
			return Err(Error::with_message(ErrorKind::InvalidData, "AAC frame too large"));
		}

		Ok(ADTSHeader {
			syncword: 0xFFF,
			id: false,
			layer: 0,
			protection_absent: true,
			profile: self.profile,
			sample_rate_index,
			private_bit: false,
			channel_config: self.channels,
			original: false,
			home: false,
			copyright_id_start: false,
			frame_length: total_frame_length,
			adts_buffer_fullness: 0x7FF,
			number_of_rdb: 0,
			crc_check: None,
		})
	}

	fn encode_pcm_to_aac(&self, pcm_data: &[u8], _channels: u8, _sample_rate: u32) -> Vec<u8> {
		pcm_data.to_vec()
	}

	fn validate_frame(&self, frame: &FrameAudio) -> IoResult<()> {
		if frame.sample_rate != self.sample_rate {
			return Err(Error::with_message(ErrorKind::InvalidData, "Frame sample rate mismatch"));
		}

		if frame.channels != self.channels {
			return Err(Error::with_message(ErrorKind::InvalidData, "Frame channel count mismatch"));
		}

		match frame.format {
			AudioFormat::PCM16 | AudioFormat::PCM24 | AudioFormat::PCM32 => Ok(()),
			_ => Err(Error::with_message(ErrorKind::InvalidData, "AAC encoder requires PCM input")),
		}
	}
}

impl Encoder for AACEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		match frame.data {
			FrameData::Audio(audio) => {
				self.validate_frame(&audio)?;

				let encoded_data = self.encode_pcm_to_aac(&audio.data, audio.channels, audio.sample_rate);

				let header = self.create_adts_header(encoded_data.len() as u16, audio.nb_samples)?;

				let mut packet_data = header.serialize();
				packet_data.extend_from_slice(&encoded_data);

				let pts =
					(self.frame_count as i64 * FRAME_SIZE_SAMPLES as i64 * 1000) / self.sample_rate as i64;

				self.frame_count += 1;

				let time = Time::new(1, self.sample_rate);
				Ok(Some(Packet::new(packet_data, self.stream_index, time).with_pts(pts).with_dts(pts)))
			}
			_ => Err(Error::with_message(ErrorKind::InvalidData, "AAC encoder expects audio frame")),
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
	fn test_encoder_creation() {
		let encoder = AACEncoder::new(48000, 2);
		assert!(encoder.is_ok());
	}

	#[test]
	fn test_invalid_sample_rate() {
		let encoder = AACEncoder::new(5000, 2);
		assert!(encoder.is_err());
	}

	#[test]
	fn test_invalid_channels() {
		let encoder = AACEncoder::new(48000, 0);
		assert!(encoder.is_err());
	}

	#[test]
	fn test_invalid_profile() {
		let encoder = AACEncoder::with_profile(48000, 2, 5, 128000);
		assert!(encoder.is_err());
	}
}
