use crate::container::wav::WavFormat;
use crate::core::{Muxer, Packet, Stream, StreamKind, Time, stream};
use crate::io::{MediaSeek, MediaWrite, Result, SeekFrom, WritePrimitives};

pub struct WavMuxer<W: MediaWrite + MediaSeek> {
	writer: W,
	#[allow(dead_code)]
	format: WavFormat,
	stream: Stream,
	data_size: u32,
	data_size_position: u64,
	file_size_position: u64,
}

impl<W: MediaWrite + MediaSeek> WavMuxer<W> {
	pub fn new(mut writer: W, format: WavFormat) -> Result<Self> {
		let file_size_position = Self::write_wav_header(&mut writer, &format)?;
		let data_size_position = writer.stream_position()?;

		writer.write_u32_le(0)?;
		writer.flush()?;

		let codec_name = format.to_codec_string().to_string();
		let time = Time::new(1, format.sample_rate);
		let stream = Stream::new(0, 0, StreamKind::Audio, codec_name, time);

		Ok(Self { writer, format, stream, data_size: 0, data_size_position, file_size_position })
	}

	fn write_wav_header(writer: &mut W, format: &WavFormat) -> Result<u64> {
		writer.write_all(b"RIFF")?;

		let file_size_position = writer.stream_position()?;
		writer.write_u32_le(0)?;

		writer.write_all(b"WAVE")?;

		writer.write_all(b"fmt ")?;

		// fmt chunk size depends on format
		let fmt_chunk_size = if format.format_code == 0x11 { 20 } else { 16 };
		writer.write_u32_le(fmt_chunk_size)?;

		writer.write_u16_le(format.format_code)?; // audio format
		writer.write_u16_le(format.channels as u16)?; // num channels
		writer.write_u32_le(format.sample_rate)?; // sample rate

		writer.write_u32_le(format.byte_rate())?;
		writer.write_u16_le(format.block_align())?;

		writer.write_u16_le(format.bit_depth)?;

		// Extra bytes for ADPCM
		if format.format_code == 0x11 {
			writer.write_u16_le(4)?; // extra bytes (samples per block)
			let samples_per_block = ((512 - 4 * format.channels as usize) * 2) + 1;
			writer.write_u16_le(samples_per_block as u16)?;
		}

		writer.write_all(b"data")?;

		Ok(file_size_position)
	}

	pub fn streams(&self) -> crate::core::stream::Streams {
		crate::core::stream::Streams::new(vec![self.stream.clone()])
	}

	pub fn write_packet(&mut self, packet: Packet) -> Result<()> {
		self.writer.write_all(&packet.data)?;
		self.data_size += packet.data.len() as u32;
		Ok(())
	}

	pub fn finalize(&mut self) -> Result<()> {
		let current_pos = self.writer.stream_position()?;

		self.writer.seek(SeekFrom::Start(self.data_size_position))?;
		self.writer.write_u32_le(self.data_size)?;

		let file_size = self.data_size + 36;
		self.writer.seek(SeekFrom::Start(self.file_size_position))?;
		self.writer.write_u32_le(file_size)?;

		self.writer.seek(SeekFrom::Start(current_pos))?;
		self.writer.flush()?;

		Ok(())
	}
}

impl<W: MediaWrite + MediaSeek> Muxer for WavMuxer<W> {
	fn streams(&self) -> &stream::Streams {
		unimplemented!("use the public streams() method instead")
	}

	fn write(&mut self, packet: Packet) -> Result<()> {
		self.write_packet(packet)
	}

	fn finalize(&mut self) -> Result<()> {
		self.finalize()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::io::MediaWrite;

	struct MockWriter {
		data: Vec<u8>,
		pos: u64,
	}

	impl MockWriter {
		fn new() -> Self {
			Self { data: Vec::new(), pos: 0 }
		}

		fn get_data(&self) -> &[u8] {
			&self.data
		}
	}

	impl MediaWrite for MockWriter {
		fn write(&mut self, buf: &[u8]) -> Result<usize> {
			self.data.extend_from_slice(buf);
			self.pos += buf.len() as u64;
			Ok(buf.len())
		}

		fn flush(&mut self) -> Result<()> {
			Ok(())
		}
	}

	impl MediaSeek for MockWriter {
		fn seek(&mut self, pos: crate::io::SeekFrom) -> Result<u64> {
			let new_pos = match pos {
				crate::io::SeekFrom::Start(n) => n,
				crate::io::SeekFrom::Current(n) => (self.pos as i64 + n) as u64,
				crate::io::SeekFrom::End(n) => (self.data.len() as i64 + n) as u64,
			};
			self.pos = new_pos;
			Ok(new_pos)
		}

		fn stream_position(&mut self) -> Result<u64> {
			Ok(self.pos)
		}
	}

	#[test]
	fn test_wav_header_format_pcm() {
		let format = WavFormat { channels: 1, sample_rate: 24000, bit_depth: 16, format_code: 1 };
		let mut writer = MockWriter::new();

		WavMuxer::write_wav_header(&mut writer, &format).expect("Failed to write header");

		let data = writer.get_data();

		// Check RIFF and WAVE
		assert_eq!(&data[0..4], b"RIFF");
		assert_eq!(&data[8..12], b"WAVE");

		// Check fmt chunk
		assert_eq!(&data[12..16], b"fmt ");

		// Check fmt subchunk1 size (should be 16 for PCM)
		let fmt_size = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
		assert_eq!(fmt_size, 16);

		// Check audio format (1 = PCM)
		let audio_format = u16::from_le_bytes([data[20], data[21]]);
		assert_eq!(audio_format, 1);

		// Check channels (should be 1)
		let channels = u16::from_le_bytes([data[22], data[23]]);
		assert_eq!(channels, 1);

		// Check sample rate (should be 24000)
		let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
		assert_eq!(sample_rate, 24000);

		// Check byte rate (should be 48000)
		let byte_rate = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
		assert_eq!(byte_rate, 48000);

		// Check block align (should be 2)
		let block_align = u16::from_le_bytes([data[32], data[33]]);
		assert_eq!(block_align, 2);

		// Check bit depth (should be 16)
		let bit_depth = u16::from_le_bytes([data[34], data[35]]);
		assert_eq!(bit_depth, 16);

		// Check data chunk identifier
		assert_eq!(&data[36..40], b"data");
	}
}
