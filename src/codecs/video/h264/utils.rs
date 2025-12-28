use crate::io::Result;

pub const START_CODE_3B: &[u8] = &[0x00, 0x00, 0x01];
pub const START_CODE_4B: &[u8] = &[0x00, 0x00, 0x00, 0x01];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NalUnitType {
	Unspecified,
	CodedSliceNonIdr,
	CodedSliceDataPartitionA,
	CodedSliceDataPartitionB,
	CodedSliceDataPartitionC,
	CodedSliceIdr,
	SupplementalEnhancementInfo,
	SequenceParameterSet,
	PictureParameterSet,
	AccessUnitDelimiter,
	EndOfSequence,
	EndOfStream,
	FilteringOperationParameterSet,
	SeqParameterSetExtension,
	PrefixNalUnit,
	SubsetSequenceParameterSet,
	Reserved16,
	CodedSliceAux,
	Reserved19,
	CodedSliceDepth,
	Reserved21,
	Reserved22,
	Reserved23,
}

impl NalUnitType {
	pub fn from_u8(value: u8) -> Self {
		match value {
			1 => NalUnitType::CodedSliceNonIdr,
			2 => NalUnitType::CodedSliceDataPartitionA,
			3 => NalUnitType::CodedSliceDataPartitionB,
			4 => NalUnitType::CodedSliceDataPartitionC,
			5 => NalUnitType::CodedSliceIdr,
			6 => NalUnitType::SupplementalEnhancementInfo,
			7 => NalUnitType::SequenceParameterSet,
			8 => NalUnitType::PictureParameterSet,
			9 => NalUnitType::AccessUnitDelimiter,
			10 => NalUnitType::EndOfSequence,
			11 => NalUnitType::EndOfStream,
			12 => NalUnitType::FilteringOperationParameterSet,
			13 => NalUnitType::SeqParameterSetExtension,
			14 => NalUnitType::PrefixNalUnit,
			15 => NalUnitType::SubsetSequenceParameterSet,
			16 => NalUnitType::Reserved16,
			17 => NalUnitType::CodedSliceAux,
			19 => NalUnitType::CodedSliceDepth,
			_ => NalUnitType::Unspecified,
		}
	}

	pub fn is_keyframe(&self) -> bool {
		matches!(
			self,
			NalUnitType::CodedSliceIdr
				| NalUnitType::SequenceParameterSet
				| NalUnitType::PictureParameterSet
		)
	}

	pub fn is_vcl(&self) -> bool {
		matches!(
			self,
			NalUnitType::CodedSliceNonIdr
				| NalUnitType::CodedSliceDataPartitionA
				| NalUnitType::CodedSliceDataPartitionB
				| NalUnitType::CodedSliceDataPartitionC
				| NalUnitType::CodedSliceIdr
				| NalUnitType::CodedSliceAux
				| NalUnitType::CodedSliceDepth
		)
	}
}

#[derive(Debug, Clone)]
pub struct NalUnit {
	pub nal_ref_idc: u8,
	pub nal_unit_type: NalUnitType,
	pub data: Vec<u8>,
}

impl NalUnit {
	pub fn new(nal_ref_idc: u8, nal_unit_type: NalUnitType, data: Vec<u8>) -> Self {
		Self { nal_ref_idc, nal_unit_type, data }
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Option<Self>> {
		if bytes.is_empty() {
			return Ok(None);
		}

		let first_byte = bytes[0];
		let nal_ref_idc = (first_byte >> 5) & 0x03;
		let nal_unit_type = NalUnitType::from_u8(first_byte & 0x1F);

		let data = if bytes.len() > 1 { bytes[1..].to_vec() } else { Vec::new() };

		Ok(Some(Self::new(nal_ref_idc, nal_unit_type, data)))
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut result = Vec::with_capacity(self.data.len() + 1);
		let first_byte = (self.nal_ref_idc << 5) | (self.nal_unit_type as u8 & 0x1F);
		result.push(first_byte);
		result.extend_from_slice(&self.data);
		result
	}
}

pub fn escape_emulation_prevention(data: &[u8]) -> Vec<u8> {
	let mut result = Vec::with_capacity(data.len());
	let mut i = 0;

	while i < data.len() {
		if i + 2 < data.len() && data[i] == 0 && data[i + 1] == 0 && data[i + 2] <= 3 {
			result.push(0);
			result.push(0);
			result.push(3);
			result.push(data[i + 2]);
			i += 3;
		} else {
			result.push(data[i]);
			i += 1;
		}
	}

	result
}

pub fn remove_emulation_prevention(data: &[u8]) -> Vec<u8> {
	let mut result = Vec::with_capacity(data.len());
	let mut i = 0;

	while i < data.len() {
		if i + 2 < data.len()
			&& data[i] == 0
			&& data[i + 1] == 0
			&& data[i + 2] == 3
			&& i + 3 < data.len()
		{
			result.push(0);
			result.push(0);
			i += 3;
		} else {
			result.push(data[i]);
			i += 1;
		}
	}

	result
}
