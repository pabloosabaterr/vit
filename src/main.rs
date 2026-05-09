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
	decay: f64,
}

impl Default for Context {
	fn default() -> Self {
		Context {
			scale: 1.0,
			decay: 1.0,
		}
	}
}

/*
 * Due to the logarithmic distances, the default scale becomes very tiny which
 * makes the plane too compressed, use 100 by default.
 * User can add another scale factor to change it as they want.
 */
const SCALE_DEF: f64 = 100.0;

impl Context {
	fn update(&mut self, key: &str, value: &str) -> bool {
		let field = match key {
			"scale" => &mut self.scale,
			"decay" => &mut self.decay,
			_ => return false,
		};
		*field = value.parse().unwrap_or(*field);
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
			/*
			 * Compresses the distances to avoid big words shadowing
			 * smaller ones even if they have bigger weights.
			 */
			dist: (num as f64).ln() * SCALE_DEF * ctx.scale,
		}
	}

	fn to_cartesian(&self) -> (f64, f64) {
		(self.dist * self.angle.cos(), self.dist * self.angle.sin())
	}
}

impl VectorInfo {
	fn from_word(word: &str, ctx: &Context) -> Self {
		let (x, y) = PolarInfo::from_number(word_to_number(word), ctx)
				        .to_cartesian();
		VectorInfo { x, y, z: 0 }
	}

	/*
	 * Calculates the center of the vector words of a message.
	 * Each word has a decaying weight making first words to influence more
	 * over the final position than later ones. This makes keywords commonly
	 * found in formal commit messages like:
	 *
	 *   "fix: ...", "feat: ...", "login: ..."
	 *
	 * To become the carriers of the message vector, while the rest will serve
	 * for the position around the cluster.
	 */
	fn from_message(message: &str, ctx: &Context) -> Self {
		let mut x = 0.0;
		let mut y = 0.0;
		let mut weight = 0.0;

		for (i, word) in message.split_whitespace().enumerate() {
			/*
			 * Similar to gravity (1 / r^2).
			 * Powering widens exponentially the gap between
			 * words making the decay more effective after distance
			 * becomes compressed.
			 *
			 * Users can set a decay factor to change the decay
			 * aggressiveness.
			 */
			let w = 1.0 / (1.0 + ctx.decay * i as f64).powi(2);
			let v = VectorInfo::from_word(word, ctx);
			x += v.x * w;
			y += v.y * w;
			weight += w;
		}

		VectorInfo { x: x / weight, y: y / weight, z: 0 }
	}
}

/*
 * Each character is multiplied by its position to avoid anagrams from having
 * the same hash.
 */
fn word_to_number(word: &str) -> u64 {
	word.chars()
		.enumerate()
		.map(|(i, c)| (c as u64) * (i as u64 + 1))
		.sum()
}

fn main() {
	let ctx = Context::load();
	let message = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
	let info = VectorInfo::from_message(&message, &ctx);
	println!("\"{}\" :\n  x = {:.4},\n  y = {:.4}", message, info.x, info.y);
}
