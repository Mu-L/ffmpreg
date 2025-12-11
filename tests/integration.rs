mod cli;
mod codecs;
mod common;
mod container;
mod core;
mod transform;

use ffmpreg::codecs::{PcmDecoder, PcmEncoder, RawVideoDecoder, RawVideoEncoder};
use ffmpreg::container::{WavFormat, WavReader, WavWriter, Y4mReader, Y4mWriter};
use ffmpreg::core::{Decoder, Demuxer, Encoder, Muxer, Timebase, Transform};
use ffmpreg::transform::{Gain, Normalize, TransformChain};
use std::io::{BufWriter, Cursor};

#[test]
fn test_full_wav_pipeline() {
	let wav_data = common::create_test_wav_data();
	let cursor = Cursor::new(wav_data.clone());

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			if let Some(pkt) = encoder.encode(frame).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_full_wav_pipeline_with_gain() {
	let wav_data = common::create_test_wav_data();
	let cursor = Cursor::new(wav_data);

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);
	let mut gain = Gain::new(2.0);

	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			let processed = gain.apply(frame).unwrap();
			if let Some(pkt) = encoder.encode(processed).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_full_wav_pipeline_with_chain() {
	let wav_data = common::create_test_wav_data();
	let cursor = Cursor::new(wav_data);

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	let mut chain = TransformChain::new();
	chain.add(Box::new(Gain::new(0.5)));
	chain.add(Box::new(Normalize::new(0.9)));

	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			let processed = chain.apply(frame).unwrap();
			if let Some(pkt) = encoder.encode(processed).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_full_y4m_pipeline() {
	let y4m_data = common::create_test_y4m_data();
	let cursor = Cursor::new(y4m_data.clone());

	let mut reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let buf_writer = BufWriter::new(output_buffer);
	let mut writer = Y4mWriter::new(buf_writer, format.clone()).unwrap();

	let timebase = Timebase::new(format.framerate_den, format.framerate_num);
	let mut decoder = RawVideoDecoder::new(format);
	let mut encoder = RawVideoEncoder::new(timebase);

	let mut frame_count = 0;
	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			if let Some(pkt) = encoder.encode(frame).unwrap() {
				writer.write_packet(pkt).unwrap();
				frame_count += 1;
			}
		}
	}

	writer.finalize().unwrap();
	assert_eq!(frame_count, 3);
}

#[test]
fn test_stereo_wav_pipeline() {
	let wav_data = common::create_test_wav_stereo_data();
	let cursor = Cursor::new(wav_data);

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	assert_eq!(format.channels, 2);

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			if let Some(pkt) = encoder.encode(frame).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_y4m_aspect_ratio_preservation() {
	let y4m_data = common::create_test_y4m_no_colorspace();
	let cursor = Cursor::new(y4m_data);

	let mut reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	assert!(format.aspect_ratio.is_some());
	let aspect = format.aspect_ratio.unwrap();
	assert_eq!(aspect.num, 128);
	assert_eq!(aspect.den, 117);

	let output_buffer = Cursor::new(Vec::new());
	let buf_writer = BufWriter::new(output_buffer);
	let mut writer = Y4mWriter::new(buf_writer, format.clone()).unwrap();

	while let Some(packet) = reader.read_packet().unwrap() {
		writer.write_packet(packet).unwrap();
	}

	writer.finalize().unwrap();
}

#[test]
fn test_wav_format_properties() {
	let format = WavFormat { channels: 2, sample_rate: 48000, bit_depth: 16 };

	assert_eq!(format.bytes_per_sample(), 2);
	assert_eq!(format.bytes_per_frame(), 4);
}

#[test]
fn test_multiple_wav_files_pipeline() {
	for _ in 0..3 {
		let wav_data = common::create_test_wav_data();
		let cursor = Cursor::new(wav_data);

		let mut reader = WavReader::new(cursor).unwrap();
		let format = reader.format();

		let output_buffer = Cursor::new(Vec::new());
		let mut writer = WavWriter::new(output_buffer, format).unwrap();

		let mut decoder = PcmDecoder::new(format);
		let timebase = Timebase::new(1, format.sample_rate);
		let mut encoder = PcmEncoder::new(timebase);

		while let Some(packet) = reader.read_packet().unwrap() {
			if let Some(frame) = decoder.decode(packet).unwrap() {
				if let Some(pkt) = encoder.encode(frame).unwrap() {
					writer.write_packet(pkt).unwrap();
				}
			}
		}

		writer.finalize().unwrap();
	}
}
