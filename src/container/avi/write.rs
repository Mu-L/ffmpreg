use super::{AVI_SIGNATURE, AviFormat, LIST_SIGNATURE, RIFF_SIGNATURE};
use crate::core::{Muxer, Packet};
use crate::io::{IoResult, MediaSeek, MediaWrite, SeekFrom, WritePrimitives};

pub struct AviWriter<W: MediaWrite + MediaSeek> {
	writer: W,
	format: AviFormat,
	frame_count: u32,
	movi_start: u64,
	index_entries: Vec<IndexEntry>,
}

struct IndexEntry {
	chunk_id: [u8; 4],
	flags: u32,
	offset: u32,
	size: u32,
}

impl<W: MediaWrite + MediaSeek> AviWriter<W> {
	pub fn new(mut writer: W, format: AviFormat) -> IoResult<Self> {
		let movi_start = Self::write_header(&mut writer, &format)?;
		Ok(Self { writer, format, frame_count: 0, movi_start, index_entries: Vec::new() })
	}

	fn write_header(writer: &mut W, format: &AviFormat) -> IoResult<u64> {
		writer.write_all(RIFF_SIGNATURE)?;
		writer.write_u32_le(0)?;

		writer.write_all(AVI_SIGNATURE)?;

		writer.write_all(LIST_SIGNATURE)?;
		let hdrl_size_pos = writer.stream_position()?;
		writer.write_u32_le(0)?;
		writer.write_all(b"hdrl")?;

		Self::write_avih(writer, format)?;

		for stream in &format.streams {
			Self::write_strl(writer, stream)?;
		}

		let hdrl_end = writer.stream_position()?;
		let hdrl_size = (hdrl_end - hdrl_size_pos - 4) as u32;
		writer.seek(SeekFrom::Start(hdrl_size_pos))?;
		writer.write_u32_le(hdrl_size)?;
		writer.seek(SeekFrom::Start(hdrl_end))?;

		writer.write_all(LIST_SIGNATURE)?;
		writer.write_u32_le(0)?;
		writer.write_all(b"movi")?;

		Ok(writer.stream_position()?)
	}

	fn write_avih(writer: &mut W, format: &AviFormat) -> IoResult<()> {
		writer.write_all(b"avih")?;
		writer.write_u32_le(56)?;

		writer.write_u32_le(format.main_header.microseconds_per_frame)?;
		writer.write_u32_le(format.main_header.max_bytes_per_sec)?;
		writer.write_u32_le(format.main_header.padding_granularity)?;
		writer.write_u32_le(format.main_header.flags)?;
		writer.write_u32_le(format.main_header.total_frames)?;
		writer.write_u32_le(format.main_header.initial_frames)?;
		writer.write_u32_le(format.main_header.streams)?;
		writer.write_u32_le(format.main_header.suggested_buffer_size)?;
		writer.write_u32_le(format.main_header.width)?;
		writer.write_u32_le(format.main_header.height)?;
		writer.write_all(&[0u8; 16])?;

		Ok(())
	}

	fn write_strl(writer: &mut W, stream: &super::AviStream) -> IoResult<()> {
		writer.write_all(LIST_SIGNATURE)?;
		let strl_size_pos = writer.stream_position()?;
		writer.write_u32_le(0)?;
		writer.write_all(b"strl")?;

		writer.write_all(b"strh")?;
		writer.write_u32_le(56)?;
		writer.write_all(&stream.header.stream_type.as_fourcc())?;
		writer.write_all(&stream.header.handler)?;
		writer.write_u32_le(stream.header.flags)?;
		writer.write_u16_le(stream.header.priority)?;
		writer.write_u16_le(stream.header.language)?;
		writer.write_u32_le(stream.header.initial_frames)?;
		writer.write_u32_le(stream.header.scale)?;
		writer.write_u32_le(stream.header.rate)?;
		writer.write_u32_le(stream.header.start)?;
		writer.write_u32_le(stream.header.length)?;
		writer.write_u32_le(stream.header.suggested_buffer_size)?;
		writer.write_u32_le(stream.header.quality)?;
		writer.write_u32_le(stream.header.sample_size)?;
		for &v in &stream.header.rect {
			writer.write_u16_le(v)?;
		}

		if let Some(ref vf) = stream.video_format {
			writer.write_all(b"strf")?;
			writer.write_u32_le(40)?;
			writer.write_u32_le(vf.size)?;
			writer.write_i32_le(vf.width)?;
			writer.write_i32_le(vf.height)?;
			writer.write_u16_le(vf.planes)?;
			writer.write_u16_le(vf.bit_count)?;
			writer.write_all(&vf.compression)?;
			writer.write_u32_le(vf.size_image)?;
			writer.write_i32_le(vf.x_pels_per_meter)?;
			writer.write_i32_le(vf.y_pels_per_meter)?;
			writer.write_u32_le(vf.clr_used)?;
			writer.write_u32_le(vf.clr_important)?;
		}

		if let Some(ref af) = stream.audio_format {
			writer.write_all(b"strf")?;
			writer.write_u32_le(16)?;
			writer.write_u16_le(af.format_tag)?;
			writer.write_u16_le(af.channels)?;
			writer.write_u32_le(af.samples_per_sec)?;
			writer.write_u32_le(af.avg_bytes_per_sec)?;
			writer.write_u16_le(af.block_align)?;
			writer.write_u16_le(af.bits_per_sample)?;
		}

		let strl_end = writer.stream_position()?;
		let strl_size = (strl_end - strl_size_pos - 4) as u32;
		writer.seek(SeekFrom::Start(strl_size_pos))?;
		writer.write_u32_le(strl_size)?;
		writer.seek(SeekFrom::Start(strl_end))?;

		Ok(())
	}

	fn write_index(&mut self) -> IoResult<()> {
		self.writer.write_all(b"idx1")?;
		self.writer.write_u32_le((self.index_entries.len() * 16) as u32)?;

		for entry in &self.index_entries {
			self.writer.write_all(&entry.chunk_id)?;
			self.writer.write_u32_le(entry.flags)?;
			self.writer.write_u32_le(entry.offset)?;
			self.writer.write_u32_le(entry.size)?;
		}

		Ok(())
	}
}

impl<W: MediaWrite + MediaSeek> Muxer for AviWriter<W> {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
		let stream_idx = packet.stream_index;
		let chunk_id = if stream_idx < self.format.streams.len() {
			let stream = &self.format.streams[stream_idx];
			match stream.header.stream_type {
				super::StreamType::Video => {
					[b'0' + (stream_idx / 10) as u8, b'0' + (stream_idx % 10) as u8, b'd', b'c']
				}
				super::StreamType::Audio => {
					[b'0' + (stream_idx / 10) as u8, b'0' + (stream_idx % 10) as u8, b'w', b'b']
				}
				_ => [b'0' + (stream_idx / 10) as u8, b'0' + (stream_idx % 10) as u8, b'd', b'c'],
			}
		} else {
			[b'0', b'0', b'd', b'c']
		};

		let offset = (self.writer.stream_position()? - self.movi_start + 4) as u32;

		self.writer.write_all(&chunk_id)?;
		self.writer.write_u32_le(packet.data.len() as u32)?;
		self.writer.write_all(&packet.data)?;

		if packet.data.len() % 2 == 1 {
			self.writer.write_u8(0)?;
		}

		self.index_entries.push(IndexEntry {
			chunk_id,
			flags: 0x10,
			offset,
			size: packet.data.len() as u32,
		});

		self.frame_count += 1;
		Ok(())
	}

	fn finalize(&mut self) -> IoResult<()> {
		let movi_end = self.writer.stream_position()?;
		let movi_size = (movi_end - self.movi_start + 4) as u32;

		self.writer.seek(SeekFrom::Start(self.movi_start - 4))?;
		self.writer.write_u32_le(movi_size)?;
		self.writer.seek(SeekFrom::Start(movi_end))?;

		self.write_index()?;

		let file_end = self.writer.stream_position()?;
		let file_size = (file_end - 8) as u32;
		self.writer.seek(SeekFrom::Start(4))?;
		self.writer.write_u32_le(file_size)?;

		self.writer.seek(SeekFrom::Start(48))?;
		self.writer.write_u32_le(self.frame_count)?;
		self.writer.seek(SeekFrom::Start(file_end))?;

		self.writer.flush()?;
		Ok(())
	}
}
