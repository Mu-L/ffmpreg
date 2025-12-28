#[cfg(test)]
mod tests {
	use ffmpreg::cli::track::{Track, video};

	#[test]
	fn parse_video_track_options_with_track_and_codec() {
		let tokens = vec!["track=0".to_string(), "codec=h264".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(0));
		assert_eq!(opts.options.codec, Some("h264".to_string()));
	}

	#[test]
	fn parse_video_track_options_with_all_options() {
		let tokens = vec![
			"track=0".to_string(),
			"codec=h265".to_string(),
			"scale=1920:1080".to_string(),
			"width=1920".to_string(),
			"height=1080".to_string(),
			"fps=30".to_string(),
			"bitrate=5000k".to_string(),
			"aspect_ratio=16:9".to_string(),
			"rotate=90".to_string(),
			"brightness=1.2".to_string(),
			"contrast=1.1".to_string(),
		];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(0));
		assert_eq!(opts.options.codec, Some("h265".to_string()));
		assert_eq!(opts.options.scale, Some("1920:1080".to_string()));
		assert_eq!(opts.options.width, Some("1920".to_string()));
		assert_eq!(opts.options.height, Some("1080".to_string()));
		assert_eq!(opts.options.fps, Some("30".to_string()));
		assert_eq!(opts.options.bitrate, Some("5000k".to_string()));
		assert_eq!(opts.options.aspect_ratio, Some("16:9".to_string()));
		assert_eq!(opts.options.rotate, Some("90".to_string()));
		assert_eq!(opts.options.brightness, Some("1.2".to_string()));
		assert_eq!(opts.options.contrast, Some("1.1".to_string()));
	}

	#[test]
	fn parse_video_track_options_default_track() {
		let tokens = vec!["codec=h264".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
		assert_eq!(opts.options.codec, Some("h264".to_string()));
	}

	#[test]
	fn parse_video_track_options_only_track() {
		let tokens = vec!["track=1".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(1));
		assert!(opts.options.codec.is_none());
		assert!(opts.options.scale.is_none());
		assert!(opts.options.fps.is_none());
	}

	#[test]
	fn parse_video_track_options_empty_tokens() {
		let tokens: Vec<String> = vec![];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
		assert!(opts.options.codec.is_none());
	}

	#[test]
	fn parse_video_track_options_unknown_keys_ignored() {
		let tokens = vec![
			"track=0".to_string(),
			"codec=h264".to_string(),
			"unknown_video_option=value".to_string(),
		];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::One(0));
		assert_eq!(opts.options.codec, Some("h264".to_string()));
	}

	#[test]
	fn parse_video_track_options_track_all() {
		let tokens = vec!["track=all".to_string(), "codec=h264".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
	}

	#[test]
	fn parse_video_track_options_track_asterisk() {
		let tokens = vec!["track=*".to_string(), "fps=60".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.track, Track::All);
	}

	#[test]
	fn parse_video_track_options_invalid_track_fails() {
		let tokens = vec!["track=notatrack".to_string(), "codec=h264".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_err());
	}

	#[test]
	fn parse_video_track_options_dimension_options() {
		let tokens = vec!["track=0".to_string(), "width=1280".to_string(), "height=720".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.width, Some("1280".to_string()));
		assert_eq!(opts.options.height, Some("720".to_string()));
	}

	#[test]
	fn parse_video_track_options_scale_option() {
		let tokens = vec!["track=0".to_string(), "scale=1920:1080".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.scale, Some("1920:1080".to_string()));
	}

	#[test]
	fn parse_video_track_options_fps_option() {
		let tokens = vec!["track=0".to_string(), "fps=25".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.fps, Some("25".to_string()));
	}

	#[test]
	fn parse_video_track_options_bitrate_option() {
		let tokens = vec!["track=0".to_string(), "bitrate=8000k".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.bitrate, Some("8000k".to_string()));
	}

	#[test]
	fn parse_video_track_options_aspect_ratio_option() {
		let tokens = vec!["track=0".to_string(), "aspect_ratio=4:3".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.aspect_ratio, Some("4:3".to_string()));
	}

	#[test]
	fn parse_video_track_options_rotate_option() {
		let tokens = vec!["track=0".to_string(), "rotate=180".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.rotate, Some("180".to_string()));
	}

	#[test]
	fn parse_video_track_options_brightness_option() {
		let tokens = vec!["track=0".to_string(), "brightness=0.8".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.brightness, Some("0.8".to_string()));
	}

	#[test]
	fn parse_video_track_options_contrast_option() {
		let tokens = vec!["track=0".to_string(), "contrast=1.3".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.contrast, Some("1.3".to_string()));
	}

	#[test]
	fn parse_video_track_options_duplicate_keys_last_wins() {
		let tokens = vec!["track=0".to_string(), "codec=h264".to_string(), "codec=vp9".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.codec, Some("vp9".to_string()));
	}

	#[test]
	fn parse_video_track_options_empty_codec_value() {
		let tokens = vec!["track=0".to_string(), "codec=".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.codec, Some("".to_string()));
	}

	#[test]
	fn parse_video_track_options_filter_like_value_with_colons() {
		let tokens = vec!["track=0".to_string(), "scale=1920:1080".to_string()];
		let result = video::parse_video_track_options(tokens);

		assert!(result.is_ok());
		let opts = result.unwrap();
		assert_eq!(opts.options.scale, Some("1920:1080".to_string()));
	}
}
