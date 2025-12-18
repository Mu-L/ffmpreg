use super::{BoxType, Mp4Format};
use crate::core::{Muxer, Packet};
use crate::io::{IoResult, MediaSeek, MediaWrite, SeekFrom, WritePrimitives};

pub struct Mp4Writer<W: MediaWrite + MediaSeek> {
	writer: W,
	format: Mp4Format,
	samples: Vec<SampleInfo>,
	mdat_start: u64,
	mdat_size: u64,
}

struct SampleInfo {
	size: u32,
	#[allow(dead_code)]
	duration: u32,
	stream_index: usize,
}

impl<W: MediaWrite + MediaSeek> Mp4Writer<W> {
	pub fn new(mut writer: W, format: Mp4Format) -> IoResult<Self> {
		Self::write_ftyp(&mut writer, &format)?;
		let mdat_start = Self::write_mdat_header(&mut writer)?;

		Ok(Self { writer, format, samples: Vec::new(), mdat_start, mdat_size: 0 })
	}

	fn write_ftyp(writer: &mut W, format: &Mp4Format) -> IoResult<()> {
		let brands_size = format.compatible_brands.len() * 4;
		let box_size = (8 + 8 + brands_size) as u32;

		writer.write_u32_be(box_size)?;
		writer.write_all(&BoxType::Ftyp.as_fourcc())?;
		writer.write_all(&format.major_brand)?;
		writer.write_u32_be(format.minor_version)?;

		for brand in &format.compatible_brands {
			writer.write_all(brand)?;
		}

		Ok(())
	}

	fn write_mdat_header(writer: &mut W) -> IoResult<u64> {
		writer.write_u32_be(1)?;
		writer.write_all(&BoxType::Mdat.as_fourcc())?;
		writer.write_u64_be(0)?;
		Ok(writer.stream_position()?)
	}

	fn write_moov(&mut self) -> IoResult<()> {
		let moov_start = self.writer.stream_position()?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(&BoxType::Moov.as_fourcc())?;

		self.write_mvhd()?;

		let tracks: Vec<_> = self.format.tracks.iter().cloned().collect();
		for (track_idx, track) in tracks.iter().enumerate() {
			self.write_trak(track_idx, track)?;
		}

		let moov_end = self.writer.stream_position()?;
		let moov_size = (moov_end - moov_start) as u32;
		self.writer.seek(SeekFrom::Start(moov_start))?;
		self.writer.write_u32_be(moov_size)?;
		self.writer.seek(SeekFrom::Start(moov_end))?;

		Ok(())
	}

	fn write_mvhd(&mut self) -> IoResult<()> {
		let mvhd_size = 108u32;
		self.writer.write_u32_be(mvhd_size)?;
		self.writer.write_all(&BoxType::Mvhd.as_fourcc())?;

		self.writer.write_u8(0)?;
		self.writer.write_all(&[0u8; 3])?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(self.format.timescale)?;
		self.writer.write_u32_be(self.format.duration as u32)?;
		self.writer.write_u32_be(0x00010000)?;
		self.writer.write_u16_be(0x0100)?;
		self.writer.write_all(&[0u8; 10])?;

		let matrix: [u32; 9] = [0x00010000, 0, 0, 0, 0x00010000, 0, 0, 0, 0x40000000];
		for &val in &matrix {
			self.writer.write_u32_be(val)?;
		}

		self.writer.write_all(&[0u8; 24])?;
		self.writer.write_u32_be((self.format.tracks.len() + 1) as u32)?;

		Ok(())
	}

	fn write_trak(&mut self, track_idx: usize, track: &super::Mp4Track) -> IoResult<()> {
		let trak_start = self.writer.stream_position()?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(&BoxType::Trak.as_fourcc())?;

		self.write_tkhd(track)?;
		self.write_mdia(track_idx, track)?;

		let trak_end = self.writer.stream_position()?;
		let trak_size = (trak_end - trak_start) as u32;
		self.writer.seek(SeekFrom::Start(trak_start))?;
		self.writer.write_u32_be(trak_size)?;
		self.writer.seek(SeekFrom::Start(trak_end))?;

		Ok(())
	}

	fn write_tkhd(&mut self, track: &super::Mp4Track) -> IoResult<()> {
		let tkhd_size = 92u32;
		self.writer.write_u32_be(tkhd_size)?;
		self.writer.write_all(&BoxType::Tkhd.as_fourcc())?;

		self.writer.write_u8(0)?;
		self.writer.write_all(&[0x00, 0x00, 0x03])?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(track.track_id)?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(track.duration as u32)?;
		self.writer.write_all(&[0u8; 8])?;
		self.writer.write_u16_be(0)?;
		self.writer.write_u16_be(0)?;
		self.writer.write_u16_be(0x0100)?;
		self.writer.write_u16_be(0)?;

		let matrix: [u32; 9] = [0x00010000, 0, 0, 0, 0x00010000, 0, 0, 0, 0x40000000];
		for &val in &matrix {
			self.writer.write_u32_be(val)?;
		}

		self.writer.write_u32_be(track.width << 16)?;
		self.writer.write_u32_be(track.height << 16)?;

		Ok(())
	}

	fn write_mdia(&mut self, track_idx: usize, track: &super::Mp4Track) -> IoResult<()> {
		let mdia_start = self.writer.stream_position()?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(&BoxType::Mdia.as_fourcc())?;

		self.write_mdhd(track)?;
		self.write_hdlr(track)?;
		self.write_minf(track_idx, track)?;

		let mdia_end = self.writer.stream_position()?;
		let mdia_size = (mdia_end - mdia_start) as u32;
		self.writer.seek(SeekFrom::Start(mdia_start))?;
		self.writer.write_u32_be(mdia_size)?;
		self.writer.seek(SeekFrom::Start(mdia_end))?;

		Ok(())
	}

	fn write_mdhd(&mut self, track: &super::Mp4Track) -> IoResult<()> {
		let mdhd_size = 32u32;
		self.writer.write_u32_be(mdhd_size)?;
		self.writer.write_all(&BoxType::Mdhd.as_fourcc())?;

		self.writer.write_u8(0)?;
		self.writer.write_all(&[0u8; 3])?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(track.timescale)?;
		self.writer.write_u32_be(track.duration as u32)?;
		self.writer.write_u16_be(0x55C4)?;
		self.writer.write_u16_be(0)?;

		Ok(())
	}

	fn write_hdlr(&mut self, track: &super::Mp4Track) -> IoResult<()> {
		let handler_type = match track.track_type {
			super::TrackType::Video => b"vide",
			super::TrackType::Audio => b"soun",
			super::TrackType::Hint => b"hint",
			super::TrackType::Text => b"text",
			super::TrackType::Unknown => b"    ",
		};

		let name = match track.track_type {
			super::TrackType::Video => b"VideoHandler\0",
			super::TrackType::Audio => b"SoundHandler\0",
			_ => b"DataHandler\0\0",
		};

		let hdlr_size = (8 + 4 + 4 + 4 + 12 + name.len()) as u32;
		self.writer.write_u32_be(hdlr_size)?;
		self.writer.write_all(&BoxType::Hdlr.as_fourcc())?;

		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(handler_type)?;
		self.writer.write_all(&[0u8; 12])?;
		self.writer.write_all(name)?;

		Ok(())
	}

	fn write_minf(&mut self, track_idx: usize, track: &super::Mp4Track) -> IoResult<()> {
		let minf_start = self.writer.stream_position()?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(&BoxType::Minf.as_fourcc())?;

		match track.track_type {
			super::TrackType::Video => self.write_vmhd()?,
			super::TrackType::Audio => self.write_smhd()?,
			_ => {}
		}

		self.write_dinf()?;
		self.write_stbl(track_idx, track)?;

		let minf_end = self.writer.stream_position()?;
		let minf_size = (minf_end - minf_start) as u32;
		self.writer.seek(SeekFrom::Start(minf_start))?;
		self.writer.write_u32_be(minf_size)?;
		self.writer.seek(SeekFrom::Start(minf_end))?;

		Ok(())
	}

	fn write_vmhd(&mut self) -> IoResult<()> {
		self.writer.write_u32_be(20)?;
		self.writer.write_all(b"vmhd")?;
		self.writer.write_u32_be(0x0001)?;
		self.writer.write_all(&[0u8; 8])?;
		Ok(())
	}

	fn write_smhd(&mut self) -> IoResult<()> {
		self.writer.write_u32_be(16)?;
		self.writer.write_all(b"smhd")?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(&[0u8; 4])?;
		Ok(())
	}

	fn write_dinf(&mut self) -> IoResult<()> {
		self.writer.write_u32_be(36)?;
		self.writer.write_all(b"dinf")?;

		self.writer.write_u32_be(28)?;
		self.writer.write_all(b"dref")?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(1)?;

		self.writer.write_u32_be(12)?;
		self.writer.write_all(b"url ")?;
		self.writer.write_u32_be(0x0001)?;

		Ok(())
	}

	fn write_stbl(&mut self, track_idx: usize, track: &super::Mp4Track) -> IoResult<()> {
		let stbl_start = self.writer.stream_position()?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(&BoxType::Stbl.as_fourcc())?;

		self.write_stsd(track)?;
		self.write_stts(track_idx)?;
		self.write_stsc(track_idx)?;
		self.write_stsz(track_idx)?;
		self.write_stco(track_idx)?;

		let stbl_end = self.writer.stream_position()?;
		let stbl_size = (stbl_end - stbl_start) as u32;
		self.writer.seek(SeekFrom::Start(stbl_start))?;
		self.writer.write_u32_be(stbl_size)?;
		self.writer.seek(SeekFrom::Start(stbl_end))?;

		Ok(())
	}

	fn write_stsd(&mut self, track: &super::Mp4Track) -> IoResult<()> {
		let stsd_start = self.writer.stream_position()?;
		self.writer.write_u32_be(0)?;
		self.writer.write_all(&BoxType::Stsd.as_fourcc())?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(1)?;

		match track.track_type {
			super::TrackType::Video => {
				self.writer.write_u32_be(86)?;
				self.writer.write_all(b"avc1")?;
				self.writer.write_all(&[0u8; 6])?;
				self.writer.write_u16_be(1)?;
				self.writer.write_all(&[0u8; 16])?;
				self.writer.write_u16_be(track.width as u16)?;
				self.writer.write_u16_be(track.height as u16)?;
				self.writer.write_u32_be(0x00480000)?;
				self.writer.write_u32_be(0x00480000)?;
				self.writer.write_u32_be(0)?;
				self.writer.write_u16_be(1)?;
				self.writer.write_all(&[0u8; 32])?;
				self.writer.write_u16_be(0x0018)?;
				self.writer.write_i16_be(-1)?;
			}
			super::TrackType::Audio => {
				self.writer.write_u32_be(36)?;
				self.writer.write_all(b"mp4a")?;
				self.writer.write_all(&[0u8; 6])?;
				self.writer.write_u16_be(1)?;
				self.writer.write_all(&[0u8; 8])?;
				self.writer.write_u16_be(track.channels)?;
				self.writer.write_u16_be(16)?;
				self.writer.write_u32_be(0)?;
				self.writer.write_u32_be(track.sample_rate << 16)?;
			}
			_ => {}
		}

		let stsd_end = self.writer.stream_position()?;
		let stsd_size = (stsd_end - stsd_start) as u32;
		self.writer.seek(SeekFrom::Start(stsd_start))?;
		self.writer.write_u32_be(stsd_size)?;
		self.writer.seek(SeekFrom::Start(stsd_end))?;

		Ok(())
	}

	fn write_stts(&mut self, track_idx: usize) -> IoResult<()> {
		let track_samples: Vec<_> =
			self.samples.iter().filter(|s| s.stream_index == track_idx).collect();

		let stts_size = (16 + 8 * 1) as u32;
		self.writer.write_u32_be(stts_size)?;
		self.writer.write_all(&BoxType::Stts.as_fourcc())?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(1)?;
		self.writer.write_u32_be(track_samples.len() as u32)?;
		self.writer.write_u32_be(if track_samples.is_empty() { 1 } else { 1024 })?;

		Ok(())
	}

	fn write_stsc(&mut self, _track_idx: usize) -> IoResult<()> {
		self.writer.write_u32_be(28)?;
		self.writer.write_all(&BoxType::Stsc.as_fourcc())?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(1)?;
		self.writer.write_u32_be(1)?;
		self.writer.write_u32_be(1)?;
		self.writer.write_u32_be(1)?;

		Ok(())
	}

	fn write_stsz(&mut self, track_idx: usize) -> IoResult<()> {
		let track_samples: Vec<_> =
			self.samples.iter().filter(|s| s.stream_index == track_idx).collect();

		let stsz_size = (20 + 4 * track_samples.len()) as u32;
		self.writer.write_u32_be(stsz_size)?;
		self.writer.write_all(&BoxType::Stsz.as_fourcc())?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(track_samples.len() as u32)?;

		for sample in &track_samples {
			self.writer.write_u32_be(sample.size)?;
		}

		Ok(())
	}

	fn write_stco(&mut self, track_idx: usize) -> IoResult<()> {
		let track_samples: Vec<_> =
			self.samples.iter().filter(|s| s.stream_index == track_idx).collect();

		let stco_size = (16 + 4 * track_samples.len()) as u32;
		self.writer.write_u32_be(stco_size)?;
		self.writer.write_all(&BoxType::Stco.as_fourcc())?;
		self.writer.write_u32_be(0)?;
		self.writer.write_u32_be(track_samples.len() as u32)?;

		let mut offset = self.mdat_start as u32;
		for sample in &track_samples {
			self.writer.write_u32_be(offset)?;
			offset += sample.size;
		}

		Ok(())
	}
}

impl<W: MediaWrite + MediaSeek> Muxer for Mp4Writer<W> {
	fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
		let size = packet.data.len() as u32;
		self.writer.write_all(&packet.data)?;
		self.mdat_size += size as u64;

		self.samples.push(SampleInfo { size, duration: 1024, stream_index: packet.stream_index });

		Ok(())
	}

	fn finalize(&mut self) -> IoResult<()> {
		let mdat_end = self.writer.stream_position()?;
		let mdat_total_size = mdat_end - self.mdat_start + 16;
		self.writer.seek(SeekFrom::Start(self.mdat_start - 8))?;
		self.writer.write_u64_be(mdat_total_size)?;
		self.writer.seek(SeekFrom::Start(mdat_end))?;

		self.write_moov()?;

		self.writer.flush()?;
		Ok(())
	}
}
