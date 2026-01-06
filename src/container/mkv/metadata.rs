use crate::core::ebml::types::EbmlElement;

pub struct MkvMetadata {
	pub title: Option<String>,
	pub muxing_app: Option<String>,
	pub writing_app: Option<String>,
	pub duration: Option<f64>,
	pub date_utc: Option<i64>,
}

impl MkvMetadata {
	pub fn new() -> Self {
		MkvMetadata { title: None, muxing_app: None, writing_app: None, duration: None, date_utc: None }
	}

	pub fn from_element(elem: &EbmlElement) -> Option<Self> {
		if elem.id != 0x1254C367 {
			return None;
		}

		Some(MkvMetadata::new())
	}

	pub fn is_empty(&self) -> bool {
		self.title.is_none()
			&& self.muxing_app.is_none()
			&& self.writing_app.is_none()
			&& self.duration.is_none()
			&& self.date_utc.is_none()
	}
}

impl Default for MkvMetadata {
	fn default() -> Self {
		Self::new()
	}
}
