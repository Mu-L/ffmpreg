use crate::cli::transcoder::media;
use crate::cli::{Cli, sink, source, track, utils};
use crate::core::compatible;
use crate::{container, io};
use std::collections::HashMap;

struct Context {
	source: source::Source,
	sink: sink::Sink,
	audio_tracks: track::audio::AudioTrackOptions,
	video_tracks: track::video::VideoTrackOptions,
	subtitle_tracks: track::subtitle::SubtitleTrackOptions,
	transforms_tracks: track::transform::TransformTrackOptions,
}

impl Context {
	fn new(source: source::Source, sink: sink::Sink) -> Self {
		Self {
			source,
			sink,
			subtitle_tracks: Default::default(),
			transforms_tracks: Default::default(),
			video_tracks: Default::default(),
			audio_tracks: Default::default(),
		}
	}

	pub fn with_audio(mut self, audio_tracks: track::audio::AudioTrackOptions) -> Self {
		self.audio_tracks = audio_tracks;
		self
	}

	pub fn with_video(mut self, video_tracks: track::video::VideoTrackOptions) -> Self {
		self.video_tracks = video_tracks;
		self
	}

	pub fn with_subtitle(mut self, subtitle_tracks: track::subtitle::SubtitleTrackOptions) -> Self {
		self.subtitle_tracks = subtitle_tracks;
		self
	}

	pub fn with_transforms(
		mut self,
		transforms_tracks: track::transform::TransformTrackOptions,
	) -> Self {
		self.transforms_tracks = transforms_tracks;
		self
	}
}

pub fn execute(cli: Cli) -> io::Result<()> {
	let compatible = compatible::Compatible::new();

	let audio_tracks = track::audio::parse_audio_track_options(cli.audio)?;
	let video_tracks = track::video::parse_video_track_options(cli.video)?;
	let subtitle_tracks = track::subtitle::parse_subtitle_track_options(cli.subtitle)?;
	let transforms_tracks = track::transform::parse_transform_track_options(cli.apply)?;

	let input_ext = utils::extension(cli.input.as_str())?;
	let output_ext = utils::extension(cli.output.as_str())?;
	compatible.assert_container_supported(&input_ext)?;
	compatible.assert_container_supported(&output_ext)?;

	if let Some(codec) = audio_tracks.options.codec.as_ref() {
		compatible.assert_audio_supported(&input_ext, codec)?;
	}

	if let Some(codec) = video_tracks.options.codec.as_ref() {
		compatible.assert_video_supported(&input_ext, codec)?;
	}

	if let Some(codec) = subtitle_tracks.options.codec.as_ref() {
		compatible.assert_subtitle_supported(&input_ext, codec)?;
	}

	let source = source::open_source_demuxer(&cli.input, &input_ext)?;
	let sink_metadata = sink::Metadata::with_input_format(&output_ext, Some(&source.metadata));
	let sink = sink::open_sink_muxer(&cli.output, sink_metadata)?;

	let context = Context::new(source, sink);
	let context = context
		.with_audio(audio_tracks)
		.with_video(video_tracks)
		.with_subtitle(subtitle_tracks)
		.with_transforms(transforms_tracks);

	return execute_pipeline(context);
}

fn execute_pipeline(mut ctx: Context) -> io::Result<()> {
	let input_ext = utils::extension(&ctx.source.path)?;
	// let output_ext = utils::extension(&ctx.sink.path)?;

	let mut transcoders: HashMap<usize, media::Transcoder> = HashMap::new();
	let stream_audio_id = ctx.audio_tracks.track.uncheck_resolve();
	let audio_transcoder = create_audio_transcoder(&ctx, &input_ext)?;
	transcoders.insert(stream_audio_id, audio_transcoder);

	while let Some(packet) = ctx.source.demuxer.read_packet()? {
		if let Some(transcoder) = transcoders.get_mut(&packet.stream_index) {
			for out_packet in transcoder.transcode(packet)? {
				ctx.sink.muxer.write(out_packet)?;
			}
		} else {
			ctx.sink.muxer.write(packet)?;
		}
	}

	for transcoder in transcoders.values_mut() {
		for packet in transcoder.flush()? {
			ctx.sink.muxer.write(packet)?;
		}
	}

	return ctx.sink.muxer.finalize();
}

fn create_audio_transcoder(ctx: &Context, input_ext: &str) -> io::Result<media::Transcoder> {
	use crate::codecs::audio::adpcm::AdpcmEncoder;
	use crate::codecs::audio::pcm::PcmDecoder;
	match input_ext {
		container::WAV => {
			let source_metadata = &ctx.source.metadata;
			if let source::Metadata::Wav(metadata) = source_metadata {
				let pcm_decoder = Box::new(PcmDecoder::new_from_metadata(metadata));
				let adpcm_encoder = Box::new(AdpcmEncoder::new_from_metadata(metadata));
				return Ok(media::Transcoder::new(pcm_decoder, adpcm_encoder));
			}
			let message = format!("expected 'wav' metadata, but found '{}'", source_metadata.name());
			return Err(io::Error::invalid_data(message));
		}
		_ => unimplemented!(),
	}
}
