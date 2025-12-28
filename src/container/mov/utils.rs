pub const FTYP_BOX: &[u8; 4] = b"ftyp";
pub const MOOV_BOX: &[u8; 4] = b"moov";
pub const MDAT_BOX: &[u8; 4] = b"mdat";
pub const MVHD_BOX: &[u8; 4] = b"mvhd";
pub const TRAK_BOX: &[u8; 4] = b"trak";
pub const TKHD_BOX: &[u8; 4] = b"tkhd";
pub const MDIA_BOX: &[u8; 4] = b"mdia";
pub const MDHD_BOX: &[u8; 4] = b"mdhd";
pub const MINF_BOX: &[u8; 4] = b"minf";
pub const STBL_BOX: &[u8; 4] = b"stbl";
pub const SMHD_BOX: &[u8; 4] = b"smhd";
pub const VMHD_BOX: &[u8; 4] = b"vmhd";
pub const DINF_BOX: &[u8; 4] = b"dinf";
pub const DREF_BOX: &[u8; 4] = b"dref";
pub const STSD_BOX: &[u8; 4] = b"stsd";
pub const STTS_BOX: &[u8; 4] = b"stts";
pub const STSS_BOX: &[u8; 4] = b"stss";
pub const STSZ_BOX: &[u8; 4] = b"stsz";
pub const STCO_BOX: &[u8; 4] = b"stco";
pub const HDLR_BOX: &[u8; 4] = b"hdlr";
pub const UDTA_BOX: &[u8; 4] = b"udta";

#[derive(Debug, Clone)]
pub struct BoxHeader {
	pub size: u32,
	pub fourcc: [u8; 4],
	pub position: u64,
}

impl BoxHeader {
	pub fn new(size: u32, fourcc: [u8; 4], position: u64) -> Self {
		Self { size, fourcc, position }
	}

	pub fn is_type(&self, fcc: &[u8; 4]) -> bool {
		&self.fourcc == fcc
	}
}

#[derive(Debug, Clone)]
pub struct AacSpecificBox {
	pub profile: u8,
	pub sample_rate_idx: u8,
	pub channels: u8,
	pub frame_length: u16,
}

#[derive(Debug, Clone)]
pub struct H264SpecificBox {
	pub profile_idc: u8,
	pub profile_compatibility: u8,
	pub level_idc: u8,
	pub sps: Vec<u8>,
	pub pps: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AudioSampleEntry {
	pub channels: u16,
	pub sample_size: u16,
	pub sample_rate: u32,
	pub aac_specific: Option<AacSpecificBox>,
}

#[derive(Debug, Clone)]
pub struct VideoSampleEntry {
	pub width: u16,
	pub height: u16,
	pub h264_specific: Option<H264SpecificBox>,
}

pub fn fourcc_to_string(fourcc: &[u8; 4]) -> String {
	String::from_utf8_lossy(fourcc).to_string()
}

pub fn string_to_fourcc(s: &str) -> Option<[u8; 4]> {
	let bytes = s.as_bytes();
	if bytes.len() == 4 {
		let mut arr = [0u8; 4];
		arr.copy_from_slice(bytes);
		Some(arr)
	} else {
		None
	}
}
