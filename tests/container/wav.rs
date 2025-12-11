use ffmpreg::container::{WavFormat, WavReader, WavWriter};
use ffmpreg::core::{Demuxer, Muxer, Packet, Timebase};
use std::io::Cursor;

fn create_test_wav() -> Vec<u8> {
	let sample_rate: u32 = 44100;
	let channels: u16 = 1;
	let bits_per_sample: u16 = 16;
	let num_samples: u32 = 1024;

	let data_size = num_samples * (bits_per_sample as u32 / 8) * channels as u32;
	let file_size = 36 + data_size;

	let mut wav = Vec::new();

	wav.extend_from_slice(b"RIFF");
	wav.extend_from_slice(&file_size.to_le_bytes());
	wav.extend_from_slice(b"WAVE");

	wav.extend_from_slice(b"fmt ");
	wav.extend_from_slice(&16u32.to_le_bytes());
	wav.extend_from_slice(&1u16.to_le_bytes());
	wav.extend_from_slice(&channels.to_le_bytes());
	wav.extend_from_slice(&sample_rate.to_le_bytes());
	let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
	wav.extend_from_slice(&byte_rate.to_le_bytes());
	let block_align = channels * bits_per_sample / 8;
	wav.extend_from_slice(&block_align.to_le_bytes());
	wav.extend_from_slice(&bits_per_sample.to_le_bytes());

	wav.extend_from_slice(b"data");
	wav.extend_from_slice(&data_size.to_le_bytes());

	for i in 0..num_samples {
		let sample = ((i as f32 / num_samples as f32) * 16000.0) as i16;
		wav.extend_from_slice(&sample.to_le_bytes());
	}

	wav
}

fn create_stereo_wav() -> Vec<u8> {
	let sample_rate: u32 = 48000;
	let channels: u16 = 2;
	let bits_per_sample: u16 = 16;
	let num_samples: u32 = 256;

	let data_size = num_samples * (bits_per_sample as u32 / 8) * channels as u32;
	let file_size = 36 + data_size;

	let mut wav = Vec::new();

	wav.extend_from_slice(b"RIFF");
	wav.extend_from_slice(&file_size.to_le_bytes());
	wav.extend_from_slice(b"WAVE");

	wav.extend_from_slice(b"fmt ");
	wav.extend_from_slice(&16u32.to_le_bytes());
	wav.extend_from_slice(&1u16.to_le_bytes());
	wav.extend_from_slice(&channels.to_le_bytes());
	wav.extend_from_slice(&sample_rate.to_le_bytes());
	let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
	wav.extend_from_slice(&byte_rate.to_le_bytes());
	let block_align = channels * bits_per_sample / 8;
	wav.extend_from_slice(&block_align.to_le_bytes());
	wav.extend_from_slice(&bits_per_sample.to_le_bytes());

	wav.extend_from_slice(b"data");
	wav.extend_from_slice(&data_size.to_le_bytes());

	for i in 0..num_samples {
		let left = (i as i16) * 100;
		let right = -(i as i16) * 100;
		wav.extend_from_slice(&left.to_le_bytes());
		wav.extend_from_slice(&right.to_le_bytes());
	}

	wav
}

#[test]
fn test_wav_reader_format() {
	let wav_data = create_test_wav();
	let cursor = Cursor::new(wav_data);
	let reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	assert_eq!(format.sample_rate, 44100);
	assert_eq!(format.channels, 1);
	assert_eq!(format.bit_depth, 16);
}

#[test]
fn test_wav_reader_stereo() {
	let wav_data = create_stereo_wav();
	let cursor = Cursor::new(wav_data);
	let reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	assert_eq!(format.sample_rate, 48000);
	assert_eq!(format.channels, 2);
}

#[test]
fn test_wav_reader_read_packets() {
	let wav_data = create_test_wav();
	let cursor = Cursor::new(wav_data);
	let mut reader = WavReader::new(cursor).unwrap();

	let mut packet_count = 0;
	let mut total_bytes = 0;

	while let Some(packet) = reader.read_packet().unwrap() {
		packet_count += 1;
		total_bytes += packet.size();
	}

	assert!(packet_count > 0);
	assert_eq!(total_bytes, 2048);
}

#[test]
fn test_wav_reader_stream_count() {
	let wav_data = create_test_wav();
	let cursor = Cursor::new(wav_data);
	let reader = WavReader::new(cursor).unwrap();

	assert_eq!(reader.stream_count(), 1);
}

#[test]
fn test_wav_format_bytes_per_sample() {
	let format = WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 };

	assert_eq!(format.bytes_per_sample(), 2);
	assert_eq!(format.bytes_per_frame(), 4);
}

#[test]
fn test_wav_writer_basic() {
	let format = WavFormat { channels: 1, sample_rate: 44100, bit_depth: 16 };

	let buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(buffer, format).unwrap();

	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![0u8; 1024], 0, timebase);
	writer.write_packet(packet).unwrap();
	writer.finalize().unwrap();
}

#[test]
fn test_wav_roundtrip() {
	let original_wav = create_test_wav();
	let cursor = Cursor::new(original_wav.clone());
	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	while let Some(packet) = reader.read_packet().unwrap() {
		writer.write_packet(packet).unwrap();
	}

	writer.finalize().unwrap();
}

#[test]
fn test_wav_invalid_file() {
	let invalid_data = b"NOT A WAV FILE".to_vec();
	let cursor = Cursor::new(invalid_data);
	let result = WavReader::new(cursor);

	assert!(result.is_err());
}

#[test]
fn test_wav_pts_increments() {
	let wav_data = create_test_wav();
	let cursor = Cursor::new(wav_data);
	let mut reader = WavReader::new(cursor).unwrap();

	let mut last_pts: Option<i64> = None;

	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(prev) = last_pts {
			assert!(packet.pts >= prev);
		}
		last_pts = Some(packet.pts);
	}
}
