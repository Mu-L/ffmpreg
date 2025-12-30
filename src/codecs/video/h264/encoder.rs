use super::utils::{NalUnit, NalUnitType, escape_emulation_prevention};
use crate::core::{Encoder, Frame, FrameVideo, Packet, Time};
use crate::io::Result;

#[allow(dead_code)]
pub struct H264Encoder {
	width: u32,
	height: u32,
	time_scale: u32,
	frame_count: u64,
	sps_generated: bool,
	pps_generated: bool,
}

impl H264Encoder {
	pub fn new(width: u32, height: u32, time_scale: u32) -> Self {
		Self { width, height, time_scale, frame_count: 0, sps_generated: false, pps_generated: false }
	}

	fn generate_sps(&self) -> NalUnit {
		let profile_idc = 66;
		let level_idc = 30;

		let mut data = vec![profile_idc, 0, level_idc];

		for _ in 0..10 {
			data.push(0xFF);
		}

		NalUnit::new(3, NalUnitType::SequenceParameterSet, data)
	}

	fn generate_pps(&self) -> NalUnit {
		let pps_id = 0;
		let sps_id = 0;

		let data = vec![pps_id | (sps_id << 4)];

		NalUnit::new(3, NalUnitType::PictureParameterSet, data)
	}

	fn build_slice_header(is_keyframe: bool, slice_num: u64) -> Vec<u8> {
		let mut header = Vec::new();

		let slice_type = if is_keyframe { 7 } else { 1 };
		header.push(slice_type);

		let poc_lsb = (slice_num & 0xFF) as u8;
		header.push(poc_lsb);

		header
	}

	fn build_frame_data(video: &FrameVideo, is_keyframe: bool) -> Vec<u8> {
		let mut frame_data = Vec::new();

		frame_data.extend_from_slice(&[0, 0, 0, 1]);

		let slice_header = Self::build_slice_header(is_keyframe, 0);
		let nal_unit = NalUnit::new(
			if is_keyframe { 3 } else { 2 },
			if is_keyframe { NalUnitType::CodedSliceIdr } else { NalUnitType::CodedSliceNonIdr },
			slice_header,
		);

		let nal_bytes = nal_unit.to_bytes();
		let escaped_nal = escape_emulation_prevention(&nal_bytes);
		frame_data.extend_from_slice(&escaped_nal);

		frame_data.extend_from_slice(&video.data);

		frame_data
	}
}

impl Encoder for H264Encoder {
	fn encode(&mut self, frame: Frame) -> Result<Option<Packet>> {
		let video = frame.video().ok_or_else(|| crate::io::Error::invalid_data("not a video frame"))?;

		let is_keyframe = video.keyframe;

		let mut packet_data = Vec::new();

		if !self.sps_generated {
			let sps = self.generate_sps();
			packet_data.extend_from_slice(&[0, 0, 0, 1]);
			packet_data.extend_from_slice(&sps.to_bytes());
			self.sps_generated = true;
		}

		if !self.pps_generated {
			let pps = self.generate_pps();
			packet_data.extend_from_slice(&[0, 0, 0, 1]);
			packet_data.extend_from_slice(&pps.to_bytes());
			self.pps_generated = true;
		}

		let frame_data = Self::build_frame_data(video, is_keyframe);
		packet_data.extend_from_slice(&frame_data);

		let time = Time::new(1, self.time_scale);
		let pts = self.frame_count as i64;

		let packet = Packet::new(packet_data, frame.stream_index, time).with_pts(pts).with_dts(pts);

		self.frame_count += 1;

		Ok(Some(packet))
	}

	fn flush(&mut self) -> Result<Option<Packet>> {
		Ok(None)
	}
}
