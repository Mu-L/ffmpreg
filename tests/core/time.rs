use ffmpreg::core::Timebase;

#[test]
fn test_timebase_creation() {
	let tb = Timebase::new(1, 44100);
	assert_eq!(tb.num, 1);
	assert_eq!(tb.den, 44100);
}

#[test]
fn test_timebase_to_seconds() {
	let tb = Timebase::new(1, 1000);
	let seconds = tb.to_seconds(500);
	assert!((seconds - 0.5).abs() < 0.0001);
}

#[test]
fn test_timebase_to_seconds_audio() {
	let tb = Timebase::new(1, 44100);
	let seconds = tb.to_seconds(44100);
	assert!((seconds - 1.0).abs() < 0.0001);
}

#[test]
fn test_timebase_from_seconds() {
	let tb = Timebase::new(1, 1000);
	let pts = tb.from_seconds(0.5);
	assert_eq!(pts, 500);
}

#[test]
fn test_timebase_from_seconds_audio() {
	let tb = Timebase::new(1, 44100);
	let pts = tb.from_seconds(1.0);
	assert_eq!(pts, 44100);
}

#[test]
fn test_timebase_video_framerate() {
	let tb = Timebase::new(1001, 30000);
	let frame_duration = tb.to_seconds(1);
	assert!((frame_duration - 0.0333666).abs() < 0.0001);
}

#[test]
fn test_timebase_equality() {
	let tb1 = Timebase::new(1, 44100);
	let tb2 = Timebase::new(1, 44100);
	let tb3 = Timebase::new(1, 48000);

	assert_eq!(tb1, tb2);
	assert_ne!(tb1, tb3);
}

#[test]
fn test_timebase_copy() {
	let tb1 = Timebase::new(1, 44100);
	let tb2 = tb1;
	assert_eq!(tb1, tb2);
}
