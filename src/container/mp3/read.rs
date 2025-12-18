use super::{Mp3Format, parse_frame_header};
use crate::core::{Demuxer, Packet, Timebase};
use crate::io::{IoError, IoResult, MediaRead, ReadPrimitives};

pub struct Mp3Reader<R: MediaRead> {
	reader: R,
	format: Mp3Format,
	timebase: Timebase,
	frame_count: u64,
	first_frame: Option<Vec<u8>>,
	eof: bool,
}

impl<R: MediaRead> Mp3Reader<R> {
	pub fn new(mut reader: R) -> IoResult<Self> {
		let (format, first_frame) = Self::read_header(&mut reader)?;
		let timebase = Timebase::new(1, format.sample_rate);

		Ok(Self {
			reader,
			format,
			timebase,
			frame_count: 0,
			first_frame: Some(first_frame),
			eof: false,
		})
	}

	pub fn format(&self) -> &Mp3Format {
		&self.format
	}

	fn read_header(reader: &mut R) -> IoResult<(Mp3Format, Vec<u8>)> {
		let mut initial_buf = [0u8; 4];
		Self::skip_id3v2(reader, &mut initial_buf)?;
		Self::find_sync_and_read_first_frame(reader, initial_buf)
	}

	fn skip_id3v2(reader: &mut R, initial_buf: &mut [u8; 4]) -> IoResult<()> {
		let mut buf = [0u8; 10];
		reader.read_exact(&mut buf)?;

		if &buf[0..3] == b"ID3" {
			let size = ((buf[6] as u32 & 0x7F) << 21)
				| ((buf[7] as u32 & 0x7F) << 14)
				| ((buf[8] as u32 & 0x7F) << 7)
				| (buf[9] as u32 & 0x7F);

			let mut skip = vec![0u8; size as usize];
			reader.read_exact(&mut skip)?;
			reader.read_exact(initial_buf)?;
		} else {
			initial_buf[0] = buf[0];
			initial_buf[1] = buf[1];
			initial_buf[2] = buf[2];
			initial_buf[3] = buf[3];
		}

		Ok(())
	}

	fn find_sync_and_read_first_frame(
		reader: &mut R,
		mut buf: [u8; 4],
	) -> IoResult<(Mp3Format, Vec<u8>)> {
		let mut attempts = 0;
		const MAX_ATTEMPTS: usize = 8192;

		loop {
			if attempts >= MAX_ATTEMPTS {
				return Err(IoError::invalid_data("no valid MP3 frame found"));
			}

			if let Some(format) = parse_frame_header(buf) {
				let frame_size = format.frame_size();
				if frame_size >= 4 {
					let mut frame_data = vec![0u8; frame_size];
					frame_data[0..4].copy_from_slice(&buf);
					reader.read_exact(&mut frame_data[4..])?;
					return Ok((format, frame_data));
				}
			}

			buf[0] = buf[1];
			buf[1] = buf[2];
			buf[2] = buf[3];
			match reader.read_exact(&mut buf[3..4]) {
				Ok(()) => {}
				Err(_) => return Err(IoError::invalid_data("no valid MP3 frame found")),
			}
			attempts += 1;
		}
	}

	fn read_frame(&mut self) -> IoResult<Option<Vec<u8>>> {
		if self.eof {
			return Ok(None);
		}

		if let Some(frame) = self.first_frame.take() {
			return Ok(Some(frame));
		}

		let mut header = [0u8; 4];
		match self.reader.read_exact(&mut header) {
			Ok(()) => {}
			Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
				self.eof = true;
				return Ok(None);
			}
			Err(e) => return Err(e),
		}

		let format = match parse_frame_header(header) {
			Some(f) => f,
			None => {
				self.eof = true;
				return Ok(None);
			}
		};

		let frame_size = format.frame_size();
		if frame_size < 4 {
			self.eof = true;
			return Ok(None);
		}

		let mut data = vec![0u8; frame_size];
		data[0..4].copy_from_slice(&header);

		match self.reader.read_exact(&mut data[4..]) {
			Ok(()) => Ok(Some(data)),
			Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
				self.eof = true;
				Ok(None)
			}
			Err(e) => Err(e),
		}
	}
}

impl<R: MediaRead> Demuxer for Mp3Reader<R> {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		match self.read_frame()? {
			Some(data) => {
				let samples_per_frame = self.format.samples_per_frame() as i64;
				let pts = self.frame_count as i64 * samples_per_frame;
				self.frame_count += 1;

				Ok(Some(Packet::new(data, 0, self.timebase).with_pts(pts)))
			}
			None => Ok(None),
		}
	}

	fn stream_count(&self) -> usize {
		1
	}
}
