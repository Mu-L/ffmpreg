use rustc_hash::FxHashMap;

use crate::io::{Error, Result};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Track {
	One(usize),
	#[default]
	All,
}

impl Track {
	pub fn resolve(&self, max_track: usize) -> Result<Vec<usize>> {
		if let Track::One(idx) = self {
			if *idx >= max_track {
				let msg = format!("track {} out of bounds (max {})", idx, max_track - 1);
				return Err(Error::invalid_data(msg));
			}
			return Ok(vec![*idx]);
		}

		if max_track == 0 {
			return Err(Error::invalid_data("no streams"));
		}

		Ok((0..max_track).collect())
	}

	pub fn uncheck_resolve(&self) -> usize {
		match self {
			Track::One(idx) => *idx,
			Track::All => 0,
		}
	}
}

pub fn parse_track_id(map: &FxHashMap<String, String>) -> Result<Option<usize>> {
	match map.get("track") {
		None => Ok(None),
		Some(s) if s == "all" || s == "*" => Ok(None),
		Some(s) => {
			s.parse::<usize>().map(Some).map_err(|_| Error::invalid_data(format!("invalid track: {}", s)))
		}
	}
}
