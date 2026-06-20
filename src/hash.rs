use crate::config::Context;

/*
 * Due to the logarithmic distances, the default scale becomes very tiny which
 * makes the plane too compressed, use 100 by default.
 * User can add another scale factor to change it as they want.
 */
const SCALE_DEF: f64 = 100.0;

pub struct PolarInfo {
    pub angle: f64,
    pub dist: f64,
}

impl PolarInfo {
    pub fn from_number(num: u64, ctx: &Context) -> Self {
        PolarInfo {
            angle: ((num % 360) as f64).to_radians(),
            /*
             * Compresses the distances to avoid big words shadowing
             * smaller ones even if they have bigger weights.
             */
            dist: (num as f64).ln() * SCALE_DEF * ctx.scale,
        }
    }

    pub fn to_cartesian(&self) -> (f64, f64) {
        (self.dist * self.angle.cos(), self.dist * self.angle.sin())
    }
}

/*
 * Each character is multiplied by its position to avoid anagrams from having
 * the same hash.
 */
pub fn word_to_number(word: &str) -> u64 {
    word.chars()
        .enumerate()
        .map(|(i, c)| (c as u64) * (i as u64 + 1))
        .sum()
}
