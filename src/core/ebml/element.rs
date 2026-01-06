use super::types::{EbmlElement, EbmlError, EbmlResult, EbmlType};
use super::vint::{is_unknown_size, parse_vint};
use nom::IResult;

pub fn parse_element_id(input: &[u8]) -> IResult<&[u8], u64> {
	if input.is_empty() {
		return Err(nom::Err::Incomplete(nom::Needed::Unknown));
	}

	let first_byte = input[0];
	let length = 1 + first_byte.leading_zeros() as usize;

	if length > 4 {
		return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TooLarge)));
	}

	if input.len() < length {
		return Err(nom::Err::Incomplete(nom::Needed::Size(
			std::num::NonZeroUsize::new(length).unwrap(),
		)));
	}

	let mut id = 0u64;
	for i in 0..length {
		id = (id << 8) | (input[i] as u64);
	}

	Ok((&input[length..], id))
}

pub fn parse_element_size(input: &[u8]) -> IResult<&[u8], (u64, bool)> {
	let (remaining, vint) = parse_vint(input)?;
	let unknown = is_unknown_size(&vint);
	Ok((remaining, (vint.value, unknown)))
}

pub fn parse_element_header(input: &[u8]) -> IResult<&[u8], (u64, u64, bool)> {
	let (after_id, id) = parse_element_id(input)?;
	let (after_size, (size, unknown)) = parse_element_size(after_id)?;
	Ok((after_size, (id, size, unknown)))
}

pub fn parse_element_data(input: &[u8], size: u64) -> IResult<&[u8], Vec<u8>> {
	let size = size as usize;

	if input.len() < size {
		return Err(nom::Err::Incomplete(nom::Needed::Size(
			std::num::NonZeroUsize::new(size).unwrap(),
		)));
	}

	let (remaining, data) = nom::bytes::complete::take(size)(input)?;
	Ok((remaining, data.to_vec()))
}

pub fn parse_master_element(input: &[u8], size: u64) -> IResult<&[u8], Vec<EbmlElement>> {
	let size = size as usize;

	if input.len() < size {
		return Err(nom::Err::Incomplete(nom::Needed::Size(
			std::num::NonZeroUsize::new(size).unwrap(),
		)));
	}

	let mut children = Vec::new();
	let mut pos = 0;
	let end = size;

	while pos < end {
		let chunk = &input[pos..end];
		let (remaining, child) = parse_element(chunk)?;
		let consumed = chunk.len() - remaining.len();
		children.push(child);
		pos += consumed;
	}

	Ok((&input[size..], children))
}

pub fn parse_integer(data: &[u8]) -> EbmlResult<i64> {
	if data.len() > 8 {
		return Err(EbmlError::InvalidSize {
			element: "Signed Integer".to_string(),
			size: data.len(),
			max: 8,
		});
	}

	if data.is_empty() {
		return Ok(0);
	}

	let mut value = if data[0] & 0x80 == 0x80 { -1i64 } else { 0i64 };

	for byte in data {
		value = (value << 8) | (*byte as i64);
	}

	Ok(value)
}

pub fn parse_uinteger(data: &[u8]) -> EbmlResult<u64> {
	if data.len() > 8 {
		return Err(EbmlError::InvalidSize {
			element: "Unsigned Integer".to_string(),
			size: data.len(),
			max: 8,
		});
	}

	let mut value = 0u64;

	for byte in data {
		value = (value << 8) | (*byte as u64);
	}

	Ok(value)
}

pub fn parse_float(data: &[u8]) -> EbmlResult<f64> {
	match data.len() {
		0 => Ok(0.0),
		4 => {
			let mut bytes = [0u8; 4];
			bytes.copy_from_slice(&data[..4]);
			Ok(f32::from_be_bytes(bytes) as f64)
		}
		8 => {
			let mut bytes = [0u8; 8];
			bytes.copy_from_slice(&data[..8]);
			Ok(f64::from_be_bytes(bytes))
		}
		_ => Err(EbmlError::InvalidFloatSize(data.len())),
	}
}

pub fn parse_string(data: &[u8]) -> EbmlResult<String> {
	for &byte in data {
		if byte < 0x20 || byte > 0x7E {
			return Err(EbmlError::InvalidStringChar(byte));
		}
	}

	Ok(String::from_utf8_lossy(data).to_string())
}

pub fn parse_utf8(data: &[u8]) -> EbmlResult<String> {
	String::from_utf8(data.to_vec()).map_err(|_| EbmlError::InvalidUtf8)
}

pub fn parse_element(input: &[u8]) -> IResult<&[u8], EbmlElement> {
	let (after_header, (id, size, unknown_size)) = parse_element_header(input)?;

	let (remaining, data) = if size == 0 && !unknown_size {
		(after_header, EbmlType::Binary(Vec::new()))
	} else {
		let (remaining, data_bytes) = parse_element_data(after_header, size)?;
		(remaining, EbmlType::Binary(data_bytes))
	};

	let element = EbmlElement::new(id as u32, size, unknown_size, data);
	Ok((remaining, element))
}
