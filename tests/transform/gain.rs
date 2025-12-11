use ffmpreg::core::{Frame, Timebase, Transform};
use ffmpreg::transform::Gain;

fn create_test_frame(samples: Vec<i16>) -> Frame {
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let timebase = Timebase::new(1, 44100);
	Frame::new(data, timebase, 44100, 1, samples.len())
}

fn extract_samples(frame: &Frame) -> Vec<i16> {
	frame.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

#[test]
fn test_gain_unity() {
	let mut gain = Gain::new(1.0);
	let frame = create_test_frame(vec![100, 200, 300, 400]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![100, 200, 300, 400]);
}

#[test]
fn test_gain_double() {
	let mut gain = Gain::new(2.0);
	let frame = create_test_frame(vec![100, 200, 300, 400]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![200, 400, 600, 800]);
}

#[test]
fn test_gain_half() {
	let mut gain = Gain::new(0.5);
	let frame = create_test_frame(vec![100, 200, 300, 400]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![50, 100, 150, 200]);
}

#[test]
fn test_gain_negative_samples() {
	let mut gain = Gain::new(2.0);
	let frame = create_test_frame(vec![-100, -200, -300, -400]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![-200, -400, -600, -800]);
}

#[test]
fn test_gain_clipping_positive() {
	let mut gain = Gain::new(10.0);
	let frame = create_test_frame(vec![10000, 20000]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output[0], 32767);
	assert_eq!(output[1], 32767);
}

#[test]
fn test_gain_clipping_negative() {
	let mut gain = Gain::new(10.0);
	let frame = create_test_frame(vec![-10000, -20000]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output[0], -32768);
	assert_eq!(output[1], -32768);
}

#[test]
fn test_gain_zero() {
	let mut gain = Gain::new(0.0);
	let frame = create_test_frame(vec![100, 200, 300, 400]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![0, 0, 0, 0]);
}

#[test]
fn test_gain_name() {
	let gain = Gain::new(1.0);
	assert_eq!(gain.name(), "gain");
}

#[test]
fn test_gain_preserves_metadata() {
	let mut gain = Gain::new(1.5);

	let samples: Vec<i16> = vec![100, 200];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let timebase = Timebase::new(1, 48000);
	let frame = Frame::new(data, timebase, 48000, 2, 1).with_pts(12345);

	let result = gain.apply(frame).unwrap();

	assert_eq!(result.pts, 12345);
	assert_eq!(result.sample_rate, 48000);
	assert_eq!(result.channels, 2);
}

#[test]
fn test_gain_empty_frame() {
	let mut gain = Gain::new(2.0);
	let frame = create_test_frame(vec![]);

	let result = gain.apply(frame).unwrap();

	assert!(result.data.is_empty());
}

#[test]
fn test_gain_large_frame() {
	let mut gain = Gain::new(1.5);
	let samples: Vec<i16> = (0i16..1024).map(|i| i % 1000).collect();
	let frame = create_test_frame(samples);

	let result = gain.apply(frame).unwrap();

	assert_eq!(result.data.len(), 2048);
}

#[test]
fn test_gain_fractional() {
	let mut gain = Gain::new(1.25);
	let frame = create_test_frame(vec![1000]);

	let result = gain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output[0], 1250);
}
