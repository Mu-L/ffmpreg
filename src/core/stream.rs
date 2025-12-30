use crate::core::time::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamKind {
	Audio,
	Video,
	Subtitle,
}

#[derive(Debug, Clone)]
pub struct Stream {
	pub id: u32,
	pub index: usize,
	pub kind: StreamKind,
	pub codec: String,
	pub time: Time,
}

impl Stream {
	pub fn new(id: u32, index: usize, kind: StreamKind, codec: String, time: Time) -> Self {
		Self { id, index, kind, codec, time }
	}

	#[inline(always)]
	pub fn is_audio(&self) -> bool {
		matches!(self.kind, StreamKind::Audio)
	}

	#[inline(always)]
	pub fn is_video(&self) -> bool {
		matches!(self.kind, StreamKind::Video)
	}

	#[inline(always)]
	pub fn is_subtitle(&self) -> bool {
		matches!(self.kind, StreamKind::Subtitle)
	}

	pub fn display_name(&self) -> String {
		format!("stream {} ({:?}) [{}]", self.index, self.kind, self.codec)
	}
}

pub struct Streams {
	inner: Vec<Stream>,
}

impl Streams {
	pub fn new(inner: Vec<Stream>) -> Self {
		Self { inner }
	}

	pub fn new_empty() -> Self {
		Self { inner: Vec::new() }
	}

	pub fn add(&mut self, stream: Stream) {
		self.inner.push(stream);
	}

	pub fn all(&self) -> &[Stream] {
		&self.inner
	}

	pub fn get(&self, index: usize) -> Option<&Stream> {
		self.inner.get(index)
	}

	pub fn audio(&self) -> impl Iterator<Item = &Stream> {
		self.inner.iter().filter(|s| s.is_audio())
	}

	// pub fn par_audio(&self) -> impl ParallelIterator<Item = &Stream> {
	// 	self.inner.par_iter().filter(|s| s.is_audio())
	// }

	pub fn video(&self) -> impl Iterator<Item = &Stream> {
		self.inner.iter().filter(|s| s.is_video())
	}

	pub fn subtitle(&self) -> impl Iterator<Item = &Stream> {
		self.inner.iter().filter(|s| s.is_subtitle())
	}

	pub fn count_audio(&self) -> usize {
		self.audio().count()
	}
}
