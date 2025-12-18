use ffmpreg::codecs::Mp3Decoder;
use ffmpreg::core::{Decoder, Timebase};

#[test]
fn test_mp3_decoder_new() {
	let decoder = Mp3Decoder::new(44100, 2);
	assert!(true);
}

#[test]
fn test_mp3_decoder_flush() {
	let mut decoder = Mp3Decoder::new(44100, 2);
	let result = decoder.flush().unwrap();
	assert!(result.is_none());
}

#[test]
fn test_mp3_decoder_from_header_invalid() {
	let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
	let result = Mp3Decoder::from_header(&invalid_data);
	assert!(result.is_none());
}

#[test]
fn test_mp3_decoder_empty_packet() {
	use ffmpreg::core::Packet;

	let mut decoder = Mp3Decoder::new(44100, 2);
	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![], 0, timebase);

	let result = decoder.decode(packet).unwrap();
	assert!(result.is_none());
}

#[test]
fn test_mp3_decoder_invalid_data() {
	use ffmpreg::core::Packet;

	let mut decoder = Mp3Decoder::new(44100, 2);
	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05], 0, timebase);

	let result = decoder.decode(packet).unwrap();
	assert!(result.is_none());
}
