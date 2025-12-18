use ffmpreg::codecs::{MsAdpcmDecoder, MsAdpcmEncoder};
use ffmpreg::container::WavFormat;
use ffmpreg::core::{Decoder, Encoder, Frame, Packet, Timebase};

fn create_mono_format() -> WavFormat {
	WavFormat { channels: 1, sample_rate: 44100, bit_depth: 16 }
}

fn create_stereo_format() -> WavFormat {
	WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 }
}

#[test]
fn test_ms_adpcm_encoder_basic() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = MsAdpcmEncoder::new(timebase, 1, 256);

	let samples: Vec<i16> = (0..256).map(|i| (i * 100) as i16).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 256);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(!packet.data.is_empty());
	assert!(packet.data.len() < 512);
}

#[test]
fn test_ms_adpcm_decoder_basic() {
	let format = create_mono_format();
	let mut decoder = MsAdpcmDecoder::new(format, 256);

	let mut block_data = Vec::new();
	block_data.push(0);
	block_data.extend_from_slice(&16i16.to_le_bytes());
	block_data.extend_from_slice(&0i16.to_le_bytes());
	block_data.extend_from_slice(&0i16.to_le_bytes());

	for _ in 0..10 {
		block_data.push(0x00);
	}

	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(block_data, 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.channels, 1);
	assert_eq!(frame.sample_rate, 44100);
}

#[test]
fn test_ms_adpcm_stereo_encoder() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = MsAdpcmEncoder::new(timebase, 2, 256);

	let mut samples: Vec<i16> = Vec::with_capacity(512);
	for i in 0..256 {
		samples.push((i * 50) as i16);
		samples.push((-i * 50) as i16);
	}
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 2, 256);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(!packet.data.is_empty());
}

#[test]
fn test_ms_adpcm_stereo_decoder() {
	let format = create_stereo_format();
	let mut decoder = MsAdpcmDecoder::new(format, 256);

	let mut block_data = Vec::new();
	block_data.push(0);
	block_data.push(0);
	block_data.extend_from_slice(&16i16.to_le_bytes());
	block_data.extend_from_slice(&16i16.to_le_bytes());
	block_data.extend_from_slice(&0i16.to_le_bytes());
	block_data.extend_from_slice(&0i16.to_le_bytes());
	block_data.extend_from_slice(&0i16.to_le_bytes());
	block_data.extend_from_slice(&0i16.to_le_bytes());

	for _ in 0..10 {
		block_data.push(0x00);
	}

	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(block_data, 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.channels, 2);
}

#[test]
fn test_ms_adpcm_roundtrip() {
	let format = create_mono_format();
	let timebase = Timebase::new(1, 44100);

	let samples: Vec<i16> = (0..64).map(|i| ((i as f32 * 0.1).sin() * 10000.0) as i16).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 64).with_pts(1000);

	let mut encoder = MsAdpcmEncoder::new(timebase, 1, 64);
	let packet = encoder.encode(frame).unwrap().unwrap();

	let mut decoder = MsAdpcmDecoder::new(format, 64);
	let decoded = decoder.decode(packet).unwrap().unwrap();

	assert!(decoded.nb_samples > 0);
	assert_eq!(decoded.pts, 1000);
}

#[test]
fn test_ms_adpcm_preserves_pts() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = MsAdpcmEncoder::new(timebase, 1, 256);

	let samples: Vec<i16> = vec![0; 256];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 256).with_pts(9999);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.pts, 9999);
}

#[test]
fn test_ms_adpcm_encoder_flush() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = MsAdpcmEncoder::new(timebase, 1, 256);

	let result = encoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_ms_adpcm_decoder_flush() {
	let format = create_mono_format();
	let mut decoder = MsAdpcmDecoder::new(format, 256);

	let result = decoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_ms_adpcm_compression() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = MsAdpcmEncoder::new(timebase, 1, 1024);

	let samples: Vec<i16> = (0..1024).map(|i| (i * 10) as i16).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let original_size = data.len();
	let frame = Frame::new(data, timebase, 44100, 1, 1024);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(packet.data.len() < original_size);
}
