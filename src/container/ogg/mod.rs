pub mod read;
pub mod write;

pub use read::OggReader;
pub use write::OggWriter;

pub const OGG_SIGNATURE: &[u8; 4] = b"OggS";

#[derive(Debug, Clone)]
pub struct OggPage {
	pub version: u8,
	pub header_type: u8,
	pub granule_position: i64,
	pub bitstream_serial: u32,
	pub page_sequence: u32,
	pub checksum: u32,
	pub segment_count: u8,
	pub segment_table: Vec<u8>,
	pub data: Vec<u8>,
}

impl OggPage {
	pub fn is_continued(&self) -> bool {
		(self.header_type & 0x01) != 0
	}

	pub fn is_bos(&self) -> bool {
		(self.header_type & 0x02) != 0
	}

	pub fn is_eos(&self) -> bool {
		(self.header_type & 0x04) != 0
	}
}

#[derive(Debug, Clone)]
pub struct OggFormat {
	pub sample_rate: u32,
	pub channels: u8,
	pub bitstream_serial: u32,
}

impl Default for OggFormat {
	fn default() -> Self {
		Self { sample_rate: 44100, channels: 2, bitstream_serial: 0 }
	}
}

pub fn compute_crc32(data: &[u8]) -> u32 {
	const CRC_LOOKUP: [u32; 256] = generate_crc_table();
	let mut crc: u32 = 0;
	for &byte in data {
		let index = ((crc >> 24) ^ (byte as u32)) & 0xFF;
		crc = (crc << 8) ^ CRC_LOOKUP[index as usize];
	}
	crc
}

const fn generate_crc_table() -> [u32; 256] {
	let mut table = [0u32; 256];
	let polynomial: u32 = 0x04C11DB7;
	let mut i = 0;
	while i < 256 {
		let mut crc = (i as u32) << 24;
		let mut j = 0;
		while j < 8 {
			if crc & 0x80000000 != 0 {
				crc = (crc << 1) ^ polynomial;
			} else {
				crc <<= 1;
			}
			j += 1;
		}
		table[i] = crc;
		i += 1;
	}
	table
}
