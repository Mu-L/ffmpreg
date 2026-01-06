use nom::IResult;
use nom::error::{Error, ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VintResult {
	pub value: u64,
	pub length: usize,
}

pub fn parse_vint(input: &[u8]) -> IResult<&[u8], VintResult> {
	if input.is_empty() {
		return Err(nom::Err::Incomplete(nom::Needed::Unknown));
	}

	let first_byte = input[0];
	let length = vint_length(first_byte);

	if length == 0 {
		return Err(nom::Err::Error(Error::new(input, ErrorKind::TooLarge)));
	}

	if input.len() < length {
		return Err(nom::Err::Incomplete(nom::Needed::Size(
			std::num::NonZeroUsize::new(length).unwrap(),
		)));
	}

	let value = vint_value(&input[..length], length);
	Ok((&input[length..], VintResult { value, length }))
}

pub fn vint_length(first_byte: u8) -> usize {
	match first_byte {
		0x80..=0xFF => 1,
		0x40..=0x7F => 2,
		0x20..=0x3F => 3,
		0x10..=0x1F => 4,
		0x08..=0x0F => 5,
		0x04..=0x07 => 6,
		0x02..=0x03 => 7,
		0x01 => 8,
		_ => 0,
	}
}

fn vint_value(bytes: &[u8], length: usize) -> u64 {
	let mask = (1u8 << (8 - length as u8)) - 1;
	let mut value = (bytes[0] & mask) as u64;

	for i in 1..length {
		value = (value << 8) | bytes[i] as u64;
	}

	value
}

pub fn is_unknown_size(vint: &VintResult) -> bool {
	let bits = 7 + (vint.length - 1) * 8;
	let mask = (1u64 << bits) - 1;
	vint.value == mask
}

pub fn validate_element_id(vint: &VintResult) -> bool {
	!is_all_ones_data(vint)
}

fn is_all_ones_data(vint: &VintResult) -> bool {
	let bits = 7 + (vint.length - 1) * 8;
	let mask = (1u64 << bits) - 1;
	vint.value == mask
}
