pub struct BitReader<'a> {
	data: &'a [u8],
	byte_pos: usize,
	bit_pos: u8,
}

impl<'a> BitReader<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self { data, byte_pos: 0, bit_pos: 0 }
	}

	#[inline]
	pub fn position_bits(&self) -> usize {
		self.byte_pos * 8 + self.bit_pos as usize
	}

	#[inline]
	pub fn remaining_bits(&self) -> usize {
		if self.byte_pos >= self.data.len() {
			return 0;
		}
		(self.data.len() - self.byte_pos) * 8 - self.bit_pos as usize
	}

	#[inline]
	pub fn skip_bits(&mut self, n: usize) {
		let total = self.bit_pos as usize + n;
		self.byte_pos += total / 8;
		self.bit_pos = (total % 8) as u8;
	}

	#[inline]
	pub fn read_bit(&mut self) -> Option<bool> {
		if self.byte_pos >= self.data.len() {
			return None;
		}
		let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1;
		self.bit_pos += 1;
		if self.bit_pos == 8 {
			self.bit_pos = 0;
			self.byte_pos += 1;
		}
		Some(bit != 0)
	}

	#[inline]
	pub fn read_bits(&mut self, n: u32) -> Option<u32> {
		if n == 0 {
			return Some(0);
		}
		if n > 32 || self.remaining_bits() < n as usize {
			return None;
		}

		let mut result: u32 = 0;
		let mut bits_remaining = n;

		while bits_remaining > 0 {
			let bits_in_byte = 8 - self.bit_pos as u32;
			let bits_to_read = bits_remaining.min(bits_in_byte);

			let mask = (1u32 << bits_to_read) - 1;
			let shift = bits_in_byte - bits_to_read;
			let value = ((self.data[self.byte_pos] as u32) >> shift) & mask;

			result = (result << bits_to_read) | value;

			self.bit_pos += bits_to_read as u8;
			if self.bit_pos >= 8 {
				self.bit_pos = 0;
				self.byte_pos += 1;
			}
			bits_remaining -= bits_to_read;
		}

		Some(result)
	}

	#[inline]
	pub fn read_bits_signed(&mut self, n: u32) -> Option<i32> {
		let val = self.read_bits(n)?;
		if n == 0 {
			return Some(0);
		}
		let sign_bit = 1u32 << (n - 1);
		if val & sign_bit != 0 {
			Some(val as i32 - (1i32 << n))
		} else {
			Some(val as i32)
		}
	}

	pub fn align_to_byte(&mut self) {
		if self.bit_pos != 0 {
			self.bit_pos = 0;
			self.byte_pos += 1;
		}
	}

	pub fn set_position(&mut self, bit_position: usize) {
		self.byte_pos = bit_position / 8;
		self.bit_pos = (bit_position % 8) as u8;
	}
}
