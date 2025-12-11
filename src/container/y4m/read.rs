use super::{AspectRatio, Colorspace, Interlacing, Y4mFormat};
use crate::core::{Demuxer, Packet, Timebase};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Result};

pub struct Y4mReader<R: Read> {
	reader: BufReader<R>,
	format: Y4mFormat,
	timebase: Timebase,
	frame_count: u64,
}

impl<R: Read> Y4mReader<R> {
	pub fn new(reader: R) -> Result<Self> {
		let mut buf_reader = BufReader::new(reader);
		let format = Self::read_header(&mut buf_reader)?;
		let timebase = Timebase::new(format.framerate_den, format.framerate_num);

		Ok(Self { reader: buf_reader, format, timebase, frame_count: 0 })
	}

	pub fn format(&self) -> Y4mFormat {
		self.format.clone()
	}

	fn read_header(reader: &mut BufReader<R>) -> Result<Y4mFormat> {
		let mut header = String::new();
		reader.read_line(&mut header)?;

		if !header.starts_with("YUV4MPEG2") {
			return Err(Error::new(ErrorKind::InvalidData, "not a Y4M file"));
		}

		let mut format = Y4mFormat::default();

		for param in header.split_whitespace().skip(1) {
			if param.is_empty() {
				continue;
			}
			let (key, value) = param.split_at(1);
			match key {
				"W" => format.width = value.parse().unwrap_or(format.width),
				"H" => format.height = value.parse().unwrap_or(format.height),
				"F" => {
					if let Some((num, den)) = value.split_once(':') {
						format.framerate_num = num.parse().unwrap_or(30);
						format.framerate_den = den.parse().unwrap_or(1);
					}
				}
				"I" => {
					if let Some(c) = value.chars().next() {
						format.interlacing = Interlacing::from_char(c).unwrap_or(Interlacing::Progressive);
					}
				}
				"C" => {
					format.colorspace = Colorspace::from_str(value);
				}
				"A" => {
					format.aspect_ratio = AspectRatio::from_str(value);
				}
				_ => {}
			}
		}

		Ok(format)
	}

	fn read_frame_header(&mut self) -> Result<bool> {
		let mut header = String::new();
		let bytes_read = self.reader.read_line(&mut header)?;

		if bytes_read == 0 {
			return Ok(false);
		}

		if !header.starts_with("FRAME") {
			return Err(Error::new(ErrorKind::InvalidData, "expected FRAME header"));
		}

		Ok(true)
	}
}

impl<R: Read> Demuxer for Y4mReader<R> {
	fn read_packet(&mut self) -> Result<Option<Packet>> {
		if !self.read_frame_header()? {
			return Ok(None);
		}

		let frame_size = self.format.frame_size();
		let mut data = vec![0u8; frame_size];
		self.reader.read_exact(&mut data)?;

		let pts = self.frame_count as i64;
		self.frame_count += 1;

		Ok(Some(Packet::new(data, 0, self.timebase).with_pts(pts)))
	}

	fn stream_count(&self) -> usize {
		1
	}
}
