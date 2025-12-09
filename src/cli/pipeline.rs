use crate::codecs::{PcmDecoder, PcmEncoder};
use crate::container::{WavReader, WavWriter};
use crate::core::{Decoder, Demuxer, Encoder, Muxer, Timebase};
use std::fs::File;
use std::io::Result;

pub struct Pipeline {
	input_path: String,
	output_path: String,
}

impl Pipeline {
	pub fn new(input_path: String, output_path: String) -> Self {
		Self { input_path, output_path }
	}

	pub fn run(&self) -> Result<()> {
		let input = File::open(&self.input_path)?;
		let mut reader = WavReader::new(input)?;
		let format = reader.format();

		let output = File::create(&self.output_path)?;
		let mut writer = WavWriter::new(output, format)?;

		let mut decoder = PcmDecoder::new(format);
		let timebase = Timebase::new(1, format.sample_rate as u32);
		let mut encoder = PcmEncoder::new(timebase);

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					if let Some(frame) = decoder.decode(packet)? {
						if let Some(pkt) = encoder.encode(frame)? {
							writer.write_packet(pkt)?;
						}
					}
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}
}
