#[cfg(test)]
mod tests {
	use ffmpreg::cli::track::{Track, subtitle};

	#[test]
	fn parse_subtitle_track_options_with_track() {
		let tokens = vec!["track=0".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(0)));
		assert!(opts.lang.is_none());
	}

	#[test]
	fn parse_subtitle_track_options_with_language() {
		let tokens = vec!["lang=en".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert!(opts.track.is_none());
		assert_eq!(opts.lang, Some("en".to_string()));
	}

	#[test]
	fn parse_subtitle_track_options_with_track_and_language() {
		let tokens = vec!["track=0".to_string(), "lang=en".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(0)));
		assert_eq!(opts.lang, Some("en".to_string()));
	}

	#[test]
	fn parse_subtitle_track_options_missing_selector_fails() {
		let tokens: Vec<String> = vec![];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_err());
		let err = result.unwrap_err();
		assert_eq!(err.message().unwrap(), &"subtitle missing track or lang".to_string());
	}

	#[test]
	fn parse_subtitle_track_options_track_all() {
		let tokens = vec!["track=all".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::All));
	}

	#[test]
	fn parse_subtitle_track_options_track_asterisk() {
		let tokens = vec!["track=*".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::All));
	}

	#[test]
	fn parse_subtitle_track_options_invalid_track_fails() {
		let tokens = vec!["track=invalid".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_err());
	}

	#[test]
	fn parse_subtitle_track_options_language_code() {
		let tokens = vec!["lang=fr".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.lang, Some("fr".to_string()));
	}

	#[test]
	fn parse_subtitle_track_options_language_extended() {
		let tokens = vec!["lang=en-US".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.lang, Some("en-US".to_string()));
	}

	#[test]
	fn parse_subtitle_track_options_with_codec() {
		let tokens = vec!["track=0".to_string(), "codec=subrip".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(0)));
	}

	#[test]
	fn parse_subtitle_track_options_with_font_size() {
		let tokens = vec!["lang=en".to_string(), "font_size=20".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_color() {
		let tokens = vec!["track=0".to_string(), "color=white".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_position() {
		let tokens = vec!["lang=en".to_string(), "position=bottom".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_shift() {
		let tokens = vec!["track=0".to_string(), "shift=500".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_default() {
		let tokens = vec!["lang=en".to_string(), "default=yes".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_fps() {
		let tokens = vec!["track=0".to_string(), "fps=25".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_encoding() {
		let tokens = vec!["lang=en".to_string(), "encoding=utf-8".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_translate() {
		let tokens = vec!["track=0".to_string(), "translate=fr".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_with_multiple_options() {
		let tokens = vec![
			"track=1".to_string(),
			"codec=ass".to_string(),
			"font_size=24".to_string(),
			"color=yellow".to_string(),
			"shift=1000".to_string(),
		];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(1)));
	}

	#[test]
	fn parse_subtitle_track_options_unknown_keys_ignored() {
		let tokens = vec!["track=0".to_string(), "unknown_option=value".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(0)));
	}

	#[test]
	fn parse_subtitle_track_options_duplicate_lang_last_wins() {
		let tokens = vec!["lang=en".to_string(), "lang=fr".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.lang, Some("fr".to_string()));
	}

	#[test]
	fn parse_subtitle_track_options_duplicate_track_last_wins() {
		let tokens = vec!["track=0".to_string(), "track=1".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(1)));
	}

	#[test]
	fn parse_subtitle_track_options_empty_codec_value() {
		let tokens = vec!["track=0".to_string(), "codec=".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let _opts = result.unwrap();
	}

	#[test]
	fn parse_subtitle_track_options_empty_lang_value() {
		let tokens = vec!["track=0".to_string(), "lang=".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.lang, Some("".to_string()));
	}

	#[test]
	fn parse_subtitle_track_options_zero_track() {
		let tokens = vec!["track=0".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(0)));
	}

	#[test]
	fn parse_subtitle_track_options_large_track_number() {
		let tokens = vec!["track=100".to_string()];
		let result = subtitle::parse_subtitle_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Some(Track::One(100)));
	}
}
