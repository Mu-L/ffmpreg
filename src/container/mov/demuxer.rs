use super::utils::{BoxHeader, *};
use crate::core::{Demuxer, Packet, Stream, StreamKind, Time, stream};
use crate::io::{Error, MediaRead, MediaSeek, ReadPrimitives, Result, SeekFrom};

#[allow(dead_code)]
struct StsdEntry {
	fourcc: [u8; 4],
	audio_entry: Option<AudioSampleEntry>,
	video_entry: Option<VideoSampleEntry>,
}

#[allow(dead_code)]
pub struct MovDemuxer<R: MediaRead + MediaSeek> {
	reader: R,
	streams: Vec<Stream>,
	sample_tables: Vec<SampleTable>,
	packet_index: usize,
	mdat_offset: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SampleTable {
	stream_index: usize,
	sample_sizes: Vec<u32>,
	sample_offsets: Vec<u64>,
	keyframes: Vec<bool>,
}

impl<R: MediaRead + MediaSeek> MovDemuxer<R> {
	pub fn new(mut reader: R) -> Result<Self> {
		let mut ftyp_found = false;
		let mut moov_pos = None;
		let mut mdat_offset = 0;
		let mut streams = Vec::new();

		reader.rewind()?;

		loop {
			let header = Self::read_box_header(&mut reader)?;
			if header.size == 0 {
				break;
			}

			if header.is_type(FTYP_BOX) {
				ftyp_found = true;
			} else if header.is_type(MOOV_BOX) {
				moov_pos = Some(header.position);
			} else if header.is_type(MDAT_BOX) {
				mdat_offset = header.position + 8;
			}

			let next_pos = header.position + header.size as u64;
			reader.seek(SeekFrom::Start(next_pos))?;

			if moov_pos.is_some() && mdat_offset > 0 {
				break;
			}
		}

		if !ftyp_found {
			return Err(Error::invalid_data("ftyp box not found"));
		}
		if moov_pos.is_none() {
			return Err(Error::invalid_data("moov box not found"));
		}

		let moov_pos = moov_pos.unwrap();
		reader.seek(SeekFrom::Start(moov_pos))?;

		let moov_header = Self::read_box_header(&mut reader)?;
		let moov_end = moov_pos + moov_header.size as u64;

		while reader.stream_position()? < moov_end {
			let header = Self::read_box_header(&mut reader)?;
			if header.is_type(TRAK_BOX) {
				if let Ok(stream) = Self::parse_trak(&mut reader, &header) {
					streams.push(stream);
				}
			} else {
				let next = header.position + header.size as u64;
				reader.seek(SeekFrom::Start(next))?;
			}
		}

		let sample_tables = Vec::new();

		Ok(Self { reader, streams, sample_tables, packet_index: 0, mdat_offset })
	}

	fn read_box_header(reader: &mut R) -> Result<BoxHeader> {
		let position = reader.stream_position()?;
		let size = reader.read_u32_be()?;
		let mut fourcc = [0u8; 4];
		reader.read_exact(&mut fourcc)?;

		if size < 8 {
			return Err(Error::invalid_data("invalid box size"));
		}

		Ok(BoxHeader::new(size, fourcc, position))
	}

	fn parse_trak(reader: &mut R, trak_header: &BoxHeader) -> Result<Stream> {
		let trak_end = trak_header.position + trak_header.size as u64;
		let mut stream_id = 0;
		let mut mdia_pos = None;

		while reader.stream_position()? < trak_end {
			let header = Self::read_box_header(reader)?;

			if header.is_type(TKHD_BOX) {
				reader.seek(SeekFrom::Current(4))?;
				reader.seek(SeekFrom::Current(4))?;
				stream_id = reader.read_u32_be()?;
				reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
			} else if header.is_type(MDIA_BOX) {
				mdia_pos = Some(header.position);
				reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
			} else {
				reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
			}
		}

		if let Some(pos) = mdia_pos {
			reader.seek(SeekFrom::Start(pos))?;
			let mdia_header = Self::read_box_header(reader)?;
			return Self::parse_mdia(reader, &mdia_header, stream_id);
		}

		Err(Error::invalid_data("mdia box not found"))
	}

	fn parse_mdia(reader: &mut R, mdia_header: &BoxHeader, stream_id: u32) -> Result<Stream> {
		let mdia_end = mdia_header.position + mdia_header.size as u64;
		let mut mdhd_data = None;
		let mut hdlr_type = None;
		let mut stbl_pos = None;

		while reader.stream_position()? < mdia_end {
			let header = Self::read_box_header(reader)?;

			if header.is_type(MDHD_BOX) {
				mdhd_data = Some(Self::read_mdhd(reader, &header)?);
			} else if header.is_type(HDLR_BOX) {
				hdlr_type = Some(Self::read_hdlr(reader, &header)?);
			} else if header.is_type(MINF_BOX) {
				stbl_pos = Some(Self::find_stbl(reader, &header)?);
				reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
			} else {
				reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
			}
		}

		let (sample_rate, time_scale) = mdhd_data.unwrap_or((48000, 48000));
		let handler = hdlr_type.unwrap_or_default();

		if handler.is_audio() {
			Self::parse_audio_stream(reader, stream_id, stbl_pos, sample_rate, time_scale)
		} else if handler.is_video() {
			Self::parse_video_stream(reader, stream_id, stbl_pos, sample_rate, time_scale)
		} else {
			Err(Error::invalid_data("unknown handler type"))
		}
	}

	fn read_mdhd(reader: &mut R, header: &BoxHeader) -> Result<(u32, u32)> {
		reader.seek(SeekFrom::Current(4))?;
		let time_scale = reader.read_u32_be()?;
		reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
		Ok((time_scale, time_scale))
	}

	fn read_hdlr(reader: &mut R, header: &BoxHeader) -> Result<HandlerType> {
		reader.seek(SeekFrom::Current(4))?;
		reader.seek(SeekFrom::Current(4))?;
		let mut handler_type = [0u8; 4];
		reader.read_exact(&mut handler_type)?;
		reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
		Ok(HandlerType::from_bytes(&handler_type))
	}

	fn find_stbl(reader: &mut R, minf_header: &BoxHeader) -> Result<u64> {
		let minf_end = minf_header.position + minf_header.size as u64;

		while reader.stream_position()? < minf_end {
			let header = Self::read_box_header(reader)?;
			if header.is_type(STBL_BOX) {
				return Ok(header.position);
			}
			reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
		}

		Err(Error::invalid_data("stbl box not found"))
	}

	fn parse_audio_stream(
		reader: &mut R,
		stream_id: u32,
		stbl_pos: Option<u64>,
		_sample_rate: u32,
		time_scale: u32,
	) -> Result<Stream> {
		if let Some(pos) = stbl_pos {
			reader.seek(SeekFrom::Start(pos))?;
			let codec = Self::detect_audio_codec(reader, pos)?;
			let time = Time::new(1, time_scale);
			return Ok(Stream::new(stream_id, 0, StreamKind::Audio, codec, time));
		}

		Err(Error::invalid_data("no stbl found for audio"))
	}

	fn parse_video_stream(
		reader: &mut R,
		stream_id: u32,
		stbl_pos: Option<u64>,
		_sample_rate: u32,
		time_scale: u32,
	) -> Result<Stream> {
		if let Some(pos) = stbl_pos {
			reader.seek(SeekFrom::Start(pos))?;
			let codec = Self::detect_video_codec(reader, pos)?;
			let time = Time::new(1, time_scale);
			return Ok(Stream::new(stream_id, 1, StreamKind::Video, codec, time));
		}

		Err(Error::invalid_data("no stbl found for video"))
	}

	fn detect_audio_codec(reader: &mut R, stbl_pos: u64) -> Result<String> {
		reader.seek(SeekFrom::Start(stbl_pos))?;
		let stbl_header = Self::read_box_header(reader)?;
		let stbl_end = stbl_pos + stbl_header.size as u64;

		while reader.stream_position()? < stbl_end {
			let header = Self::read_box_header(reader)?;
			if header.is_type(STSD_BOX) {
				return Self::read_stsd_audio(reader, &header);
			}
			reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
		}

		Err(Error::invalid_data("stsd not found"))
	}

	fn detect_video_codec(reader: &mut R, stbl_pos: u64) -> Result<String> {
		reader.seek(SeekFrom::Start(stbl_pos))?;
		let stbl_header = Self::read_box_header(reader)?;
		let stbl_end = stbl_pos + stbl_header.size as u64;

		while reader.stream_position()? < stbl_end {
			let header = Self::read_box_header(reader)?;
			if header.is_type(STSD_BOX) {
				return Self::read_stsd_video(reader, &header);
			}
			reader.seek(SeekFrom::Start(header.position + header.size as u64))?;
		}

		Err(Error::invalid_data("stsd not found"))
	}

	fn read_stsd_audio(reader: &mut R, _header: &BoxHeader) -> Result<String> {
		reader.seek(SeekFrom::Current(4))?;
		let entry_count = reader.read_u32_be()?;

		if entry_count == 0 {
			return Err(Error::invalid_data("no audio entries in stsd"));
		}

		let _sample_entry_size = reader.read_u32_be()?;
		let mut codec = [0u8; 4];
		reader.read_exact(&mut codec)?;

		if codec == *b"mp4a" {
			Ok("aac".to_string())
		} else {
			Err(Error::invalid_data("unsupported audio codec"))
		}
	}

	fn read_stsd_video(reader: &mut R, _header: &BoxHeader) -> Result<String> {
		reader.seek(SeekFrom::Current(4))?;
		let entry_count = reader.read_u32_be()?;

		if entry_count == 0 {
			return Err(Error::invalid_data("no video entries in stsd"));
		}

		let _sample_entry_size = reader.read_u32_be()?;
		let mut codec = [0u8; 4];
		reader.read_exact(&mut codec)?;

		if codec == *b"avc1" {
			Ok("h264".to_string())
		} else {
			Err(Error::invalid_data("unsupported video codec"))
		}
	}

	pub fn streams(&self) -> stream::Streams {
		stream::Streams::new(self.streams.clone())
	}

	pub fn read_packet(&mut self) -> Result<Option<Packet>> {
		if self.sample_tables.is_empty() {
			return Ok(None);
		}

		if self.packet_index == 0 {
			return Ok(None);
		}

		Ok(None)
	}
}

#[derive(Debug, Clone, Copy)]
struct HandlerType {
	value: [u8; 4],
}

impl Default for HandlerType {
	fn default() -> Self {
		Self { value: [0; 4] }
	}
}

impl HandlerType {
	fn from_bytes(bytes: &[u8; 4]) -> Self {
		Self { value: *bytes }
	}

	fn is_audio(&self) -> bool {
		self.value == *b"soun"
	}

	fn is_video(&self) -> bool {
		self.value == *b"vide"
	}
}

impl<R: MediaRead + MediaSeek> Demuxer for MovDemuxer<R> {
	fn streams(&self) -> &stream::Streams {
		unimplemented!("use the public streams() method instead")
	}

	fn read_packet(&mut self) -> Result<Option<Packet>> {
		self.read_packet()
	}
}
