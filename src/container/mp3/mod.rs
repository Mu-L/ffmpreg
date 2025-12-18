pub mod read;
pub mod write;

pub use read::Mp3Reader;
pub use write::Mp3Writer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegVersion {
	Mpeg1,
	Mpeg2,
	Mpeg25,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
	Layer1,
	Layer2,
	Layer3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMode {
	Stereo,
	JointStereo,
	DualChannel,
	Mono,
}

#[derive(Debug, Clone)]
pub struct Mp3Format {
	pub version: MpegVersion,
	pub layer: Layer,
	pub bitrate: u32,
	pub sample_rate: u32,
	pub channels: u8,
	pub channel_mode: ChannelMode,
	pub padding: bool,
}

impl Default for Mp3Format {
	fn default() -> Self {
		Self {
			version: MpegVersion::Mpeg1,
			layer: Layer::Layer3,
			bitrate: 128,
			sample_rate: 44100,
			channels: 2,
			channel_mode: ChannelMode::Stereo,
			padding: false,
		}
	}
}

impl Mp3Format {
	pub fn samples_per_frame(&self) -> usize {
		match (self.version, self.layer) {
			(MpegVersion::Mpeg1, Layer::Layer1) => 384,
			(MpegVersion::Mpeg1, Layer::Layer2) => 1152,
			(MpegVersion::Mpeg1, Layer::Layer3) => 1152,
			(_, Layer::Layer1) => 384,
			(_, Layer::Layer2) => 1152,
			(_, Layer::Layer3) => 576,
		}
	}

	pub fn frame_size(&self) -> usize {
		let samples = self.samples_per_frame();
		let padding = if self.padding { 1 } else { 0 };

		match self.layer {
			Layer::Layer1 => ((12 * self.bitrate * 1000 / self.sample_rate) + padding) as usize * 4,
			Layer::Layer2 | Layer::Layer3 => {
				((samples as u32 / 8 * self.bitrate * 1000 / self.sample_rate) + padding) as usize
			}
		}
	}
}

const BITRATE_TABLE_V1_L3: [u32; 16] =
	[0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
const BITRATE_TABLE_V1_L2: [u32; 16] =
	[0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0];
const BITRATE_TABLE_V1_L1: [u32; 16] =
	[0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0];
const BITRATE_TABLE_V2_L3: [u32; 16] =
	[0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0];
const BITRATE_TABLE_V2_L2: [u32; 16] =
	[0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0];
const BITRATE_TABLE_V2_L1: [u32; 16] =
	[0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0];

const SAMPLE_RATE_TABLE_V1: [u32; 4] = [44100, 48000, 32000, 0];
const SAMPLE_RATE_TABLE_V2: [u32; 4] = [22050, 24000, 16000, 0];
const SAMPLE_RATE_TABLE_V25: [u32; 4] = [11025, 12000, 8000, 0];

pub fn parse_frame_header(header: [u8; 4]) -> Option<Mp3Format> {
	if header[0] != 0xFF || (header[1] & 0xE0) != 0xE0 {
		return None;
	}

	let version_bits = (header[1] >> 3) & 0x03;
	let version = match version_bits {
		0 => MpegVersion::Mpeg25,
		2 => MpegVersion::Mpeg2,
		3 => MpegVersion::Mpeg1,
		_ => return None,
	};

	let layer_bits = (header[1] >> 1) & 0x03;
	let layer = match layer_bits {
		1 => Layer::Layer3,
		2 => Layer::Layer2,
		3 => Layer::Layer1,
		_ => return None,
	};

	let bitrate_index = (header[2] >> 4) & 0x0F;
	let bitrate = match (version, layer) {
		(MpegVersion::Mpeg1, Layer::Layer1) => BITRATE_TABLE_V1_L1[bitrate_index as usize],
		(MpegVersion::Mpeg1, Layer::Layer2) => BITRATE_TABLE_V1_L2[bitrate_index as usize],
		(MpegVersion::Mpeg1, Layer::Layer3) => BITRATE_TABLE_V1_L3[bitrate_index as usize],
		(_, Layer::Layer1) => BITRATE_TABLE_V2_L1[bitrate_index as usize],
		(_, Layer::Layer2) => BITRATE_TABLE_V2_L2[bitrate_index as usize],
		(_, Layer::Layer3) => BITRATE_TABLE_V2_L3[bitrate_index as usize],
	};

	if bitrate == 0 {
		return None;
	}

	let sample_rate_index = (header[2] >> 2) & 0x03;
	let sample_rate = match version {
		MpegVersion::Mpeg1 => SAMPLE_RATE_TABLE_V1[sample_rate_index as usize],
		MpegVersion::Mpeg2 => SAMPLE_RATE_TABLE_V2[sample_rate_index as usize],
		MpegVersion::Mpeg25 => SAMPLE_RATE_TABLE_V25[sample_rate_index as usize],
	};

	if sample_rate == 0 {
		return None;
	}

	let padding = (header[2] >> 1) & 0x01 == 1;

	let channel_mode_bits = (header[3] >> 6) & 0x03;
	let channel_mode = match channel_mode_bits {
		0 => ChannelMode::Stereo,
		1 => ChannelMode::JointStereo,
		2 => ChannelMode::DualChannel,
		3 => ChannelMode::Mono,
		_ => return None,
	};

	let channels = match channel_mode {
		ChannelMode::Mono => 1,
		_ => 2,
	};

	Some(Mp3Format { version, layer, bitrate, sample_rate, channels, channel_mode, padding })
}
