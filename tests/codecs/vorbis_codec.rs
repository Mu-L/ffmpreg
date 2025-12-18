use ffmpreg::codecs::VorbisDecoder;
use ffmpreg::container::OggFormat;
use ffmpreg::core::{Decoder, Packet, Timebase};

#[test]
fn test_vorbis_decoder_new() {
	let format = OggFormat::default();
	let decoder = VorbisDecoder::new(format);
	assert_eq!(decoder.sample_rate(), 44100);
	assert_eq!(decoder.channels(), 2);
}

#[test]
fn test_vorbis_decoder_flush() {
	let format = OggFormat::default();
	let mut decoder = VorbisDecoder::new(format);
	let result = decoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_vorbis_decoder_empty_packet() {
	let format = OggFormat::default();
	let mut decoder = VorbisDecoder::new(format);
	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![], 0, timebase);

	let result = decoder.decode(packet).unwrap();
	assert!(result.is_none());
}

#[test]
fn test_vorbis_decoder_invalid_data() {
	let format = OggFormat::default();
	let mut decoder = VorbisDecoder::new(format);
	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![0x00, 0x01, 0x02, 0x03], 0, timebase);

	let result = decoder.decode(packet).unwrap();
	assert!(result.is_none());
}

#[test]
fn test_vorbis_decoder_custom_format() {
	let format = OggFormat { sample_rate: 48000, channels: 1, bitstream_serial: 12345 };
	let decoder = VorbisDecoder::new(format);
	assert_eq!(decoder.sample_rate(), 48000);
	assert_eq!(decoder.channels(), 1);
}

#[test]
fn test_vorbis_from_ogg_data_invalid() {
	let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
	let result = VorbisDecoder::from_ogg_data(&invalid_data);
	assert!(result.is_err());
}
