use std::io::{Read, Write};

pub struct StdinAdapter;

impl StdinAdapter {
	pub fn new() -> Self {
		Self
	}
}

impl crate::io::MediaRead for StdinAdapter {
	fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
		std::io::stdin().read(buf).map_err(crate::io::Error::from)
	}
}

pub struct StdoutAdapter;

impl StdoutAdapter {
	pub fn new() -> Self {
		Self
	}
}

impl crate::io::MediaWrite for StdoutAdapter {
	fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
		std::io::stdout().write(buf).map_err(crate::io::Error::from)
	}

	fn flush(&mut self) -> crate::io::Result<()> {
		std::io::stdout().flush().map_err(crate::io::Error::from)
	}
}

pub enum StdioSource {
	Stdin(StdinAdapter),
	File(std::fs::File),
}

impl crate::io::MediaRead for StdioSource {
	fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
		match self {
			StdioSource::Stdin(stdin) => stdin.read(buf),
			StdioSource::File(file) => {
				use std::io::Read;
				file.read(buf).map_err(crate::io::Error::from)
			}
		}
	}
}

pub enum StdioSink {
	Stdout(StdoutAdapter),
	File(std::fs::File),
}

impl crate::io::MediaWrite for StdioSink {
	fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
		match self {
			StdioSink::Stdout(stdout) => stdout.write(buf),
			StdioSink::File(file) => {
				use std::io::Write;
				file.write(buf).map_err(crate::io::Error::from)
			}
		}
	}

	fn flush(&mut self) -> crate::io::Result<()> {
		match self {
			StdioSink::Stdout(stdout) => stdout.flush(),
			StdioSink::File(file) => {
				use std::io::Write;
				file.flush().map_err(crate::io::Error::from)
			}
		}
	}
}
