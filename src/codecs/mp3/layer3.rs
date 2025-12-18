use super::bits::BitReader;
use super::header::{FrameHeader, MpegVersion};
use super::huffman::{decode_huffman_pair, decode_huffman_quad};
use super::sideinfo::{GranuleChannel, SideInfo};
use super::synth::{apply_window, imdct_36, SynthesisFilterbank};
use super::tables::{
	CA, CS, PRETAB, SCALEFACTOR_BAND_LONG, SCALEFACTOR_BAND_LONG_MPEG2, SCALEFACTOR_BAND_SHORT,
	SCALEFACTOR_BAND_SHORT_MPEG2, SLEN_TABLE, get_sample_rate_index, HUFFMAN_TABLES,
};

thread_local! {
	static REQUANT_TABLE: [f32; 256] = {
		let mut table = [0.0f32; 256];
		for i in 0..256 {
			table[i] = 2.0_f32.powf(0.25 * (i as f32 - 210.0));
		}
		table
	};
}

pub struct Layer3Decoder {
	synth: [SynthesisFilterbank; 2],
	prev_samples: [[f32; 576]; 2],
	main_data_buffer: Vec<u8>,
}

impl Default for Layer3Decoder {
	fn default() -> Self {
		Self::new()
	}
}

impl Layer3Decoder {
	pub fn new() -> Self {
		Self {
			synth: [SynthesisFilterbank::new(), SynthesisFilterbank::new()],
			prev_samples: [[0.0; 576]; 2],
			main_data_buffer: Vec::with_capacity(2048),
		}
	}

	pub fn decode_frame(
		&mut self,
		header: &FrameHeader,
		frame_data: &[u8],
	) -> Option<Vec<i16>> {
		let header_size = 4;
		let crc_size = if header.crc_protection { 2 } else { 0 };
		let side_info_start = header_size + crc_size;

		if frame_data.len() < side_info_start + header.side_info_size() {
			return None;
		}

		let side_info_data = &frame_data[side_info_start..];
		let mut si_reader = BitReader::new(side_info_data);
		let side_info = SideInfo::parse(&mut si_reader, header)?;

		let main_data_start = side_info_start + header.side_info_size();
		let main_data = &frame_data[main_data_start..];

		self.main_data_buffer.extend_from_slice(main_data);

		let main_data_begin = side_info.main_data_begin as usize;
		if self.main_data_buffer.len() < main_data_begin {
			return None;
		}

		let buffer_offset = self.main_data_buffer.len() - main_data_begin - main_data.len();
		let decode_data = self.main_data_buffer[buffer_offset..].to_vec(); // Copy to avoid borrow issues

		let channels = header.channels as usize;
		let is_mpeg1 = header.version == MpegVersion::Mpeg1;
		let num_granules = if is_mpeg1 { 2 } else { 1 };
		let samples_per_granule = if is_mpeg1 { 576 } else { 576 };

		let mut output = Vec::with_capacity(samples_per_granule * num_granules * channels);

		{
			let mut reader = BitReader::new(&decode_data);
			
			for gr in 0..num_granules {
				let mut samples = [[0.0f32; 576]; 2];
				let mut scalefactors = [[[0u8; 22]; 3]; 2];

				for ch in 0..channels {
					let gc = &side_info.granules[gr].channels[ch];

					self.decode_scalefactors(
						&mut reader,
						header,
						gc,
						&side_info.scfsi[ch],
						gr,
						&mut scalefactors[ch],
					)?;

					self.decode_huffman(&mut reader, header, gc, &mut samples[ch])?;

					self.requantize(header, gc, &scalefactors[ch], &mut samples[ch]);
				}

				if channels == 2 {
					self.process_stereo(header, &side_info.granules[gr], &mut samples);
				}

				// Process granule data (needs &mut self)
				for ch in 0..channels {
					let gc = &side_info.granules[gr].channels[ch];
					self.reorder(header, gc, &mut samples[ch]);
					self.alias_reduction(gc, &mut samples[ch]);
					self.imdct(gc, &mut samples[ch], ch);
					self.frequency_inversion(&mut samples[ch]);
				}

				for i in 0..18 {
					for ch in 0..channels {
						let mut subband = [0.0f32; 32];
						for sb in 0..32 {
							subband[sb] = samples[ch][sb * 18 + i];
						}

						let mut pcm = [0.0f32; 32];
						self.synth[ch].process(&subband, ch, &mut pcm);

						for sample in &pcm {
							let clamped = (*sample * 32768.0).clamp(-32768.0, 32767.0) as i16;
							output.push(clamped);
						}
					}
				}
			}
		}

		let max_buffer_size = 2048;
		if self.main_data_buffer.len() > max_buffer_size {
			let drain = self.main_data_buffer.len() - max_buffer_size;
			self.main_data_buffer.drain(0..drain);
		}

		Some(output)
	}

	fn decode_scalefactors(
		&self,
		reader: &mut BitReader,
		header: &FrameHeader,
		gc: &GranuleChannel,
		scfsi: &[bool; 4],
		_gr: usize,
		scalefactors: &mut [[u8; 22]; 3],
	) -> Option<()> {
		let is_mpeg1 = header.version == MpegVersion::Mpeg1;

		if is_mpeg1 {
			let slen0 = SLEN_TABLE[0][gc.scalefac_compress as usize] as u32;
			let slen1 = SLEN_TABLE[1][gc.scalefac_compress as usize] as u32;

			if gc.window_switching && gc.block_type == 2 {
				if gc.mixed_block {
					for i in 0..8 {
						scalefactors[0][i] = reader.read_bits(slen0)? as u8;
					}
					for i in 3..6 {
						for sfb in 0..3 {
							scalefactors[sfb][i] = reader.read_bits(slen0)? as u8;
						}
					}
					for i in 6..12 {
						for sfb in 0..3 {
							scalefactors[sfb][i] = reader.read_bits(slen1)? as u8;
						}
					}
				} else {
					for i in 0..6 {
						for sfb in 0..3 {
							scalefactors[sfb][i] = reader.read_bits(slen0)? as u8;
						}
					}
					for i in 6..12 {
						for sfb in 0..3 {
							scalefactors[sfb][i] = reader.read_bits(slen1)? as u8;
						}
					}
				}
			} else {
				let bands = [0, 6, 11, 16, 21];
				for region in 0..4 {
					let slen = if region < 2 { slen0 } else { slen1 };
					if !scfsi[region] {
						for i in bands[region]..bands[region + 1] {
							scalefactors[0][i] = reader.read_bits(slen)? as u8;
						}
					}
				}
			}
		} else {
			let slen = [
				(gc.scalefac_compress >> 4) as u32,
				((gc.scalefac_compress >> 2) & 0x03) as u32,
				(gc.scalefac_compress & 0x03) as u32,
			];

			if gc.window_switching && gc.block_type == 2 {
				for i in 0..6 {
					for sfb in 0..3 {
						if slen[0] > 0 {
							scalefactors[sfb][i] = reader.read_bits(slen[0])? as u8;
						}
					}
				}
				for i in 6..12 {
					for sfb in 0..3 {
						if slen[1] > 0 {
							scalefactors[sfb][i] = reader.read_bits(slen[1])? as u8;
						}
					}
				}
			} else {
				for i in 0..6 {
					if slen[0] > 0 {
						scalefactors[0][i] = reader.read_bits(slen[0])? as u8;
					}
				}
				for i in 6..12 {
					if slen[1] > 0 {
						scalefactors[0][i] = reader.read_bits(slen[1])? as u8;
					}
				}
				for i in 12..21 {
					if slen[2] > 0 {
						scalefactors[0][i] = reader.read_bits(slen[2])? as u8;
					}
				}
			}
		}

		Some(())
	}

	fn decode_huffman(
		&self,
		reader: &mut BitReader,
		header: &FrameHeader,
		gc: &GranuleChannel,
		samples: &mut [f32; 576],
	) -> Option<()> {
		let sr_idx = get_sample_rate_index(header.sample_rate);
		let is_mpeg1 = header.version == MpegVersion::Mpeg1;

		let sfb_long = if is_mpeg1 { &SCALEFACTOR_BAND_LONG[sr_idx] } else { &SCALEFACTOR_BAND_LONG_MPEG2[sr_idx] };

		let region1_start = sfb_long[(gc.region0_count + 1) as usize].min(576);
		let region2_start = sfb_long[(gc.region0_count + gc.region1_count + 2) as usize].min(576);

		let big_values_end = (gc.big_values as usize * 2).min(576);

		let mut i = 0;

		while i < region1_start.min(big_values_end) {
			let table = gc.table_select[0] as usize;
			if table == 0 {
				samples[i] = 0.0;
				samples[i + 1] = 0.0;
			} else {
				let linbits = HUFFMAN_TABLES[table.min(33)].linbits;
				let (x, y) = decode_huffman_pair(reader, table, linbits)?;
				samples[i] = x as f32;
				samples[i + 1] = y as f32;
			}
			i += 2;
		}

		while i < region2_start.min(big_values_end) {
			let table = gc.table_select[1] as usize;
			if table == 0 {
				samples[i] = 0.0;
				samples[i + 1] = 0.0;
			} else {
				let linbits = HUFFMAN_TABLES[table.min(33)].linbits;
				let (x, y) = decode_huffman_pair(reader, table, linbits)?;
				samples[i] = x as f32;
				samples[i + 1] = y as f32;
			}
			i += 2;
		}

		while i < big_values_end {
			let table = gc.table_select[2] as usize;
			if table == 0 {
				samples[i] = 0.0;
				samples[i + 1] = 0.0;
			} else {
				let linbits = HUFFMAN_TABLES[table.min(33)].linbits;
				let (x, y) = decode_huffman_pair(reader, table, linbits)?;
				samples[i] = x as f32;
				samples[i + 1] = y as f32;
			}
			i += 2;
		}

		let quad_table = if gc.count1table_select { 1 } else { 0 };
		while i + 4 <= 576 {
			let (v, w, x, y) = decode_huffman_quad(reader, quad_table)?;
			if v == 0 && w == 0 && x == 0 && y == 0 {
				break;
			}
			samples[i] = v as f32;
			samples[i + 1] = w as f32;
			samples[i + 2] = x as f32;
			samples[i + 3] = y as f32;
			i += 4;
		}

		while i < 576 {
			samples[i] = 0.0;
			i += 1;
		}

		Some(())
	}

	fn requantize(
		&self,
		header: &FrameHeader,
		gc: &GranuleChannel,
		scalefactors: &[[u8; 22]; 3],
		samples: &mut [f32; 576],
	) {
		let sr_idx = get_sample_rate_index(header.sample_rate);
		let is_mpeg1 = header.version == MpegVersion::Mpeg1;

		let sfb_long = if is_mpeg1 { &SCALEFACTOR_BAND_LONG[sr_idx] } else { &SCALEFACTOR_BAND_LONG_MPEG2[sr_idx] };
		let sfb_short = if is_mpeg1 { &SCALEFACTOR_BAND_SHORT[sr_idx] } else { &SCALEFACTOR_BAND_SHORT_MPEG2[sr_idx] };

		let global_gain = gc.global_gain as f32;
		let scalefac_scale = if gc.scalefac_scale { 1.0 } else { 0.5 };

		if gc.window_switching && gc.block_type == 2 {
			if gc.mixed_block {
				for sfb in 0..8 {
					let start = sfb_long[sfb];
					let end = sfb_long[sfb + 1];
					let sf = scalefactors[0][sfb] as f32;
					let pretab = if gc.preflag { PRETAB[sfb] as f32 } else { 0.0 };

					for i in start..end {
						if samples[i] != 0.0 {
							let sign = samples[i].signum();
							let val = samples[i].abs();
							let exp = global_gain - 210.0 - 8.0 * gc.subblock_gain[0] as f32
								- scalefac_scale * (sf + pretab);
							samples[i] = sign * val.powf(4.0 / 3.0) * 2.0f32.powf(exp * 0.25);
						}
					}
				}

				for sfb in 3..13 {
					for window in 0..3 {
						let start = sfb_short[sfb] * 3 + window;
						let end = sfb_short[sfb + 1] * 3 + window;
						let sf = scalefactors[window][sfb] as f32;

						for i in (start..end).step_by(3) {
							if i < 576 && samples[i] != 0.0 {
								let sign = samples[i].signum();
								let val = samples[i].abs();
								let exp = global_gain - 210.0 - 8.0 * gc.subblock_gain[window] as f32
									- scalefac_scale * sf;
								samples[i] = sign * val.powf(4.0 / 3.0) * 2.0f32.powf(exp * 0.25);
							}
						}
					}
				}
			} else {
				for sfb in 0..13 {
					for window in 0..3 {
						let start = sfb_short[sfb] * 3 + window;
						let end = sfb_short[sfb + 1] * 3 + window;
						let sf = scalefactors[window][sfb.min(11)] as f32;

						for i in (start..end.min(576)).step_by(3) {
							if samples[i] != 0.0 {
								let sign = samples[i].signum();
								let val = samples[i].abs();
								let exp = global_gain - 210.0 - 8.0 * gc.subblock_gain[window] as f32
									- scalefac_scale * sf;
								samples[i] = sign * val.powf(4.0 / 3.0) * 2.0f32.powf(exp * 0.25);
							}
						}
					}
				}
			}
		} else {
			for sfb in 0..22 {
				let start = sfb_long[sfb];
				let end = sfb_long[sfb + 1].min(576);
				let sf = scalefactors[0][sfb.min(21)] as f32;
				let pretab = if gc.preflag { PRETAB[sfb.min(21)] as f32 } else { 0.0 };

				for i in start..end {
					if samples[i] != 0.0 {
						let sign = samples[i].signum();
						let val = samples[i].abs();
						let exp = global_gain - 210.0 - scalefac_scale * (sf + pretab);
						samples[i] = sign * val.powf(4.0 / 3.0) * 2.0f32.powf(exp * 0.25);
					}
				}
			}
		}
	}

	fn process_stereo(
		&self,
		header: &FrameHeader,
		_granule: &super::sideinfo::Granule,
		samples: &mut [[f32; 576]; 2],
	) {
		if header.is_ms_stereo() {
			for i in 0..576 {
				let m = samples[0][i];
				let s = samples[1][i];
				samples[0][i] = (m + s) * 0.707106781;
				samples[1][i] = (m - s) * 0.707106781;
			}
		}
	}

	fn reorder(&self, header: &FrameHeader, gc: &GranuleChannel, samples: &mut [f32; 576]) {
		if !gc.window_switching || gc.block_type != 2 {
			return;
		}

		let sr_idx = get_sample_rate_index(header.sample_rate);
		let is_mpeg1 = header.version == MpegVersion::Mpeg1;
		let sfb_short = if is_mpeg1 { &SCALEFACTOR_BAND_SHORT[sr_idx] } else { &SCALEFACTOR_BAND_SHORT_MPEG2[sr_idx] };

		let mut temp = [0.0f32; 576];
		let start_sfb = if gc.mixed_block { 3 } else { 0 };

		for sfb in start_sfb..13 {
			let width = sfb_short[sfb + 1] - sfb_short[sfb];
			for window in 0..3 {
				for i in 0..width {
					let src = sfb_short[sfb] * 3 + window * width + i;
					let dst = sfb_short[sfb] * 3 + i * 3 + window;
					if src < 576 && dst < 576 {
						temp[dst] = samples[src];
					}
				}
			}
		}

		let start = if gc.mixed_block { sfb_short[3] * 3 } else { 0 };
		samples[start..576].copy_from_slice(&temp[start..576]);
	}

	fn alias_reduction(&self, gc: &GranuleChannel, samples: &mut [f32; 576]) {
		if gc.window_switching && gc.block_type == 2 && !gc.mixed_block {
			return;
		}

		let bands = if gc.window_switching && gc.mixed_block { 1 } else { 31 };

		for sb in 0..bands {
			for i in 0..8 {
				let idx1 = 18 * (sb + 1) - 1 - i;
				let idx2 = 18 * (sb + 1) + i;

				if idx2 >= 576 {
					break;
				}

				let a = samples[idx1];
				let b = samples[idx2];

				samples[idx1] = a * CS[i] - b * CA[i];
				samples[idx2] = b * CS[i] + a * CA[i];
			}
		}
	}

	fn imdct(&mut self, gc: &GranuleChannel, samples: &mut [f32; 576], ch: usize) {
		let block_type = gc.block_type;
		let mixed = gc.mixed_block;

		for sb in 0..32 {
			let mut input = [0.0f32; 18];
			for i in 0..18 {
				input[i] = samples[sb * 18 + i];
			}

			let sb_block_type = if mixed && sb < 2 { 0 } else { block_type };

			if sb_block_type == 2 {
				let mut output = [0.0f32; 36];
				for window in 0..3 {
					let mut short_input = [0.0f32; 6];
					for i in 0..6 {
						short_input[i] = input[window * 6 + i];
					}
					let mut short_output = [0.0f32; 12];
					super::synth::imdct_12(&short_input, &mut short_output);

					for i in 0..12 {
						output[6 + window + i * 3] += short_output[i];
					}
				}
				apply_window(&mut output, 2, false);

				for i in 0..18 {
					samples[sb * 18 + i] = output[i] + self.prev_samples[ch][sb * 18 + i];
					self.prev_samples[ch][sb * 18 + i] = output[i + 18];
				}
			} else {
				let mut output = [0.0f32; 36];
				imdct_36(&input, &mut output);
				apply_window(&mut output, sb_block_type, mixed);

				for i in 0..18 {
					samples[sb * 18 + i] = output[i] + self.prev_samples[ch][sb * 18 + i];
					self.prev_samples[ch][sb * 18 + i] = output[i + 18];
				}
			}
		}
	}

	fn frequency_inversion(&self, samples: &mut [f32; 576]) {
		for sb in (1..32).step_by(2) {
			for i in (1..18).step_by(2) {
				samples[sb * 18 + i] = -samples[sb * 18 + i];
			}
		}
	}

	pub fn reset(&mut self) {
		self.synth[0].reset();
		self.synth[1].reset();
		self.prev_samples = [[0.0; 576]; 2];
		self.main_data_buffer.clear();
	}
}
