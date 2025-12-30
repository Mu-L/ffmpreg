mod cursor;
mod file;
mod reader;
mod seek;
pub mod stdio;
mod writer;

pub use cursor::Cursor;
pub use file::File;
pub use reader::{
	BufferedReader, BufferedWriter, DEFAULT_BUFFER_SIZE, MediaRead, ReadPrimitives, StdReadAdapter,
};

pub use seek::{MediaSeek, SeekFrom, SeekableReader, SeekableWriter, StdSeekAdapter};
pub use writer::{MediaWrite, StdWriteAdapter, WritePrimitives};

#[derive(Debug)]
pub enum ErrorKind {
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
pub struct Error {
	kind: ErrorKind,
	message: Option<String>,
}

impl Error {
	#[inline]
	pub const fn new(kind: ErrorKind) -> Self {
		Self { kind, message: None }
	}

	#[inline]
	pub fn with_message(kind: ErrorKind, message: impl Into<String>) -> Self {
		let message: String = message.into();
		Self { kind, message: Some(message) }
	}

	#[inline]
	pub fn kind(&self) -> &ErrorKind {
		&self.kind
	}

	#[inline]
	pub fn message(&self) -> Option<&String> {
		self.message.as_ref()
	}

	#[inline]
	pub fn unexpected_eof() -> Self {
		Self::new(ErrorKind::UnexpectedEof)
	}

	#[inline]
	pub fn write_zero() -> Self {
		Self::new(ErrorKind::WriteZero)
	}

	#[inline]
	pub fn invalid_data(message: impl Into<String>) -> Self {
		Self::with_message(ErrorKind::InvalidData, message.into())
	}

	#[inline]
	pub fn not_seekable() -> Self {
		Self::new(ErrorKind::NotSeekable)
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match &self.message {
			Some(message) => write!(f, "{}", message),
			None => write!(f, "{:?}", self.kind),
		}
	}
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		let kind = match err.kind() {
			std::io::ErrorKind::UnexpectedEof => ErrorKind::UnexpectedEof,
			std::io::ErrorKind::WriteZero => ErrorKind::WriteZero,
			std::io::ErrorKind::Interrupted => ErrorKind::Interrupted,
			std::io::ErrorKind::InvalidData => ErrorKind::InvalidData,
			std::io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
			std::io::ErrorKind::NotFound => ErrorKind::NotFound,
			std::io::ErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
			std::io::ErrorKind::WouldBlock => ErrorKind::WouldBlock,
			_ => ErrorKind::Other,
		};
		Self::new(kind)
	}
}

pub type Result<T> = std::result::Result<T, Error>;
