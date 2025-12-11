use ffmpreg::codecs::{RawVideoDecoder, RawVideoEncoder};
use ffmpreg::container::Y4mFormat;
use ffmpreg::core::{Decoder, Encoder, Frame, Packet, Timebase};

fn create_test_format() -> Y4mFormat {
	Y4mFormat {
		width: 16,
		height: 16,
		framerate_num: 30,
		framerate_den: 1,
		colorspace: None,
		interlacing: ffmpreg::container::y4m::Interlacing::Progressive,
		aspect_ratio: None,
	}
}

#[test]
fn test_rawvideo_decoder_basic() {
	let format = create_test_format();
	let frame_size = format.frame_size();
	let mut decoder = RawVideoDecoder::new(format);

	let timebase = Timebase::new(1, 30);
	let data = vec![128u8; frame_size];
	let packet = Packet::new(data, 0, timebase).with_pts(0);

	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.data.len(), frame_size);
	assert_eq!(frame.pts, 0);
}

#[test]
fn test_rawvideo_decoder_preserves_data() {
	let format = create_test_format();
	let frame_size = format.frame_size();
	let mut decoder = RawVideoDecoder::new(format);

	let timebase = Timebase::new(1, 30);
	let original_data: Vec<u8> = (0..frame_size).map(|i| (i % 256) as u8).collect();
	let packet = Packet::new(original_data.clone(), 0, timebase);

	let frame = decoder.decode(packet).unwrap().unwrap();
	assert_eq!(frame.data, original_data);
}

#[test]
fn test_rawvideo_decoder_preserves_pts() {
	let format = create_test_format();
	let frame_size = format.frame_size();
	let mut decoder = RawVideoDecoder::new(format);

	let timebase = Timebase::new(1, 30);
	let data = vec![0u8; frame_size];
	let packet = Packet::new(data, 0, timebase).with_pts(12345);

	let frame = decoder.decode(packet).unwrap().unwrap();
	assert_eq!(frame.pts, 12345);
}

#[test]
fn test_rawvideo_decoder_flush() {
	let format = create_test_format();
	let mut decoder = RawVideoDecoder::new(format);

	let result = decoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_rawvideo_encoder_basic() {
	let timebase = Timebase::new(1, 30);
	let mut encoder = RawVideoEncoder::new(timebase);

	let data = vec![128u8; 384];
	let frame = Frame::new(data, timebase, 30, 1, 1).with_pts(0);

	let packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(packet.data.len(), 384);
	assert_eq!(packet.pts, 0);
}

#[test]
fn test_rawvideo_encoder_preserves_data() {
	let timebase = Timebase::new(1, 30);
	let mut encoder = RawVideoEncoder::new(timebase);

	let original_data: Vec<u8> = (0..256).map(|i| i as u8).collect();
	let frame = Frame::new(original_data.clone(), timebase, 30, 1, 1);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.data, original_data);
}

#[test]
fn test_rawvideo_encoder_preserves_pts() {
	let timebase = Timebase::new(1, 30);
	let mut encoder = RawVideoEncoder::new(timebase);

	let data = vec![0u8; 100];
	let frame = Frame::new(data, timebase, 30, 1, 1).with_pts(9999);

	let packet = encoder.encode(frame).unwrap().unwrap();
	assert_eq!(packet.pts, 9999);
}

#[test]
fn test_rawvideo_encoder_flush() {
	let timebase = Timebase::new(1, 30);
	let mut encoder = RawVideoEncoder::new(timebase);

	let result = encoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_rawvideo_roundtrip() {
	let format = create_test_format();
	let timebase = Timebase::new(1, 30);

	let mut decoder = RawVideoDecoder::new(format.clone());
	let mut encoder = RawVideoEncoder::new(timebase);

	let frame_size = format.frame_size();
	let original_data: Vec<u8> = (0..frame_size).map(|i| (i * 3 % 256) as u8).collect();

	let packet = Packet::new(original_data.clone(), 0, timebase).with_pts(100);
	let frame = decoder.decode(packet).unwrap().unwrap();
	let output_packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(output_packet.data, original_data);
	assert_eq!(output_packet.pts, 100);
}

#[test]
fn test_rawvideo_multiple_frames() {
	let format = create_test_format();
	let timebase = Timebase::new(1, 30);

	let mut decoder = RawVideoDecoder::new(format.clone());
	let mut encoder = RawVideoEncoder::new(timebase);

	let frame_size = format.frame_size();

	for i in 0..5 {
		let data: Vec<u8> = (0..frame_size).map(|j| ((i + j) % 256) as u8).collect();
		let packet = Packet::new(data.clone(), 0, timebase).with_pts(i as i64);

		let frame = decoder.decode(packet).unwrap().unwrap();
		let output = encoder.encode(frame).unwrap().unwrap();

		assert_eq!(output.data, data);
		assert_eq!(output.pts, i as i64);
	}
}

#[test]
fn test_rawvideo_frame_size_c420() {
	let format = Y4mFormat {
		width: 8,
		height: 8,
		framerate_num: 30,
		framerate_den: 1,
		colorspace: Some(ffmpreg::container::y4m::Colorspace::C420),
		interlacing: ffmpreg::container::y4m::Interlacing::Progressive,
		aspect_ratio: None,
	};

	let luma = 8 * 8;
	let chroma = luma / 4 * 2;
	assert_eq!(format.frame_size(), luma + chroma);
}
