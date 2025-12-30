use crate::io::{Error, ErrorKind, Result};

pub trait MediaWrite {
	fn write(&mut self, buf: &[u8]) -> Result<usize>;

	fn flush(&mut self) -> Result<()>;
}

pub trait WritePrimitives: MediaWrite {
	fn write_all(&mut self, buf: &[u8]) -> Result<()> {
		let mut written = 0;
		while written < buf.len() {
			match self.write(&buf[written..]) {
				Ok(0) => return Err(Error::write_zero()),
				Ok(n) => written += n,
				Err(e) if matches!(e.kind(), ErrorKind::Interrupted) => continue,
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}

	#[inline]
	fn write_u8(&mut self, value: u8) -> Result<()> {
		self.write_all(&[value])
	}

	#[inline]
	fn write_u16_be(&mut self, value: u16) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_u16_le(&mut self, value: u16) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}

	#[inline]
	fn write_u32_be(&mut self, value: u32) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_u32_le(&mut self, value: u32) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}

	#[inline]
	fn write_u64_be(&mut self, value: u64) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_u64_le(&mut self, value: u64) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}

	#[inline]
	fn write_i8(&mut self, value: i8) -> Result<()> {
		self.write_all(&[value as u8])
	}

	#[inline]
	fn write_i16_be(&mut self, value: i16) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_i16_le(&mut self, value: i16) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}

	#[inline]
	fn write_i32_be(&mut self, value: i32) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_i32_le(&mut self, value: i32) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}

	#[inline]
	fn write_i64_be(&mut self, value: i64) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_i64_le(&mut self, value: i64) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}

	#[inline]
	fn write_f32_be(&mut self, value: f32) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_f32_le(&mut self, value: f32) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}

	#[inline]
	fn write_f64_be(&mut self, value: f64) -> Result<()> {
		self.write_all(&value.to_be_bytes())
	}

	#[inline]
	fn write_f64_le(&mut self, value: f64) -> Result<()> {
		self.write_all(&value.to_le_bytes())
	}
}

impl<T: MediaWrite> WritePrimitives for T {}

pub struct StdWriteAdapter<W> {
	inner: W,
}

impl<W> StdWriteAdapter<W> {
	#[inline]
	pub const fn new(inner: W) -> Self {
		Self { inner }
	}

	#[inline]
	pub fn into_inner(self) -> W {
		self.inner
	}

	#[inline]
	pub const fn get_ref(&self) -> &W {
		&self.inner
	}

	#[inline]
	pub fn get_mut(&mut self) -> &mut W {
		&mut self.inner
	}
}

impl<W: std::io::Write> MediaWrite for StdWriteAdapter<W> {
	#[inline]
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.inner.write(buf).map_err(Error::from)
	}

	#[inline]
	fn flush(&mut self) -> Result<()> {
		self.inner.flush().map_err(Error::from)
	}
}

impl MediaWrite for Vec<u8> {
	#[inline]
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.extend_from_slice(buf);
		Ok(buf.len())
	}

	#[inline]
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}
