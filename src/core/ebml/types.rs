use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum EbmlError {
	InvalidSize { element: String, size: usize, max: usize },
	InvalidFloatSize(usize),
	InvalidStringChar(u8),
	InvalidUtf8,
	InvalidElementId,
	UnknownSizeNotAllowed,
	Custom(String),
}

pub type EbmlResult<T> = Result<T, EbmlError>;

impl fmt::Display for EbmlError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			EbmlError::InvalidSize { element, size, max } => {
				write!(f, "{} size {} exceeds maximum {}", element, size, max)
			}
			EbmlError::InvalidFloatSize(size) => {
				write!(f, "Float must be 0, 4, or 8 octets, got {}", size)
			}
			EbmlError::InvalidStringChar(byte) => {
				write!(f, "String contains invalid ASCII character: 0x{:02X}", byte)
			}
			EbmlError::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
			EbmlError::InvalidElementId => {
				write!(f, "Element ID has reserved value (all VINT_DATA bits are 1)")
			}
			EbmlError::UnknownSizeNotAllowed => {
				write!(f, "Unknown size not allowed for this element")
			}
			EbmlError::Custom(msg) => write!(f, "{}", msg),
		}
	}
}

/// EBML Element Data Types per spec §4
#[derive(Debug, Clone, PartialEq)]
pub enum EbmlType {
	Master(Vec<EbmlElement>),
	Integer(i64),
	Uinteger(u64),
	Float(f64),
	String(String),
	Binary(Vec<u8>),
	Date(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EbmlElement {
	pub id: u32,
	pub size: u64,
	pub unknown_size: bool,
	pub data: EbmlType,
}

impl EbmlElement {
	pub fn new(id: u32, size: u64, unknown_size: bool, data: EbmlType) -> Self {
		EbmlElement { id, size, unknown_size, data }
	}

	pub fn is_master(&self) -> bool {
		matches!(self.data, EbmlType::Master(_))
	}

	pub fn as_master(&self) -> Option<&Vec<EbmlElement>> {
		match &self.data {
			EbmlType::Master(children) => Some(children),
			_ => None,
		}
	}

	pub fn as_master_mut(&mut self) -> Option<&mut Vec<EbmlElement>> {
		match &mut self.data {
			EbmlType::Master(children) => Some(children),
			_ => None,
		}
	}
}

impl fmt::Display for EbmlType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			EbmlType::Master(_) => write!(f, "Master"),
			EbmlType::Integer(v) => write!(f, "Integer({})", v),
			EbmlType::Uinteger(v) => write!(f, "Uinteger({})", v),
			EbmlType::Float(v) => write!(f, "Float({})", v),
			EbmlType::String(v) => write!(f, "String({})", v),
			EbmlType::Binary(v) => write!(f, "Binary({}B)", v.len()),
			EbmlType::Date(v) => write!(f, "Date({})", v),
		}
	}
}
