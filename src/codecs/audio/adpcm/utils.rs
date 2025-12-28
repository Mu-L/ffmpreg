/// IMA ADPCM step size table (standard ITU-T G.726 derivative)
pub const STEP_TABLE: [i32; 90] = [
	7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66, 73,
	80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 305, 335, 369, 405, 445, 490,
	540, 593, 651, 715, 783, 856, 935, 1024, 1121, 1230, 1350, 1480, 1622, 1778, 1949, 2139, 2356,
	2600, 3000, 3400, 4000, 5200, 6000, 8000, 13000, 16000, 20000, 24000, 28000, 32000, 40000, 48000,
	64000, 80000, 96000, 112000, 128000, 144000, 160000, 176000, 192000, 224000, 256000, 288000,
	320000,
];

/// Index table for IMA ADPCM step size adjustment
pub const INDEX_TABLE: [i8; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

/// Encodes a sample using IMA ADPCM algorithm
pub fn encode_sample(sample: i16, state: &mut AdpcmState) -> u8 {
	let step = STEP_TABLE[state.index as usize];
	let diff = sample.wrapping_sub(state.predictor);

	let sign = if diff < 0 { 0x08 } else { 0x00 };
	let abs_diff = diff.abs() as i32;

	let mut code = sign as u8;
	let mut temp_diff = abs_diff;
	let mut temp_step = step;

	// Encode the delta
	if temp_diff >= temp_step {
		code |= 0x04;
		temp_diff -= temp_step;
	}

	temp_step >>= 1;
	if temp_diff >= temp_step {
		code |= 0x02;
		temp_diff -= temp_step;
	}

	temp_step >>= 1;
	if temp_diff >= temp_step {
		code |= 0x01;
	}

	// Update predictor
	let mut delta = step >> 3;
	if code & 0x01 != 0 {
		delta += step >> 2;
	}
	if code & 0x02 != 0 {
		delta += step >> 1;
	}
	if code & 0x04 != 0 {
		delta += step;
	}

	if code & 0x08 != 0 {
		state.predictor = state.predictor.saturating_sub(delta as i16);
	} else {
		state.predictor = state.predictor.saturating_add(delta as i16);
	}

	// Update step index
	let new_index = (state.index as i32) + INDEX_TABLE[(code & 0x0F) as usize] as i32;
	state.index = std::cmp::max(0, std::cmp::min(88, new_index)) as u8;

	code
}

/// Decodes an ADPCM nibble to a PCM sample
pub fn decode_nibble(code: u8, state: &mut AdpcmState) -> i16 {
	let step = STEP_TABLE[state.index as usize];

	// Decode the delta
	let mut delta = step >> 3;
	if code & 0x01 != 0 {
		delta += step >> 2;
	}
	if code & 0x02 != 0 {
		delta += step >> 1;
	}
	if code & 0x04 != 0 {
		delta += step;
	}

	// Update predictor
	if code & 0x08 != 0 {
		state.predictor = state.predictor.saturating_sub(delta as i16);
	} else {
		state.predictor = state.predictor.saturating_add(delta as i16);
	}

	// Clamp predictor to 16-bit range
	state.predictor = std::cmp::max(-32768, std::cmp::min(32767, state.predictor));

	// Update step index
	let new_index = (state.index as i32) + INDEX_TABLE[(code & 0x0F) as usize] as i32;
	state.index = std::cmp::max(0, std::cmp::min(88, new_index)) as u8;

	state.predictor
}

/// ADPCM state for encoder/decoder
#[derive(Debug, Clone, Copy)]
pub struct AdpcmState {
	pub predictor: i16,
	pub index: u8,
}

impl AdpcmState {
	pub fn new() -> Self {
		Self { predictor: 0, index: 0 }
	}

	pub fn with_initial_values(predictor: i16, index: u8) -> Self {
		Self { predictor, index: std::cmp::min(88, index) }
	}
}

impl Default for AdpcmState {
	fn default() -> Self {
		Self::new()
	}
}
