pub mod channel_mixer;
pub mod eq;
pub mod fade;
pub mod gain;
pub mod highpass;
pub mod lowpass;
pub mod normalize;
pub mod peak_limiter;
pub mod resample;
pub mod rms_limiter;
pub mod video;
pub mod volume;

pub use channel_mixer::{ChannelLayout, ChannelMixer};
pub use eq::{EqBand, Equalizer, FilterType};
pub use fade::{Crossfade, FadeIn, FadeOut};
pub use gain::Gain;
pub use highpass::Highpass;
pub use lowpass::Lowpass;
pub use normalize::Normalize;
pub use peak_limiter::PeakLimiter;
pub use resample::Resample;
pub use rms_limiter::RmsLimiter;
pub use video::{
	Blur, Brightness, Contrast, Crop, Flip, FlipDirection, FrameRateConverter, Pad, Rotate,
	RotateAngle, Scale, ScaleMode,
};
pub use volume::Volume;

use crate::core::Transform;
use crate::io::{IoError, IoErrorKind, IoResult};

pub fn parse_transform(spec: &str) -> IoResult<Box<dyn Transform>> {
	let parts: Vec<&str> = spec.splitn(2, '=').collect();
	let name = parts[0];

	match name {
		"volume" => {
			let factor = parts
				.get(1)
				.ok_or_else(|| {
					IoError::with_message(
						IoErrorKind::InvalidData,
						"volume requires a value (e.g., volume=2.0)",
					)
				})?
				.parse::<f32>()
				.map_err(|_| {
					IoError::with_message(IoErrorKind::InvalidData, "volume value must be a number")
				})?;
			Ok(Box::new(Volume::new(factor)))
		}
		"normalize" => {
			let peak = parts.get(1).map(|v| v.parse::<f32>().unwrap_or(0.95)).unwrap_or(0.95);
			Ok(Box::new(Normalize::new(peak)))
		}
		"highpass" => {
			let cutoff = parts
				.get(1)
				.ok_or_else(|| {
					IoError::with_message(
						IoErrorKind::InvalidData,
						"highpass requires cutoff frequency (e.g., highpass=200)",
					)
				})?
				.parse::<f32>()
				.map_err(|_| {
					IoError::with_message(IoErrorKind::InvalidData, "highpass cutoff must be a number")
				})?;
			Ok(Box::new(Highpass::new(cutoff)))
		}
		"lowpass" => {
			let cutoff = parts
				.get(1)
				.ok_or_else(|| {
					IoError::with_message(
						IoErrorKind::InvalidData,
						"lowpass requires cutoff frequency (e.g., lowpass=5000)",
					)
				})?
				.parse::<f32>()
				.map_err(|_| {
					IoError::with_message(IoErrorKind::InvalidData, "lowpass cutoff must be a number")
				})?;
			Ok(Box::new(Lowpass::new(cutoff)))
		}
		"fadein" => {
			let duration_ms = parts
				.get(1)
				.ok_or_else(|| {
					IoError::with_message(
						IoErrorKind::InvalidData,
						"fadein requires duration in ms (e.g., fadein=1000)",
					)
				})?
				.parse::<f32>()
				.map_err(|_| {
					IoError::with_message(IoErrorKind::InvalidData, "fadein duration must be a number")
				})?;
			Ok(Box::new(FadeIn::new(duration_ms, 44100)))
		}
		"resample" => {
			let rate = parts
				.get(1)
				.ok_or_else(|| {
					IoError::with_message(
						IoErrorKind::InvalidData,
						"resample requires target rate (e.g., resample=48000)",
					)
				})?
				.parse::<u32>()
				.map_err(|_| {
					IoError::with_message(IoErrorKind::InvalidData, "resample rate must be a number")
				})?;
			Ok(Box::new(Resample::new(rate)))
		}
		"mono" => Ok(Box::new(ChannelMixer::stereo_to_mono())),
		"stereo" => Ok(Box::new(ChannelMixer::mono_to_stereo())),
		"eq3" => {
			let params = parts.get(1).unwrap_or(&"0,0,0");
			let values: Vec<f32> = params.split(',').filter_map(|v| v.parse::<f32>().ok()).collect();
			let bass = values.first().copied().unwrap_or(0.0);
			let mid = values.get(1).copied().unwrap_or(0.0);
			let treble = values.get(2).copied().unwrap_or(0.0);
			Ok(Box::new(Equalizer::three_band(bass, mid, treble)))
		}
		"peak_limiter" | "limiter" => {
			let threshold_db = parts.get(1).map(|v| v.parse::<f32>().unwrap_or(-1.0)).unwrap_or(-1.0);
			Ok(Box::new(PeakLimiter::new(threshold_db)))
		}
		"rms_limiter" => {
			let threshold_db = parts.get(1).map(|v| v.parse::<f32>().unwrap_or(-10.0)).unwrap_or(-10.0);
			Ok(Box::new(RmsLimiter::new(threshold_db, 50.0, 44100)))
		}
		_ => Err(IoError::with_message(IoErrorKind::InvalidData, "unknown transform")),
	}
}

pub struct TransformChain {
	transforms: Vec<Box<dyn Transform>>,
}

impl TransformChain {
	pub fn new() -> Self {
		Self { transforms: Vec::new() }
	}

	pub fn add(&mut self, transform: Box<dyn Transform>) {
		self.transforms.push(transform);
	}

	pub fn is_empty(&self) -> bool {
		self.transforms.is_empty()
	}
}

impl Default for TransformChain {
	fn default() -> Self {
		Self::new()
	}
}

impl Transform for TransformChain {
	fn apply(&mut self, mut frame: crate::core::Frame) -> IoResult<crate::core::Frame> {
		for transform in &mut self.transforms {
			frame = transform.apply(frame)?;
		}
		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"chain"
	}
}
