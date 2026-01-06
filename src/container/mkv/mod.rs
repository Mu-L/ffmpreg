pub mod cluster;
pub mod codec;
pub mod cues;
pub mod elements;
pub mod header;
pub mod metadata;
pub mod parser;
pub mod track;

use crate::core::ebml::parser::EbmlParser;
use crate::core::ebml::types::EbmlElement;
use cluster::Cluster;
use cues::Cues;
use header::MkvHeader;
use metadata::MkvMetadata;
use nom::IResult;
use track::{MkvTrack, MkvTracks};

pub struct MkvFile {
	pub ebml: Option<EbmlElement>,
	pub segment: Option<EbmlElement>,
	pub header: Option<MkvHeader>,
	pub metadata: MkvMetadata,
	pub tracks: MkvTracks,
	pub clusters: Vec<Cluster>,
	pub cues: Option<Cues>,
	pub attachments: Vec<EbmlElement>,
	pub chapters: Option<EbmlElement>,
	pub tags: Vec<EbmlElement>,
}

impl MkvFile {
	pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
		let (remaining, elements) = EbmlParser::parse_document(input)?;
		let mut file = Self::default();

		for elem in elements {
			match elem.id {
				0x1A45DFA3 => {
					file.ebml = Some(elem.clone());
					file.header = MkvHeader::from_element(&elem);
				}
				0x18538067 => {
					file.segment = Some(elem.clone());
					Self::parse_segment(&elem, &mut file);
				}
				_ => {}
			}
		}

		Ok((remaining, file))
	}

	fn parse_segment(segment: &EbmlElement, file: &mut Self) {
		let children = match segment.as_master() {
			Some(c) => c,
			None => return,
		};

		for child in children {
			match child.id {
				0x1549A966 => {
					if let Some(meta) = MkvMetadata::from_element(child) {
						file.metadata = meta;
					}
				}
				0x1654AE6B => {
					file.tracks = parse_tracks(child);
				}
				0x1F43B675 => {
					if let Some(cluster) = Cluster::from_element(child) {
						file.clusters.push(cluster);
					}
				}
				0x1C53BB6B => {
					file.cues = Cues::from_element(child);
				}
				0x1941A469 => {
					file.attachments.push(child.clone());
				}
				0x1043A770 => {
					file.chapters = Some(child.clone());
				}
				0x1254C367 => {
					file.tags.push(child.clone());
				}
				_ => {}
			}
		}
	}

	pub fn has_header(&self) -> bool {
		self.header.is_some()
	}

	pub fn has_segment(&self) -> bool {
		self.segment.is_some()
	}

	pub fn segment_count(&self) -> usize {
		if self.segment.is_some() { 1 } else { 0 }
	}

	pub fn cluster_count(&self) -> usize {
		self.clusters.len()
	}

	pub fn track_count(&self) -> usize {
		self.tracks.tracks.len()
	}

	pub fn video_track_count(&self) -> usize {
		self.tracks.video_tracks().len()
	}

	pub fn audio_track_count(&self) -> usize {
		self.tracks.audio_tracks().len()
	}

	pub fn subtitle_track_count(&self) -> usize {
		self.tracks.subtitle_tracks().len()
	}

	pub fn has_attachments(&self) -> bool {
		!self.attachments.is_empty()
	}

	pub fn has_chapters(&self) -> bool {
		self.chapters.is_some()
	}

	pub fn has_cues(&self) -> bool {
		self.cues.is_some()
	}

	pub fn has_tags(&self) -> bool {
		!self.tags.is_empty()
	}

	pub fn total_blocks(&self) -> usize {
		self.clusters.iter().map(|c| c.block_count()).sum()
	}
}

impl Default for MkvFile {
	fn default() -> Self {
		MkvFile {
			ebml: None,
			segment: None,
			header: None,
			metadata: MkvMetadata::default(),
			tracks: MkvTracks::new(),
			clusters: Vec::new(),
			cues: None,
			attachments: Vec::new(),
			chapters: None,
			tags: Vec::new(),
		}
	}
}

fn parse_tracks(elem: &EbmlElement) -> MkvTracks {
	let mut tracks = MkvTracks::new();

	let children = match elem.as_master() {
		Some(c) => c.clone(),
		None => {
			if let crate::core::ebml::types::EbmlType::Binary(data) = &elem.data {
				match EbmlParser::parse_document(data) {
					Ok((_, parsed)) => parsed,
					Err(_) => return tracks,
				}
			} else {
				return tracks;
			}
		}
	};

	for child in &children {
		if child.id == 0xAE {
			if let Some(track) = MkvTrack::from_element(child) {
				tracks.tracks.push(track);
			}
		}
	}

	tracks
}

pub fn parse_mkv(input: &[u8]) -> IResult<&[u8], MkvFile> {
	MkvFile::parse(input)
}
