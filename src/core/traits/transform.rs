use crate::core::Frame;
use crate::io::Result;

pub trait Transform: Send {
	fn apply(&mut self, frame: Frame) -> Result<Frame>;
	fn name(&self) -> &'static str;
}
