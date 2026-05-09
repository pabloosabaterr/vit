struct CartesianInfo {
	x: f64,
	y: f64,
}
struct PolarInfo {
	angle: f64,
	dist: f64,
}
struct VectorInfo {
	x: f64,
	y: f64,
	#[allow(unused)]
	z: u64,
}

struct Context {
	scale: f64,
}

impl Default for Context {
	fn default() -> Self {
		Context {
			scale: 1.0,
		}
	}
}

impl Context {
	fn update(&mut self, key: &str, value: &str) -> bool {
		match key {
			"scale" => self.scale = value.parse().unwrap_or(self.scale),
			_ => return false,
		}
		true
	}

	fn try_load() -> Result<Self, Box<dyn std::error::Error>> {
		let content = std::fs::read_to_string(".vitrc")?;
		let mut ctx = Self::default();

		for line in content.lines() {
			let line = line.trim();
			if line.is_empty() || line.starts_with('#') {
				continue;
			}

			if let Some((key, value)) = line.split_once('=') {
				ctx.update(key.trim(), value.trim());
			};
		}
		Ok(ctx)
	}

	fn load() -> Self {
		Self::try_load().unwrap_or_default()
	}
}

impl PolarInfo {
	fn from_number(num: u64, ctx: &Context) -> Self {
		PolarInfo {
			angle: ((num % 360) as f64).to_radians(),
			dist: (num as f64) * ctx.scale,
		}
	}

	fn to_cartesian(&self) -> CartesianInfo {
		CartesianInfo {
			x: self.dist * self.angle.cos(),
			y: self.dist * self.angle.sin(),
		}
	}
}

fn word_to_number(word: &str) -> u64 {
	word.chars()
		.enumerate()
		.map(|(i, c)| (c as u64) * (i as u64 + 1))
		.sum()
}

fn hash_info_algo(word: &str, ctx: &Context) -> VectorInfo {
	let n = word_to_number(word);
	let polar = PolarInfo::from_number(n, ctx);
	let cart = polar.to_cartesian();
	VectorInfo {
		x: cart.x,
		y: cart.y,
		z: 0,
	}
}
fn main() {
	let ctx = Context::load();
	let args: Vec<String> = std::env::args().skip(1).collect();
	for word in &args {
		let info = hash_info_algo(word, &ctx);
		println!("\"{}\" :\n  x = {:.4},\n  y = {:.4}", word, info.x, info.y);
	}
}
