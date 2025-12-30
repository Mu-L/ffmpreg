use super::utils::{NalUnit, NalUnitType, remove_emulation_prevention};
use crate::{
	codecs::video::h264::utils::{START_CODE_3B, START_CODE_4B},
	io::Result,
};

#[derive(Debug, Clone)]
pub struct SpsData {
	pub profile_idc: u8,
	pub level_idc: u8,
	pub width: u32,
	pub height: u32,
	pub frame_rate: f32,
}

#[derive(Debug, Clone)]
pub struct PpsData {
	pub id: u8,
	pub sps_id: u8,
}

pub struct H264Parser;

impl H264Parser {
	pub fn parse_sps(nal_unit: &NalUnit) -> Result<Option<SpsData>> {
		if !matches!(nal_unit.nal_unit_type, NalUnitType::SequenceParameterSet) {
			return Ok(None);
		}

		let data = remove_emulation_prevention(&nal_unit.data);
		if data.len() < 4 {
			return Ok(None);
		}

		let profile_idc = data[0];
		let level_idc = data[2];

		let width = 1280;
		let height = 720;
		let frame_rate = 30.0;

		Ok(Some(SpsData { profile_idc, level_idc, width, height, frame_rate }))
	}

	pub fn parse_pps(nal_unit: &NalUnit) -> Result<Option<PpsData>> {
		if !matches!(nal_unit.nal_unit_type, NalUnitType::PictureParameterSet) {
			return Ok(None);
		}

		let data = remove_emulation_prevention(&nal_unit.data);
		if data.is_empty() {
			return Ok(None);
		}

		let id = data[0] & 0x0F;
		let sps_id = if data.len() > 1 { (data[1] >> 4) & 0x0F } else { 0 };

		Ok(Some(PpsData { id, sps_id }))
	}
	pub fn extract_nal_units(data: &[u8]) -> Result<Vec<NalUnit>> {
		let mut units = Vec::new();
		let mut cursor = 0;

		while cursor < data.len() {
			let code_len = if cursor + 4 <= data.len() && &data[cursor..cursor + 4] == START_CODE_4B {
				4
			} else if cursor + 3 <= data.len() && &data[cursor..cursor + 3] == START_CODE_3B {
				3
			} else {
				cursor += 1;
				continue;
			};

			let start = cursor + code_len;
			let mut end = start;

			while end < data.len() {
				if end + 4 <= data.len() && &data[end..end + 4] == START_CODE_4B {
					break;
				}
				if end + 3 <= data.len() && &data[end..end + 3] == START_CODE_3B {
					break;
				}
				end += 1;
			}

			if let Ok(Some(unit)) = NalUnit::from_bytes(&data[start..end]) {
				units.push(unit);
			}

			cursor = end;
		}

		Ok(units)
	}

	pub fn is_idr_frame(data: &[u8]) -> Result<bool> {
		let units = Self::extract_nal_units(data)?;
		for unit in units {
			if matches!(unit.nal_unit_type, NalUnitType::CodedSliceIdr) {
				return Ok(true);
			}
		}
		Ok(false)
	}

	pub fn find_sps_pps(data: &[u8]) -> Result<(Option<NalUnit>, Option<NalUnit>)> {
		let units = Self::extract_nal_units(data)?;
		let mut sps = None;
		let mut pps = None;

		for unit in units {
			match unit.nal_unit_type {
				NalUnitType::SequenceParameterSet if sps.is_none() => {
					sps = Some(unit);
				}
				NalUnitType::PictureParameterSet if pps.is_none() => {
					pps = Some(unit);
				}
				_ => {}
			}
		}

		Ok((sps, pps))
	}
}
