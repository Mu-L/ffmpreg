use crate::io;

pub fn get_extension(path: &str) -> io::Result<String> {
	std::path::Path::new(path)
		.extension()
		.and_then(|e| e.to_str())
		.map(|s| s.to_lowercase())
		.ok_or_else(|| io::Error::invalid_data("no file extension"))
}
