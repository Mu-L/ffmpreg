use ffmpreg::codecs::{PcmDecoder, PcmEncoder};
use ffmpreg::container::WavFormat;
use ffmpreg::core::{Decoder, Encoder, Packet, Timebase};

fn create_test_format() -> WavFormat {
	WavFormat { channels: 1, sample_rate: 44100, bit_depth: 16 }
}

fn create_stereo_format() -> WavFormat {
	WavFormat { channels: 2, sample_rate: 48000, bit_depth: 16 }
}

#[test]
fn test_pcm_decoder_basic() {
	let format = create_test_format();
	let mut decoder = PcmDecoder::new(format);

	let timebase = Timebase::new(1, 44100);
	let data = vec![0u8; 1024];
	let packet = Packet::new(data, 0, timebase).with_pts(0);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.sample_rate, 44100);
	assert_eq!(frame.channels, 1);
	assert_eq!(frame.nb_samples, 512);
}

#[test]
fn test_pcm_decoder_stereo() {
	let format = create_stereo_format();
	let mut decoder = PcmDecoder::new(format);

	let timebase = Timebase::new(1, 48000);
	let data = vec![0u8; 2048];
	let packet = Packet::new(data, 0, timebase).with_pts(0);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.sample_rate, 48000);
	assert_eq!(frame.channels, 2);
	assert_eq!(frame.nb_samples, 512);
}

#[test]
fn test_pcm_decoder_preserves_pts() {
	let format = create_test_format();
	let mut decoder = PcmDecoder::new(format);

	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![0u8; 512], 0, timebase).with_pts(12345);

	let frame = decoder.decode(packet).unwrap().unwrap();
	assert_eq!(frame.pts, 12345);
}

#[test]
fn test_pcm_decoder_preserves_data() {
	let format = create_test_format();
	let mut decoder = PcmDecoder::new(format);

	let timebase = Timebase::new(1, 44100);
	let original_data = vec![1, 2, 3, 4, 5, 6, 7, 8];
	let packet = Packet::new(original_data.clone(), 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();
	assert_eq!(frame.data, original_data);
}

#[test]
fn test_pcm_decoder_flush() {
	let format = create_test_format();
	let mut decoder = PcmDecoder::new(format);

	let result = decoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_pcm_encoder_basic() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = PcmEncoder::new(timebase);

	let frame = ffmpreg::core::Frame::new(vec![0u8; 1024], timebase, 44100, 1, 512).with_pts(0);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(packet.size(), 1024);
	assert_eq!(packet.pts, 0);
}

#[test]
fn test_pcm_encoder_preserves_pts() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = PcmEncoder::new(timebase);

	let frame = ffmpreg::core::Frame::new(vec![0u8; 256], timebase, 44100, 1, 128).with_pts(9999);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.pts, 9999);
}

#[test]
fn test_pcm_encoder_preserves_data() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = PcmEncoder::new(timebase);

	let original_data = vec![10, 20, 30, 40, 50, 60, 70, 80];
	let frame = ffmpreg::core::Frame::new(original_data.clone(), timebase, 44100, 1, 4);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.data, original_data);
}

#[test]
fn test_pcm_encoder_flush() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = PcmEncoder::new(timebase);

	let result = encoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_pcm_roundtrip() {
	let format = create_test_format();
	let timebase = Timebase::new(1, 44100);

	let mut decoder = PcmDecoder::new(format);
	let mut encoder = PcmEncoder::new(timebase);

	let samples: Vec<i16> = vec![100, -200, 300, -400, 500, -600, 700, -800];
	let original_data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();

	let packet = Packet::new(original_data.clone(), 0, timebase).with_pts(1000);
	let frame = decoder.decode(packet).unwrap().unwrap();
	let output_packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(output_packet.data, original_data);
	assert_eq!(output_packet.pts, 1000);
}

#[test]
fn test_pcm_multiple_packets() {
	let format = create_test_format();
	let timebase = Timebase::new(1, 44100);

	let mut decoder = PcmDecoder::new(format);
	let mut encoder = PcmEncoder::new(timebase);

	for i in 0..5 {
		let data = vec![(i * 10) as u8; 256];
		let packet = Packet::new(data.clone(), 0, timebase).with_pts(i as i64 * 128);

		let frame = decoder.decode(packet).unwrap().unwrap();
		let output = encoder.encode(frame).unwrap().unwrap();

		assert_eq!(output.data, data);
		assert_eq!(output.pts, i as i64 * 128);
	}
}
