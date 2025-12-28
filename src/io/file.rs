use crate::io;

use super::{MediaRead, MediaSeek, MediaWrite, SeekFrom};
use std::io::{Read, Seek, Write};

#[derive(Debug)]
pub struct File {
	file: std::fs::File,
}

pub fn mapper_error<T>(result: std::io::Result<T>, path: &str) -> io::Result<T> {
	match result {
		Ok(file) => Ok(file),
		Err(error) => match error.kind() {
			std::io::ErrorKind::AlreadyExists => {
				let message = format!("'{}' already exists", path);
				return Err(io::Error::with_message(io::ErrorKind::AlreadyExists, message));
			}
			std::io::ErrorKind::PermissionDenied => {
				let message = format!("permission denied for '{}'", path);
				return Err(io::Error::with_message(io::ErrorKind::PermissionDenied, message));
			}
			std::io::ErrorKind::NotFound => {
				let message = format!("'{}' not found", path);
				return Err(io::Error::with_message(io::ErrorKind::NotFound, message));
			}
			std::io::ErrorKind::Other => {
				let message = format!("failed to create '{}'", path);
				return Err(io::Error::with_message(io::ErrorKind::Other, message));
			}
			std::io::ErrorKind::Interrupted => {
				let message = format!("interrupted while creating '{}'", path);
				return Err(io::Error::with_message(io::ErrorKind::Interrupted, message));
			}
			_ => return Err(io::Error::from(error)),
		},
	}
}

impl File {
	pub fn open(path: &str) -> io::Result<Self> {
		let file = mapper_error(std::fs::File::open(path), path)?;
		Ok(Self { file })
	}

	pub fn create(path: &str) -> io::Result<Self> {
		let file = mapper_error(std::fs::File::create(path), path)?;
		Ok(Self { file })
	}
}

impl MediaRead for File {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.file.read(buf).map_err(Into::into)
	}
}

impl MediaWrite for File {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.file.write(buf).map_err(Into::into)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.file.flush().map_err(Into::into)
	}
}

impl MediaSeek for File {
	fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
		let std_position = match position {
			SeekFrom::Start(offset) => std::io::SeekFrom::Start(offset),
			SeekFrom::Current(offset) => std::io::SeekFrom::Current(offset),
			SeekFrom::End(offset) => std::io::SeekFrom::End(offset),
		};
		self.file.seek(std_position).map_err(Into::into)
	}
}

impl Read for File {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.file.read(buf)
	}
}

impl Seek for File {
	fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
		self.file.seek(pos)
	}
}

impl Write for File {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.file.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.file.flush()
	}
}
