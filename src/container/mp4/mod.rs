pub mod read;
pub mod write;

pub use read::Mp4Reader;
pub use write::Mp4Writer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
	Ftyp,
	Moov,
	Mvhd,
	Trak,
	Tkhd,
	Mdia,
	Mdhd,
	Hdlr,
	Minf,
	Stbl,
	Stsd,
	Stts,
	Stsc,
	Stsz,
	Stco,
	Co64,
	Ctts,
	Stss,
	Mdat,
	Free,
	Skip,
	Udta,
	Meta,
	Edts,
	Elst,
	Unknown,
}

impl BoxType {
	pub fn from_fourcc(fourcc: &[u8; 4]) -> Self {
		match fourcc {
			b"ftyp" => BoxType::Ftyp,
			b"moov" => BoxType::Moov,
			b"mvhd" => BoxType::Mvhd,
			b"trak" => BoxType::Trak,
			b"tkhd" => BoxType::Tkhd,
			b"mdia" => BoxType::Mdia,
			b"mdhd" => BoxType::Mdhd,
			b"hdlr" => BoxType::Hdlr,
			b"minf" => BoxType::Minf,
			b"stbl" => BoxType::Stbl,
			b"stsd" => BoxType::Stsd,
			b"stts" => BoxType::Stts,
			b"stsc" => BoxType::Stsc,
			b"stsz" => BoxType::Stsz,
			b"stco" => BoxType::Stco,
			b"co64" => BoxType::Co64,
			b"ctts" => BoxType::Ctts,
			b"stss" => BoxType::Stss,
			b"mdat" => BoxType::Mdat,
			b"free" => BoxType::Free,
			b"skip" => BoxType::Skip,
			b"udta" => BoxType::Udta,
			b"meta" => BoxType::Meta,
			b"edts" => BoxType::Edts,
			b"elst" => BoxType::Elst,
			_ => BoxType::Unknown,
		}
	}

	pub fn as_fourcc(&self) -> [u8; 4] {
		match self {
			BoxType::Ftyp => *b"ftyp",
			BoxType::Moov => *b"moov",
			BoxType::Mvhd => *b"mvhd",
			BoxType::Trak => *b"trak",
			BoxType::Tkhd => *b"tkhd",
			BoxType::Mdia => *b"mdia",
			BoxType::Mdhd => *b"mdhd",
			BoxType::Hdlr => *b"hdlr",
			BoxType::Minf => *b"minf",
			BoxType::Stbl => *b"stbl",
			BoxType::Stsd => *b"stsd",
			BoxType::Stts => *b"stts",
			BoxType::Stsc => *b"stsc",
			BoxType::Stsz => *b"stsz",
			BoxType::Stco => *b"stco",
			BoxType::Co64 => *b"co64",
			BoxType::Ctts => *b"ctts",
			BoxType::Stss => *b"stss",
			BoxType::Mdat => *b"mdat",
			BoxType::Free => *b"free",
			BoxType::Skip => *b"skip",
			BoxType::Udta => *b"udta",
			BoxType::Meta => *b"meta",
			BoxType::Edts => *b"edts",
			BoxType::Elst => *b"elst",
			BoxType::Unknown => *b"    ",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
	Video,
	Audio,
	Hint,
	Text,
	Unknown,
}

#[derive(Debug, Clone)]
pub struct Mp4Format {
	pub major_brand: [u8; 4],
	pub minor_version: u32,
	pub compatible_brands: Vec<[u8; 4]>,
	pub timescale: u32,
	pub duration: u64,
	pub tracks: Vec<Mp4Track>,
}

impl Default for Mp4Format {
	fn default() -> Self {
		Self {
			major_brand: *b"isom",
			minor_version: 512,
			compatible_brands: vec![*b"isom", *b"iso2", *b"mp41"],
			timescale: 1000,
			duration: 0,
			tracks: Vec::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Mp4Track {
	pub track_id: u32,
	pub track_type: TrackType,
	pub timescale: u32,
	pub duration: u64,
	pub width: u32,
	pub height: u32,
	pub sample_rate: u32,
	pub channels: u16,
	pub sample_sizes: Vec<u32>,
	pub chunk_offsets: Vec<u64>,
	pub sample_to_chunk: Vec<(u32, u32, u32)>,
	pub time_to_sample: Vec<(u32, u32)>,
}

impl Default for Mp4Track {
	fn default() -> Self {
		Self {
			track_id: 1,
			track_type: TrackType::Video,
			timescale: 90000,
			duration: 0,
			width: 1920,
			height: 1080,
			sample_rate: 0,
			channels: 0,
			sample_sizes: Vec::new(),
			chunk_offsets: Vec::new(),
			sample_to_chunk: Vec::new(),
			time_to_sample: Vec::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct BoxHeader {
	pub size: u64,
	pub box_type: BoxType,
	pub header_size: u8,
}
