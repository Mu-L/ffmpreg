use ffmpreg::core::{Decoder, Demuxer, Encoder, Frame, Muxer, Packet, Timebase, Transform};
use ffmpreg::io::IoResult;

struct MockDemuxer {
	packets: Vec<Packet>,
	index: usize,
}

impl Demuxer for MockDemuxer {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		if self.index < self.packets.len() {
			let pkt = self.packets[self.index].clone();
			self.index += 1;
			Ok(Some(pkt))
		} else {
			Ok(None)
		}
	}

	fn stream_count(&self) -> usize {
		1
	}
}

struct MockMuxer {
	packets: Vec<Packet>,
}

impl Muxer for MockMuxer {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
		self.packets.push(packet);
		Ok(())
	}

	fn finalize(&mut self) -> IoResult<()> {
		Ok(())
	}
}

struct MockDecoder;

impl Decoder for MockDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		let frame = Frame::new(packet.data, packet.timebase, 44100, 1, 256).with_pts(packet.pts);
		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}

struct MockEncoder {
	timebase: Timebase,
}

impl Encoder for MockEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		let packet = Packet::new(frame.data, 0, self.timebase).with_pts(frame.pts);
		Ok(Some(packet))
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}

struct MockTransform {
	multiplier: i16,
}

impl Transform for MockTransform {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		for chunk in frame.data.chunks_exact_mut(2) {
			let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
			let modified = sample.saturating_mul(self.multiplier);
			let bytes = modified.to_le_bytes();
			chunk[0] = bytes[0];
			chunk[1] = bytes[1];
		}
		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"mock"
	}
}

#[test]
fn test_demuxer_trait() {
	let timebase = Timebase::new(1, 44100);
	let packets = vec![
		Packet::new(vec![1, 2], 0, timebase).with_pts(0),
		Packet::new(vec![3, 4], 0, timebase).with_pts(1),
	];

	let mut demuxer = MockDemuxer { packets, index: 0 };

	let pkt1 = demuxer.read_packet().unwrap();
	assert!(pkt1.is_some());
	assert_eq!(pkt1.unwrap().data, vec![1, 2]);

	let pkt2 = demuxer.read_packet().unwrap();
	assert!(pkt2.is_some());

	let pkt3 = demuxer.read_packet().unwrap();
	assert!(pkt3.is_none());

	assert_eq!(demuxer.stream_count(), 1);
}

#[test]
fn test_muxer_trait() {
	let timebase = Timebase::new(1, 44100);
	let mut muxer = MockMuxer { packets: Vec::new() };

	muxer.write_packet(Packet::new(vec![1, 2], 0, timebase)).unwrap();
	muxer.write_packet(Packet::new(vec![3, 4], 0, timebase)).unwrap();
	muxer.finalize().unwrap();

	assert_eq!(muxer.packets.len(), 2);
}

#[test]
fn test_decoder_trait() {
	let timebase = Timebase::new(1, 44100);
	let mut decoder = MockDecoder;

	let packet = Packet::new(vec![1, 2, 3, 4], 0, timebase).with_pts(100);
	let frame = decoder.decode(packet).unwrap().unwrap();

	assert_eq!(frame.pts, 100);
	assert_eq!(frame.data, vec![1, 2, 3, 4]);
}

#[test]
fn test_encoder_trait() {
	let timebase = Timebase::new(1, 44100);
	let mut encoder = MockEncoder { timebase };

	let frame = Frame::new(vec![1, 2, 3, 4], timebase, 44100, 1, 2).with_pts(200);
	let packet = encoder.encode(frame).unwrap().unwrap();

	assert_eq!(packet.pts, 200);
	assert_eq!(packet.data, vec![1, 2, 3, 4]);
}

#[test]
fn test_transform_trait() {
	let timebase = Timebase::new(1, 44100);
	let mut transform = MockTransform { multiplier: 2 };

	let samples: Vec<i16> = vec![100, 200];
	let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
	let frame = Frame::new(data, timebase, 44100, 1, 2);

	let result = transform.apply(frame).unwrap();
	let output: Vec<i16> = result.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();

	assert_eq!(output, vec![200, 400]);
	assert_eq!(transform.name(), "mock");
}
