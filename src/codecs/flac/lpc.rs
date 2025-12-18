pub fn compute_autocorrelation(samples: &[i32], order: usize) -> Vec<f64> {
	let mut autocorr = vec![0.0f64; order + 1];
	let n = samples.len();

	for lag in 0..=order {
		let mut sum = 0.0f64;
		for i in lag..n {
			sum += samples[i] as f64 * samples[i - lag] as f64;
		}
		autocorr[lag] = sum;
	}

	autocorr
}

pub fn levinson_durbin(autocorr: &[f64], order: usize) -> Option<(Vec<f64>, f64)> {
	if autocorr.is_empty() || autocorr[0] <= 0.0 {
		return None;
	}

	let mut lpc = vec![0.0f64; order];
	let mut error = autocorr[0];

	for i in 0..order {
		let mut lambda = 0.0f64;
		for j in 0..i {
			lambda += lpc[j] * autocorr[i - j];
		}
		lambda = (autocorr[i + 1] - lambda) / error;

		for j in 0..(i / 2 + 1) {
			let tmp = lpc[j];
			if j < i - j {
				lpc[j] = tmp + lambda * lpc[i - 1 - j];
				lpc[i - 1 - j] = lpc[i - 1 - j] + lambda * tmp;
			} else {
				lpc[j] = tmp + lambda * lpc[i - 1 - j];
			}
		}
		lpc[i] = lambda;

		error *= 1.0 - lambda * lambda;
		if error <= 0.0 {
			return None;
		}
	}

	Some((lpc, error))
}

pub fn quantize_lpc_coefficients(lpc: &[f64], precision: u8) -> (Vec<i32>, i8) {
	let max_coef = lpc.iter().map(|c| c.abs()).fold(0.0f64, f64::max);

	if max_coef == 0.0 {
		return (vec![0i32; lpc.len()], 0);
	}

	let shift = (precision as i32 - 1) - (max_coef.log2().ceil() as i32);
	let shift = shift.clamp(-16, 15) as i8;

	let scale = (1i64 << shift.max(0)) as f64;
	let quantized: Vec<i32> = lpc.iter().map(|&c| (c * scale).round() as i32).collect();

	(quantized, shift)
}

pub fn apply_lpc_prediction(samples: &[i32], lpc: &[i32], shift: i8, output: &mut [i32]) {
	let order = lpc.len();

	for i in 0..order.min(output.len()) {
		output[i] = samples[i];
	}

	for i in order..samples.len() {
		let mut prediction = 0i64;
		for (j, &coef) in lpc.iter().enumerate() {
			prediction += coef as i64 * samples[i - 1 - j] as i64;
		}
		if shift >= 0 {
			prediction >>= shift;
		} else {
			prediction <<= -shift;
		}
		output[i] = samples[i] - prediction as i32;
	}
}

pub fn restore_lpc_signal(
	residuals: &[i32],
	warmup: &[i32],
	lpc: &[i32],
	shift: i8,
	output: &mut Vec<i32>,
) {
	let order = lpc.len();

	output.extend_from_slice(warmup);

	for i in 0..residuals.len() {
		let mut prediction = 0i64;
		for (j, &coef) in lpc.iter().enumerate() {
			prediction += coef as i64 * output[order - 1 - j + i] as i64;
		}
		if shift >= 0 {
			prediction >>= shift;
		} else {
			prediction <<= -shift;
		}
		output.push(residuals[i] + prediction as i32);
	}
}

pub fn apply_fixed_prediction(samples: &[i32], order: usize, output: &mut [i32]) {
	match order {
		0 => {
			output.copy_from_slice(samples);
		}
		1 => {
			if !samples.is_empty() {
				output[0] = samples[0];
			}
			for i in 1..samples.len() {
				output[i] = samples[i] - samples[i - 1];
			}
		}
		2 => {
			for i in 0..2.min(samples.len()) {
				output[i] = samples[i];
			}
			for i in 2..samples.len() {
				output[i] = samples[i] - 2 * samples[i - 1] + samples[i - 2];
			}
		}
		3 => {
			for i in 0..3.min(samples.len()) {
				output[i] = samples[i];
			}
			for i in 3..samples.len() {
				output[i] = samples[i] - 3 * samples[i - 1] + 3 * samples[i - 2] - samples[i - 3];
			}
		}
		4 => {
			for i in 0..4.min(samples.len()) {
				output[i] = samples[i];
			}
			for i in 4..samples.len() {
				output[i] = samples[i] - 4 * samples[i - 1] + 6 * samples[i - 2] - 4 * samples[i - 3]
					+ samples[i - 4];
			}
		}
		_ => {
			output.copy_from_slice(samples);
		}
	}
}

pub fn restore_fixed_signal(
	residuals: &[i32],
	warmup: &[i32],
	order: usize,
	output: &mut Vec<i32>,
) {
	output.extend_from_slice(warmup);

	match order {
		0 => {
			output.extend_from_slice(residuals);
		}
		1 => {
			for &residual in residuals {
				let last = *output.last().unwrap_or(&0);
				output.push(residual + last);
			}
		}
		2 => {
			for &residual in residuals {
				let len = output.len();
				let prediction = 2 * output[len - 1] - output[len - 2];
				output.push(residual + prediction);
			}
		}
		3 => {
			for &residual in residuals {
				let len = output.len();
				let prediction = 3 * output[len - 1] - 3 * output[len - 2] + output[len - 3];
				output.push(residual + prediction);
			}
		}
		4 => {
			for &residual in residuals {
				let len = output.len();
				let prediction =
					4 * output[len - 1] - 6 * output[len - 2] + 4 * output[len - 3] - output[len - 4];
				output.push(residual + prediction);
			}
		}
		_ => {}
	}
}
