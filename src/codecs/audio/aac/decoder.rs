use crate::core::traits::Decoder;
use crate::core::{AudioFormat, Frame, FrameAudio, Packet, Time};
use crate::io::Result as IoResult;
use crate::io::{Error, ErrorKind};

use super::parser::{ADTSHeader, ADTSParser};

pub struct AACDecoder {
	parser: ADTSParser,
	last_header: Option<ADTSHeader>,
	#[allow(dead_code)]
	sample_buffer: Vec<u8>,
	pending_frames: std::collections::VecDeque<Frame>,
}

impl AACDecoder {
	pub fn new() -> Self {
		Self {
			parser: ADTSParser::new(),
			last_header: None,
			sample_buffer: Vec::new(),
			pending_frames: std::collections::VecDeque::new(),
		}
	}

	pub fn validate_header(header: &ADTSHeader) -> IoResult<()> {
		header.get_sample_rate()?;

		if header.get_channels() == 0 {
			return Err(Error::with_message(ErrorKind::InvalidData, "AAC channels cannot be 0"));
		}

		if header.profile > 3 {
			return Err(Error::with_message(ErrorKind::InvalidData, "AAC profile must be 0-3"));
		}

		Ok(())
	}

	fn extract_raw_aac_frame(frame_data: &[u8], header: &ADTSHeader) -> Vec<u8> {
		let header_size = header.header_size();
		if frame_data.len() > header_size { frame_data[header_size..].to_vec() } else { Vec::new() }
	}

	fn create_frame(
		raw_data: Vec<u8>,
		header: &ADTSHeader,
		packet_pts: i64,
		stream_index: usize,
	) -> IoResult<Frame> {
		let sample_rate = header.get_sample_rate()?;
		let channels = header.get_channels();

		let audio =
			FrameAudio::new(raw_data, sample_rate, channels, AudioFormat::AAC).with_nb_samples(1024);

		let time = Time::new(1, sample_rate);

		Ok(Frame::new_audio(audio, time, stream_index, 0).with_pts(packet_pts))
	}
}

impl Decoder for AACDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		if !packet.is_empty() {
			self.parser.feed(&packet.data);
		}

		match self.parser.extract_frame()? {
			Some((header, frame_data)) => {
				Self::validate_header(&header)?;
				self.last_header = Some(header);

				let raw_aac = Self::extract_raw_aac_frame(&frame_data, &header);
				let frame = Self::create_frame(raw_aac, &header, packet.pts, packet.stream_index)?;

				Ok(Some(frame))
			}
			None => Ok(None),
		}
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		if self.pending_frames.is_empty() && self.parser.buffer_size() == 0 {
			return Ok(None);
		}

		if let Some(frame) = self.pending_frames.pop_front() {
			return Ok(Some(frame));
		}

		match self.parser.extract_frame()? {
			Some((header, frame_data)) => {
				Self::validate_header(&header)?;

				let raw_aac = Self::extract_raw_aac_frame(&frame_data, &header);
				let frame = Self::create_frame(raw_aac, &header, 0, 0)?;

				Ok(Some(frame))
			}
			None => Ok(None),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decoder_creation() {
		let decoder = AACDecoder::new();
		assert_eq!(decoder.parser.buffer_size(), 0);
	}

	#[test]
	fn test_validate_valid_profile() {
		let header = ADTSHeader {
			syncword: 0xFFF,
			id: false,
			layer: 0,
			protection_absent: true,
			profile: 1,
			sample_rate_index: 4,
			private_bit: false,
			channel_config: 2,
			original: false,
			home: false,
			copyright_id_start: false,
			frame_length: 256,
			adts_buffer_fullness: 0,
			number_of_rdb: 0,
			crc_check: None,
		};

		assert!(AACDecoder::validate_header(&header).is_ok());
	}
}
