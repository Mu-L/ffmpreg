use crate::core::Frame;
use crate::io::Result;

pub struct FrameRateConverter {
	src_fps_num: u32,
	src_fps_den: u32,
	dst_fps_num: u32,
	dst_fps_den: u32,
	frame_count: u64,
	output_count: u64,
	last_frame: Option<Frame>,
}

impl FrameRateConverter {
	pub fn new(src_fps_num: u32, src_fps_den: u32, dst_fps_num: u32, dst_fps_den: u32) -> Self {
		Self {
			src_fps_num,
			src_fps_den,
			dst_fps_num,
			dst_fps_den,
			frame_count: 0,
			output_count: 0,
			last_frame: None,
		}
	}

	pub fn double() -> Self {
		Self::new(30, 1, 60, 1)
	}

	pub fn halve() -> Self {
		Self::new(60, 1, 30, 1)
	}

	pub fn process(&mut self, frame: Frame) -> Result<Vec<Frame>> {
		let src_fps = self.src_fps_num as f64 / self.src_fps_den as f64;
		let dst_fps = self.dst_fps_num as f64 / self.dst_fps_den as f64;

		let src_time = self.frame_count as f64 / src_fps;
		let next_src_time = (self.frame_count + 1) as f64 / src_fps;

		let mut output_frames = Vec::new();

		loop {
			let output_time = self.output_count as f64 / dst_fps;

			if output_time >= next_src_time {
				break;
			}

			if output_time >= src_time {
				let mut out_frame = frame.clone();
				out_frame.pts = self.output_count as i64;
				output_frames.push(out_frame);
				self.output_count += 1;
			} else if output_time < src_time {
				if let Some(ref last) = self.last_frame {
					let mut out_frame = last.clone();
					out_frame.pts = self.output_count as i64;
					output_frames.push(out_frame);
				}
				self.output_count += 1;
			}

			if output_frames.len() > 10 {
				break;
			}
		}

		self.last_frame = Some(frame);
		self.frame_count += 1;

		Ok(output_frames)
	}

	pub fn flush(&mut self) -> Result<Vec<Frame>> {
		Ok(Vec::new())
	}
}
