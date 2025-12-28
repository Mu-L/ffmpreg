use crate::core::{Muxer, Packet, Stream};
use crate::io::{MediaSeek, MediaWrite, Result, SeekFrom, WritePrimitives};

pub struct MovMuxer<W: MediaWrite + MediaSeek> {
	writer: W,
	streams: Vec<Stream>,
	next_stream_id: u32,
	#[allow(dead_code)]
	ftyp_position: u64,
	moov_position: u64,
	mdat_position: u64,
	mdat_size: u64,
	packets: Vec<(usize, u64, u32)>,
}

impl<W: MediaWrite + MediaSeek> MovMuxer<W> {
	pub fn new(mut writer: W) -> Result<Self> {
		let ftyp_position = writer.stream_position()?;
		Self::write_ftyp(&mut writer)?;

		let moov_position = writer.stream_position()?;
		Self::write_placeholder_moov(&mut writer)?;

		let mdat_position = writer.stream_position()?;
		Self::write_mdat_header(&mut writer)?;

		Ok(Self {
			writer,
			streams: Vec::new(),
			next_stream_id: 1,
			ftyp_position,
			moov_position,
			mdat_position,
			mdat_size: 0,
			packets: Vec::new(),
		})
	}

	pub fn add_stream(&mut self, stream: Stream) -> Result<()> {
		let mut s = stream.clone();
		s.id = self.next_stream_id;
		self.next_stream_id += 1;
		self.streams.push(s);
		Ok(())
	}

	fn write_ftyp(writer: &mut W) -> Result<()> {
		let ftyp_size = 20u32;
		writer.write_u32_be(ftyp_size)?;
		writer.write_all(b"ftyp")?;
		writer.write_all(b"isom")?;
		writer.write_u32_be(512)?;
		writer.write_all(b"iso2")?;
		writer.write_all(b"avc1")?;
		writer.write_all(b"mp42")?;
		Ok(())
	}

	fn write_placeholder_moov(writer: &mut W) -> Result<()> {
		let placeholder_size = 4u32;
		writer.write_u32_be(placeholder_size)?;
		writer.write_all(b"moov")?;
		Ok(())
	}

	fn write_mdat_header(writer: &mut W) -> Result<()> {
		writer.write_u32_be(0)?;
		writer.write_all(b"mdat")?;
		Ok(())
	}

	pub fn streams(&self) -> crate::core::stream::Streams {
		crate::core::stream::Streams::new(self.streams.clone())
	}

	pub fn write_packet(&mut self, packet: Packet) -> Result<()> {
		let offset = self.mdat_size;
		let size = packet.data.len() as u32;

		self.writer.write_all(&packet.data)?;
		self.mdat_size += packet.data.len() as u64;

		self.packets.push((packet.stream_index, offset, size));

		Ok(())
	}

	pub fn finalize(&mut self) -> Result<()> {
		let final_position = self.writer.stream_position()?;

		self.update_mdat_size()?;
		self.update_moov_box()?;

		self.writer.seek(SeekFrom::Start(final_position))?;
		self.writer.flush()?;

		Ok(())
	}

	fn update_mdat_size(&mut self) -> Result<()> {
		let mdat_size = (self.mdat_size + 8) as u32;
		let current_pos = self.writer.stream_position()?;

		self.writer.seek(SeekFrom::Start(self.mdat_position))?;
		self.writer.write_u32_be(mdat_size)?;

		self.writer.seek(SeekFrom::Start(current_pos))?;
		Ok(())
	}

	fn update_moov_box(&mut self) -> Result<()> {
		let moov_data = self.build_moov_box()?;
		let current_pos = self.writer.stream_position()?;

		self.writer.seek(SeekFrom::Start(self.moov_position))?;
		self.writer.write_all(&moov_data)?;

		self.writer.seek(SeekFrom::Start(current_pos))?;
		Ok(())
	}

	fn build_moov_box(&self) -> Result<Vec<u8>> {
		let mut moov = Vec::new();

		let moov_content = self.build_moov_content()?;
		let moov_size = (moov_content.len() + 8) as u32;

		moov.extend_from_slice(&moov_size.to_be_bytes());
		moov.extend_from_slice(b"moov");
		moov.extend_from_slice(&moov_content);

		Ok(moov)
	}

	fn build_moov_content(&self) -> Result<Vec<u8>> {
		let mut content = Vec::new();

		let mvhd = self.build_mvhd_box()?;
		content.extend_from_slice(&mvhd);

		for stream in &self.streams {
			let trak = self.build_trak_box(stream)?;
			content.extend_from_slice(&trak);
		}

		Ok(content)
	}

	fn build_mvhd_box(&self) -> Result<Vec<u8>> {
		let mut mvhd = Vec::with_capacity(108);
		mvhd.extend_from_slice(&108u32.to_be_bytes());
		mvhd.extend_from_slice(b"mvhd");
		mvhd.extend_from_slice(&[0u8; 24]);
		mvhd.extend_from_slice(&1000u32.to_be_bytes());
		mvhd.extend_from_slice(&1u32.to_be_bytes());
		mvhd.extend_from_slice(&0x00010000u32.to_be_bytes());
		mvhd.extend_from_slice(&[0u8; 10]);
		mvhd.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]);
		mvhd.extend_from_slice(&[0u8; 24]);
		mvhd.extend_from_slice(&2u32.to_be_bytes());
		Ok(mvhd)
	}

	fn build_trak_box(&self, stream: &Stream) -> Result<Vec<u8>> {
		let mut trak = Vec::new();

		let tkhd = self.build_tkhd_box(stream)?;
		let mdia = self.build_mdia_box(stream)?;

		let trak_content_size = tkhd.len() + mdia.len();
		let trak_size = (trak_content_size + 8) as u32;

		trak.extend_from_slice(&trak_size.to_be_bytes());
		trak.extend_from_slice(b"trak");
		trak.extend_from_slice(&tkhd);
		trak.extend_from_slice(&mdia);

		Ok(trak)
	}

	fn build_tkhd_box(&self, stream: &Stream) -> Result<Vec<u8>> {
		let mut tkhd = Vec::with_capacity(92);
		tkhd.extend_from_slice(&92u32.to_be_bytes());
		tkhd.extend_from_slice(b"tkhd");
		tkhd.extend_from_slice(&[0u8; 8]);
		tkhd.extend_from_slice(&stream.id.to_be_bytes());
		tkhd.extend_from_slice(&[0u8; 8]);
		tkhd.extend_from_slice(&1u32.to_be_bytes());
		tkhd.extend_from_slice(&[0u8; 48]);
		Ok(tkhd)
	}

	fn build_mdia_box(&self, stream: &Stream) -> Result<Vec<u8>> {
		let mut mdia = Vec::new();

		let mdhd = self.build_mdhd_box(stream)?;
		let hdlr = self.build_hdlr_box(stream)?;
		let minf = self.build_minf_box(stream)?;

		let mdia_content_size = mdhd.len() + hdlr.len() + minf.len();
		let mdia_size = (mdia_content_size + 8) as u32;

		mdia.extend_from_slice(&mdia_size.to_be_bytes());
		mdia.extend_from_slice(b"mdia");
		mdia.extend_from_slice(&mdhd);
		mdia.extend_from_slice(&hdlr);
		mdia.extend_from_slice(&minf);

		Ok(mdia)
	}

	fn build_mdhd_box(&self, stream: &Stream) -> Result<Vec<u8>> {
		let mut mdhd = Vec::with_capacity(32);
		mdhd.extend_from_slice(&32u32.to_be_bytes());
		mdhd.extend_from_slice(b"mdhd");
		mdhd.extend_from_slice(&[0u8; 8]);
		mdhd.extend_from_slice(&stream.time.den.to_be_bytes());
		mdhd.extend_from_slice(&[0u8; 4]);
		Ok(mdhd)
	}

	fn build_hdlr_box(&self, stream: &Stream) -> Result<Vec<u8>> {
		let mut hdlr = Vec::with_capacity(33);
		hdlr.extend_from_slice(&33u32.to_be_bytes());
		hdlr.extend_from_slice(b"hdlr");
		hdlr.extend_from_slice(&[0u8; 8]);
		let handler = if stream.is_audio() { b"soun" } else { b"vide" };
		hdlr.extend_from_slice(handler);
		hdlr.extend_from_slice(&[0u8; 12]);
		Ok(hdlr)
	}

	fn build_minf_box(&self, stream: &Stream) -> Result<Vec<u8>> {
		let mut minf = Vec::new();

		let smhd_or_vmhd =
			if stream.is_audio() { self.build_smhd_box()? } else { self.build_vmhd_box()? };

		let dinf = self.build_dinf_box()?;
		let stbl = self.build_stbl_box(stream)?;

		let minf_content_size = smhd_or_vmhd.len() + dinf.len() + stbl.len();
		let minf_size = (minf_content_size + 8) as u32;

		minf.extend_from_slice(&minf_size.to_be_bytes());
		minf.extend_from_slice(b"minf");
		minf.extend_from_slice(&smhd_or_vmhd);
		minf.extend_from_slice(&dinf);
		minf.extend_from_slice(&stbl);

		Ok(minf)
	}

	fn build_smhd_box(&self) -> Result<Vec<u8>> {
		let mut smhd = Vec::with_capacity(16);
		smhd.extend_from_slice(&16u32.to_be_bytes());
		smhd.extend_from_slice(b"smhd");
		smhd.extend_from_slice(&[0u8; 8]);
		Ok(smhd)
	}

	fn build_vmhd_box(&self) -> Result<Vec<u8>> {
		let mut vmhd = Vec::with_capacity(20);
		vmhd.extend_from_slice(&20u32.to_be_bytes());
		vmhd.extend_from_slice(b"vmhd");
		vmhd.extend_from_slice(&[0u8; 16]);
		Ok(vmhd)
	}

	fn build_dinf_box(&self) -> Result<Vec<u8>> {
		let dref = self.build_dref_box()?;
		let dinf_size = (dref.len() + 8) as u32;
		let mut dinf = Vec::with_capacity(dinf_size as usize);
		dinf.extend_from_slice(&dinf_size.to_be_bytes());
		dinf.extend_from_slice(b"dinf");
		dinf.extend_from_slice(&dref);
		Ok(dinf)
	}

	fn build_dref_box(&self) -> Result<Vec<u8>> {
		let mut dref = Vec::with_capacity(16);
		dref.extend_from_slice(&16u32.to_be_bytes());
		dref.extend_from_slice(b"dref");
		dref.extend_from_slice(&[0u8; 4]);
		dref.extend_from_slice(&1u32.to_be_bytes());
		Ok(dref)
	}

	fn build_stbl_box(&self, stream: &Stream) -> Result<Vec<u8>> {
		let mut stbl = Vec::new();

		let stsd = self.build_stsd_box(stream)?;
		let stts = self.build_stts_box()?;
		let stss = self.build_stss_box()?;
		let stsz = self.build_stsz_box()?;
		let stco = self.build_stco_box()?;

		let stbl_content_size = stsd.len() + stts.len() + stss.len() + stsz.len() + stco.len();
		let stbl_size = (stbl_content_size + 8) as u32;

		stbl.extend_from_slice(&stbl_size.to_be_bytes());
		stbl.extend_from_slice(b"stbl");
		stbl.extend_from_slice(&stsd);
		stbl.extend_from_slice(&stts);
		stbl.extend_from_slice(&stss);
		stbl.extend_from_slice(&stsz);
		stbl.extend_from_slice(&stco);

		Ok(stbl)
	}

	fn build_stsd_box(&self, _stream: &Stream) -> Result<Vec<u8>> {
		let mut stsd = Vec::with_capacity(20);
		stsd.extend_from_slice(&20u32.to_be_bytes());
		stsd.extend_from_slice(b"stsd");
		stsd.extend_from_slice(&[0u8; 4]);
		stsd.extend_from_slice(&1u32.to_be_bytes());
		Ok(stsd)
	}

	fn build_stts_box(&self) -> Result<Vec<u8>> {
		let mut stts = Vec::with_capacity(16);
		stts.extend_from_slice(&16u32.to_be_bytes());
		stts.extend_from_slice(b"stts");
		stts.extend_from_slice(&[0u8; 8]);
		Ok(stts)
	}

	fn build_stss_box(&self) -> Result<Vec<u8>> {
		let mut stss = Vec::with_capacity(16);
		stss.extend_from_slice(&16u32.to_be_bytes());
		stss.extend_from_slice(b"stss");
		stss.extend_from_slice(&[0u8; 8]);
		Ok(stss)
	}

	fn build_stsz_box(&self) -> Result<Vec<u8>> {
		let mut stsz = Vec::with_capacity(20);
		stsz.extend_from_slice(&20u32.to_be_bytes());
		stsz.extend_from_slice(b"stsz");
		stsz.extend_from_slice(&[0u8; 12]);
		Ok(stsz)
	}

	fn build_stco_box(&self) -> Result<Vec<u8>> {
		let mut stco = Vec::with_capacity(16);
		stco.extend_from_slice(&16u32.to_be_bytes());
		stco.extend_from_slice(b"stco");
		stco.extend_from_slice(&[0u8; 8]);
		Ok(stco)
	}
}

impl<W: MediaWrite + MediaSeek> Muxer for MovMuxer<W> {
	fn streams(&self) -> &crate::core::stream::Streams {
		unimplemented!("use the public streams() method instead")
	}

	fn write(&mut self, packet: Packet) -> Result<()> {
		self.write_packet(packet)
	}

	fn finalize(&mut self) -> Result<()> {
		self.finalize()
	}
}
