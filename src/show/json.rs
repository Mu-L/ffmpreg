use super::format::hex_without_spaces;
use super::types::{AudioStreamInfo, FrameInfo, MediaInfo, StreamInfo, VideoStreamInfo};

pub fn render(info: &MediaInfo) {
	print!("{{");
	render_file_info(info);
	render_streams(&info.streams);
	render_frames(&info.frames);
	println!("}}");
}

fn render_file_info(info: &MediaInfo) {
	let path = escape(&info.file.path);
	let duration = info.file.duration;
	let size = info.file.size;

	print!("\"file\":\"{}\",", path);
	print!("\"duration\":{:.2},", duration);
	print!("\"size\":{},", size);
}

fn render_streams(streams: &[StreamInfo]) {
	print!("\"streams\":[");

	for (idx, stream) in streams.iter().enumerate() {
		let needs_comma = idx > 0;

		if needs_comma {
			print!(",");
		}

		match stream {
			StreamInfo::Video(v) => render_video_stream(v),
			StreamInfo::Audio(a) => render_audio_stream(a),
		}
	}

	print!("],");
}

fn render_video_stream(stream: &VideoStreamInfo) {
	print!("{{");
	print!("\"index\":{},", stream.index);
	print!("\"type\":\"video\",");
	print!("\"codec\":\"{}\",", escape(&stream.codec));
	print!("\"pix_fmt\":\"{}\",", escape(&stream.pix_fmt));
	print!("\"width\":{},", stream.width);
	print!("\"height\":{},", stream.height);
	print!("\"frame_rate\":\"{}\"", escape(&stream.frame_rate));
	print!("}}");
}

fn render_audio_stream(stream: &AudioStreamInfo) {
	print!("{{");
	print!("\"index\":{},", stream.index);
	print!("\"type\":\"audio\",");
	print!("\"codec\":\"{}\",", escape(&stream.codec));
	print!("\"sample_rate\":{},", stream.sample_rate);
	print!("\"channels\":{},", stream.channels);
	print!("\"bit_depth\":{}", stream.bit_depth);
	print!("}}");
}

fn render_frames(frames: &[FrameInfo]) {
	print!("\"frames\":[");

	for (idx, frame) in frames.iter().enumerate() {
		let needs_comma = idx > 0;

		if needs_comma {
			print!(",");
		}

		render_frame(frame);
	}

	print!("]");
}

fn render_frame(frame: &FrameInfo) {
	let hex = hex_without_spaces(&frame.hex);

	print!("{{");
	print!("\"index\":{},", frame.index);
	print!("\"pts\":{},", frame.pts);
	print!("\"keyframe\":{},", frame.keyframe);
	print!("\"size\":{},", frame.size);
	print!("\"hex\":\"{}\"", escape(&hex));
	print!("}}");
}

fn escape(s: &str) -> String {
	s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
}
