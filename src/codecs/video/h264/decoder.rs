use super::parser::{H264Parser, SpsData};
use crate::core::{Decoder, Frame, FrameVideo, Packet, Time, VideoFormat};
use crate::io::{Error, Result};

pub struct H264Decoder {
	sps_data: Option<SpsData>,
	frame_count: u64,
	width: u32,
	height: u32,
	time_scale: u32,
}

impl H264Decoder {
	pub fn new(time_scale: u32) -> Self {
		Self { sps_data: None, frame_count: 0, width: 1280, height: 720, time_scale }
	}

	pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
		self.width = width;
		self.height = height;
		self
	}

	fn extract_frame_data(packet: &Packet) -> Result<Vec<u8>> {
		if packet.data.is_empty() {
			return Err(Error::invalid_data("empty packet data"));
		}

		let mut frame_bytes = Vec::with_capacity(packet.data.len());
		let raw_data = &packet.data;
		let mut cursor = 0;

		while cursor < raw_data.len() {
			if cursor + 4 <= raw_data.len() && &raw_data[cursor..cursor + 4] == [0, 0, 0, 1] {
				cursor += 4;
				continue;
			}

			if cursor + 3 <= raw_data.len() && &raw_data[cursor..cursor + 3] == [0, 0, 1] {
				cursor += 3;
				continue;
			}

			frame_bytes.push(raw_data[cursor]);
			cursor += 1;
		}

		Ok(frame_bytes)
	}
}

impl Decoder for H264Decoder {
	fn decode(&mut self, packet: Packet) -> Result<Option<Frame>> {
		let frame_data = Self::extract_frame_data(&packet)?;

		if frame_data.is_empty() {
			return Ok(None);
		}

		let is_keyframe = H264Parser::is_idr_frame(&packet.data)?;

		if is_keyframe {
			if let Ok((Some(sps), _)) = H264Parser::find_sps_pps(&packet.data) {
				if let Ok(Some(sps_data)) = H264Parser::parse_sps(&sps) {
					self.sps_data = Some(sps_data.clone());
					self.width = sps_data.width;
					self.height = sps_data.height;
				}
			}
		}

		let video =
			FrameVideo::new(frame_data, self.width, self.height, VideoFormat::YUV420, is_keyframe);

		let time = Time::new(1, self.time_scale);
		let pts = self.frame_count as i64;

		let frame = Frame::new_video(video, time, packet.stream_index, 0).with_pts(pts).with_dts(pts);

		self.frame_count += 1;

		Ok(Some(frame))
	}

	fn flush(&mut self) -> Result<Option<Frame>> {
		Ok(None)
	}
}
