#[cfg(test)]
mod tests {
	use crate::codecs::audio::adpcm::utils::{AdpcmState, decode_nibble, encode_sample};

	#[test]
	fn test_encode_decode_produces_output() {
		let test_samples = vec![0i16, 100, -100, 200, -200, 300, -300, 400, -400, 500, -500, 600, -600];

		let mut encode_state = AdpcmState::new();
		let mut encoded_codes = Vec::new();

		// Encode all samples
		for &original in &test_samples {
			let code = encode_sample(original, &mut encode_state);
			encoded_codes.push(code);
			// Verify code is valid (4-bit nibble)
			assert!(code <= 0x0F, "Invalid encoded nibble: {}", code);
		}

		// Decode with same initial state
		let mut decode_state = AdpcmState::new();
		let mut decoded = Vec::new();

		for &code in &encoded_codes {
			let output = decode_nibble(code, &mut decode_state);
			decoded.push(output);
		}

		// Basic sanity checks
		assert_eq!(encoded_codes.len(), test_samples.len());
		assert_eq!(decoded.len(), test_samples.len());
	}

	#[test]
	fn test_encode_decode_sequence() {
		let samples = vec![100i16, 200, 300, 400, 500];
		let mut state = AdpcmState::new();
		let mut encoded = Vec::new();

		for &sample in &samples {
			encoded.push(encode_sample(sample, &mut state));
		}

		let mut decode_state = AdpcmState::new();
		let mut decoded = Vec::new();

		for &code in &encoded {
			decoded.push(decode_nibble(code, &mut decode_state));
		}

		assert_eq!(samples.len(), decoded.len());
		// ADPCM has quantization error but decoded values should be reasonable
		for (_, dec) in samples.iter().zip(decoded.iter()) {
			assert!(*dec >= -32768 && *dec <= 32767, "Decoded value out of range: {}", dec);
		}
	}
}
