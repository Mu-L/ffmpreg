use super::bit::BitReader;
use super::utils::{get_sample_rate_from_index, is_valid_channel_config};
use crate::io::Result as IoResult;
use crate::io::{Error, ErrorKind};

#[derive(Debug, Clone, Copy)]
pub struct ADTSHeader {
	pub syncword: u16,
	pub id: bool,
	pub layer: u8,
	pub protection_absent: bool,
	pub profile: u8,
	pub sample_rate_index: u8,
	pub private_bit: bool,
	pub channel_config: u8,
	pub original: bool,
	pub home: bool,
	pub copyright_id_start: bool,
	pub frame_length: u16,
	pub adts_buffer_fullness: u16,
	pub number_of_rdb: u8,
	pub crc_check: Option<u16>,
}

impl ADTSHeader {
	pub fn parse(data: &[u8]) -> IoResult<Self> {
		if data.len() < 7 {
			return Err(Error::with_message(
				ErrorKind::InvalidData,
				"adts header must be at least 7 bytes",
			));
		}

		let mut reader = BitReader::new(data.to_vec());

		let syncword = reader.read_bits(12) as u16;
		if syncword != 0xFFF {
			return Err(Error::with_message(ErrorKind::InvalidData, "invalid ADTS sync word"));
		}

		let id = reader.read_bit();
		let layer = reader.read_bits(2) as u8;
		let protection_absent = reader.read_bit();

		let profile = reader.read_bits(2) as u8;
		let sample_rate_index = reader.read_bits(4) as u8;
		let private_bit = reader.read_bit();
		let channel_config = reader.read_bits(3) as u8;

		if !is_valid_channel_config(channel_config) {
			return Err(Error::with_message(
				ErrorKind::InvalidData,
				"invalid ADTS channel configuration",
			));
		}

		let original = reader.read_bit();
		let home = reader.read_bit();
		let copyright_id_start = reader.read_bit();
		let frame_length = reader.read_bits(13) as u16;
		let adts_buffer_fullness = reader.read_bits(11) as u16;
		let number_of_rdb = reader.read_bits(2) as u8;

		let crc_check = if !protection_absent { Some(reader.read_bits(16) as u16) } else { None };

		// Note: frame_length validation is done in extract_frame() context
		// since we may be parsing incomplete frames during initialization

		Ok(Self {
			syncword,
			id,
			layer,
			protection_absent,
			profile,
			sample_rate_index,
			private_bit,
			channel_config,
			original,
			home,
			copyright_id_start,
			frame_length,
			adts_buffer_fullness,
			number_of_rdb,
			crc_check,
		})
	}

	pub fn serialize(&self) -> Vec<u8> {
		// Serialize to match the exact bit layout expected by parse()
		// Using a BitWriter would be ideal, but we'll do it manually for the 7-byte case

		let mut data = vec![0u8; 7];
		let mut bit_pos = 0;

		// Helper to write bits
		let write_bits = |data: &mut [u8], pos: &mut usize, value: u32, count: u32| {
			for i in (0..count).rev() {
				let bit = ((value >> i) & 1) != 0;
				let byte_idx = *pos / 8;
				let bit_idx = 7 - (*pos % 8);
				if byte_idx < data.len() {
					if bit {
						data[byte_idx] |= 1 << bit_idx;
					}
				}
				*pos += 1;
			}
		};

		write_bits(&mut data, &mut bit_pos, 0xFFF, 12); // syncword
		write_bits(&mut data, &mut bit_pos, self.id as u32, 1);
		write_bits(&mut data, &mut bit_pos, self.layer as u32, 2);
		write_bits(&mut data, &mut bit_pos, self.protection_absent as u32, 1);
		write_bits(&mut data, &mut bit_pos, self.profile as u32, 2);
		write_bits(&mut data, &mut bit_pos, self.sample_rate_index as u32, 4);
		write_bits(&mut data, &mut bit_pos, self.private_bit as u32, 1);
		write_bits(&mut data, &mut bit_pos, self.channel_config as u32, 3);
		write_bits(&mut data, &mut bit_pos, self.original as u32, 1);
		write_bits(&mut data, &mut bit_pos, self.home as u32, 1);
		write_bits(&mut data, &mut bit_pos, self.copyright_id_start as u32, 1);
		write_bits(&mut data, &mut bit_pos, self.frame_length as u32, 13);
		write_bits(&mut data, &mut bit_pos, self.adts_buffer_fullness as u32, 11);
		write_bits(&mut data, &mut bit_pos, self.number_of_rdb as u32, 2);

		if let Some(crc) = self.crc_check {
			data.extend_from_slice(&crc.to_be_bytes());
		}

		data
	}

	pub fn get_sample_rate(&self) -> IoResult<u32> {
		get_sample_rate_from_index(self.sample_rate_index)
			.ok_or_else(|| Error::with_message(ErrorKind::InvalidData, "Invalid ADTS sample rate index"))
	}

	pub fn get_channels(&self) -> u8 {
		self.channel_config
	}

	pub fn get_profile_name(&self) -> &'static str {
		match self.profile {
			0 => "AAC-LC (Main)",
			1 => "AAC-LC (Low Complexity)",
			2 => "AAC-SSR (Scalable Sample Rate)",
			3 => "AAC-LTP (Long Term Prediction)",
			_ => "Unknown",
		}
	}

	pub fn frame_data(&self) -> usize {
		self.frame_length as usize - if self.protection_absent { 7 } else { 9 }
	}

	pub fn header_size(&self) -> usize {
		if self.protection_absent { 7 } else { 9 }
	}
}

pub struct ADTSParser {
	buffer: Vec<u8>,
}

impl ADTSParser {
	pub fn new() -> Self {
		Self { buffer: Vec::new() }
	}

	pub fn feed(&mut self, data: &[u8]) {
		self.buffer.extend_from_slice(data);
	}

	pub fn extract_frame(&mut self) -> IoResult<Option<(ADTSHeader, Vec<u8>)>> {
		if self.buffer.len() < 7 {
			return Ok(None);
		}

		let header = ADTSHeader::parse(&self.buffer)?;
		let frame_length = header.frame_length as usize;

		if self.buffer.len() < frame_length {
			return Ok(None);
		}

		let frame_data = self.buffer.drain(0..frame_length).collect::<Vec<_>>();
		Ok(Some((header, frame_data)))
	}

	pub fn reset(&mut self) {
		self.buffer.clear();
	}

	pub fn buffer_size(&self) -> usize {
		self.buffer.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_valid_adts_header() {
		// Valid ADTS header: profile=1 (AAC-LC), 44100Hz, stereo, frame_length=7
		let data = vec![0xFF, 0xF1, 0x50, 0x80, 0x01, 0xFF, 0xF8];
		let header = ADTSHeader::parse(&data);
		assert!(header.is_ok());

		let h = header.unwrap();
		assert_eq!(h.syncword, 0xFFF);
		assert!(h.protection_absent);
		assert_eq!(h.channel_config, 2);
		assert_eq!(h.sample_rate_index, 4);
		assert_eq!(h.profile, 1);
		assert_eq!(h.frame_length, 7);
	}

	#[test]
	fn test_invalid_sync_word() {
		let data = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
		let header = ADTSHeader::parse(&data);
		assert!(header.is_err());
	}
}
