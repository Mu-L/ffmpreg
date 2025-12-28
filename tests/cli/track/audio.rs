#[cfg(test)]
mod tests {
	use ffmpreg::cli::track::{Track, audio};

	#[test]
	fn parse_audio_track_options_with_track_and_codec() {
		let tokens = vec!["track=0".to_string(), "codec=aac".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(0));
		assert_eq!(opts.options.codec, Some("aac".to_string()));
	}

	#[test]
	fn parse_audio_track_options_with_all_options() {
		let tokens = vec![
			"track=1".to_string(),
			"codec=mp3".to_string(),
			"channels=2".to_string(),
			"sample_rate=44100".to_string(),
			"volume=0.8".to_string(),
		];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(1));
		assert_eq!(opts.options.codec, Some("mp3".to_string()));
		assert_eq!(opts.options.channels, Some("2".to_string()));
		assert_eq!(opts.options.sample_rate, Some("44100".to_string()));
		assert_eq!(opts.options.volume, Some("0.8".to_string()));
	}

	#[test]
	fn parse_audio_track_options_default_track() {
		let tokens = vec!["codec=aac".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
		assert_eq!(opts.options.codec, Some("aac".to_string()));
	}

	#[test]
	fn parse_audio_track_options_only_track() {
		let tokens = vec!["track=0".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(0));
		assert!(opts.options.codec.is_none());
		assert!(opts.options.channels.is_none());
		assert!(opts.options.sample_rate.is_none());
		assert!(opts.options.volume.is_none());
	}

	#[test]
	fn parse_audio_track_options_empty_tokens() {
		let tokens: Vec<String> = vec![];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
		assert!(opts.options.codec.is_none());
	}

	#[test]
	fn parse_audio_track_options_unknown_keys_ignored() {
		let tokens =
			vec!["track=0".to_string(), "codec=aac".to_string(), "unknown_key=value".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(0));
		assert_eq!(opts.options.codec, Some("aac".to_string()));
	}

	#[test]
	fn parse_audio_track_options_track_zero() {
		let tokens = vec!["track=0".to_string(), "codec=flac".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(0));
	}

	#[test]
	fn parse_audio_track_options_track_all() {
		let tokens = vec!["track=all".to_string(), "codec=aac".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
	}

	#[test]
	fn parse_audio_track_options_track_asterisk() {
		let tokens = vec!["track=*".to_string(), "codec=aac".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
	}

	#[test]
	fn parse_audio_track_options_invalid_track_fails() {
		let tokens = vec!["track=invalid".to_string(), "codec=aac".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_err());
	}

	#[test]
	fn parse_audio_track_options_channels_only() {
		let tokens = vec!["track=0".to_string(), "channels=5.1".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.channels, Some("5.1".to_string()));
		assert!(opts.options.codec.is_none());
	}

	#[test]
	fn parse_audio_track_options_sample_rate_only() {
		let tokens = vec!["track=1".to_string(), "sample_rate=48000".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.sample_rate, Some("48000".to_string()));
	}

	#[test]
	fn parse_audio_track_options_volume_only() {
		let tokens = vec!["track=0".to_string(), "volume=1.5".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.volume, Some("1.5".to_string()));
	}

	#[test]
	fn parse_audio_track_options_duplicate_keys_last_wins() {
		let tokens = vec!["track=0".to_string(), "codec=aac".to_string(), "codec=mp3".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.codec, Some("mp3".to_string()));
	}

	#[test]
	fn parse_audio_track_options_empty_codec_value() {
		let tokens = vec!["track=0".to_string(), "codec=".to_string()];
		let result = audio::parse_audio_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.codec, Some("".to_string()));
	}
}
