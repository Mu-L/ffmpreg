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

	pub fn encode_sample(&mut self, sample: i16) -> u8 {
		let step = super::table::STEP_TABLE[self.index as usize];
		let diff = sample.wrapping_sub(self.predictor);

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
			self.predictor = self.predictor.saturating_sub(delta as i16);
		} else {
			self.predictor = self.predictor.saturating_add(delta as i16);
		}

		// Update step index
		let new_index = (self.index as i32) + super::table::INDEX_TABLE[(code & 0x0F) as usize] as i32;
		self.index = std::cmp::max(0, std::cmp::min(88, new_index)) as u8;

		code
	}

	pub fn decode_nibble(&mut self, code: u8) -> i16 {
		let step = super::table::STEP_TABLE[self.index as usize];

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
			self.predictor = self.predictor.saturating_sub(delta as i16);
		} else {
			self.predictor = self.predictor.saturating_add(delta as i16);
		}

		// Clamp predictor to 16-bit range
		self.predictor = std::cmp::max(-32768, std::cmp::min(32767, self.predictor));

		// Update step index
		let new_index = (self.index as i32) + super::table::INDEX_TABLE[(code & 0x0F) as usize] as i32;
		self.index = std::cmp::max(0, std::cmp::min(88, new_index)) as u8;

		self.predictor
	}
}

impl Default for AdpcmState {
	fn default() -> Self {
		Self::new()
	}
}
