// use std::io::Cursor;

pub fn create_test_wav_data() -> Vec<u8> {
	let sample_rate: u32 = 44100;
	let channels: u16 = 1;
	let bits_per_sample: u16 = 16;
	let num_samples: u32 = 1024;

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
		let sample = ((i as f32 / num_samples as f32) * 32767.0 * 0.5) as i16;
		wav.extend_from_slice(&sample.to_le_bytes());
	}

	wav
}

pub fn create_test_wav_stereo_data() -> Vec<u8> {
	let sample_rate: u32 = 44100;
	let channels: u16 = 2;
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
		let left = ((i as f32 / num_samples as f32) * 16000.0) as i16;
		let right = (-(i as f32 / num_samples as f32) * 16000.0) as i16;
		wav.extend_from_slice(&left.to_le_bytes());
		wav.extend_from_slice(&right.to_le_bytes());
	}

	wav
}

pub fn create_test_y4m_data() -> Vec<u8> {
	let width: u32 = 8;
	let height: u32 = 8;
	let num_frames = 3;

	let mut y4m = Vec::new();

	let header = format!("YUV4MPEG2 W{} H{} F30:1 Ip A1:1 C420\n", width, height);
	y4m.extend_from_slice(header.as_bytes());

	let luma_size = (width * height) as usize;
	let chroma_size = luma_size / 4;
	// let frame_size = luma_size + chroma_size * 2;

	for frame_idx in 0..num_frames {
		y4m.extend_from_slice(b"FRAME\n");

		for i in 0..luma_size {
			y4m.push(((frame_idx * 30 + i) % 256) as u8);
		}
		for _ in 0..chroma_size {
			y4m.push(128);
		}
		for _ in 0..chroma_size {
			y4m.push(128);
		}
	}

	y4m
}

pub fn create_test_y4m_no_colorspace() -> Vec<u8> {
	let width: u32 = 4;
	let height: u32 = 4;

	let mut y4m = Vec::new();

	let header = format!("YUV4MPEG2 W{} H{} F25:1 Ip A128:117\n", width, height);
	y4m.extend_from_slice(header.as_bytes());

	let luma_size = (width * height) as usize;
	let chroma_size = luma_size / 4;

	y4m.extend_from_slice(b"FRAME\n");
	for i in 0..luma_size {
		y4m.push((i * 10) as u8);
	}
	for _ in 0..chroma_size {
		y4m.push(128);
	}
	for _ in 0..chroma_size {
		y4m.push(128);
	}

	y4m
}

// pub fn cursor_from_bytes(data: Vec<u8>) -> Cursor<Vec<u8>> {
// 	Cursor::new(data)
// }
