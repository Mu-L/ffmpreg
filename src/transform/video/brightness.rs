use crate::core::Frame;
use crate::io::IoResult;

pub struct Brightness {
	factor: f32,
	width: u32,
	height: u32,
}

impl Brightness {
	pub fn new(width: u32, height: u32, factor: f32) -> Self {
		Self { factor, width, height }
	}

	pub fn apply_yuv420(&self, frame: &Frame) -> IoResult<Frame> {
		let y_size = (self.width * self.height) as usize;
		let uv_size = y_size / 4;

		let mut dst_data = frame.data.clone();

		for i in 0..y_size {
			let y = dst_data[i] as f32;
			let adjusted = (y + self.factor * 255.0).clamp(0.0, 255.0);
			dst_data[i] = adjusted as u8;
		}

		Ok(Frame {
			data: dst_data,
			pts: frame.pts,
			timebase: frame.timebase,
			sample_rate: frame.sample_rate,
			channels: frame.channels,
			nb_samples: y_size + 2 * uv_size,
		})
	}
}
