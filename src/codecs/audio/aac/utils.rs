pub const SAMPLE_RATES: &[u32] =
	&[96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000];

pub fn get_sample_rate_index(sample_rate: u32) -> Option<u8> {
	SAMPLE_RATES.iter().position(|&sr| sr == sample_rate).map(|i| i as u8)
}

pub fn get_sample_rate_from_index(index: u8) -> Option<u32> {
	SAMPLE_RATES.get(index as usize).copied()
}

pub const CHANNEL_CONFIGURATIONS: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 8];

pub fn is_valid_channel_config(config: u8) -> bool {
	// Valid channel configs: 0 (specific), 1-6 (mono to 5.1), 7 (7.1) sometimes reserved
	// Accept 0-7 for now, though some files may use non-standard values
	config < 8
}

pub fn get_channels_from_config(config: u8) -> Option<u8> {
	match config {
		0 => Some(2), // Default to stereo if not specified
		1..=7 => Some(config),
		_ => None,
	}
}

pub const FRAME_SIZE_SAMPLES: usize = 1024;

pub fn calculate_frame_size_bytes(sample_rate: u32, bit_rate: u32) -> usize {
	((bit_rate as usize * FRAME_SIZE_SAMPLES) / (8 * sample_rate as usize)) + 1
}
