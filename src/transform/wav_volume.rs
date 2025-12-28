use crate::core::{AudioFormat, Frame, Transform};
use crate::io::Result;

pub struct WavVolume {
	factor: f32,
}

impl WavVolume {
	pub fn new(factor: f32) -> Self {
		Self { factor }
	}
}

impl Transform for WavVolume {
	fn apply(&mut self, mut frame: Frame) -> Result<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			match audio_frame.format {
				AudioFormat::PCM16 => {
					self.apply_pcm16(&mut audio_frame.data);
				}
				AudioFormat::PCM24 => {
					self.apply_pcm24(&mut audio_frame.data);
				}
				AudioFormat::PCM32 => {
					self.apply_pcm32(&mut audio_frame.data);
				}
				_ => {}
			}
		}
		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"wav_volume"
	}
}

impl WavVolume {
	fn apply_pcm16(&self, data: &mut [u8]) {
		let samples = data.len() / 2;
		for i in 0..samples {
			let offset = i * 2;
			let sample = i16::from_le_bytes([data[offset], data[offset + 1]]);
			let amplified = (sample as f32 * self.factor).clamp(-32768.0, 32767.0) as i16;
			let bytes = amplified.to_le_bytes();
			data[offset] = bytes[0];
			data[offset + 1] = bytes[1];
		}
	}

	fn apply_pcm24(&self, data: &mut [u8]) {
		let samples = data.len() / 3;
		for i in 0..samples {
			let offset = i * 3;
			let sample = Self::read_i24_le(&data[offset..offset + 3]);
			let amplified = (sample as f32 * self.factor).clamp(-8388608.0, 8388607.0) as i32;
			Self::write_i24_le(&mut data[offset..offset + 3], amplified);
		}
	}

	fn apply_pcm32(&self, data: &mut [u8]) {
		let samples = data.len() / 4;
		for i in 0..samples {
			let offset = i * 4;
			let sample =
				f32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
			let amplified = (sample * self.factor).clamp(-1.0, 1.0);
			let bytes = amplified.to_le_bytes();
			data[offset] = bytes[0];
			data[offset + 1] = bytes[1];
			data[offset + 2] = bytes[2];
			data[offset + 3] = bytes[3];
		}
	}

	fn read_i24_le(bytes: &[u8]) -> i32 {
		let b0 = bytes[0] as u32;
		let b1 = bytes[1] as u32;
		let b2 = bytes[2] as i32;

		let value = (b0 | (b1 << 8) | ((b2 as u32) << 16)) as i32;
		if value & 0x800000 != 0 { value | 0xff000000 } else { value }
	}

	fn write_i24_le(bytes: &mut [u8], value: i32) {
		bytes[0] = (value & 0xff) as u8;
		bytes[1] = ((value >> 8) & 0xff) as u8;
		bytes[2] = ((value >> 16) & 0xff) as u8;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_read_write_i24_le() {
		let mut bytes = [0u8; 3];
		WavVolume::write_i24_le(&mut bytes, 1000);
		let read_back = WavVolume::read_i24_le(&bytes);
		assert_eq!(read_back, 1000);

		WavVolume::write_i24_le(&mut bytes, -5000);
		let read_back = WavVolume::read_i24_le(&bytes);
		assert_eq!(read_back, -5000);
	}

	#[test]
	fn test_pcm16_volume() {
		let mut transform = WavVolume::new(0.5);
		let data = vec![0x00, 0x40, 0x00, 0x80];
		let frame_audio = crate::core::FrameAudio::new(data, 44100, 2, AudioFormat::PCM16);
		let time = crate::core::Time::new(1, 44100);
		let frame = Frame::new_audio(frame_audio, time, 0, 0);
		let result = transform.apply(frame).unwrap();
		if let Some(audio) = result.audio() {
			assert_eq!(audio.data.len(), 4);
		}
	}
}
