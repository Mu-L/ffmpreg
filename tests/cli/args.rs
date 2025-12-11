use clap::Parser;
use ffmpreg::cli::Args;

#[test]
fn test_args_basic() {
	let args = Args::try_parse_from(["ffmpreg", "-i", "input.wav", "-o", "output.wav"]).unwrap();

	assert_eq!(args.input, "input.wav");
	assert_eq!(args.output, Some("output.wav".to_string()));
	assert!(!args.show);
	assert!(args.transforms.is_empty());
}

#[test]
fn test_args_show_mode() {
	let args = Args::try_parse_from(["ffmpreg", "-i", "input.wav", "--show"]).unwrap();

	assert_eq!(args.input, "input.wav");
	assert!(args.show);
	assert!(args.output.is_none());
}

#[test]
fn test_args_single_transform() {
	let args =
		Args::try_parse_from(["ffmpreg", "-i", "input.wav", "-o", "output.wav", "--apply", "gain=2.0"])
			.unwrap();

	assert_eq!(args.transforms.len(), 1);
	assert_eq!(args.transforms[0], "gain=2.0");
}

#[test]
fn test_args_multiple_transforms() {
	let args = Args::try_parse_from([
		"ffmpreg",
		"-i",
		"input.wav",
		"-o",
		"output.wav",
		"--apply",
		"gain=2.0",
		"--apply",
		"normalize",
	])
	.unwrap();

	assert_eq!(args.transforms.len(), 2);
	assert_eq!(args.transforms[0], "gain=2.0");
	assert_eq!(args.transforms[1], "normalize");
}

#[test]
fn test_args_codec() {
	let args =
		Args::try_parse_from(["ffmpreg", "-i", "input.wav", "-o", "output.wav", "--codec", "adpcm"])
			.unwrap();

	assert_eq!(args.codec, Some("adpcm".to_string()));
}

#[test]
fn test_args_long_form() {
	let args =
		Args::try_parse_from(["ffmpreg", "--input", "input.wav", "--output", "output.wav"]).unwrap();

	assert_eq!(args.input, "input.wav");
	assert_eq!(args.output, Some("output.wav".to_string()));
}

#[test]
fn test_args_y4m() {
	let args = Args::try_parse_from(["ffmpreg", "-i", "input.y4m", "-o", "output.y4m"]).unwrap();

	assert_eq!(args.input, "input.y4m");
	assert_eq!(args.output, Some("output.y4m".to_string()));
}

#[test]
fn test_args_glob_pattern() {
	let args = Args::try_parse_from(["ffmpreg", "-i", "folder/*.wav", "-o", "out/"]).unwrap();

	assert_eq!(args.input, "folder/*.wav");
	assert_eq!(args.output, Some("out/".to_string()));
}

#[test]
fn test_args_missing_input() {
	let result = Args::try_parse_from(["ffmpreg", "-o", "output.wav"]);
	assert!(result.is_err());
}

#[test]
fn test_args_all_options() {
	let args = Args::try_parse_from([
		"ffmpreg",
		"-i",
		"input.wav",
		"-o",
		"output.wav",
		"--apply",
		"gain=1.5",
		"--apply",
		"normalize=0.9",
		"--codec",
		"pcm",
	])
	.unwrap();

	assert_eq!(args.input, "input.wav");
	assert_eq!(args.output, Some("output.wav".to_string()));
	assert_eq!(args.transforms.len(), 2);
	assert_eq!(args.codec, Some("pcm".to_string()));
	assert!(!args.show);
}
