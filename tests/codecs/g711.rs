use ffmpreg::codecs::{AlawDecoder, AlawEncoder, UlawDecoder, UlawEncoder};
use ffmpreg::container::WavFormat;
use ffmpreg::core::{Decoder, Encoder, Frame, Packet, Timebase};

fn create_mono_format() -> WavFormat {
	WavFormat { channels: 1, sample_rate: 8000, bit_depth: 16 }
}

fn create_stereo_format() -> WavFormat {
	WavFormat { channels: 2, sample_rate: 8000, bit_depth: 16 }
}

#[test]
fn test_ulaw_encoder_basic() {
	let timebase = Timebase::new(1, 8000);
	let mut encoder = UlawEncoder::new(timebase, 1);

	let samples: Vec<i16> = vec![0, 1000, -1000, 5000, -5000, 10000, -10000, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 8000, 1, 8);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(packet.data.len(), 8);
}

#[test]
fn test_ulaw_decoder_basic() {
	let format = create_mono_format();
	let mut decoder = UlawDecoder::new(format);

	let timebase = Timebase::new(1, 8000);
	let packet = Packet::new(vec![0xFF, 0x00, 0x80, 0x7F], 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.channels, 1);
	assert_eq!(frame.sample_rate, 8000);
	assert_eq!(frame.data.len(), 8);
}

#[test]
fn test_ulaw_roundtrip() {
	let format = create_mono_format();
	let timebase = Timebase::new(1, 8000);

	let samples: Vec<i16> = vec![0, 500, 1000, 2000, 4000, 8000, 16000, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 8000, 1, 8).with_pts(100);

	let mut encoder = UlawEncoder::new(timebase, 1);
	let packet = encoder.encode(frame).unwrap().unwrap();

	let mut decoder = UlawDecoder::new(format);
	let decoded = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(decoded.nb_samples, 8);
	assert_eq!(decoded.pts, 100);
}

#[test]
fn test_ulaw_compression_ratio() {
	let timebase = Timebase::new(1, 8000);
	let mut encoder = UlawEncoder::new(timebase, 1);

	let samples: Vec<i16> = (0..256).map(|i| (i * 100) as i16).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data.clone(), timebase, 8000, 1, 256);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(packet.data.len(), 256);
	assert_eq!(data.len(), 512);
}

#[test]
fn test_alaw_encoder_basic() {
	let timebase = Timebase::new(1, 8000);
	let mut encoder = AlawEncoder::new(timebase, 1);

	let samples: Vec<i16> = vec![0, 1000, -1000, 5000, -5000, 10000, -10000, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 8000, 1, 8);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(packet.data.len(), 8);
}

#[test]
fn test_alaw_decoder_basic() {
	let format = create_mono_format();
	let mut decoder = AlawDecoder::new(format);

	let timebase = Timebase::new(1, 8000);
	let packet = Packet::new(vec![0xD5, 0x55, 0x80, 0x00], 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.channels, 1);
	assert_eq!(frame.sample_rate, 8000);
	assert_eq!(frame.data.len(), 8);
}

#[test]
fn test_alaw_roundtrip() {
	let format = create_mono_format();
	let timebase = Timebase::new(1, 8000);

	let samples: Vec<i16> = vec![0, 500, 1000, 2000, 4000, 8000, 16000, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 8000, 1, 8).with_pts(200);

	let mut encoder = AlawEncoder::new(timebase, 1);
	let packet = encoder.encode(frame).unwrap().unwrap();

	let mut decoder = AlawDecoder::new(format);
	let decoded = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(decoded.nb_samples, 8);
	assert_eq!(decoded.pts, 200);
}

#[test]
fn test_alaw_stereo() {
	let format = create_stereo_format();
	let timebase = Timebase::new(1, 8000);

	let mut decoder = AlawDecoder::new(format);
	let packet = Packet::new(vec![0xD5, 0x55, 0x80, 0x00, 0xD5, 0x55, 0x80, 0x00], 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.channels, 2);
	assert_eq!(frame.nb_samples, 4);
}

#[test]
fn test_ulaw_preserves_pts() {
	let timebase = Timebase::new(1, 8000);
	let mut encoder = UlawEncoder::new(timebase, 1);

	let samples: Vec<i16> = vec![0, 0, 0, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 8000, 1, 4).with_pts(12345);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.pts, 12345);
}

#[test]
fn test_alaw_preserves_pts() {
	let timebase = Timebase::new(1, 8000);
	let mut encoder = AlawEncoder::new(timebase, 1);

	let samples: Vec<i16> = vec![0, 0, 0, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 8000, 1, 4).with_pts(54321);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.pts, 54321);
}

#[test]
fn test_ulaw_flush() {
	let timebase = Timebase::new(1, 8000);
	let mut encoder = UlawEncoder::new(timebase, 1);

	let result = encoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_alaw_flush() {
	let timebase = Timebase::new(1, 8000);
	let mut encoder = AlawEncoder::new(timebase, 1);

	let result = encoder.flush().unwrap();
	assert!(result.is_none());
}
