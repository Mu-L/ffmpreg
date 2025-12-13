use crate::core::Frame;
use crate::io::IoResult;

pub trait Transform: Send {
	fn apply(&mut self, frame: Frame) -> IoResult<Frame>;
	fn name(&self) -> &'static str;
}
