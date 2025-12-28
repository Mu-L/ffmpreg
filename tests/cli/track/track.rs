#[cfg(test)]
mod tests {
	use ffmpreg::cli::track;
	use std::collections::HashMap;

	#[test]
	fn parse_track_default_is_all() {
		let kv = HashMap::new();
		let result = track::parse_track(&kv);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), track::Track::All);
	}

	#[test]
	fn parse_track_explicit_all() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "all".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), track::Track::All);
	}

	#[test]
	fn parse_track_asterisk_is_all() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "*".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), track::Track::All);
	}

	#[test]
	fn parse_track_zero() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "0".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), track::Track::One(0));
	}

	#[test]
	fn parse_track_positive_integer() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "1".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), track::Track::One(1));
	}

	#[test]
	fn parse_track_large_integer() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "999".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), track::Track::One(999));
	}

	#[test]
	fn parse_track_invalid_non_numeric() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "invalid".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_err());
		let err = result.unwrap_err();
		assert_eq!(err.message().unwrap(), &"invalid track specifier invalid".to_string());
	}

	#[test]
	fn parse_track_negative_integer_fails() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "-1".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_err());
	}

	#[test]
	fn parse_track_empty_string_fails() {
		let mut kv = HashMap::new();
		kv.insert("track".to_string(), "".to_string());

		let result = track::parse_track(&kv);

		assert!(result.is_err());
	}

	#[test]
	fn track_resolve_one_within_bounds() {
		let track = track::Track::One(0);
		let result = track.resolve(5);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), vec![0]);
	}

	#[test]
	fn track_resolve_one_at_upper_bound() {
		let track = track::Track::One(4);
		let result = track.resolve(5);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), vec![4]);
	}

	#[test]
	fn track_resolve_one_out_of_bounds() {
		let track = track::Track::One(5);
		let result = track.resolve(5);

		assert!(result.is_err());
		let err = result.unwrap_err();
		assert!(err.message().unwrap().contains("out of bounds"));
	}

	#[test]
	fn track_resolve_all_with_multiple_tracks() {
		let track = track::Track::All;
		let result = track.resolve(5);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), vec![0, 1, 2, 3, 4]);
	}

	#[test]
	fn track_resolve_all_with_one_track() {
		let track = track::Track::All;
		let result = track.resolve(1);

		assert!(result.is_ok());
		assert_eq!(result.unwrap(), vec![0]);
	}

	#[test]
	fn track_resolve_all_with_zero_tracks() {
		let track = track::Track::All;
		let result = track.resolve(0);

		assert!(result.is_err());
		let err = result.unwrap_err();
		assert_eq!(err.message().unwrap(), &"no streams available".to_string());
	}
}
