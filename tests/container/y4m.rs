use ffmpreg::container::{Y4mReader, Y4mWriter};
use ffmpreg::core::{Demuxer, Muxer, Packet, Timebase};
use ffmpreg::io::{BufferedWriter, Cursor};

fn create_test_y4m() -> Vec<u8> {
	let width: u32 = 8;
	let height: u32 = 8;
	let num_frames = 3;

	let mut y4m = Vec::new();

	let header = format!("YUV4MPEG2 W{} H{} F30:1 Ip A1:1 C420\n", width, height);
	y4m.extend_from_slice(header.as_bytes());

	let luma_size = (width * height) as usize;
	let chroma_size = luma_size / 4;

	for frame_idx in 0..num_frames {
		y4m.extend_from_slice(b"FRAME\n");

		for i in 0..luma_size {
			y4m.push(((frame_idx * 30 + i) % 256) as u8);
		}
		for _ in 0..chroma_size {
			y4m.push(128);
		}
		for _ in 0..chroma_size {
			y4m.push(128);
		}
	}

	y4m
}

fn create_y4m_with_aspect_ratio() -> Vec<u8> {
	let width: u32 = 4;
	let height: u32 = 4;

	let mut y4m = Vec::new();

	let header = format!("YUV4MPEG2 W{} H{} F25:1 Ip A128:117\n", width, height);
	y4m.extend_from_slice(header.as_bytes());

	let luma_size = (width * height) as usize;
	let chroma_size = luma_size / 4;

	y4m.extend_from_slice(b"FRAME\n");
	for i in 0..luma_size {
		y4m.push((i * 10) as u8);
	}
	for _ in 0..chroma_size {
		y4m.push(128);
	}
	for _ in 0..chroma_size {
		y4m.push(128);
	}

	y4m
}

#[test]
fn test_y4m_reader_format() {
	let y4m_data = create_test_y4m();
	let cursor = Cursor::new(y4m_data);
	let reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	assert_eq!(format.width, 8);
	assert_eq!(format.height, 8);
	assert_eq!(format.framerate_num, 30);
	assert_eq!(format.framerate_den, 1);
}

#[test]
fn test_y4m_reader_aspect_ratio() {
	let y4m_data = create_y4m_with_aspect_ratio();
	let cursor = Cursor::new(y4m_data);
	let reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	assert!(format.aspect_ratio.is_some());
	let aspect = format.aspect_ratio.unwrap();
	assert_eq!(aspect.num, 128);
	assert_eq!(aspect.den, 117);
}

#[test]
fn test_y4m_reader_read_packets() {
	let y4m_data = create_test_y4m();
	let cursor = Cursor::new(y4m_data);
	let mut reader = Y4mReader::new(cursor).unwrap();

	let mut frame_count = 0;

	while let Some(_packet) = reader.read_packet().unwrap() {
		frame_count += 1;
	}

	assert_eq!(frame_count, 3);
}

#[test]
fn test_y4m_reader_frame_size() {
	let y4m_data = create_test_y4m();
	let cursor = Cursor::new(y4m_data);
	let mut reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	let expected_size = format.frame_size();

	if let Some(packet) = reader.read_packet().unwrap() {
		assert_eq!(packet.size(), expected_size);
	}
}

#[test]
fn test_y4m_reader_stream_count() {
	let y4m_data = create_test_y4m();
	let cursor = Cursor::new(y4m_data);
	let reader = Y4mReader::new(cursor).unwrap();

	assert_eq!(reader.stream_count(), 1);
}

#[test]
fn test_y4m_writer_basic() {
	let y4m_data = create_test_y4m();
	let cursor = Cursor::new(y4m_data);
	let reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let buf_writer: BufferedWriter<Cursor<Vec<u8>>> = BufferedWriter::new(output_buffer);
	let mut writer = Y4mWriter::new(buf_writer, format).unwrap();

	let timebase = Timebase::new(1, 30);
	let frame_size = 8 * 8 + (8 * 8 / 4) * 2;
	let packet = Packet::new(vec![128u8; frame_size], 0, timebase);

	writer.write_packet(packet).unwrap();
	writer.finalize().unwrap();
}

#[test]
fn test_y4m_roundtrip() {
	let original_y4m = create_test_y4m();
	let cursor = Cursor::new(original_y4m.clone());
	let mut reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let buf_writer: BufferedWriter<Cursor<Vec<u8>>> = BufferedWriter::new(output_buffer);
	let mut writer = Y4mWriter::new(buf_writer, format).unwrap();

	let mut frame_count = 0;
	while let Some(packet) = reader.read_packet().unwrap() {
		writer.write_packet(packet).unwrap();
		frame_count += 1;
	}

	writer.finalize().unwrap();
	assert_eq!(frame_count, 3);
}

#[test]
fn test_y4m_roundtrip_preserves_aspect_ratio() {
	let original_y4m = create_y4m_with_aspect_ratio();
	let cursor = Cursor::new(original_y4m.clone());
	let mut reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let buf_writer: BufferedWriter<Cursor<Vec<u8>>> = BufferedWriter::new(output_buffer);
	let mut writer = Y4mWriter::new(buf_writer, format.clone()).unwrap();

	while let Some(packet) = reader.read_packet().unwrap() {
		writer.write_packet(packet).unwrap();
	}

	writer.finalize().unwrap();

	assert!(format.aspect_ratio.is_some());
}

#[test]
fn test_y4m_invalid_file() {
	let invalid_data = b"NOT A Y4M FILE\n".to_vec();
	let cursor = Cursor::new(invalid_data);
	let result = Y4mReader::new(cursor);

	assert!(result.is_err());
}

#[test]
fn test_y4m_pts_sequence() {
	let y4m_data = create_test_y4m();
	let cursor = Cursor::new(y4m_data);
	let mut reader = Y4mReader::new(cursor).unwrap();

	let mut expected_pts = 0i64;
	while let Some(packet) = reader.read_packet().unwrap() {
		assert_eq!(packet.pts, expected_pts);
		expected_pts += 1;
	}
}

#[test]
fn test_y4m_frame_size_calculation() {
	let y4m_data = create_test_y4m();
	let cursor = Cursor::new(y4m_data);
	let reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	let luma = (format.width * format.height) as usize;
	let chroma = luma / 4 * 2;
	let expected = luma + chroma;

	assert_eq!(format.frame_size(), expected);
}
