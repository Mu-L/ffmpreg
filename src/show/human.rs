use prettytable::{Cell, Row, Table, format};

use super::format::{format_duration, format_size};
use super::types::{
	AudioStreamInfo, FrameInfo, MediaInfo, ShowOptions, StreamInfo, VideoStreamInfo,
};

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
#[allow(dead_code)]
const GREEN: &str = "\x1b[32m";
#[allow(dead_code)]
const RED: &str = "\x1b[31m";

pub fn render(info: &MediaInfo, opts: &ShowOptions) {
	render_file_header(info);
	render_streams(info, opts);
	render_frames(info, opts);
}

fn render_file_header(info: &MediaInfo) {
	let duration = format_duration(info.file.duration);
	let size = format_size(info.file.size);
	let stream_count = info.streams.len();

	println!();
	println!("{}▶ File{}      : {}", BOLD, RESET, info.file.path);
	println!("  Duration  : {}", duration);
	println!("  Size      : {}", size);
	println!("  Streams   : {}", stream_count);
	println!();
}

fn render_streams(info: &MediaInfo, opts: &ShowOptions) {
	for stream in &info.streams {
		let should_skip = opts.stream_filter.is_some_and(|f| f != stream.index());

		if should_skip {
			continue;
		}

		match stream {
			StreamInfo::Video(v) => render_video_stream(v),
			StreamInfo::Audio(a) => render_audio_stream(a),
		}
	}
}

fn render_video_stream(stream: &VideoStreamInfo) {
	let fps_decimal = calculate_fps(&stream.frame_rate);
	let frame_rate_display = format!("{:.2} fps ({})", fps_decimal, stream.frame_rate);
	let aspect_display = format_aspect_ratio(stream);

	println!("{}Video Stream #{}{}", BOLD, stream.index, RESET);
	println!("  Codec        : {}", stream.codec);
	println!("  Resolution   : {} x {}", stream.width, stream.height);
	println!("  Frame Rate   : {}", frame_rate_display);
	println!("  Pixel Format : {}", stream.pix_fmt);

	if let Some(aspect) = aspect_display {
		println!("  Aspect Ratio : {}", aspect);
	}

	println!("  Field Order  : {}", stream.field_order);
	println!();
}

fn calculate_fps(frame_rate: &str) -> f64 {
	let parts: Vec<&str> = frame_rate.split('/').collect();

	if parts.len() != 2 {
		return 0.0;
	}

	let num: f64 = parts[0].parse().unwrap_or(0.0);
	let den: f64 = parts[1].parse().unwrap_or(1.0);

	if den == 0.0 {
		return 0.0;
	}

	num / den
}

fn format_aspect_ratio(stream: &VideoStreamInfo) -> Option<String> {
	let sar = stream.aspect_ratio.as_ref()?;
	let dar = stream.display_aspect.as_ref()?;
	let ratio = calculate_aspect_decimal(sar);

	Some(format!("{:.2} ({} -> {})", ratio, sar, dar))
}

fn calculate_aspect_decimal(aspect: &str) -> f64 {
	let parts: Vec<&str> = aspect.split(':').collect();

	if parts.len() != 2 {
		return 0.0;
	}

	let num: f64 = parts[0].parse().unwrap_or(0.0);
	let den: f64 = parts[1].parse().unwrap_or(1.0);

	if den == 0.0 {
		return 0.0;
	}

	num / den
}

fn render_audio_stream(stream: &AudioStreamInfo) {
	println!("{}Audio Stream #{}{}", BOLD, stream.index, RESET);
	println!("  Codec        : {}", stream.codec);
	println!("  Sample Rate  : {} Hz", stream.sample_rate);
	println!("  Channels     : {}", stream.channels);
	println!("  Bit Depth    : {}-bit", stream.bit_depth);
	println!();
}

fn render_frames(info: &MediaInfo, opts: &ShowOptions) {
	let has_frames = !info.frames.is_empty();

	if !has_frames {
		return;
	}

	println!("{}Frames{} (hex preview, first {} bytes)", BOLD, RESET, opts.hex_limit);

	let mut table = Table::new();
	let table_format =
		format::FormatBuilder::new().column_separator(' ').borders(' ').padding(1, 1).build();
	table.set_format(table_format);

	table.add_row(Row::new(vec![
		Cell::new("index"),
		Cell::new("pts"),
		// Cell::new("key"),
		Cell::new("size"),
		Cell::new("hexdump"),
	]));

	let frames_to_show = info.frames.iter().take(opts.frame_limit);

	for frame in frames_to_show {
		let row = build_frame_row(frame, opts);
		table.add_row(row);
	}

	table.printstd();

	render_remaining_count(&info.frames, opts.frame_limit);
	println!();
}

fn build_frame_row(frame: &FrameInfo, opts: &ShowOptions) -> Row {
	let _key_icon = format_keyframe_icon(frame.keyframe);
	let hex_display = truncate_hex(&frame.hex, opts.hex_limit);
	let size_display = format_frame_size(frame.size);

	Row::new(vec![
		Cell::new(&frame.index.to_string()),
		Cell::new(&frame.pts.to_string()),
		// Cell::new(&key_icon),
		Cell::new(&size_display),
		Cell::new(&hex_display),
	])
}

fn format_frame_size(bytes: usize) -> String {
	const KB: usize = 1024;
	const MB: usize = KB * 1024;

	if bytes >= MB {
		return format!("{:.1} MB", bytes as f64 / MB as f64);
	}

	if bytes >= KB {
		return format!("{:.1} KB", bytes as f64 / KB as f64);
	}

	format!("{} B", bytes)
}

fn format_keyframe_icon(is_keyframe: bool) -> String {
	if is_keyframe {
		return "✓".to_string();
	}
	"✗".to_string()
}

fn truncate_hex(hex: &str, limit: usize) -> String {
	let parts: Vec<&str> = hex.split_whitespace().collect();
	let take = parts.len().min(limit);

	parts[..take].join(" ")
}

fn render_remaining_count(frames: &[FrameInfo], limit: usize) {
	let total = frames.len();
	let remaining = total.saturating_sub(limit);

	if remaining == 0 {
		return;
	}

	println!("... {} more frames", remaining);
}
