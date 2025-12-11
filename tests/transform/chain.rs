use ffmpreg::core::{Frame, Timebase, Transform};
use ffmpreg::transform::{Gain, Normalize, TransformChain, parse_transform};

fn create_test_frame(samples: Vec<i16>) -> Frame {
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let timebase = Timebase::new(1, 44100);
	Frame::new(data, timebase, 44100, 1, samples.len())
}

fn extract_samples(frame: &Frame) -> Vec<i16> {
	frame.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

#[test]
fn test_chain_empty() {
	let mut chain = TransformChain::new();
	assert!(chain.is_empty());

	let frame = create_test_frame(vec![100, 200, 300]);
	let result = chain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![100, 200, 300]);
}

#[test]
fn test_chain_single_transform() {
	let mut chain = TransformChain::new();
	chain.add(Box::new(Gain::new(2.0)));

	assert!(!chain.is_empty());

	let frame = create_test_frame(vec![100, 200]);
	let result = chain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![200, 400]);
}

#[test]
fn test_chain_multiple_transforms() {
	let mut chain = TransformChain::new();
	chain.add(Box::new(Gain::new(2.0)));
	chain.add(Box::new(Gain::new(2.0)));

	let frame = create_test_frame(vec![100]);
	let result = chain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert_eq!(output, vec![400]);
}

#[test]
fn test_chain_gain_then_normalize() {
	let mut chain = TransformChain::new();
	chain.add(Box::new(Gain::new(0.5)));
	chain.add(Box::new(Normalize::new(1.0)));

	let frame = create_test_frame(vec![1000, -1000]);
	let result = chain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert!(output[0] >= 32766);
	assert!(output[1] <= -32766);
}

#[test]
fn test_chain_normalize_then_gain() {
	let mut chain = TransformChain::new();
	chain.add(Box::new(Normalize::new(0.5)));
	chain.add(Box::new(Gain::new(2.0)));

	let frame = create_test_frame(vec![1000, -1000]);
	let result = chain.apply(frame).unwrap();
	let output = extract_samples(&result);

	assert!(output[0] >= 32000);
}

#[test]
fn test_chain_name() {
	let chain = TransformChain::new();
	assert_eq!(chain.name(), "chain");
}

#[test]
fn test_chain_preserves_metadata() {
	let mut chain = TransformChain::new();
	chain.add(Box::new(Gain::new(1.0)));

	let samples: Vec<i16> = vec![100, 200];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let timebase = Timebase::new(1, 48000);
	let frame = Frame::new(data, timebase, 48000, 2, 1).with_pts(5555);

	let result = chain.apply(frame).unwrap();

	assert_eq!(result.pts, 5555);
	assert_eq!(result.sample_rate, 48000);
	assert_eq!(result.channels, 2);
}

#[test]
fn test_parse_transform_gain() {
	let transform = parse_transform("gain=2.0").unwrap();
	assert_eq!(transform.name(), "gain");
}

#[test]
fn test_parse_transform_gain_integer() {
	let transform = parse_transform("gain=3").unwrap();
	assert_eq!(transform.name(), "gain");
}

#[test]
fn test_parse_transform_normalize() {
	let transform = parse_transform("normalize").unwrap();
	assert_eq!(transform.name(), "normalize");
}

#[test]
fn test_parse_transform_normalize_with_value() {
	let transform = parse_transform("normalize=0.8").unwrap();
	assert_eq!(transform.name(), "normalize");
}

#[test]
fn test_parse_transform_unknown() {
	let result = parse_transform("unknown_filter");
	assert!(result.is_err());
}

#[test]
fn test_parse_transform_gain_missing_value() {
	let result = parse_transform("gain");
	assert!(result.is_err());
}

#[test]
fn test_parse_transform_gain_invalid_value() {
	let result = parse_transform("gain=abc");
	assert!(result.is_err());
}

#[test]
fn test_chain_default() {
	let chain = TransformChain::default();
	assert!(chain.is_empty());
}

#[test]
fn test_chain_three_transforms() {
	let mut chain = TransformChain::new();
	chain.add(Box::new(Gain::new(0.5)));
	chain.add(Box::new(Normalize::new(1.0)));
	chain.add(Box::new(Gain::new(0.5)));

	let frame = create_test_frame(vec![1000, -1000]);
	let result = chain.apply(frame).unwrap();
	let output = extract_samples(&result);

	let max_abs = output.iter().map(|s| s.abs()).max().unwrap();
	assert!(max_abs > 16000 && max_abs < 16400);
}
