use crate::core::{Frame, Transform};
use crate::io::Result;

pub struct Normalize {}

impl Normalize {}

impl Transform for Normalize {
	fn apply(&mut self, mut frame: Frame) -> Result<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {}
		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"normalize"
	}
}
