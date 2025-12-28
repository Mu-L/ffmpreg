pub struct Cursor<T> {
	inner: T,
	pos: u64,
}

impl<T> Cursor<T> {
	#[inline]
	pub const fn new(inner: T) -> Self {
		Self { inner, pos: 0 }
	}

	#[inline]
	pub fn into_inner(self) -> T {
		self.inner
	}

	#[inline]
	pub const fn get_ref(&self) -> &T {
		&self.inner
	}

	#[inline]
	pub fn get_mut(&mut self) -> &mut T {
		&mut self.inner
	}

	#[inline]
	pub const fn position(&self) -> u64 {
		self.pos
	}

	#[inline]
	pub fn set_position(&mut self, pos: u64) {
		self.pos = pos;
	}
}

impl<T: AsRef<[u8]>> crate::io::MediaRead for Cursor<T> {
	fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
		let slice = self.inner.as_ref();
		let pos = self.pos as usize;
		if pos >= slice.len() {
			return Ok(0);
		}
		let remaining = &slice[pos..];
		let amt = core::cmp::min(remaining.len(), buf.len());
		buf[..amt].copy_from_slice(&remaining[..amt]);
		self.pos += amt as u64;
		Ok(amt)
	}
}

impl crate::io::MediaWrite for Cursor<Vec<u8>> {
	fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
		let pos = self.pos as usize;
		let len = self.inner.len();

		if pos > len {
			self.inner.resize(pos, 0);
		}

		let space = self.inner.len().saturating_sub(pos);
		let overwrite = core::cmp::min(space, buf.len());
		self.inner[pos..pos + overwrite].copy_from_slice(&buf[..overwrite]);

		if buf.len() > overwrite {
			self.inner.extend_from_slice(&buf[overwrite..]);
		}

		self.pos += buf.len() as u64;
		Ok(buf.len())
	}

	#[inline]
	fn flush(&mut self) -> crate::io::Result<()> {
		Ok(())
	}
}

impl<T: AsRef<[u8]>> crate::io::MediaSeek for Cursor<T> {
	fn seek(&mut self, pos: crate::io::SeekFrom) -> crate::io::Result<u64> {
		let len = self.inner.as_ref().len() as i64;
		let new_pos = match pos {
			crate::io::SeekFrom::Start(n) => n as i64,
			crate::io::SeekFrom::End(n) => len + n,
			crate::io::SeekFrom::Current(n) => self.pos as i64 + n,
		};
		if new_pos < 0 {
			return Err(crate::io::Error::invalid_data("seek to negative position"));
		}
		self.pos = new_pos as u64;
		Ok(self.pos)
	}
}
