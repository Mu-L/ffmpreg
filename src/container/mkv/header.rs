use crate::core::ebml::types::EbmlElement;

pub struct MkvHeader {
	pub ebml_version: u64,
	pub ebml_read_version: u64,
	pub max_id_length: u64,
	pub max_size_length: u64,
	pub doc_type: String,
	pub doc_type_version: u64,
	pub doc_type_read_version: u64,
}

impl MkvHeader {
	pub fn from_element(elem: &EbmlElement) -> Option<Self> {
		if elem.id != 0x1A45DFA3 {
			return None;
		}

		Some(MkvHeader {
			ebml_version: 1,
			ebml_read_version: 1,
			max_id_length: 4,
			max_size_length: 8,
			doc_type: "matroska".to_string(),
			doc_type_version: 4,
			doc_type_read_version: 4,
		})
	}

	pub fn validate(&self) -> bool {
		self.max_id_length > 0 && self.max_size_length > 0
	}
}
