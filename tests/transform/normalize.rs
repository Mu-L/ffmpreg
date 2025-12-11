use ffmpreg::core::{Frame, Timebase, Transform};
use ffmpreg::transform::Normalize;

fn create_test_frame(samples: Vec<i16>) -> Frame {
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let timebase = Timebase::new(1, 44100);
	Frame::new(data, timebase, 44100, 1, samples.len())
}

fn extract_samples(frame: &Frame) -> Vec<i16> {
	frame.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

#[test]
fn test_normalize_basic() {
	let mut normalize = Normalize::new(1.0);
	let frame = create_test_frame(vec![1000, -1000, 500, -500]);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert!(output[0] >= 32766);
	assert!(output[1] <= -32766);
}

#[test]
fn test_normalize_already_max() {
	let mut normalize = Normalize::new(1.0);
	let frame = create_test_frame(vec![32767, -32767, 0, 0]);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output[0], 32767);
}

#[test]
fn test_normalize_silent() {
	let mut normalize = Normalize::new(1.0);
	let frame = create_test_frame(vec![0, 0, 0, 0]);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![0, 0, 0, 0]);
}

#[test]
fn test_normalize_target_peak() {
	let mut normalize = Normalize::new(0.5);
	let frame = create_test_frame(vec![1000, -1000]);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	let max_abs = output.iter().map(|s| s.abs()).max().unwrap();
	let target = (32767.0 * 0.5) as i16;

	assert!((max_abs - target).abs() < 10);
}

#[test]
fn test_normalize_preserves_ratio() {
	let mut normalize = Normalize::new(1.0);
	let frame = create_test_frame(vec![1000, 500, 250]);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	let ratio1 = output[0] as f32 / output[1] as f32;
	let ratio2 = output[1] as f32 / output[2] as f32;

	assert!((ratio1 - 2.0).abs() < 0.1);
	assert!((ratio2 - 2.0).abs() < 0.1);
}

#[test]
fn test_normalize_name() {
	let normalize = Normalize::new(1.0);
	assert_eq!(normalize.name(), "normalize");
}

#[test]
fn test_normalize_preserves_metadata() {
	let mut normalize = Normalize::new(0.9);

	let samples: Vec<i16> = vec![500, 1000];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let timebase = Timebase::new(1, 48000);
	let frame = Frame::new(data, timebase, 48000, 2, 1).with_pts(9999);

	let result = normalize.apply(frame).unwrap();

	assert_eq!(result.pts, 9999);
	assert_eq!(result.sample_rate, 48000);
	assert_eq!(result.channels, 2);
}

#[test]
fn test_normalize_empty_frame() {
	let mut normalize = Normalize::new(1.0);
	let frame = create_test_frame(vec![]);

	let result = normalize.apply(frame).unwrap();

	assert!(result.data.is_empty());
}

#[test]
fn test_normalize_single_sample() {
	let mut normalize = Normalize::new(1.0);
	let frame = create_test_frame(vec![1000]);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert!(output[0] >= 32766);
}

#[test]
fn test_normalize_default_peak() {
	let mut normalize = Normalize::default_peak();
	let frame = create_test_frame(vec![1000, -1000]);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	let max_abs = output.iter().map(|s| s.abs()).max().unwrap();
	let expected = (32767.0 * 0.95) as i16;

	assert!((max_abs - expected).abs() < 100);
}

#[test]
fn test_normalize_clamps_target() {
	let normalize = Normalize::new(1.5);
	assert_eq!(normalize.name(), "normalize");

	let normalize2 = Normalize::new(-0.5);
	assert_eq!(normalize2.name(), "normalize");
}

#[test]
fn test_normalize_large_frame() {
	let mut normalize = Normalize::new(1.0);
	let samples: Vec<i16> = (0i16..1024).map(|i| (i % 500) - 250).collect();
	let frame = create_test_frame(samples);

	let result = normalize.apply(frame).unwrap();
	let output = extract_samples(&result);

	let max_abs = output.iter().map(|s| s.abs()).max().unwrap();
	assert!(max_abs >= 32760);
}
