use ffmpreg::core::{Frame, Timebase};

#[test]
fn test_frame_creation() {
	let data = vec![0u8; 1024];
	let timebase = Timebase::new(1, 44100);
	let frame = Frame::new(data.clone(), timebase, 44100, 2, 256);

	assert_eq!(frame.sample_rate, 44100);
	assert_eq!(frame.channels, 2);
	assert_eq!(frame.nb_samples, 256);
	assert_eq!(frame.size(), 1024);
}

#[test]
fn test_frame_with_pts() {
	let data = vec![0u8; 512];
	let timebase = Timebase::new(1, 48000);
	let frame = Frame::new(data, timebase, 48000, 1, 256).with_pts(1024);

	assert_eq!(frame.pts, 1024);
}

#[test]
fn test_frame_empty() {
	let data = vec![];
	let timebase = Timebase::new(1, 44100);
	let frame = Frame::new(data, timebase, 44100, 1, 0);

	assert!(frame.is_empty());
	assert_eq!(frame.size(), 0);
}

#[test]
fn test_frame_clone() {
	let data = vec![1, 2, 3, 4];
	let timebase = Timebase::new(1, 44100);
	let frame = Frame::new(data, timebase, 44100, 1, 2).with_pts(100);
	let cloned = frame.clone();

	assert_eq!(cloned.pts, frame.pts);
	assert_eq!(cloned.data, frame.data);
	assert_eq!(cloned.sample_rate, frame.sample_rate);
}
