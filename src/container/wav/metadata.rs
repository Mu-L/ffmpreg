use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WavMetadata {
	pub fields: HashMap<String, String>,
}

impl WavMetadata {
	pub fn new() -> Self {
		Self { fields: HashMap::new() }
	}

	pub fn set(&mut self, key: &str, value: String) {
		self.fields.insert(key.to_string(), value);
	}

	pub fn get(&self, key: &str) -> Option<&str> {
		self.fields.get(key).map(|s| s.as_str())
	}

	pub fn artist(&self) -> Option<&str> {
		self.get("artist")
	}

	pub fn title(&self) -> Option<&str> {
		self.get("title")
	}

	pub fn set_artist(&mut self, artist: String) {
		self.set("artist", artist);
	}

	pub fn set_title(&mut self, title: String) {
		self.set("title", title);
	}

	pub fn all_fields(&self) -> &HashMap<String, String> {
		&self.fields
	}

	pub fn is_empty(&self) -> bool {
		self.fields.is_empty()
	}
}

impl Default for WavMetadata {
	fn default() -> Self {
		Self::new()
	}
}
