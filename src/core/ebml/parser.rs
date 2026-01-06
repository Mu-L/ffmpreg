use super::element::{
	parse_element_header, parse_float, parse_integer, parse_master_element, parse_string,
	parse_uinteger, parse_utf8,
};
use super::types::{EbmlElement, EbmlResult, EbmlType};
use nom::IResult;

pub struct EbmlParser;

impl EbmlParser {
	pub fn parse_document(input: &[u8]) -> IResult<&[u8], Vec<EbmlElement>> {
		let mut elements = Vec::new();
		let mut remaining = input;

		while !remaining.is_empty() {
			match Self::parse_top_level(remaining) {
				Ok((next, elem)) => {
					elements.push(elem);
					remaining = next;
				}
				Err(nom::Err::Incomplete(needed)) => {
					return Err(nom::Err::Incomplete(needed));
				}
				Err(e) => return Err(e),
			}
		}

		Ok((remaining, elements))
	}

	pub fn parse_top_level(input: &[u8]) -> IResult<&[u8], EbmlElement> {
		let (after_header, (id, size, unknown_size)) = parse_element_header(input)?;

		let (remaining, data) = match id {
			0x1A45DFA3 => Self::parse_ebml(after_header, size)?,
			0x18538067 => Self::parse_segment(after_header, size)?,
			_ => Self::parse_generic(after_header, size)?,
		};

		let element = EbmlElement::new(id as u32, size, unknown_size, data);
		Ok((remaining, element))
	}

	fn parse_ebml(input: &[u8], size: u64) -> IResult<&[u8], EbmlType> {
		let (remaining, children) = parse_master_element(input, size)?;
		Ok((remaining, EbmlType::Master(children)))
	}

	fn parse_segment(input: &[u8], size: u64) -> IResult<&[u8], EbmlType> {
		let (remaining, children) = parse_master_element(input, size)?;
		Ok((remaining, EbmlType::Master(children)))
	}

	fn parse_generic(input: &[u8], size: u64) -> IResult<&[u8], EbmlType> {
		let size = size as usize;

		if size == 0 {
			return Ok((&input[0..], EbmlType::Binary(Vec::new())));
		}

		if input.len() < size {
			return Err(nom::Err::Incomplete(nom::Needed::Size(
				std::num::NonZeroUsize::new(size).unwrap(),
			)));
		}

		let (remaining, data) = nom::bytes::complete::take(size)(input)?;
		Ok((remaining, EbmlType::Binary(data.to_vec())))
	}

	pub fn element_type_id(id: u64) -> ElementKind {
		match id {
			0x1A45DFA3 => ElementKind::Master,
			0x18538067 => ElementKind::Master,
			0x114D9B74 => ElementKind::Master,
			0x1549A966 => ElementKind::Master,
			0x1C53BB6B => ElementKind::Master,
			0x1F43B675 => ElementKind::Master,
			0x1E1FF8B => ElementKind::Master,
			0x4286 => ElementKind::Uinteger,
			0x42F7 => ElementKind::Uinteger,
			0x42F2 => ElementKind::Uinteger,
			0x42F3 => ElementKind::Uinteger,
			0x4282 => ElementKind::String,
			0x4287 => ElementKind::Uinteger,
			0x42F1 => ElementKind::Uinteger,
			_ => ElementKind::Binary,
		}
	}

	pub fn infer_type(id: u64, data: &[u8]) -> EbmlResult<EbmlType> {
		let kind = Self::element_type_id(id);

		match kind {
			ElementKind::Master => Ok(EbmlType::Master(Vec::new())),
			ElementKind::Integer => {
				let val = parse_integer(data)?;
				Ok(EbmlType::Integer(val))
			}
			ElementKind::Uinteger => {
				let val = parse_uinteger(data)?;
				Ok(EbmlType::Uinteger(val))
			}
			ElementKind::Float => {
				let val = parse_float(data)?;
				Ok(EbmlType::Float(val))
			}
			ElementKind::String => {
				let val = parse_string(data)?;
				Ok(EbmlType::String(val))
			}
			ElementKind::Utf8 => {
				let val = parse_utf8(data)?;
				Ok(EbmlType::String(val))
			}
			ElementKind::Date => {
				let val = parse_integer(data)?;
				Ok(EbmlType::Date(val))
			}
			ElementKind::Binary => Ok(EbmlType::Binary(data.to_vec())),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
	Master,
	Integer,
	Uinteger,
	Float,
	String,
	Utf8,
	Date,
	Binary,
}
