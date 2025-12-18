use super::bits::BitReader;
use super::tables::{HUFFMAN_TABLES, HUFFMAN_TREE_DATA, QUAD_TABLE_A, QUAD_TABLE_B};

pub fn decode_huffman_pair(
	reader: &mut BitReader,
	table_num: usize,
	linbits: u8,
) -> Option<(i32, i32)> {
	if table_num == 0 {
		return Some((0, 0));
	}

	let table = &HUFFMAN_TABLES[table_num];
	if table.tree_len == 0 {
		return Some((0, 0));
	}

	let mut idx = table.tree_offset;
	let end = table.tree_offset + table.tree_len;

	loop {
		if idx >= end || idx >= HUFFMAN_TREE_DATA.len() {
			return None;
		}

		let node = HUFFMAN_TREE_DATA[idx];

		if node >= 0 {
			let x = (node >> 4) & 0x0F;
			let y = node & 0x0F;
			let mut x_val = x as i32;
			let mut y_val = y as i32;

			if linbits > 0 {
				if x_val == 15 {
					x_val += reader.read_bits(linbits as u32)? as i32;
				}
				if y_val == 15 {
					y_val += reader.read_bits(linbits as u32)? as i32;
				}
			}

			if x_val != 0 {
				if reader.read_bit()? {
					x_val = -x_val;
				}
			}
			if y_val != 0 {
				if reader.read_bit()? {
					y_val = -y_val;
				}
			}

			return Some((x_val, y_val));
		}

		let bit = reader.read_bit()?;
		let offset = (-node) as usize;
		idx = table.tree_offset + offset + if bit { 1 } else { 0 };
	}
}

pub fn decode_huffman_quad(
	reader: &mut BitReader,
	table_num: usize,
) -> Option<(i32, i32, i32, i32)> {
	let (v, w, x, y) = if table_num == 0 {
		let bits = reader.read_bits(4)? as usize;
		QUAD_TABLE_A[bits]
	} else {
		let mut code = 0u32;
		for _ in 0..4 {
			code = (code << 1) | (reader.read_bit()? as u32);
			if code < 8 {
				break;
			}
		}
		QUAD_TABLE_B[code.min(15) as usize]
	};

	let mut v_val = v as i32;
	let mut w_val = w as i32;
	let mut x_val = x as i32;
	let mut y_val = y as i32;

	if v != 0 && reader.read_bit()? {
		v_val = -v_val;
	}
	if w != 0 && reader.read_bit()? {
		w_val = -w_val;
	}
	if x != 0 && reader.read_bit()? {
		x_val = -x_val;
	}
	if y != 0 && reader.read_bit()? {
		y_val = -y_val;
	}

	Some((v_val, w_val, x_val, y_val))
}
