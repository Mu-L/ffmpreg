pub mod audio;
pub mod subtitle;
pub mod track;
pub mod transform;
pub mod video;
use rustc_hash::FxHashMap;

pub use track::Track;

pub use audio::{AudioConfig, parse_audio};
pub use subtitle::{SubtitleConfig, parse_subtitle};
pub use transform::{TransformConfig, parse_transform};
pub use video::{VideoConfig, parse_video};

pub fn parse_flags(tokens: Vec<String>, boolean_value: bool) -> FxHashMap<String, String> {
	let mut map = FxHashMap::default();
	for token in tokens {
		if let Some((key, value)) = token.split_once('=') {
			map.insert(key.to_string(), value.to_string());
		} else if boolean_value {
			map.insert(token, "true".to_string());
		}
	}
	map
}
