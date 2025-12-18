use super::{BoxHeader, BoxType, Mp4Format, Mp4Track, TrackType};
use crate::core::{Demuxer, Packet, Timebase};
use crate::io::{IoResult, MediaRead, ReadPrimitives};

pub struct Mp4Reader<R: MediaRead> {
	reader: R,
	format: Mp4Format,
	timebase: Timebase,
	current_track: usize,
	current_sample: usize,
	#[allow(dead_code)]
	mdat_offset: u64,
	#[allow(dead_code)]
	mdat_size: u64,
	bytes_read: u64,
	eof: bool,
}

impl<R: MediaRead> Mp4Reader<R> {
	pub fn new(mut reader: R) -> IoResult<Self> {
		let (format, mdat_offset, mdat_size) = Self::parse_file(&mut reader)?;
		let timescale = if format.timescale > 0 { format.timescale } else { 1000 };
		let timebase = Timebase::new(1, timescale);

		Ok(Self {
			reader,
			format,
			timebase,
			current_track: 0,
			current_sample: 0,
			mdat_offset,
			mdat_size,
			bytes_read: 0,
			eof: false,
		})
	}

	pub fn format(&self) -> &Mp4Format {
		&self.format
	}

	fn parse_file(reader: &mut R) -> IoResult<(Mp4Format, u64, u64)> {
		let mut format = Mp4Format::default();
		let mut mdat_offset: u64 = 0;
		let mut mdat_size: u64 = 0;
		let mut pos: u64 = 0;

		loop {
			let header = match Self::read_box_header(reader) {
				Ok(h) => h,
				Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => break,
				Err(e) => return Err(e),
			};

			let content_size = header.size.saturating_sub(header.header_size as u64);
			pos += header.header_size as u64;

			match header.box_type {
				BoxType::Ftyp => {
					Self::parse_ftyp(reader, content_size, &mut format)?;
				}
				BoxType::Moov => {
					Self::parse_moov(reader, content_size, &mut format)?;
				}
				BoxType::Mdat => {
					mdat_offset = pos;
					mdat_size = content_size;
					Self::skip_bytes(reader, content_size)?;
				}
				_ => {
					Self::skip_bytes(reader, content_size)?;
				}
			}

			pos += content_size;
		}

		Ok((format, mdat_offset, mdat_size))
	}

	fn read_box_header(reader: &mut R) -> IoResult<BoxHeader> {
		let size = reader.read_u32_be()? as u64;
		let mut fourcc = [0u8; 4];
		reader.read_exact(&mut fourcc)?;

		let box_type = BoxType::from_fourcc(&fourcc);

		let (actual_size, header_size) = if size == 1 {
			let large_size = reader.read_u64_be()?;
			(large_size, 16u8)
		} else if size == 0 {
			(u64::MAX, 8u8)
		} else {
			(size, 8u8)
		};

		Ok(BoxHeader { size: actual_size, box_type, header_size })
	}

	fn parse_ftyp(reader: &mut R, size: u64, format: &mut Mp4Format) -> IoResult<()> {
		reader.read_exact(&mut format.major_brand)?;
		format.minor_version = reader.read_u32_be()?;

		let brands_count = (size - 8) / 4;
		format.compatible_brands.clear();
		for _ in 0..brands_count {
			let mut brand = [0u8; 4];
			reader.read_exact(&mut brand)?;
			format.compatible_brands.push(brand);
		}

		Ok(())
	}

	fn parse_moov(reader: &mut R, size: u64, format: &mut Mp4Format) -> IoResult<()> {
		let mut remaining = size;

		while remaining >= 8 {
			let header = Self::read_box_header(reader)?;
			remaining -= header.header_size as u64;
			let content_size = header.size.saturating_sub(header.header_size as u64);

			match header.box_type {
				BoxType::Mvhd => {
					Self::parse_mvhd(reader, content_size, format)?;
				}
				BoxType::Trak => {
					let track = Self::parse_trak(reader, content_size)?;
					format.tracks.push(track);
				}
				_ => {
					Self::skip_bytes(reader, content_size)?;
				}
			}

			remaining = remaining.saturating_sub(content_size);
		}

		Ok(())
	}

	fn parse_mvhd(reader: &mut R, size: u64, format: &mut Mp4Format) -> IoResult<()> {
		let version = reader.read_u8()?;
		let mut _flags = [0u8; 3];
		reader.read_exact(&mut _flags)?;

		if version == 1 {
			let _creation_time = reader.read_u64_be()?;
			let _modification_time = reader.read_u64_be()?;
			format.timescale = reader.read_u32_be()?;
			format.duration = reader.read_u64_be()?;
			Self::skip_bytes(reader, size - 28)?;
		} else {
			let _creation_time = reader.read_u32_be()?;
			let _modification_time = reader.read_u32_be()?;
			format.timescale = reader.read_u32_be()?;
			format.duration = reader.read_u32_be()? as u64;
			Self::skip_bytes(reader, size - 16)?;
		}

		Ok(())
	}

	fn parse_trak(reader: &mut R, size: u64) -> IoResult<Mp4Track> {
		let mut track = Mp4Track::default();
		let mut remaining = size;

		while remaining >= 8 {
			let header = Self::read_box_header(reader)?;
			remaining -= header.header_size as u64;
			let content_size = header.size.saturating_sub(header.header_size as u64);

			match header.box_type {
				BoxType::Tkhd => {
					Self::parse_tkhd(reader, content_size, &mut track)?;
				}
				BoxType::Mdia => {
					Self::parse_mdia(reader, content_size, &mut track)?;
				}
				_ => {
					Self::skip_bytes(reader, content_size)?;
				}
			}

			remaining = remaining.saturating_sub(content_size);
		}

		Ok(track)
	}

	fn parse_tkhd(reader: &mut R, size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let version = reader.read_u8()?;
		let mut _flags = [0u8; 3];
		reader.read_exact(&mut _flags)?;

		if version == 1 {
			let _creation_time = reader.read_u64_be()?;
			let _modification_time = reader.read_u64_be()?;
			track.track_id = reader.read_u32_be()?;
			let _reserved = reader.read_u32_be()?;
			track.duration = reader.read_u64_be()?;
			Self::skip_bytes(reader, size - 32)?;
		} else {
			let _creation_time = reader.read_u32_be()?;
			let _modification_time = reader.read_u32_be()?;
			track.track_id = reader.read_u32_be()?;
			let _reserved = reader.read_u32_be()?;
			track.duration = reader.read_u32_be()? as u64;
			Self::skip_bytes(reader, size - 20)?;
		}

		Ok(())
	}

	fn parse_mdia(reader: &mut R, size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let mut remaining = size;

		while remaining >= 8 {
			let header = Self::read_box_header(reader)?;
			remaining -= header.header_size as u64;
			let content_size = header.size.saturating_sub(header.header_size as u64);

			match header.box_type {
				BoxType::Mdhd => {
					Self::parse_mdhd(reader, content_size, track)?;
				}
				BoxType::Hdlr => {
					Self::parse_hdlr(reader, content_size, track)?;
				}
				BoxType::Minf => {
					Self::parse_minf(reader, content_size, track)?;
				}
				_ => {
					Self::skip_bytes(reader, content_size)?;
				}
			}

			remaining = remaining.saturating_sub(content_size);
		}

		Ok(())
	}

	fn parse_mdhd(reader: &mut R, size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let version = reader.read_u8()?;
		let mut _flags = [0u8; 3];
		reader.read_exact(&mut _flags)?;

		if version == 1 {
			let _creation_time = reader.read_u64_be()?;
			let _modification_time = reader.read_u64_be()?;
			track.timescale = reader.read_u32_be()?;
			track.duration = reader.read_u64_be()?;
			Self::skip_bytes(reader, size - 28)?;
		} else {
			let _creation_time = reader.read_u32_be()?;
			let _modification_time = reader.read_u32_be()?;
			track.timescale = reader.read_u32_be()?;
			track.duration = reader.read_u32_be()? as u64;
			Self::skip_bytes(reader, size - 16)?;
		}

		Ok(())
	}

	fn parse_hdlr(reader: &mut R, size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let _version_flags = reader.read_u32_be()?;
		let _pre_defined = reader.read_u32_be()?;

		let mut handler_type = [0u8; 4];
		reader.read_exact(&mut handler_type)?;

		track.track_type = match &handler_type {
			b"vide" => TrackType::Video,
			b"soun" => TrackType::Audio,
			b"hint" => TrackType::Hint,
			b"text" => TrackType::Text,
			_ => TrackType::Unknown,
		};

		Self::skip_bytes(reader, size - 12)?;
		Ok(())
	}

	fn parse_minf(reader: &mut R, size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let mut remaining = size;

		while remaining >= 8 {
			let header = Self::read_box_header(reader)?;
			remaining -= header.header_size as u64;
			let content_size = header.size.saturating_sub(header.header_size as u64);

			match header.box_type {
				BoxType::Stbl => {
					Self::parse_stbl(reader, content_size, track)?;
				}
				_ => {
					Self::skip_bytes(reader, content_size)?;
				}
			}

			remaining = remaining.saturating_sub(content_size);
		}

		Ok(())
	}

	fn parse_stbl(reader: &mut R, size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let mut remaining = size;

		while remaining >= 8 {
			let header = Self::read_box_header(reader)?;
			remaining -= header.header_size as u64;
			let content_size = header.size.saturating_sub(header.header_size as u64);

			match header.box_type {
				BoxType::Stts => {
					Self::parse_stts(reader, content_size, track)?;
				}
				BoxType::Stsc => {
					Self::parse_stsc(reader, content_size, track)?;
				}
				BoxType::Stsz => {
					Self::parse_stsz(reader, content_size, track)?;
				}
				BoxType::Stco => {
					Self::parse_stco(reader, content_size, track)?;
				}
				BoxType::Co64 => {
					Self::parse_co64(reader, content_size, track)?;
				}
				_ => {
					Self::skip_bytes(reader, content_size)?;
				}
			}

			remaining = remaining.saturating_sub(content_size);
		}

		Ok(())
	}

	fn parse_stts(reader: &mut R, _size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let _version_flags = reader.read_u32_be()?;
		let entry_count = reader.read_u32_be()?;

		track.time_to_sample.clear();
		for _ in 0..entry_count {
			let sample_count = reader.read_u32_be()?;
			let sample_delta = reader.read_u32_be()?;
			track.time_to_sample.push((sample_count, sample_delta));
		}

		Ok(())
	}

	fn parse_stsc(reader: &mut R, _size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let _version_flags = reader.read_u32_be()?;
		let entry_count = reader.read_u32_be()?;

		track.sample_to_chunk.clear();
		for _ in 0..entry_count {
			let first_chunk = reader.read_u32_be()?;
			let samples_per_chunk = reader.read_u32_be()?;
			let sample_description_index = reader.read_u32_be()?;
			track.sample_to_chunk.push((first_chunk, samples_per_chunk, sample_description_index));
		}

		Ok(())
	}

	fn parse_stsz(reader: &mut R, _size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let _version_flags = reader.read_u32_be()?;
		let sample_size = reader.read_u32_be()?;
		let sample_count = reader.read_u32_be()?;

		track.sample_sizes.clear();
		if sample_size == 0 {
			for _ in 0..sample_count {
				let size = reader.read_u32_be()?;
				track.sample_sizes.push(size);
			}
		} else {
			for _ in 0..sample_count {
				track.sample_sizes.push(sample_size);
			}
		}

		Ok(())
	}

	fn parse_stco(reader: &mut R, _size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let _version_flags = reader.read_u32_be()?;
		let entry_count = reader.read_u32_be()?;

		track.chunk_offsets.clear();
		for _ in 0..entry_count {
			let offset = reader.read_u32_be()? as u64;
			track.chunk_offsets.push(offset);
		}

		Ok(())
	}

	fn parse_co64(reader: &mut R, _size: u64, track: &mut Mp4Track) -> IoResult<()> {
		let _version_flags = reader.read_u32_be()?;
		let entry_count = reader.read_u32_be()?;

		track.chunk_offsets.clear();
		for _ in 0..entry_count {
			let offset = reader.read_u64_be()?;
			track.chunk_offsets.push(offset);
		}

		Ok(())
	}

	fn skip_bytes(reader: &mut R, count: u64) -> IoResult<()> {
		let mut buf = [0u8; 1024];
		let mut remaining = count;
		while remaining > 0 {
			let to_read = (remaining as usize).min(buf.len());
			reader.read_exact(&mut buf[..to_read])?;
			remaining -= to_read as u64;
		}
		Ok(())
	}
}

impl<R: MediaRead> Demuxer for Mp4Reader<R> {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		if self.eof || self.format.tracks.is_empty() {
			return Ok(None);
		}

		if self.current_track >= self.format.tracks.len() {
			self.current_track = 0;
			self.current_sample += 1;
		}

		let track = &self.format.tracks[self.current_track];
		if self.current_sample >= track.sample_sizes.len() {
			self.eof = true;
			return Ok(None);
		}

		let sample_size = track.sample_sizes[self.current_sample] as usize;
		let mut data = vec![0u8; sample_size];

		match self.reader.read_exact(&mut data) {
			Ok(()) => {}
			Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
				self.eof = true;
				return Ok(None);
			}
			Err(e) => return Err(e),
		}

		self.bytes_read += sample_size as u64;

		let pts = self.current_sample as i64;
		let stream_index = self.current_track;

		self.current_track += 1;

		Ok(Some(Packet::new(data, stream_index, self.timebase).with_pts(pts)))
	}

	fn stream_count(&self) -> usize {
		self.format.tracks.len()
	}
}
