use super::tables::PI;

thread_local! {
	static SYNTH_WINDOW: [f32; 512] = {
		let mut window = [0.0f32; 512];
		for i in 0..512 {
			let n = i as f32;
			let val = (PI / 64.0 * (n + 0.5)).cos();
			window[i] = val;
		}
		window
	};
}

pub struct SynthesisFilterbank {
	v_vec: [[f32; 1024]; 2],
	v_offset: [usize; 2],
}

impl Default for SynthesisFilterbank {
	fn default() -> Self {
		Self::new()
	}
}

impl SynthesisFilterbank {
	pub fn new() -> Self {
		Self { v_vec: [[0.0; 1024]; 2], v_offset: [0; 2] }
	}

	pub fn process(&mut self, samples: &[f32; 32], channel: usize, output: &mut [f32; 32]) {
		let v = &mut self.v_vec[channel];
		let offset = &mut self.v_offset[channel];

		*offset = (*offset + 1024 - 64) % 1024;

		for i in 0..64 {
			let mut sum = 0.0f32;
			for k in 0..32 {
				let angle = PI / 64.0 * (16.0 + i as f32) * (2.0 * k as f32 + 1.0);
				sum += samples[k] * angle.cos();
			}
			v[(*offset + i) % 1024] = sum;
		}

		SYNTH_WINDOW.with(|window| {
			for j in 0..32 {
				let mut sum = 0.0f32;
				for i in 0..16 {
					let idx1 = (*offset + j + i * 64) % 1024;
					let idx2 = (*offset + j + 32 + i * 64) % 1024;

					let win_idx1 = j + i * 64;
					let win_idx2 = j + 32 + i * 64;

					if win_idx1 < 512 && win_idx2 < 512 {
						sum += v[idx1] * window[win_idx1];
						sum += v[idx2] * window[win_idx2];
					}
				}
				output[j] = sum;
			}
		})
	}

	pub fn reset(&mut self) {
		self.v_vec = [[0.0; 1024]; 2];
		self.v_offset = [0; 2];
	}
}

pub fn imdct_36(input: &[f32; 18], output: &mut [f32; 36]) {
	for i in 0..36 {
		let mut sum = 0.0f32;
		for k in 0..18 {
			let angle = PI / 72.0 * (2.0 * i as f32 + 1.0 + 18.0) * (2.0 * k as f32 + 1.0);
			sum += input[k] * angle.cos();
		}
		output[i] = sum;
	}
}

pub fn imdct_12(input: &[f32; 6], output: &mut [f32; 12]) {
	for i in 0..12 {
		let mut sum = 0.0f32;
		for k in 0..6 {
			let angle = PI / 24.0 * (2.0 * i as f32 + 1.0 + 6.0) * (2.0 * k as f32 + 1.0);
			sum += input[k] * angle.cos();
		}
		output[i] = sum;
	}
}

#[rustfmt::skip]
const WINDOW_LONG: [f32; 36] = [
	0.043619387, 0.130526192, 0.216439614, 0.300705799, 0.382683432, 0.461748613,
	0.537299608, 0.608761429, 0.675590208, 0.737277337, 0.793353340, 0.843391446,
	0.887010833, 0.923879533, 0.953716951, 0.976296007, 0.991444861, 0.999048222,
	0.999048222, 0.991444861, 0.976296007, 0.953716951, 0.923879533, 0.887010833,
	0.843391446, 0.793353340, 0.737277337, 0.675590208, 0.608761429, 0.537299608,
	0.461748613, 0.382683432, 0.300705799, 0.216439614, 0.130526192, 0.043619387,
];

#[rustfmt::skip]
const WINDOW_SHORT: [f32; 12] = [
	0.130526192, 0.382683432, 0.608761429, 0.793353340, 0.923879533, 0.991444861,
	0.991444861, 0.923879533, 0.793353340, 0.608761429, 0.382683432, 0.130526192,
];

#[rustfmt::skip]
const WINDOW_START: [f32; 36] = [
	0.043619387, 0.130526192, 0.216439614, 0.300705799, 0.382683432, 0.461748613,
	0.537299608, 0.608761429, 0.675590208, 0.737277337, 0.793353340, 0.843391446,
	0.887010833, 0.923879533, 0.953716951, 0.976296007, 0.991444861, 0.999048222,
	1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
	0.991444861, 0.923879533, 0.793353340, 0.608761429, 0.382683432, 0.130526192,
	0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

#[rustfmt::skip]
const WINDOW_STOP: [f32; 36] = [
	0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
	0.130526192, 0.382683432, 0.608761429, 0.793353340, 0.923879533, 0.991444861,
	1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
	0.999048222, 0.991444861, 0.976296007, 0.953716951, 0.923879533, 0.887010833,
	0.843391446, 0.793353340, 0.737277337, 0.675590208, 0.608761429, 0.537299608,
	0.461748613, 0.382683432, 0.300705799, 0.216439614, 0.130526192, 0.043619387,
];

pub fn apply_window(samples: &mut [f32; 36], block_type: u8, mixed_block: bool) {
	let window = match block_type {
		0 => &WINDOW_LONG,
		1 => &WINDOW_START,
		2 if !mixed_block => {
			for i in 0..3 {
				for j in 0..12 {
					samples[i * 12 + j] *= WINDOW_SHORT[j];
				}
			}
			return;
		}
		3 => &WINDOW_STOP,
		_ => &WINDOW_LONG,
	};

	for i in 0..36 {
		samples[i] *= window[i];
	}
}
