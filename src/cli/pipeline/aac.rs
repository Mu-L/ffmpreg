use super::common::Pipeline;
use crate::io::{Error, Result};

pub fn run(_pipeline: Pipeline) -> Result<()> {
	Err(Error::invalid_data("acc pipeline not implemented"))
}
