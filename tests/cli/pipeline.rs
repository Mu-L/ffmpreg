use ffmpreg::cli::{Pipeline, is_batch_pattern, is_directory};
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

fn create_test_wav() -> Vec<u8> {
	let sample_rate: u32 = 44100;
	let channels: u16 = 1;
	let bits_per_sample: u16 = 16;
	let num_samples: u32 = 512;

	let data_size = num_samples * (bits_per_sample as u32 / 8) * channels as u32;
	let file_size = 36 + data_size;

	let mut wav = Vec::new();

	wav.extend_from_slice(b"RIFF");
	wav.extend_from_slice(&file_size.to_le_bytes());
	wav.extend_from_slice(b"WAVE");

	wav.extend_from_slice(b"fmt ");
	wav.extend_from_slice(&16u32.to_le_bytes());
	wav.extend_from_slice(&1u16.to_le_bytes());
	wav.extend_from_slice(&channels.to_le_bytes());
	wav.extend_from_slice(&sample_rate.to_le_bytes());
	let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
	wav.extend_from_slice(&byte_rate.to_le_bytes());
	let block_align = channels * bits_per_sample / 8;
	wav.extend_from_slice(&block_align.to_le_bytes());
	wav.extend_from_slice(&bits_per_sample.to_le_bytes());

	wav.extend_from_slice(b"data");
	wav.extend_from_slice(&data_size.to_le_bytes());

	for i in 0..num_samples {
		let sample = ((i as f32 / num_samples as f32) * 10000.0) as i16;
		wav.extend_from_slice(&sample.to_le_bytes());
	}

	wav
}

fn create_test_y4m() -> Vec<u8> {
	let mut y4m = Vec::new();
	y4m.extend_from_slice(b"YUV4MPEG2 W4 H4 F30:1 Ip C420\n");

	let luma_size = 16;
	let chroma_size = 4;

	y4m.extend_from_slice(b"FRAME\n");
	for i in 0..luma_size {
		y4m.push((i * 10) as u8);
	}
	for _ in 0..chroma_size * 2 {
		y4m.push(128);
	}

	y4m
}

#[test]
fn test_is_batch_pattern() {
	assert!(is_batch_pattern("folder/*.wav"));
	assert!(is_batch_pattern("*.wav"));
	assert!(is_batch_pattern("dir/**/*.wav"));
	assert!(!is_batch_pattern("file.wav"));
	assert!(!is_batch_pattern("folder/file.wav"));
}

#[test]
fn test_is_directory() {
	let dir = tempdir().unwrap();
	assert!(is_directory(dir.path().to_str().unwrap()));
	assert!(!is_directory("/nonexistent/path/12345"));
}

#[test]
fn test_pipeline_wav_passthrough() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.wav");
	let output_path = dir.path().join("output.wav");

	let wav_data = create_test_wav();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&wav_data).unwrap();

	let pipeline = Pipeline::new(
		input_path.to_str().unwrap().to_string(),
		Some(output_path.to_str().unwrap().to_string()),
		false,
		vec![],
	);

	pipeline.run().unwrap();

	let output_data = fs::read(&output_path).unwrap();
	assert_eq!(output_data, wav_data);
}

#[test]
fn test_pipeline_wav_show_mode() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.wav");

	let wav_data = create_test_wav();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&wav_data).unwrap();

	let pipeline = Pipeline::new(input_path.to_str().unwrap().to_string(), None, true, vec![]);

	let result = pipeline.run();
	assert!(result.is_ok());
}

#[test]
fn test_pipeline_wav_with_transform() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.wav");
	let output_path = dir.path().join("output.wav");

	let wav_data = create_test_wav();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&wav_data).unwrap();

	let pipeline = Pipeline::new(
		input_path.to_str().unwrap().to_string(),
		Some(output_path.to_str().unwrap().to_string()),
		false,
		vec!["gain=2.0".to_string()],
	);

	pipeline.run().unwrap();

	assert!(output_path.exists());
	let output_data = fs::read(&output_path).unwrap();
	assert!(output_data.len() > 0);
}

#[test]
fn test_pipeline_wav_chained_transforms() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.wav");
	let output_path = dir.path().join("output.wav");

	let wav_data = create_test_wav();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&wav_data).unwrap();

	let pipeline = Pipeline::new(
		input_path.to_str().unwrap().to_string(),
		Some(output_path.to_str().unwrap().to_string()),
		false,
		vec!["gain=1.5".to_string(), "normalize".to_string()],
	);

	pipeline.run().unwrap();

	assert!(output_path.exists());
}

#[test]
fn test_pipeline_y4m_passthrough() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.y4m");
	let output_path = dir.path().join("output.y4m");

	let y4m_data = create_test_y4m();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&y4m_data).unwrap();

	let pipeline = Pipeline::new(
		input_path.to_str().unwrap().to_string(),
		Some(output_path.to_str().unwrap().to_string()),
		false,
		vec![],
	);

	pipeline.run().unwrap();

	assert!(output_path.exists());
	let output_data = fs::read(&output_path).unwrap();
	assert!(output_data.len() > 0);
}

#[test]
fn test_pipeline_y4m_show_mode() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.y4m");

	let y4m_data = create_test_y4m();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&y4m_data).unwrap();

	let pipeline = Pipeline::new(input_path.to_str().unwrap().to_string(), None, true, vec![]);

	let result = pipeline.run();
	assert!(result.is_ok());
}

#[test]
fn test_pipeline_invalid_input() {
	let pipeline = Pipeline::new(
		"/nonexistent/file.wav".to_string(),
		Some("/tmp/output.wav".to_string()),
		false,
		vec![],
	);

	let result = pipeline.run();
	assert!(result.is_err());
}

#[test]
fn test_pipeline_unsupported_format() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.xyz");
	let output_path = dir.path().join("output.xyz");

	let mut file = File::create(&input_path).unwrap();
	file.write_all(b"some data").unwrap();

	let pipeline = Pipeline::new(
		input_path.to_str().unwrap().to_string(),
		Some(output_path.to_str().unwrap().to_string()),
		false,
		vec![],
	);

	let result = pipeline.run();
	assert!(result.is_err());
}

#[test]
fn test_pipeline_missing_output() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.wav");

	let wav_data = create_test_wav();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&wav_data).unwrap();

	let pipeline = Pipeline::new(input_path.to_str().unwrap().to_string(), None, false, vec![]);

	let result = pipeline.run();
	assert!(result.is_err());
}

#[test]
fn test_pipeline_invalid_transform() {
	let dir = tempdir().unwrap();
	let input_path = dir.path().join("input.wav");
	let output_path = dir.path().join("output.wav");

	let wav_data = create_test_wav();
	let mut file = File::create(&input_path).unwrap();
	file.write_all(&wav_data).unwrap();

	let pipeline = Pipeline::new(
		input_path.to_str().unwrap().to_string(),
		Some(output_path.to_str().unwrap().to_string()),
		false,
		vec!["unknown_filter".to_string()],
	);

	let result = pipeline.run();
	assert!(result.is_err());
}
