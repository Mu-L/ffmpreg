use ffmpreg::codecs::{FlacDecoder, FlacEncoder};
use ffmpreg::container::FlacFormat;
use ffmpreg::core::{Decoder, Encoder, Frame, Timebase};

fn create_default_format() -> FlacFormat {
	FlacFormat {
		min_block_size: 4096,
		max_block_size: 4096,
		min_frame_size: 0,
		max_frame_size: 0,
		sample_rate: 44100,
		channels: 2,
		bits_per_sample: 16,
		total_samples: 0,
		md5_signature: [0u8; 16],
	}
}

#[test]
fn test_flac_encoder_basic() {
	let mut encoder = FlacEncoder::new(44100, 2, 16, 4096);
	let timebase = Timebase::new(1, 44100);

	let mut samples: Vec<i16> = Vec::with_capacity(8192);
	for i in 0..4096 {
		samples.push((i * 10) as i16);
		samples.push((-i * 10) as i16);
	}
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 2, 4096);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(!packet.data.is_empty());
}

#[test]
fn test_flac_encoder_mono() {
	let mut encoder = FlacEncoder::new(44100, 1, 16, 1024);
	let timebase = Timebase::new(1, 44100);

	let samples: Vec<i16> = (0..1024).map(|i| (i * 10) as i16).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 1024);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(!packet.data.is_empty());
}

#[test]
fn test_flac_encoder_preserves_pts() {
	let mut encoder = FlacEncoder::new(44100, 1, 16, 256);
	let timebase = Timebase::new(1, 44100);

	let samples: Vec<i16> = vec![0; 256];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 256).with_pts(12345);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.pts, 12345);
}

#[test]
fn test_flac_encoder_flush() {
	let mut encoder = FlacEncoder::new(44100, 2, 16, 4096);

	let result = encoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_flac_decoder_new() {
	let format = create_default_format();
	let _decoder = FlacDecoder::new(&format);
}

#[test]
fn test_flac_decoder_flush() {
	let format = create_default_format();
	let mut decoder = FlacDecoder::new(&format);

	let result = decoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_flac_encoder_different_sample_rates() {
	for &sample_rate in &[8000u32, 16000, 22050, 44100, 48000, 96000] {
		let mut encoder = FlacEncoder::new(sample_rate, 1, 16, 256);
		let timebase = Timebase::new(1, sample_rate);

		let samples: Vec<i16> = (0..256).map(|i| i as i16).collect();
		let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
		let frame = Frame::new(data, timebase, sample_rate, 1, 256);

		let packet = encoder.encode(frame).unwrap().unwrap();
		assert!(!packet.data.is_empty());
	}
}

#[test]
fn test_flac_encoder_different_bit_depths() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = FlacEncoder::new(44100, 1, 16, 256);

	let samples: Vec<i16> = (0..256).map(|i| (i * 128) as i16).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 256);

	let result = encoder.encode(frame);
	assert!(result.is_ok());
}

#[test]
fn test_flac_encoder_silent_audio() {
	let mut encoder = FlacEncoder::new(44100, 2, 16, 1024);
	let timebase = Timebase::new(1, 44100);

	let samples: Vec<i16> = vec![0; 2048];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 2, 1024);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(!packet.data.is_empty());
}

#[test]
fn test_flac_encoder_varying_signal() {
	let mut encoder = FlacEncoder::new(44100, 1, 16, 512);
	let timebase = Timebase::new(1, 44100);

	let samples: Vec<i16> = (0..512).map(|i| ((i as f32 * 0.1).sin() * 20000.0) as i16).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 512);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(!packet.data.is_empty());
}
