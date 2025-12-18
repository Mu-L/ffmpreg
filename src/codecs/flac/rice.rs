use crate::io::{IoError, IoResult};

pub struct BitReader<'a> {
	data: &'a [u8],
	byte_pos: usize,
	bit_pos: u8,
}

impl<'a> BitReader<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self { data, byte_pos: 0, bit_pos: 0 }
	}

	pub fn position(&self) -> usize {
		self.byte_pos * 8 + self.bit_pos as usize
	}

	pub fn remaining_bits(&self) -> usize {
		if self.byte_pos >= self.data.len() {
			return 0;
		}
		(self.data.len() - self.byte_pos) * 8 - self.bit_pos as usize
	}

	pub fn read_bit(&mut self) -> IoResult<bool> {
		if self.byte_pos >= self.data.len() {
			return Err(IoError::unexpected_eof());
		}

		let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1;
		self.bit_pos += 1;
		if self.bit_pos == 8 {
			self.bit_pos = 0;
			self.byte_pos += 1;
		}
		Ok(bit != 0)
	}

	pub fn read_bits(&mut self, n: u32) -> IoResult<u32> {
		if n == 0 {
			return Ok(0);
		}
		if n > 32 {
			return Err(IoError::invalid_data("cannot read more than 32 bits"));
		}

		let mut result: u32 = 0;
		for _ in 0..n {
			result = (result << 1) | (self.read_bit()? as u32);
		}
		Ok(result)
	}

	pub fn read_bits_signed(&mut self, n: u32) -> IoResult<i32> {
		if n == 0 {
			return Ok(0);
		}
		let val = self.read_bits(n)?;
		let sign_bit = 1u32 << (n - 1);
		if val >= sign_bit { Ok(val as i32 - (1i32 << n)) } else { Ok(val as i32) }
	}

	pub fn read_unary(&mut self) -> IoResult<u32> {
		let mut count = 0u32;
		while !self.read_bit()? {
			count += 1;
			if count > 32 {
				return Err(IoError::invalid_data("unary value too large"));
			}
		}
		Ok(count)
	}

	pub fn read_utf8_u64(&mut self) -> IoResult<u64> {
		let first = self.read_bits(8)? as u8;

		if first & 0x80 == 0 {
			return Ok(first as u64);
		}

		let num_bytes = first.leading_ones() as usize;
		if num_bytes < 2 || num_bytes > 7 {
			return Err(IoError::invalid_data("invalid UTF-8 encoding"));
		}

		let mask = 0xFF >> num_bytes;
		let mut value = (first & mask) as u64;

		for _ in 1..num_bytes {
			let byte = self.read_bits(8)? as u8;
			if byte & 0xC0 != 0x80 {
				return Err(IoError::invalid_data("invalid UTF-8 continuation"));
			}
			value = (value << 6) | ((byte & 0x3F) as u64);
		}

		Ok(value)
	}

	pub fn align_to_byte(&mut self) {
		if self.bit_pos != 0 {
			self.bit_pos = 0;
			self.byte_pos += 1;
		}
	}
}

pub fn decode_rice_partition(
	reader: &mut BitReader,
	rice_param: u32,
	num_samples: usize,
	output: &mut Vec<i32>,
) -> IoResult<()> {
	for _ in 0..num_samples {
		let quotient = reader.read_unary()?;
		let remainder = if rice_param > 0 { reader.read_bits(rice_param)? } else { 0 };

		let unsigned_val = (quotient << rice_param) | remainder;
		let signed_val = if unsigned_val & 1 == 1 {
			-((unsigned_val >> 1) as i32) - 1
		} else {
			(unsigned_val >> 1) as i32
		};
		output.push(signed_val);
	}
	Ok(())
}

pub fn decode_residual(
	reader: &mut BitReader,
	predictor_order: usize,
	block_size: usize,
	output: &mut Vec<i32>,
) -> IoResult<()> {
	let coding_method = reader.read_bits(2)?;

	let param_bits = match coding_method {
		0 => 4,
		1 => 5,
		_ => return Err(IoError::invalid_data("unsupported residual coding method")),
	};

	let escape_code = (1 << param_bits) - 1;
	let partition_order = reader.read_bits(4)? as usize;
	let num_partitions = 1 << partition_order;

	let samples_in_partition = block_size >> partition_order;

	for partition in 0..num_partitions {
		let rice_param = reader.read_bits(param_bits)?;

		let partition_samples =
			if partition == 0 { samples_in_partition - predictor_order } else { samples_in_partition };

		if rice_param == escape_code {
			let bits = reader.read_bits(5)?;
			for _ in 0..partition_samples {
				let val = reader.read_bits_signed(bits)?;
				output.push(val);
			}
		} else {
			decode_rice_partition(reader, rice_param, partition_samples, output)?;
		}
	}

	Ok(())
}

pub struct BitWriter {
	data: Vec<u8>,
	current_byte: u8,
	bit_pos: u8,
}

impl BitWriter {
	pub fn new() -> Self {
		Self { data: Vec::new(), current_byte: 0, bit_pos: 0 }
	}

	pub fn write_bit(&mut self, bit: bool) {
		if bit {
			self.current_byte |= 1 << (7 - self.bit_pos);
		}
		self.bit_pos += 1;
		if self.bit_pos == 8 {
			self.data.push(self.current_byte);
			self.current_byte = 0;
			self.bit_pos = 0;
		}
	}

	pub fn write_bits(&mut self, value: u32, n: u32) {
		for i in (0..n).rev() {
			self.write_bit((value >> i) & 1 != 0);
		}
	}

	pub fn write_bits_signed(&mut self, value: i32, n: u32) {
		let unsigned = if value < 0 { ((1i64 << n) + value as i64) as u32 } else { value as u32 };
		self.write_bits(unsigned, n);
	}

	pub fn write_unary(&mut self, value: u32) {
		for _ in 0..value {
			self.write_bit(false);
		}
		self.write_bit(true);
	}

	pub fn write_utf8_u64(&mut self, value: u64) {
		if value < 0x80 {
			self.write_bits(value as u32, 8);
		} else if value < 0x800 {
			self.write_bits(0xC0 | ((value >> 6) as u32), 8);
			self.write_bits(0x80 | ((value & 0x3F) as u32), 8);
		} else if value < 0x10000 {
			self.write_bits(0xE0 | ((value >> 12) as u32), 8);
			self.write_bits(0x80 | (((value >> 6) & 0x3F) as u32), 8);
			self.write_bits(0x80 | ((value & 0x3F) as u32), 8);
		} else if value < 0x200000 {
			self.write_bits(0xF0 | ((value >> 18) as u32), 8);
			self.write_bits(0x80 | (((value >> 12) & 0x3F) as u32), 8);
			self.write_bits(0x80 | (((value >> 6) & 0x3F) as u32), 8);
			self.write_bits(0x80 | ((value & 0x3F) as u32), 8);
		} else {
			self.write_bits(0xF8 | ((value >> 24) as u32), 8);
			self.write_bits(0x80 | (((value >> 18) & 0x3F) as u32), 8);
			self.write_bits(0x80 | (((value >> 12) & 0x3F) as u32), 8);
			self.write_bits(0x80 | (((value >> 6) & 0x3F) as u32), 8);
			self.write_bits(0x80 | ((value & 0x3F) as u32), 8);
		}
	}

	pub fn align_to_byte(&mut self) {
		if self.bit_pos != 0 {
			self.data.push(self.current_byte);
			self.current_byte = 0;
			self.bit_pos = 0;
		}
	}

	pub fn finish(mut self) -> Vec<u8> {
		self.align_to_byte();
		self.data
	}

	pub fn position(&self) -> usize {
		self.data.len() * 8 + self.bit_pos as usize
	}
}

pub fn encode_rice_signed(value: i32) -> u32 {
	if value >= 0 { (value as u32) << 1 } else { ((-value as u32 - 1) << 1) | 1 }
}

pub fn encode_residual(
	writer: &mut BitWriter,
	residuals: &[i32],
	predictor_order: usize,
	_block_size: usize,
) {
	writer.write_bits(0, 2);

	let rice_param = estimate_rice_parameter(&residuals[predictor_order..]);
	let rice_param = rice_param.min(14);

	writer.write_bits(0, 4);

	writer.write_bits(rice_param as u32, 4);

	for &residual in &residuals[predictor_order..] {
		let unsigned = encode_rice_signed(residual);
		let quotient = unsigned >> rice_param;
		let remainder = unsigned & ((1 << rice_param) - 1);
		writer.write_unary(quotient);
		if rice_param > 0 {
			writer.write_bits(remainder, rice_param as u32);
		}
	}
}

fn estimate_rice_parameter(residuals: &[i32]) -> u8 {
	if residuals.is_empty() {
		return 4;
	}

	let sum: u64 = residuals.iter().map(|&r| encode_rice_signed(r) as u64).sum();
	let avg = sum / residuals.len() as u64;

	if avg == 0 {
		return 0;
	}

	let bits = 64 - avg.leading_zeros();
	(bits as u8).saturating_sub(1).min(14)
}
