use super::lpc::{restore_fixed_signal, restore_lpc_signal};
use super::rice::{BitReader, decode_residual};
use super::{ChannelAssignment, FlacStreamInfo, SubframeType};
use crate::io::{IoError, IoResult};

pub struct FlacFrame {
	pub samples: Vec<Vec<i32>>,
	pub block_size: usize,
	pub sample_rate: u32,
	pub channels: u8,
	pub bits_per_sample: u8,
}

pub fn decode_frame(data: &[u8], stream_info: &FlacStreamInfo) -> IoResult<FlacFrame> {
	let mut reader = BitReader::new(data);

	let sync = reader.read_bits(14)?;
	if sync != 0x3FFE {
		return Err(IoError::invalid_data("invalid frame sync code"));
	}

	let reserved1 = reader.read_bit()?;
	if reserved1 {
		return Err(IoError::invalid_data("reserved bit must be 0"));
	}

	let _blocking_strategy = reader.read_bit()?;

	let block_size_code = reader.read_bits(4)?;
	let sample_rate_code = reader.read_bits(4)?;
	let channel_assignment_code = reader.read_bits(4)? as u8;
	let sample_size_code = reader.read_bits(3)?;

	let reserved2 = reader.read_bit()?;
	if reserved2 {
		return Err(IoError::invalid_data("reserved bit must be 0"));
	}

	let _frame_number = reader.read_utf8_u64()?;

	let block_size = match block_size_code {
		0 => return Err(IoError::invalid_data("reserved block size")),
		1 => 192,
		2..=5 => 576 << (block_size_code - 2),
		6 => (reader.read_bits(8)? + 1) as usize,
		7 => (reader.read_bits(16)? + 1) as usize,
		8..=15 => 256 << (block_size_code - 8),
		_ => unreachable!(),
	};

	let sample_rate = match sample_rate_code {
		0 => stream_info.sample_rate,
		1 => 88200,
		2 => 176400,
		3 => 192000,
		4 => 8000,
		5 => 16000,
		6 => 22050,
		7 => 24000,
		8 => 32000,
		9 => 44100,
		10 => 48000,
		11 => 96000,
		12 => reader.read_bits(8)? * 1000,
		13 => reader.read_bits(16)?,
		14 => reader.read_bits(16)? * 10,
		15 => return Err(IoError::invalid_data("invalid sample rate")),
		_ => unreachable!(),
	};

	let channels = match channel_assignment_code {
		0..=7 => channel_assignment_code + 1,
		8..=10 => 2,
		_ => return Err(IoError::invalid_data("reserved channel assignment")),
	};

	let channel_assignment = ChannelAssignment::from_raw(channels, channel_assignment_code);

	let bits_per_sample = match sample_size_code {
		0 => stream_info.bits_per_sample,
		1 => 8,
		2 => 12,
		3 => return Err(IoError::invalid_data("reserved sample size")),
		4 => 16,
		5 => 20,
		6 => 24,
		7 => return Err(IoError::invalid_data("reserved sample size")),
		_ => unreachable!(),
	};

	let _header_crc = reader.read_bits(8)?;

	let mut channel_samples = Vec::with_capacity(channels as usize);

	for ch in 0..channels as usize {
		let extra_bits = match channel_assignment {
			ChannelAssignment::LeftSide if ch == 1 => 1,
			ChannelAssignment::RightSide if ch == 0 => 1,
			ChannelAssignment::MidSide if ch == 1 => 1,
			_ => 0,
		};

		let subframe_bps = bits_per_sample + extra_bits;
		let samples = decode_subframe(&mut reader, block_size, subframe_bps)?;
		channel_samples.push(samples);
	}

	match channel_assignment {
		ChannelAssignment::LeftSide => {
			for i in 0..block_size {
				channel_samples[1][i] = channel_samples[0][i] - channel_samples[1][i];
			}
		}
		ChannelAssignment::RightSide => {
			for i in 0..block_size {
				channel_samples[0][i] = channel_samples[0][i] + channel_samples[1][i];
			}
		}
		ChannelAssignment::MidSide => {
			for i in 0..block_size {
				let mid = channel_samples[0][i];
				let side = channel_samples[1][i];
				channel_samples[0][i] = mid + (side >> 1);
				channel_samples[1][i] = mid - (side >> 1);
			}
		}
		ChannelAssignment::Independent => {}
	}

	reader.align_to_byte();

	Ok(FlacFrame { samples: channel_samples, block_size, sample_rate, channels, bits_per_sample })
}

fn decode_subframe(reader: &mut BitReader, block_size: usize, bps: u8) -> IoResult<Vec<i32>> {
	let padding = reader.read_bit()?;
	if padding {
		return Err(IoError::invalid_data("subframe padding must be 0"));
	}

	let subframe_type_bits = reader.read_bits(6)?;

	let wasted_bits = if reader.read_bit()? {
		let mut wasted = 1u32;
		while !reader.read_bit()? {
			wasted += 1;
		}
		wasted
	} else {
		0
	};

	let effective_bps = bps - wasted_bits as u8;

	let subframe_type = match subframe_type_bits {
		0 => SubframeType::Constant,
		1 => SubframeType::Verbatim,
		2..=7 => return Err(IoError::invalid_data("reserved subframe type")),
		8..=15 => SubframeType::Fixed((subframe_type_bits - 8) as u8),
		16..=31 => return Err(IoError::invalid_data("reserved subframe type")),
		32..=63 => SubframeType::Lpc((subframe_type_bits - 31) as u8),
		_ => unreachable!(),
	};

	let mut samples = match subframe_type {
		SubframeType::Constant => {
			let value = reader.read_bits_signed(effective_bps as u32)?;
			vec![value; block_size]
		}
		SubframeType::Verbatim => {
			let mut samples = Vec::with_capacity(block_size);
			for _ in 0..block_size {
				samples.push(reader.read_bits_signed(effective_bps as u32)?);
			}
			samples
		}
		SubframeType::Fixed(order) => {
			let order = order as usize;
			let mut warmup = Vec::with_capacity(order);
			for _ in 0..order {
				warmup.push(reader.read_bits_signed(effective_bps as u32)?);
			}

			let mut residuals = Vec::new();
			decode_residual(reader, order, block_size, &mut residuals)?;

			let mut output = Vec::with_capacity(block_size);
			restore_fixed_signal(&residuals, &warmup, order, &mut output);
			output
		}
		SubframeType::Lpc(order) => {
			let order = order as usize;
			let mut warmup = Vec::with_capacity(order);
			for _ in 0..order {
				warmup.push(reader.read_bits_signed(effective_bps as u32)?);
			}

			let precision = reader.read_bits(4)? + 1;
			let shift = reader.read_bits_signed(5)? as i8;

			let mut lpc_coefs = Vec::with_capacity(order);
			for _ in 0..order {
				lpc_coefs.push(reader.read_bits_signed(precision)?);
			}

			let mut residuals = Vec::new();
			decode_residual(reader, order, block_size, &mut residuals)?;

			let mut output = Vec::with_capacity(block_size);
			restore_lpc_signal(&residuals, &warmup, &lpc_coefs, shift, &mut output);
			output
		}
	};

	if wasted_bits > 0 {
		for sample in &mut samples {
			*sample <<= wasted_bits;
		}
	}

	Ok(samples)
}

pub fn encode_frame(
	samples: &[Vec<i32>],
	frame_number: u64,
	stream_info: &FlacStreamInfo,
) -> Vec<u8> {
	use super::lpc::apply_fixed_prediction;
	use super::rice::{BitWriter, encode_residual};

	let mut writer = BitWriter::new();

	writer.write_bits(0x3FFE, 14);
	writer.write_bit(false);
	writer.write_bit(false);

	let block_size = samples[0].len();
	let block_size_code = match block_size {
		192 => 1,
		576 => 2,
		1152 => 3,
		2304 => 4,
		4608 => 5,
		256 => 8,
		512 => 9,
		1024 => 10,
		2048 => 11,
		4096 => 12,
		8192 => 13,
		16384 => 14,
		32768 => 15,
		_ if block_size <= 256 => 6,
		_ => 7,
	};
	writer.write_bits(block_size_code, 4);

	let sample_rate_code = match stream_info.sample_rate {
		88200 => 1,
		176400 => 2,
		192000 => 3,
		8000 => 4,
		16000 => 5,
		22050 => 6,
		24000 => 7,
		32000 => 8,
		44100 => 9,
		48000 => 10,
		96000 => 11,
		_ => 0,
	};
	writer.write_bits(sample_rate_code, 4);

	let channel_code = (stream_info.channels - 1) as u32;
	writer.write_bits(channel_code, 4);

	let sample_size_code = match stream_info.bits_per_sample {
		8 => 1,
		12 => 2,
		16 => 4,
		20 => 5,
		24 => 6,
		_ => 0,
	};
	writer.write_bits(sample_size_code, 3);
	writer.write_bit(false);

	writer.write_utf8_u64(frame_number);

	match block_size_code {
		6 => writer.write_bits((block_size - 1) as u32, 8),
		7 => writer.write_bits((block_size - 1) as u32, 16),
		_ => {}
	}

	let header_crc = 0u8;
	writer.write_bits(header_crc as u32, 8);

	let bps = stream_info.bits_per_sample;

	for channel in samples {
		writer.write_bit(false);

		let order = 2usize;
		let subframe_type = 8 + order as u32;
		writer.write_bits(subframe_type, 6);
		writer.write_bit(false);

		for i in 0..order {
			writer.write_bits_signed(channel[i], bps as u32);
		}

		let mut residuals = vec![0i32; block_size];
		apply_fixed_prediction(channel, order, &mut residuals);

		encode_residual(&mut writer, &residuals, order, block_size);
	}

	writer.align_to_byte();

	let frame_crc = 0u16;
	writer.write_bits(frame_crc as u32, 16);

	writer.finish()
}
