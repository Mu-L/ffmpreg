use super::common::Pipeline;
use crate::cli::transcoder::media;
use crate::codecs::audio::pcm::{PcmDecoder, PcmEncoder};
use crate::container::wav;
use crate::core::Muxer;
use crate::io::File;
use crate::io::{Error, Result};

pub fn run(pipeline: Pipeline) -> Result<()> {
	let file = File::open(&pipeline.input)?;
	let mut demuxer = wav::WavDemuxer::new(file)?;
	let input_format = demuxer.format();
	let mut output_format = demuxer.format();
	let metadata = demuxer.metadata().clone();

	if let Some(ref codec) = pipeline.audio.codec {
		output_format.apply_codec(codec).map_err(Error::invalid_data)?;
	}

	let file = File::create(&pipeline.output)?;

	let mut muxer = wav::WavMuxer::new(file, output_format)?;
	muxer.with_metadata(metadata);

	let mut transcoder = create_wav_transcoder(input_format, output_format);

	while let Some(packet) = demuxer.read_packet()? {
		for out_packet in transcoder.transcode(packet)? {
			muxer.write(out_packet)?;
		}
	}

	for packet in transcoder.flush()? {
		muxer.write(packet)?;
	}

	muxer.finalize()
}

fn create_wav_transcoder(format: wav::WavFormat, target: wav::WavFormat) -> media::Transcoder {
	let audio_format = format.audio_format();
	let target_format = target.audio_format();

	let decoder = Box::new(PcmDecoder::new_from_metadata(&format));

	if audio_format != target_format {
		let encoder = PcmEncoder::new(target.sample_rate);
		let encoder = Box::new(encoder.with_target_format(target_format));
		return media::Transcoder::new(decoder, encoder);
	}

	let encoder = Box::new(PcmEncoder::new(target.sample_rate));
	media::Transcoder::new(decoder, encoder)
}
