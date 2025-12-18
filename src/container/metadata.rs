use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MediaMetadata {
	pub tags: HashMap<String, String>,
	pub chapters: Vec<Chapter>,
}

impl MediaMetadata {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.tags.insert(key.into(), value.into());
		self
	}

	pub fn with_chapter(mut self, chapter: Chapter) -> Self {
		self.chapters.push(chapter);
		self
	}

	pub fn add_tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
		self.tags.insert(key.into(), value.into());
	}

	pub fn add_chapter(&mut self, chapter: Chapter) {
		self.chapters.push(chapter);
	}

	pub fn get_tag(&self, key: &str) -> Option<&String> {
		self.tags.get(key)
	}

	pub fn title(&self) -> Option<&String> {
		self.tags.get("title")
	}

	pub fn artist(&self) -> Option<&String> {
		self.tags.get("artist")
	}

	pub fn album(&self) -> Option<&String> {
		self.tags.get("album")
	}

	pub fn year(&self) -> Option<&String> {
		self.tags.get("year")
	}

	pub fn genre(&self) -> Option<&String> {
		self.tags.get("genre")
	}

	pub fn comment(&self) -> Option<&String> {
		self.tags.get("comment")
	}

	pub fn track_number(&self) -> Option<&String> {
		self.tags.get("track")
	}
}

#[derive(Debug, Clone)]
pub struct Tag {
	pub key: String,
	pub value: String,
}

impl Tag {
	pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
		Self { key: key.into(), value: value.into() }
	}
}

#[derive(Debug, Clone)]
pub struct Chapter {
	pub id: u32,
	pub title: Option<String>,
	pub start_time: i64,
	pub end_time: i64,
	pub timebase_num: u32,
	pub timebase_den: u32,
	pub tags: HashMap<String, String>,
}

impl Chapter {
	pub fn new(id: u32, start_time: i64, end_time: i64) -> Self {
		Self {
			id,
			title: None,
			start_time,
			end_time,
			timebase_num: 1,
			timebase_den: 1000,
			tags: HashMap::new(),
		}
	}

	pub fn with_title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	pub fn with_timebase(mut self, num: u32, den: u32) -> Self {
		self.timebase_num = num;
		self.timebase_den = den;
		self
	}

	pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.tags.insert(key.into(), value.into());
		self
	}

	pub fn duration(&self) -> i64 {
		self.end_time - self.start_time
	}

	pub fn start_seconds(&self) -> f64 {
		(self.start_time as f64) * (self.timebase_num as f64) / (self.timebase_den as f64)
	}

	pub fn end_seconds(&self) -> f64 {
		(self.end_time as f64) * (self.timebase_num as f64) / (self.timebase_den as f64)
	}

	pub fn duration_seconds(&self) -> f64 {
		self.end_seconds() - self.start_seconds()
	}
}

#[derive(Debug, Clone, Default)]
pub struct Id3v2Tag {
	pub version: u8,
	pub revision: u8,
	pub flags: u8,
	pub frames: HashMap<String, Vec<u8>>,
}

impl Id3v2Tag {
	pub fn new() -> Self {
		Self { version: 4, revision: 0, flags: 0, frames: HashMap::new() }
	}

	pub fn set_text_frame(&mut self, frame_id: &str, text: &str) {
		let mut data = vec![0x03];
		data.extend_from_slice(text.as_bytes());
		self.frames.insert(frame_id.to_string(), data);
	}

	pub fn get_text_frame(&self, frame_id: &str) -> Option<String> {
		self.frames.get(frame_id).and_then(|data| {
			if data.is_empty() {
				return None;
			}
			let encoding = data[0];
			let text_data = &data[1..];
			match encoding {
				0 => String::from_utf8(text_data.to_vec()).ok(),
				1 | 2 => {
					let chars: Vec<u16> = text_data
						.chunks(2)
						.map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
						.collect();
					String::from_utf16(&chars).ok()
				}
				3 => String::from_utf8(text_data.to_vec()).ok(),
				_ => None,
			}
		})
	}

	pub fn set_title(&mut self, title: &str) {
		self.set_text_frame("TIT2", title);
	}

	pub fn set_artist(&mut self, artist: &str) {
		self.set_text_frame("TPE1", artist);
	}

	pub fn set_album(&mut self, album: &str) {
		self.set_text_frame("TALB", album);
	}

	pub fn set_year(&mut self, year: &str) {
		self.set_text_frame("TDRC", year);
	}

	pub fn set_genre(&mut self, genre: &str) {
		self.set_text_frame("TCON", genre);
	}

	pub fn set_track(&mut self, track: &str) {
		self.set_text_frame("TRCK", track);
	}

	pub fn to_metadata(&self) -> MediaMetadata {
		let mut metadata = MediaMetadata::new();

		if let Some(title) = self.get_text_frame("TIT2") {
			metadata.add_tag("title", title);
		}
		if let Some(artist) = self.get_text_frame("TPE1") {
			metadata.add_tag("artist", artist);
		}
		if let Some(album) = self.get_text_frame("TALB") {
			metadata.add_tag("album", album);
		}
		if let Some(year) = self.get_text_frame("TDRC") {
			metadata.add_tag("year", year);
		}
		if let Some(genre) = self.get_text_frame("TCON") {
			metadata.add_tag("genre", genre);
		}
		if let Some(track) = self.get_text_frame("TRCK") {
			metadata.add_tag("track", track);
		}

		metadata
	}
}

#[derive(Debug, Clone, Default)]
pub struct VorbisComment {
	pub vendor: String,
	pub comments: HashMap<String, String>,
}

impl VorbisComment {
	pub fn new() -> Self {
		Self { vendor: String::from("ffmpreg"), comments: HashMap::new() }
	}

	pub fn with_vendor(mut self, vendor: impl Into<String>) -> Self {
		self.vendor = vendor.into();
		self
	}

	pub fn add_comment(&mut self, key: impl Into<String>, value: impl Into<String>) {
		self.comments.insert(key.into().to_uppercase(), value.into());
	}

	pub fn get_comment(&self, key: &str) -> Option<&String> {
		self.comments.get(&key.to_uppercase())
	}

	pub fn to_metadata(&self) -> MediaMetadata {
		let mut metadata = MediaMetadata::new();
		for (key, value) in &self.comments {
			metadata.add_tag(key.to_lowercase(), value.clone());
		}
		metadata
	}
}
