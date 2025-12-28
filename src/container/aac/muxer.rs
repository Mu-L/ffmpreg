use crate::codecs::audio::aac::ADTSHeader;
use crate::codecs::audio::aac::utils::get_sample_rate_index;
use crate::core::{Muxer, Packet, Stream, StreamKind, Time, stream};
use crate::io::{MediaWrite, Result, WritePrimitives};

pub struct AACMuxer<W: MediaWrite> {
	writer: W,
	stream: Stream,
	sample_rate: u32,
	channels: u8,
	sample_rate_index: u8,
}

impl<W: MediaWrite> AACMuxer<W> {
	pub fn new(writer: W, sample_rate: u32, channels: u8) -> Result<Self> {
		let codec_name = "aac";
		let time = Time::new(1, sample_rate);
		let stream = Stream::new(0, 0, StreamKind::Audio, codec_name.to_string(), time);

		let sample_rate_index = get_sample_rate_index(sample_rate)
			.ok_or_else(|| crate::io::Error::invalid_data("Unsupported sample rate for AAC"))?;

		Ok(Self { writer, stream, sample_rate, channels, sample_rate_index })
	}

	pub fn streams(&self) -> crate::core::stream::Streams {
		crate::core::stream::Streams::new(vec![self.stream.clone()])
	}

	pub fn write_packet(&mut self, packet: Packet) -> Result<()> {
		// Wrap PCM samples in ADTS frames (ADTS frame_length max is 13 bits = 8191 bytes)
		// Split large packets into multiple frames
		let mut data_offset = 0;
		while data_offset < packet.data.len() {
			let remaining = packet.data.len() - data_offset;
			// Max payload: 8191 - 7 = 8184 bytes
			let payload_size = remaining.min(8184);
			let frame_length = (payload_size + 7) as u16; // 7 bytes header + payload

			let header = ADTSHeader {
				syncword: 0xFFF,
				id: true,
				layer: 0,
				protection_absent: true,
				profile: 1, // AAC-LC
				sample_rate_index: self.sample_rate_index,
				private_bit: false,
				channel_config: self.channels.min(7), // Limit to valid range
				original: false,
				home: false,
				copyright_id_start: false,
				frame_length,
				adts_buffer_fullness: 0x7FF, // VBR mode
				number_of_rdb: 0,
				crc_check: None,
			};

			let header_bytes = header.serialize();
			self.writer.write_all(&header_bytes)?;
			self.writer.write_all(&packet.data[data_offset..data_offset + payload_size])?;
			data_offset += payload_size;
		}
		Ok(())
	}

	pub fn finalize(&mut self) -> Result<()> {
		self.writer.flush()?;
		Ok(())
	}
}

impl<W: MediaWrite> Muxer for AACMuxer<W> {
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
