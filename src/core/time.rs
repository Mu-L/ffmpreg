#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
	pub num: u32,
	pub den: u32,
}

impl Time {
	pub fn new(num: u32, den: u32) -> Self {
		assert!(num > 0 && den > 0, "time numerator and denominator must be positive");
		Self { num, den }
	}

	pub fn to_seconds(&self, pts: i64) -> f64 {
		(pts as f64) * (self.num as f64) / (self.den as f64)
	}

	pub fn from_seconds(&self, seconds: f64) -> i64 {
		((seconds * self.den as f64) / self.num as f64) as i64
	}
	pub fn scale_pts(&self, pts: i64, target: Time) -> i64 {
		(pts as i128 * target.num as i128 / target.den as i128 * self.den as i128 / self.num as i128)
			as i64
	}

	pub fn gcd(&self) -> u32 {
		fn gcd(a: u32, b: u32) -> u32 {
			if b == 0 { a } else { gcd(b, a % b) }
		}
		gcd(self.num, self.den)
	}

	pub fn simplify(&self) -> Time {
		let g = self.gcd();
		Time::new(self.num / g, self.den / g)
	}
}
