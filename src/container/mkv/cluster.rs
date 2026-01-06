use crate::core::ebml::types::{EbmlElement, EbmlType};

#[derive(Clone, Debug)]
pub struct Block {
	pub track_number: u64,
	pub timecode: i16,
	pub flags: u8,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SimpleBlock {
	pub track_number: u64,
	pub timecode: i16,
	pub flags: u8,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct BlockGroup {
	pub block: Option<Block>,
	pub block_virtual: Option<Vec<u8>>,
	pub duration: Option<u64>,
	pub references: Vec<i64>,
	pub codec_state: Option<Vec<u8>>,
	pub discard_padding: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct Cluster {
	pub timecode: u64,
	pub position: Option<u64>,
	pub prev_size: Option<u64>,
	pub blocks: Vec<SimpleBlock>,
	pub block_groups: Vec<BlockGroup>,
}

impl Block {
	pub fn new(track_number: u64, timecode: i16, flags: u8, data: Vec<u8>) -> Self {
		Block { track_number, timecode, flags, data }
	}

	pub fn is_keyframe(&self) -> bool {
		self.flags & 0x80 != 0
	}

	pub fn is_invisible(&self) -> bool {
		self.flags & 0x08 != 0
	}

	pub fn lacing_type(&self) -> u8 {
		(self.flags >> 1) & 0x03
	}
}

impl SimpleBlock {
	pub fn new(track_number: u64, timecode: i16, flags: u8, data: Vec<u8>) -> Self {
		SimpleBlock { track_number, timecode, flags, data }
	}

	pub fn is_keyframe(&self) -> bool {
		self.flags & 0x80 != 0
	}

	pub fn is_invisible(&self) -> bool {
		self.flags & 0x08 != 0
	}

	pub fn is_discardable(&self) -> bool {
		self.flags & 0x01 != 0
	}

	pub fn lacing_type(&self) -> u8 {
		(self.flags >> 1) & 0x03
	}
}

impl BlockGroup {
	pub fn new() -> Self {
		BlockGroup {
			block: None,
			block_virtual: None,
			duration: None,
			references: Vec::new(),
			codec_state: None,
			discard_padding: None,
		}
	}

	pub fn has_block(&self) -> bool {
		self.block.is_some()
	}

	pub fn duration_ms(&self, timecode_scale: u64) -> Option<f64> {
		self.duration.map(|d| (d as f64) * (timecode_scale as f64) / 1_000_000.0)
	}
}

impl Cluster {
	pub fn new(timecode: u64) -> Self {
		Cluster {
			timecode,
			position: None,
			prev_size: None,
			blocks: Vec::new(),
			block_groups: Vec::new(),
		}
	}

	pub fn block_count(&self) -> usize {
		self.blocks.len() + self.block_groups.len()
	}

	pub fn from_element(elem: &EbmlElement) -> Option<Self> {
		if elem.id != 0x1F43B675 {
			return None;
		}

		let mut cluster = Cluster::new(0);
		let children = elem.as_master()?;

		for child in children {
			match child.id {
				0xE7 => {
					if let EbmlType::Uinteger(tc) = &child.data {
						cluster.timecode = *tc;
					}
				}
				0xA7 => {
					if let EbmlType::Uinteger(pos) = &child.data {
						cluster.position = Some(*pos);
					}
				}
				0xAB => {
					if let EbmlType::Uinteger(prev) = &child.data {
						cluster.prev_size = Some(*prev);
					}
				}
				0xA3 => {
					if let Some(sb) = parse_simple_block(child) {
						cluster.blocks.push(sb);
					}
				}
				0xA0 => {
					if let Some(bg) = parse_block_group(child) {
						cluster.block_groups.push(bg);
					}
				}
				_ => {}
			}
		}

		Some(cluster)
	}
}

fn parse_simple_block(elem: &EbmlElement) -> Option<SimpleBlock> {
	let data = match &elem.data {
		EbmlType::Binary(b) => b.clone(),
		_ => return None,
	};

	if data.len() < 4 {
		return None;
	}

	let track_number = parse_vint(&data);
	let timecode = i16::from_be_bytes([data[1], data[2]]);
	let flags = data[3];

	Some(SimpleBlock::new(track_number, timecode, flags, data[4..].to_vec()))
}

fn parse_block_group(elem: &EbmlElement) -> Option<BlockGroup> {
	let children = elem.as_master()?;
	let mut group = BlockGroup::new();

	for child in children {
		match child.id {
			0xA1 => {
				if let Some(block) = parse_simple_block(child) {
					group.block =
						Some(Block::new(block.track_number, block.timecode, block.flags, block.data));
				}
			}
			0xA2 => {
				if let EbmlType::Binary(data) = &child.data {
					group.block_virtual = Some(data.clone());
				}
			}
			0x9B => {
				if let EbmlType::Uinteger(dur) = &child.data {
					group.duration = Some(*dur);
				}
			}
			0xFB => {
				if let EbmlType::Integer(ref_tc) = &child.data {
					group.references.push(*ref_tc);
				}
			}
			0xA4 => {
				if let EbmlType::Binary(state) = &child.data {
					group.codec_state = Some(state.clone());
				}
			}
			0x75A2 => {
				if let EbmlType::Integer(padding) = &child.data {
					group.discard_padding = Some(*padding);
				}
			}
			_ => {}
		}
	}

	Some(group)
}

fn parse_vint(data: &[u8]) -> u64 {
	if data.is_empty() {
		return 0;
	}

	let first = data[0];
	let length = match first {
		0x80..=0xFF => 1,
		0x40..=0x7F => 2,
		0x20..=0x3F => 3,
		0x10..=0x1F => 4,
		_ => return 0,
	};

	if data.len() < length {
		return 0;
	}

	let mask = (1u8 << (8 - length as u8)) - 1;
	let mut value = (first & mask) as u64;

	for i in 1..length {
		value = (value << 8) | (data[i] as u64);
	}

	value
}
