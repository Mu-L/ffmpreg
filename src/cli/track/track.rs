use crate::{
	cli::track::utils::Kv,
	io::{Error, Result},
};

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
				let message = format!("track index '{}' out of bounds, max '{}'", idx, max_track - 1);
				return Err(Error::invalid_data(message));
			}
			return Ok(vec![*idx]);
		}

		if max_track == 0 {
			return Err(Error::invalid_data("no streams available"));
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

pub fn parse_track(ky: &Kv) -> Result<Track> {
	let raw = ky.get("track").map(String::as_str).unwrap_or("all");
	if raw == "*" || raw == "all" {
		return Ok(Track::All);
	}
	match raw.parse::<usize>() {
		Ok(value) => Ok(Track::One(value)),
		Err(_) => Err(Error::invalid_data(format!("invalid track specifier {}", raw))),
	}
}
