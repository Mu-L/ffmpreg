use crate::core::{Frame, Transform};
use crate::io::Result;

pub struct Volume {
	factor: f32,
}

impl Volume {
	pub fn new(factor: f32) -> Self {
		Self { factor }
	}
}

impl Transform for Volume {
	fn apply(&mut self, mut frame: Frame) -> Result<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {}
		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"volume"
	}
}
