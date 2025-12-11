use ffmpreg::codecs::{AdpcmDecoder, AdpcmEncoder};
use ffmpreg::container::WavFormat;
use ffmpreg::core::{Decoder, Encoder, Frame, Packet, Timebase};

fn create_mono_format() -> WavFormat {
	WavFormat { channels: 1, sample_rate: 44100, bit_depth: 16 }
}

fn create_stereo_format() -> WavFormat {
	WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 }
}

#[test]
fn test_adpcm_decoder_basic() {
	let format = create_mono_format();
	let mut decoder = AdpcmDecoder::new(format);

	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![0x00, 0x00, 0x00, 0x00], 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.channels, 1);
	assert_eq!(frame.sample_rate, 44100);
	assert_eq!(frame.nb_samples, 8);
}

#[test]
fn test_adpcm_decoder_output_size() {
	let format = create_mono_format();
	let mut decoder = AdpcmDecoder::new(format);

	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![0x12, 0x34], 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.data.len(), 8);
}

#[test]
fn test_adpcm_decoder_preserves_pts() {
	let format = create_mono_format();
	let mut decoder = AdpcmDecoder::new(format);

	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![0x00], 0, timebase).with_pts(5000);

	let frame = decoder.decode(packet).unwrap().unwrap();
	assert_eq!(frame.pts, 5000);
}

#[test]
fn test_adpcm_decoder_flush() {
	let format = create_mono_format();
	let mut decoder = AdpcmDecoder::new(format);

	let result = decoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_adpcm_encoder_basic() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = AdpcmEncoder::new(timebase, 1);

	let samples: Vec<i16> = vec![0, 100, 200, 300];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 4);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(packet.data.len(), 2);
}

#[test]
fn test_adpcm_encoder_preserves_pts() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = AdpcmEncoder::new(timebase, 1);

	let samples: Vec<i16> = vec![0, 0, 0, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 4).with_pts(7777);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.pts, 7777);
}

#[test]
fn test_adpcm_encoder_flush() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = AdpcmEncoder::new(timebase, 1);

	let result = encoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_adpcm_roundtrip() {
	let format = create_mono_format();
	let timebase = Timebase::new(1, 44100);

	let samples: Vec<i16> = vec![0, 50, 100, 150, 200, 150, 100, 50];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 8).with_pts(1000);

	let mut encoder = AdpcmEncoder::new(timebase, 1);
	let packet = encoder.encode(frame).unwrap().unwrap();

	let mut decoder = AdpcmDecoder::new(format);
	let decoded = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(decoded.nb_samples, 8);
	assert_eq!(decoded.pts, 1000);
}

#[test]
fn test_adpcm_compression_ratio() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = AdpcmEncoder::new(timebase, 1);

	let samples: Vec<i16> = (0..256).collect();
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data.clone(), timebase, 44100, 1, 256);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert!(packet.data.len() < data.len());
	assert_eq!(packet.data.len(), 128);
}

#[test]
fn test_adpcm_stereo() {
	let format = create_stereo_format();
	let timebase = Timebase::new(1, 44100);

	let mut decoder = AdpcmDecoder::new(format);
	let packet = Packet::new(vec![0x12, 0x34, 0x56, 0x78], 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.channels, 2);
}

#[test]
fn test_adpcm_multiple_packets() {
	let format = create_mono_format();
	let timebase = Timebase::new(1, 44100);

	let mut encoder = AdpcmEncoder::new(timebase, 1);
	let mut decoder = AdpcmDecoder::new(format);

	for i in 0..3 {
		let samples: Vec<i16> = (0..64).map(|x| (x + i * 100) as i16).collect();
		let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
		let frame = Frame::new(data, timebase, 44100, 1, 64).with_pts(i as i64 * 64);

		let packet = encoder.encode(frame).unwrap().unwrap();
		let decoded = decoder.decode(packet).unwrap().unwrap();

		assert_eq!(decoded.nb_samples, 64);
	}
}

#[test]
fn test_adpcm_signal_variation() {
	let format = create_mono_format();
	let timebase = Timebase::new(1, 44100);

	let samples: Vec<i16> = vec![0, 1000, -1000, 2000, -2000, 3000, -3000, 0];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 8);

	let mut encoder = AdpcmEncoder::new(timebase, 1);
	let packet = encoder.encode(frame).unwrap().unwrap();

	let mut decoder = AdpcmDecoder::new(format);
	let decoded = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(decoded.nb_samples, 8);
}
