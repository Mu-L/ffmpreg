pub fn format_size(bytes: u64) -> String {
	const KB: u64 = 1024;
	const MB: u64 = KB * 1024;
	const GB: u64 = MB * 1024;

	if bytes >= GB {
		return format!("{:.2} GB", bytes as f64 / GB as f64);
	}

	if bytes >= MB {
		return format!("{:.2} MB", bytes as f64 / MB as f64);
	}

	if bytes >= KB {
		return format!("{:.2} KB", bytes as f64 / KB as f64);
	}

	format!("{} B", bytes)
}

pub fn format_duration(seconds: f64) -> String {
	if seconds >= 3600.0 {
		let hours = (seconds / 3600.0) as u32;
		let minutes = ((seconds % 3600.0) / 60.0) as u32;
		let secs = seconds % 60.0;
		return format!("{}:{:02}:{:05.2}", hours, minutes, secs);
	}

	if seconds >= 60.0 {
		let minutes = (seconds / 60.0) as u32;
		let secs = seconds % 60.0;
		return format!("{}:{:05.2}", minutes, secs);
	}

	format!("{:.2} s", seconds)
}

pub fn bytes_to_hex(data: &[u8], limit: usize) -> String {
	let take = data.len().min(limit);
	let bytes = &data[..take];
	let hex_parts: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
	let hex_string = hex_parts.join(" ");

	if data.len() > limit {
		return format!("{} ...", hex_string);
	}

	hex_string
}

pub fn hex_without_spaces(hex: &str) -> String {
	hex.replace(' ', "").replace(" ...", "...")
}
