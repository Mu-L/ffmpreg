use crate::io;
use crate::io::Result;
use std::path::Path;

#[allow(dead_code)]
pub fn extension(path: &str) -> Result<String> {
	Path::new(path)
		.extension()
		.and_then(|ext| ext.to_str())
		.map(|s| s.to_lowercase())
		.ok_or_else(|| io::Error::invalid_data("no file extension"))
}
