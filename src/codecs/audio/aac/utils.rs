pub struct BitReader {
	data: Vec<u8>,
	position: usize,
}

impl BitReader {
	pub fn new(data: Vec<u8>) -> Self {
		Self { data, position: 0 }
	}

	pub fn read_bit(&mut self) -> bool {
		let byte_index = self.position / 8;
		let bit_index = 7 - (self.position % 8);
		self.position += 1;

		if byte_index >= self.data.len() {
			return false;
		}

		(self.data[byte_index] >> bit_index) & 1 == 1
	}

	pub fn read_bits(&mut self, count: u32) -> u32 {
		let mut result = 0u32;
		for _ in 0..count {
			result = (result << 1) | (self.read_bit() as u32);
		}
		result
	}

	pub fn align(&mut self) {
		if self.position % 8 != 0 {
			self.position += 8 - (self.position % 8);
		}
	}

	pub fn remaining_bits(&self) -> usize {
		(self.data.len() * 8).saturating_sub(self.position)
	}
}

pub struct BitWriter {
	data: Vec<u8>,
	current_byte: u8,
	bit_position: usize,
}

impl BitWriter {
	pub fn new() -> Self {
		Self { data: Vec::new(), current_byte: 0, bit_position: 0 }
	}

	pub fn write_bit(&mut self, bit: bool) {
		if bit {
			self.current_byte |= 1 << (7 - self.bit_position);
		}
		self.bit_position += 1;

		if self.bit_position == 8 {
			self.data.push(self.current_byte);
			self.current_byte = 0;
			self.bit_position = 0;
		}
	}

	pub fn write_bits(&mut self, value: u32, count: u32) {
		for i in (0..count).rev() {
			let bit = ((value >> i) & 1) != 0;
			self.write_bit(bit);
		}
	}

	pub fn align(&mut self) {
		if self.bit_position > 0 {
			self.data.push(self.current_byte);
			self.current_byte = 0;
			self.bit_position = 0;
		}
	}

	pub fn finish(mut self) -> Vec<u8> {
		self.align();
		self.data
	}
}

pub const SAMPLE_RATES: &[u32] =
	&[96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000];

pub fn get_sample_rate_index(sample_rate: u32) -> Option<u8> {
	SAMPLE_RATES.iter().position(|&sr| sr == sample_rate).map(|i| i as u8)
}

pub fn get_sample_rate_from_index(index: u8) -> Option<u32> {
	SAMPLE_RATES.get(index as usize).copied()
}

pub const CHANNEL_CONFIGURATIONS: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 8];

pub fn is_valid_channel_config(config: u8) -> bool {
	// Valid channel configs: 0 (specific), 1-6 (mono to 5.1), 7 (7.1) sometimes reserved
	// Accept 0-7 for now, though some files may use non-standard values
	config < 8
}

pub fn get_channels_from_config(config: u8) -> Option<u8> {
	match config {
		0 => Some(2), // Default to stereo if not specified
		1..=7 => Some(config),
		_ => None,
	}
}

pub const FRAME_SIZE_SAMPLES: usize = 1024;

pub fn calculate_frame_size_bytes(sample_rate: u32, bit_rate: u32) -> usize {
	((bit_rate as usize * FRAME_SIZE_SAMPLES) / (8 * sample_rate as usize)) + 1
}
