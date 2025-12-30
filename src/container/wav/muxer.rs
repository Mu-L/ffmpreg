use crate::container::wav::{WavFormat, WavMetadata};
use crate::core::{Muxer, Packet, Stream, StreamKind, Time, stream};
use crate::io::{MediaSeek, MediaWrite, Result, SeekFrom, WritePrimitives};

pub struct WavMuxer<W: MediaWrite + MediaSeek> {
	writer: W,
	#[allow(dead_code)]
	format: WavFormat,
	streams: stream::Streams,
	metadata: Option<WavMetadata>,
	data_size: u32,
	data_size_pos: u64,
	file_size_pos: u64,
}

impl<W: MediaWrite + MediaSeek> WavMuxer<W> {
	pub fn new(mut writer: W, format: WavFormat) -> Result<Self> {
		let (file_size_pos, data_size_pos) = Self::write_header(&mut writer, &format)?;
		writer.flush()?;

		let codec_name = format.to_codec_string().to_string();
		let time = Time::new(1, format.sample_rate);
		let mut streams = stream::Streams::new_empty();
		let stream = Stream::new(0, 0, StreamKind::Audio, codec_name, time);

		streams.add(stream);

		Ok(Self { writer, format, streams, metadata: None, data_size: 0, data_size_pos, file_size_pos })
	}

	pub fn with_metadata(&mut self, metadata: WavMetadata) {
		self.metadata = Some(metadata);
	}

	fn write_header(writer: &mut W, format: &WavFormat) -> Result<(u64, u64)> {
		writer.write_all(b"RIFF")?;
		let file_size_pos = writer.stream_position()?;
		writer.write_u32_le(0)?;
		writer.write_all(b"WAVE")?;
		writer.write_all(b"fmt ")?;

		let fmt_size = match format.format_code {
			3 => 18,
			0x11 => 20,
			_ => 16,
		};
		writer.write_u32_le(fmt_size)?;
		writer.write_u16_le(format.format_code)?;
		writer.write_u16_le(format.channels as u16)?;
		writer.write_u32_le(format.sample_rate)?;
		writer.write_u32_le(format.byte_rate())?;
		writer.write_u16_le(format.block_align())?;
		writer.write_u16_le(format.bit_depth)?;

		if format.format_code == 3 {
			writer.write_u16_le(0)?;
		} else if format.format_code == 0x11 {
			writer.write_u16_le(4)?;
			let spb = ((512 - 4 * format.channels as usize) * 2 + 1) as u16;
			writer.write_u16_le(spb)?;
		}

		writer.write_all(b"data")?;
		let data_size_pos = writer.stream_position()?;
		writer.write_u32_le(0)?;
		Ok((file_size_pos, data_size_pos))
	}

	pub fn write_packet(&mut self, packet: Packet) -> Result<()> {
		self.writer.write_all(&packet.data)?;
		self.data_size += packet.data.len() as u32;
		Ok(())
	}

	pub fn finalize(&mut self) -> Result<()> {
		self.writer.seek(SeekFrom::Start(self.data_size_pos))?;
		self.writer.write_u32_le(self.data_size)?;

		let mut file_size = self.data_size + 36;

		if let Some(meta) = &self.metadata {
			if !meta.is_empty() {
				file_size += Self::calc_list_size(meta) as u32;
				self.writer.seek(SeekFrom::End(0))?;
				Self::write_list_chunk(&mut self.writer, meta)?;
			}
		}

		self.writer.seek(SeekFrom::Start(self.file_size_pos))?;
		self.writer.write_u32_le(file_size)?;
		self.writer.flush()?;
		Ok(())
	}

	fn calc_list_size(metadata: &WavMetadata) -> u64 {
		metadata.all_fields().values().fold(8, |acc, v| {
			let mut size = acc + 8 + v.len() as u64 + 1;
			if (v.len() + 1) % 2 == 1 {
				size += 1;
			}
			size
		})
	}

	fn write_list_chunk(writer: &mut W, metadata: &WavMetadata) -> Result<()> {
		if metadata.is_empty() {
			return Ok(());
		}

		let list_size = Self::calc_list_size(metadata) - 8;
		writer.write_all(b"LIST")?;
		writer.write_u32_le(list_size as u32)?;
		writer.write_all(b"INFO")?;

		for (field, value) in metadata.all_fields() {
			let id: &[u8; 4] = match field.as_str() {
				"artist" => b"IART",
				"title" => b"INAM",
				"comment" => b"ICOM",
				"copyright" => b"ICOP",
				"software" => b"ISFT",
				"genre" => b"IGNR",
				"track" => b"ITRK",
				_ => continue,
			};
			Self::write_info_chunk(writer, id, value)?;
		}
		Ok(())
	}

	fn write_info_chunk(writer: &mut W, id: &[u8; 4], value: &str) -> Result<()> {
		let data = format!("{}\0", value);
		writer.write_all(id)?;
		writer.write_u32_le(data.len() as u32)?;
		writer.write_all(data.as_bytes())?;
		if data.len() % 2 == 1 {
			writer.write_u8(0)?;
		}
		Ok(())
	}
}

impl<W: MediaWrite + MediaSeek> Muxer for WavMuxer<W> {
	fn streams(&self) -> &stream::Streams {
		&self.streams
	}
	fn write(&mut self, packet: Packet) -> Result<()> {
		self.write_packet(packet)
	}
	fn finalize(&mut self) -> Result<()> {
		self.finalize()
	}
}
