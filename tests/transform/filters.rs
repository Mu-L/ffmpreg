use ffmpreg::core::{Frame, Timebase, Transform};
use ffmpreg::transform::{
	ChannelMixer, FadeIn, FadeOut, Highpass, Lowpass, PeakLimiter, Resample, RmsLimiter,
};

fn create_test_frame(samples: Vec<i16>) -> Frame {
	let timebase = Timebase::new(1, 44100);
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	Frame::new(data, timebase, 44100, 1, samples.len())
}

fn create_stereo_frame(left: Vec<i16>, right: Vec<i16>) -> Frame {
	let timebase = Timebase::new(1, 44100);
	let mut data = Vec::new();
	for (l, r) in left.iter().zip(right.iter()) {
		data.extend_from_slice(&l.to_le_bytes());
		data.extend_from_slice(&r.to_le_bytes());
	}
	Frame::new(data, timebase, 44100, 2, left.len())
}

fn extract_samples(frame: &Frame) -> Vec<i16> {
	frame.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

#[test]
fn test_highpass_filter() {
	let samples: Vec<i16> = (0..512).map(|i| ((i as f32 * 0.1).sin() * 16000.0) as i16).collect();
	let frame = create_test_frame(samples);

	let mut highpass = Highpass::new(1000.0);
	let result = highpass.apply(frame).unwrap();

	assert_eq!(result.nb_samples, 512);
	assert_eq!(result.channels, 1);
}

#[test]
fn test_lowpass_filter() {
	let samples: Vec<i16> = (0..512).map(|i| ((i as f32 * 0.5).sin() * 16000.0) as i16).collect();
	let frame = create_test_frame(samples);

	let mut lowpass = Lowpass::new(500.0);
	let result = lowpass.apply(frame).unwrap();

	assert_eq!(result.nb_samples, 512);
	assert_eq!(result.channels, 1);
}

#[test]
fn test_peak_limiter() {
	let samples: Vec<i16> = vec![32000, -32000, 16000, -16000, 8000, -8000];
	let frame = create_test_frame(samples);

	let mut limiter = PeakLimiter::new(-6.0);
	let result = limiter.apply(frame).unwrap();

	let output = extract_samples(&result);
	for sample in output {
		let limit = (32767.0 * 0.5) as i16;
		assert!(sample.abs() <= limit + 100);
	}
}

#[test]
fn test_rms_limiter() {
	let samples: Vec<i16> = vec![20000; 256];
	let frame = create_test_frame(samples);

	let mut limiter = RmsLimiter::new(-10.0, 10.0, 44100);
	let result = limiter.apply(frame).unwrap();

	assert_eq!(result.nb_samples, 256);
}

#[test]
fn test_resample_upsample() {
	let samples: Vec<i16> = (0..100).map(|i| (i * 100) as i16).collect();
	let frame = create_test_frame(samples);

	let mut resample = Resample::new(88200);
	let result = resample.apply(frame).unwrap();

	assert!(result.nb_samples > 100);
	assert_eq!(result.sample_rate, 88200);
}

#[test]
fn test_resample_downsample() {
	let samples: Vec<i16> = (0..200).map(|i| (i * 50) as i16).collect();
	let frame = create_test_frame(samples);

	let mut resample = Resample::new(22050);
	let result = resample.apply(frame).unwrap();

	assert!(result.nb_samples < 200);
	assert_eq!(result.sample_rate, 22050);
}

#[test]
fn test_channel_mixer_mono_to_stereo() {
	let samples: Vec<i16> = vec![1000, 2000, 3000, 4000];
	let frame = create_test_frame(samples.clone());

	let mut mixer = ChannelMixer::mono_to_stereo();
	let result = mixer.apply(frame).unwrap();

	assert_eq!(result.channels, 2);
	assert_eq!(result.nb_samples, 4);

	let output = extract_samples(&result);
	assert_eq!(output[0], 1000);
	assert_eq!(output[1], 1000);
	assert_eq!(output[2], 2000);
	assert_eq!(output[3], 2000);
}

#[test]
fn test_channel_mixer_stereo_to_mono() {
	let left = vec![1000i16, 2000, 3000, 4000];
	let right = vec![1000i16, 2000, 3000, 4000];
	let frame = create_stereo_frame(left, right);

	let mut mixer = ChannelMixer::stereo_to_mono();
	let result = mixer.apply(frame).unwrap();

	assert_eq!(result.channels, 1);
	assert_eq!(result.nb_samples, 4);

	let output = extract_samples(&result);
	assert_eq!(output[0], 1000);
	assert_eq!(output[1], 2000);
}

#[test]
fn test_fade_in() {
	let samples: Vec<i16> = vec![10000; 100];
	let frame = create_test_frame(samples);

	let mut fade = FadeIn::new(2.0, 44100);
	let result = fade.apply(frame).unwrap();

	let output = extract_samples(&result);

	assert!(output[0].abs() < output[50].abs());
}

#[test]
fn test_fade_out() {
	let samples: Vec<i16> = vec![10000; 100];
	let frame = create_test_frame(samples);

	let mut fade = FadeOut::from_sample_count(50, 100);
	let result = fade.apply(frame).unwrap();

	let output = extract_samples(&result);
	assert!(output[99].abs() < output[0].abs());
}
