use ffmpreg::core::{Packet, Timebase};

#[test]
fn test_packet_creation() {
	let data = vec![0u8; 4096];
	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(data.clone(), 0, timebase);

	assert_eq!(packet.stream_index, 0);
	assert_eq!(packet.size(), 4096);
	assert_eq!(packet.pts, 0);
	assert_eq!(packet.dts, 0);
}

#[test]
fn test_packet_with_pts() {
	let data = vec![1, 2, 3, 4];
	let timebase = Timebase::new(1, 48000);
	let packet = Packet::new(data, 1, timebase).with_pts(2048);

	assert_eq!(packet.pts, 2048);
	assert_eq!(packet.stream_index, 1);
}

#[test]
fn test_packet_with_dts() {
	let data = vec![5, 6, 7, 8];
	let timebase = Timebase::new(1, 30);
	let packet = Packet::new(data, 0, timebase).with_dts(15);

	assert_eq!(packet.dts, 15);
}

#[test]
fn test_packet_empty() {
	let data = vec![];
	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(data, 0, timebase);

	assert!(packet.is_empty());
	assert_eq!(packet.size(), 0);
}

#[test]
fn test_packet_clone() {
	let data = vec![10, 20, 30, 40];
	let timebase = Timebase::new(1, 44100);
	let packet = Packet::new(data, 0, timebase).with_pts(500).with_dts(490);
	let cloned = packet.clone();

	assert_eq!(cloned.pts, packet.pts);
	assert_eq!(cloned.dts, packet.dts);
	assert_eq!(cloned.data, packet.data);
}
