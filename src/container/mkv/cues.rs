use crate::core::ebml::types::{EbmlElement, EbmlType};

#[derive(Clone, Debug, Default)]
pub struct CueReference {
	pub time: u64,
	pub cluster_position: Option<u64>,
	pub block_number: Option<u64>,
	pub codec_state: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct CueTrackPosition {
	pub track: u64,
	pub cluster_position: u64,
	pub relative_position: Option<u64>,
	pub duration: Option<u64>,
	pub block_number: Option<u64>,
	pub codec_state: Option<u64>,
	pub references: Vec<CueReference>,
}

#[derive(Clone, Debug)]
pub struct CuePoint {
	pub time: u64,
	pub positions: Vec<CueTrackPosition>,
}

#[derive(Clone, Debug)]
pub struct Cues {
	pub cue_points: Vec<CuePoint>,
}

impl CueTrackPosition {
	pub fn new(track: u64, cluster_position: u64) -> Self {
		CueTrackPosition { track, cluster_position, ..Default::default() }
	}
}

impl CuePoint {
	pub fn new(time: u64) -> Self {
		CuePoint { time, positions: Vec::new() }
	}

	pub fn position_for_track(&self, track: u64) -> Option<&CueTrackPosition> {
		self.positions.iter().find(|p| p.track == track)
	}
}

impl Cues {
	pub fn new() -> Self {
		Cues { cue_points: Vec::new() }
	}

	pub fn find_point(&self, time: u64) -> Option<&CuePoint> {
		self.cue_points.iter().find(|cp| cp.time == time)
	}

	pub fn find_point_before(&self, time: u64) -> Option<&CuePoint> {
		self.cue_points.iter().filter(|cp| cp.time <= time).last()
	}

	pub fn find_point_after(&self, time: u64) -> Option<&CuePoint> {
		self.cue_points.iter().find(|cp| cp.time >= time)
	}

	pub fn points_for_track(&self, track: u64) -> Vec<&CuePoint> {
		self.cue_points.iter().filter(|cp| cp.positions.iter().any(|p| p.track == track)).collect()
	}

	pub fn from_element(elem: &EbmlElement) -> Option<Self> {
		if elem.id != 0x1C53BB6B {
			return None;
		}

		let children = elem.as_master()?;
		let mut cues = Cues::new();

		for child in children {
			if child.id == 0xBB {
				if let Some(point) = parse_cue_point(child) {
					cues.cue_points.push(point);
				}
			}
		}

		Some(cues)
	}
}

fn parse_cue_point(elem: &EbmlElement) -> Option<CuePoint> {
	let children = elem.as_master()?;
	let mut point = CuePoint::new(0);

	for child in children {
		match child.id {
			0xB3 => {
				if let EbmlType::Uinteger(time) = &child.data {
					point.time = *time;
				}
			}
			0xB7 => {
				if let Some(pos) = parse_cue_track_position(child) {
					point.positions.push(pos);
				}
			}
			_ => {}
		}
	}

	Some(point)
}

fn parse_cue_track_position(elem: &EbmlElement) -> Option<CueTrackPosition> {
	let children = elem.as_master()?;
	let mut track = 0u64;
	let mut cluster_position = 0u64;
	let mut position = CueTrackPosition::new(0, 0);

	for child in children {
		match child.id {
			0xF7 => {
				if let EbmlType::Uinteger(t) = &child.data {
					track = *t;
				}
			}
			0xF1 => {
				if let EbmlType::Uinteger(cp) = &child.data {
					cluster_position = *cp;
				}
			}
			0x5378 => {
				if let EbmlType::Uinteger(rp) = &child.data {
					position.relative_position = Some(*rp);
				}
			}
			0xB2 => {
				if let EbmlType::Uinteger(dur) = &child.data {
					position.duration = Some(*dur);
				}
			}
			0xEA => {
				if let EbmlType::Uinteger(bn) = &child.data {
					position.block_number = Some(*bn);
				}
			}
			0x5387 => {
				if let EbmlType::Uinteger(cs) = &child.data {
					position.codec_state = Some(*cs);
				}
			}
			0x5381 => {
				if let Some(ref_) = parse_cue_reference(child) {
					position.references.push(ref_);
				}
			}
			_ => {}
		}
	}

	position.track = track;
	position.cluster_position = cluster_position;
	Some(position)
}

fn parse_cue_reference(elem: &EbmlElement) -> Option<CueReference> {
	let children = elem.as_master()?;
	let mut reference = CueReference::default();

	for child in children {
		match child.id {
			0x5388 => {
				if let EbmlType::Uinteger(time) = &child.data {
					reference.time = *time;
				}
			}
			0xEB => {
				if let EbmlType::Uinteger(cp) = &child.data {
					reference.cluster_position = Some(*cp);
				}
			}
			0x535F => {
				if let EbmlType::Uinteger(bn) = &child.data {
					reference.block_number = Some(*bn);
				}
			}
			0x6532 => {
				if let EbmlType::Uinteger(cs) = &child.data {
					reference.codec_state = Some(*cs);
				}
			}
			_ => {}
		}
	}

	Some(reference)
}

impl Default for Cues {
	fn default() -> Self {
		Self::new()
	}
}
