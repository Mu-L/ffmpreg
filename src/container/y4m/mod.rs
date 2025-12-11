pub mod read;
pub mod write;

pub use read::Y4mReader;
pub use write::Y4mWriter;

#[derive(Debug, Clone, Copy)]
pub struct AspectRatio {
	pub num: u32,
	pub den: u32,
}

impl AspectRatio {
	pub fn new(num: u32, den: u32) -> Self {
		Self { num, den }
	}

	pub fn from_str(s: &str) -> Option<Self> {
		let (num, den) = s.split_once(':')?;
		Some(Self { num: num.parse().ok()?, den: den.parse().ok()? })
	}

	pub fn to_string(&self) -> String {
		format!("{}:{}", self.num, self.den)
	}
}

#[derive(Debug, Clone)]
pub struct Y4mFormat {
	pub width: u32,
	pub height: u32,
	pub framerate_num: u32,
	pub framerate_den: u32,
	pub colorspace: Option<Colorspace>,
	pub interlacing: Interlacing,
	pub aspect_ratio: Option<AspectRatio>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colorspace {
	C420,
	C420jpeg,
	C420paldv,
	C420mpeg2,
	C422,
	C444,
	Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interlacing {
	Progressive,
	TopFieldFirst,
	BottomFieldFirst,
	Mixed,
}

impl Y4mFormat {
	pub fn frame_size(&self) -> usize {
		let luma_size = (self.width * self.height) as usize;
		let colorspace = self.colorspace.unwrap_or(Colorspace::C420);
		match colorspace {
			Colorspace::C420 | Colorspace::C420jpeg | Colorspace::C420paldv | Colorspace::C420mpeg2 => {
				luma_size + luma_size / 2
			}
			Colorspace::C422 => luma_size * 2,
			Colorspace::C444 => luma_size * 3,
			Colorspace::Mono => luma_size,
		}
	}
}

impl Default for Y4mFormat {
	fn default() -> Self {
		Self {
			width: 1920,
			height: 1080,
			framerate_num: 30,
			framerate_den: 1,
			colorspace: None,
			interlacing: Interlacing::Progressive,
			aspect_ratio: None,
		}
	}
}

impl Colorspace {
	pub fn as_str(&self) -> &'static str {
		match self {
			Colorspace::C420 => "C420",
			Colorspace::C420jpeg => "C420jpeg",
			Colorspace::C420paldv => "C420paldv",
			Colorspace::C420mpeg2 => "C420mpeg2",
			Colorspace::C422 => "C422",
			Colorspace::C444 => "C444",
			Colorspace::Mono => "Cmono",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"C420" => Some(Colorspace::C420),
			"C420jpeg" => Some(Colorspace::C420jpeg),
			"C420paldv" => Some(Colorspace::C420paldv),
			"C420mpeg2" => Some(Colorspace::C420mpeg2),
			"C422" => Some(Colorspace::C422),
			"C444" => Some(Colorspace::C444),
			"Cmono" => Some(Colorspace::Mono),
			_ => None,
		}
	}
}

impl Interlacing {
	pub fn as_char(&self) -> char {
		match self {
			Interlacing::Progressive => 'p',
			Interlacing::TopFieldFirst => 't',
			Interlacing::BottomFieldFirst => 'b',
			Interlacing::Mixed => 'm',
		}
	}

	pub fn from_char(c: char) -> Option<Self> {
		match c {
			'p' => Some(Interlacing::Progressive),
			't' => Some(Interlacing::TopFieldFirst),
			'b' => Some(Interlacing::BottomFieldFirst),
			'm' => Some(Interlacing::Mixed),
			_ => None,
		}
	}
}
