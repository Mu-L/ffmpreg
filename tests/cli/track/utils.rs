#[cfg(test)]
mod tests {
	use ffmpreg::cli::track::utils;

	#[test]
	fn parse_kv_single_key_value_pair() {
		let tokens = vec!["codec=h264".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("codec"), Some(&"h264".to_string()));
	}

	#[test]
	fn parse_kv_multiple_key_value_pairs() {
		let tokens = vec!["codec=h264".to_string(), "bitrate=5000k".to_string(), "fps=30".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("codec"), Some(&"h264".to_string()));
		assert_eq!(kv.get("bitrate"), Some(&"5000k".to_string()));
		assert_eq!(kv.get("fps"), Some(&"30".to_string()));
	}

	#[test]
	fn parse_kv_token_without_equals_defaults_to_true() {
		let tokens = vec!["default".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("default"), Some(&"true".to_string()));
	}

	#[test]
	fn parse_kv_empty_value_is_preserved() {
		let tokens = vec!["key=".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("key"), Some(&"".to_string()));
	}

	#[test]
	fn parse_kv_duplicate_keys_last_wins() {
		let tokens = vec!["codec=h264".to_string(), "codec=vp9".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("codec"), Some(&"vp9".to_string()));
	}

	#[test]
	fn parse_kv_empty_tokens_returns_empty_map() {
		let tokens: Vec<String> = vec![];
		let kv = utils::parse_kv(&tokens);

		assert!(kv.is_empty());
	}

	#[test]
	fn parse_kv_mixed_tokens_with_and_without_equals() {
		let tokens = vec!["codec=h264".to_string(), "default".to_string(), "shift=100".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("codec"), Some(&"h264".to_string()));
		assert_eq!(kv.get("default"), Some(&"true".to_string()));
		assert_eq!(kv.get("shift"), Some(&"100".to_string()));
	}

	#[test]
	fn parse_kv_value_with_equals_sign() {
		let tokens = vec!["filter=scale=1920:1080".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("filter"), Some(&"scale=1920:1080".to_string()));
	}

	#[test]
	fn parse_kv_whitespace_in_values_preserved() {
		let tokens = vec!["description=hello world".to_string()];
		let kv = utils::parse_kv(&tokens);

		assert_eq!(kv.get("description"), Some(&"hello world".to_string()));
	}

	#[test]
	fn resolve_boolean_value_key_exists() {
		use std::collections::HashMap;

		let mut kv = HashMap::new();
		kv.insert("default".to_string(), "true".to_string());

		let result = utils::resolve_boolean_value(&kv, "default");
		assert_eq!(result, Some("true".to_string()));
	}

	#[test]
	fn resolve_boolean_value_key_not_exists() {
		use std::collections::HashMap;

		let kv = HashMap::new();
		let result = utils::resolve_boolean_value(&kv, "default");
		assert_eq!(result, None);
	}
}
