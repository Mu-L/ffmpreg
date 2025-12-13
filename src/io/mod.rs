mod reader;
mod seek;
mod writer;

pub use reader::{BufferedReader, DEFAULT_BUFFER_SIZE, MediaRead, ReadPrimitives, StdReadAdapter};
pub use seek::{MediaSeek, SeekFrom, SeekableReader, SeekableWriter, StdSeekAdapter};
pub use writer::{MediaWrite, StdWriteAdapter, WritePrimitives};

#[derive(Debug)]
pub enum IoErrorKind {
	UnexpectedEof,
	WriteZero,
	Interrupted,
	InvalidData,
	NotSeekable,
	PermissionDenied,
	NotFound,
	AlreadyExists,
	WouldBlock,
	Other,
}

#[derive(Debug)]
pub struct IoError {
	kind: IoErrorKind,
	message: Option<&'static str>,
}

impl IoError {
	#[inline]
	pub const fn new(kind: IoErrorKind) -> Self {
		Self { kind, message: None }
	}

	#[inline]
	pub const fn with_message(kind: IoErrorKind, message: &'static str) -> Self {
		Self { kind, message: Some(message) }
	}

	#[inline]
	pub const fn kind(&self) -> &IoErrorKind {
		&self.kind
	}

	#[inline]
	pub const fn message(&self) -> Option<&'static str> {
		self.message
	}

	#[inline]
	pub const fn unexpected_eof() -> Self {
		Self::new(IoErrorKind::UnexpectedEof)
	}

	#[inline]
	pub const fn write_zero() -> Self {
		Self::new(IoErrorKind::WriteZero)
	}

	#[inline]
	pub const fn invalid_data(message: &'static str) -> Self {
		Self::with_message(IoErrorKind::InvalidData, message)
	}

	#[inline]
	pub const fn not_seekable() -> Self {
		Self::new(IoErrorKind::NotSeekable)
	}
}

impl core::fmt::Display for IoError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match &self.message {
			Some(msg) => write!(f, "{:?}: {}", self.kind, msg),
			None => write!(f, "{:?}", self.kind),
		}
	}
}

impl std::error::Error for IoError {}

impl From<std::io::Error> for IoError {
	fn from(err: std::io::Error) -> Self {
		let kind = match err.kind() {
			std::io::ErrorKind::UnexpectedEof => IoErrorKind::UnexpectedEof,
			std::io::ErrorKind::WriteZero => IoErrorKind::WriteZero,
			std::io::ErrorKind::Interrupted => IoErrorKind::Interrupted,
			std::io::ErrorKind::InvalidData => IoErrorKind::InvalidData,
			std::io::ErrorKind::PermissionDenied => IoErrorKind::PermissionDenied,
			std::io::ErrorKind::NotFound => IoErrorKind::NotFound,
			std::io::ErrorKind::AlreadyExists => IoErrorKind::AlreadyExists,
			std::io::ErrorKind::WouldBlock => IoErrorKind::WouldBlock,
			_ => IoErrorKind::Other,
		};
		Self::new(kind)
	}
}

pub type IoResult<T> = Result<T, IoError>;
