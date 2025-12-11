pub mod decode;
pub mod encode;

pub use decode::AdpcmDecoder;
pub use encode::AdpcmEncoder;

const INDEX_TABLE: [i8; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

const STEP_TABLE: [i16; 89] = [
	7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66, 73,
	80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449, 494,
	544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272, 2499,
	2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487,
	12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

#[derive(Debug, Clone)]
pub struct AdpcmState {
	pub predictor: i16,
	pub step_index: i8,
}

impl Default for AdpcmState {
	fn default() -> Self {
		Self { predictor: 0, step_index: 0 }
	}
}

impl AdpcmState {
	pub fn new() -> Self {
		Self::default()
	}

	fn decode_sample(&mut self, nibble: u8) -> i16 {
		let step = STEP_TABLE[self.step_index as usize];
		let nibble = nibble & 0x0F;

		let mut diff = step >> 3;
		if nibble & 4 != 0 {
			diff += step;
		}
		if nibble & 2 != 0 {
			diff += step >> 1;
		}
		if nibble & 1 != 0 {
			diff += step >> 2;
		}

		if nibble & 8 != 0 {
			self.predictor = self.predictor.saturating_sub(diff);
		} else {
			self.predictor = self.predictor.saturating_add(diff);
		}

		self.step_index = (self.step_index + INDEX_TABLE[nibble as usize]).clamp(0, 88);

		self.predictor
	}

	fn encode_sample(&mut self, sample: i16) -> u8 {
		let step = STEP_TABLE[self.step_index as usize];
		let diff = sample - self.predictor;

		let mut nibble: u8 = 0;
		let mut vpdiff = step >> 3;

		if diff < 0 {
			nibble = 8;
		}

		let abs_diff = diff.abs();

		if abs_diff >= step {
			nibble |= 4;
			vpdiff += step;
		}
		if abs_diff >= step >> 1 {
			nibble |= 2;
			vpdiff += step >> 1;
		}
		if abs_diff >= step >> 2 {
			nibble |= 1;
			vpdiff += step >> 2;
		}

		if nibble & 8 != 0 {
			self.predictor = self.predictor.saturating_sub(vpdiff);
		} else {
			self.predictor = self.predictor.saturating_add(vpdiff);
		}

		self.step_index = (self.step_index + INDEX_TABLE[nibble as usize]).clamp(0, 88);

		nibble
	}
}
