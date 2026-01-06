use super::elements::Elements;
use crate::core::ebml::types::{EbmlElement, EbmlType};

pub struct MkvParser;

impl MkvParser {
	pub fn parse_elements_recursive(elements: &[EbmlElement], depth: usize) -> Vec<ParsedElement> {
		let mut parsed = Vec::new();

		for elem in elements {
			let item = Self::parse_element(elem, depth);
			parsed.push(item);
		}

		parsed
	}

	fn parse_element(elem: &EbmlElement, depth: usize) -> ParsedElement {
		let info = Elements::info(elem.id);
		let name = Elements::name(elem.id);
		let is_master = Elements::is_master(elem.id);

		let value = if is_master {
			if let EbmlType::Master(children) = &elem.data {
				let parsed_children = Self::parse_elements_recursive(children, depth + 1);
				ElementValue::Master(parsed_children)
			} else {
				ElementValue::Unknown
			}
		} else {
			Self::parse_element_data(&elem.data, elem.id)
		};

		ParsedElement { id: elem.id, name: name.to_string(), value, info }
	}

	fn parse_element_data(data: &EbmlType, _id: u32) -> ElementValue {
		match data {
			EbmlType::Master(children) => {
				let parsed = Self::parse_elements_recursive(children, 0);
				ElementValue::Master(parsed)
			}
			EbmlType::Integer(v) => ElementValue::Integer(*v),
			EbmlType::Uinteger(v) => ElementValue::Uinteger(*v),
			EbmlType::Float(v) => ElementValue::Float(*v),
			EbmlType::String(v) => ElementValue::String(v.clone()),
			EbmlType::Binary(v) => ElementValue::Binary(v.len()),
			EbmlType::Date(v) => ElementValue::Date(*v),
		}
	}

	pub fn find_element(elements: &[ParsedElement], id: u32) -> Option<&ParsedElement> {
		elements.iter().find(|e| e.id == id)
	}

	pub fn find_elements(elements: &[ParsedElement], id: u32) -> Vec<&ParsedElement> {
		elements.iter().filter(|e| e.id == id).collect()
	}

	pub fn find_by_name<'a>(elements: &'a [ParsedElement], name: &str) -> Vec<&'a ParsedElement> {
		elements.iter().filter(|e| e.name == name).collect()
	}

	pub fn traverse_depth<F>(elements: &[ParsedElement], callback: &mut F)
	where
		F: FnMut(&ParsedElement, usize),
	{
		Self::traverse_recursive(elements, 0, callback);
	}

	fn traverse_recursive<F>(elements: &[ParsedElement], depth: usize, callback: &mut F)
	where
		F: FnMut(&ParsedElement, usize),
	{
		for elem in elements {
			callback(elem, depth);
			if let ElementValue::Master(children) = &elem.value {
				Self::traverse_recursive(children, depth + 1, callback);
			}
		}
	}

	pub fn count_elements(elements: &[ParsedElement]) -> usize {
		let mut count = elements.len();
		for elem in elements {
			if let ElementValue::Master(children) = &elem.value {
				count += Self::count_elements(children);
			}
		}
		count
	}

	pub fn get_element_size(elem: &ParsedElement) -> usize {
		match &elem.value {
			ElementValue::Binary(size) => *size,
			ElementValue::String(s) => s.len(),
			ElementValue::Master(children) => children.iter().map(|c| Self::get_element_size(c)).sum(),
			_ => 0,
		}
	}
}

#[derive(Debug, Clone)]
pub struct ParsedElement {
	pub id: u32,
	pub name: String,
	pub value: ElementValue,
	pub info: Option<super::elements::ElementInfo>,
}

#[derive(Debug, Clone)]
pub enum ElementValue {
	Master(Vec<ParsedElement>),
	Integer(i64),
	Uinteger(u64),
	Float(f64),
	String(String),
	Binary(usize),
	Date(i64),
	Unknown,
}

impl ParsedElement {
	pub fn is_master(&self) -> bool {
		matches!(self.value, ElementValue::Master(_))
	}

	pub fn as_master(&self) -> Option<&Vec<ParsedElement>> {
		match &self.value {
			ElementValue::Master(children) => Some(children),
			_ => None,
		}
	}

	pub fn as_uinteger(&self) -> Option<u64> {
		match self.value {
			ElementValue::Uinteger(v) => Some(v),
			_ => None,
		}
	}

	pub fn as_integer(&self) -> Option<i64> {
		match self.value {
			ElementValue::Integer(v) => Some(v),
			_ => None,
		}
	}

	pub fn as_float(&self) -> Option<f64> {
		match self.value {
			ElementValue::Float(v) => Some(v),
			_ => None,
		}
	}

	pub fn as_string(&self) -> Option<&String> {
		match &self.value {
			ElementValue::String(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_binary_size(&self) -> Option<usize> {
		match self.value {
			ElementValue::Binary(size) => Some(size),
			_ => None,
		}
	}

	pub fn as_date(&self) -> Option<i64> {
		match self.value {
			ElementValue::Date(v) => Some(v),
			_ => None,
		}
	}
}
