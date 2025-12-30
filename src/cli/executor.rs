use crate::cli::{config, pipeline, utils};
use crate::container;
use crate::core::compatible;
use crate::{cli, io};

pub fn execute(cli: cli::Cli) -> io::Result<()> {
	let mut pipe = pipeline::Pipeline::new(&cli.input, &cli.output);

	let audio = config::parse_audio(cli.audio)?;
	let video = config::parse_video(cli.video)?;
	let subtitle = config::parse_subtitle(cli.subtitle)?;
	let transform = config::parse_transform(cli.apply)?;
	pipe.with_transform(transform);

	let input_ext = utils::get_extension(&cli.input)?;
	let output_ext = utils::get_extension(&cli.output)?;

	let compat = compatible::Compatible::new();
	compat.assert_container_supported(&input_ext)?;
	compat.assert_container_supported(&output_ext)?;

	if let Some(codec) = &audio.codec {
		compat.assert_audio_supported(&input_ext, codec)?;
		pipe.with_audio(audio);
	}

	if let Some(codec) = &video.codec {
		compat.assert_video_supported(&input_ext, codec)?;
		pipe.with_video(video);
	}

	if let Some(codec) = &subtitle.codec {
		compat.assert_subtitle_supported(&input_ext, codec)?;
		pipe.with_subtitle(subtitle);
	}

	match input_ext.as_str() {
		container::WAV => pipeline::run_wav(pipe),
		container::AAC => pipeline::run_aac(pipe),
		container::MOV => pipeline::run_mov(pipe),

		_ => Err(io::Error::invalid_data(format!("unsupported '{}' format", input_ext))),
	}
}
