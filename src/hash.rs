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
    /*
     * Maps a character to a polar point based on its position
     * in the alphabet. Each letter gets an equal slice of 360°.
     *
     *   'a' =   0.0°
     *   'b' =  13.8°
     *   'z' = 346.2°
     */
    pub fn char_to_polar(c: char, ctx: &Context) -> Self {
        PolarInfo {
            angle: ((c as u8 - b'a') as f64
                * (360.0 / 26.0))
                .to_radians(),
            dist: SCALE_DEF * ctx.scale,
        }
    }

    pub fn to_cartesian(&self) -> (f64, f64) {
        (self.dist * self.angle.cos(), self.dist * self.angle.sin())
    }
}

pub struct PointInfo {
    pub x: f64,
    pub y: f64,
}

impl PointInfo {
    /*
     * Different from message vectorization, words get their char
     * vectors concatenated with a decaying weight, making first
     * chars more relevant to get the most of the lexeme while
     * the rest tweaks the final position.
     */
    pub fn from_word(word: &str, ctx: &Context) -> Self {
        let mut x = 0.0;
        let mut y = 0.0;
        for (i, c) in word.chars().enumerate() {
            let weight =
                1.0 / (1.0 + ctx.char_decay * i as f64).powi(2);
            let (dx, dy) =
                PolarInfo::char_to_polar(c, ctx).to_cartesian();
            x += dx * weight;
            y += dy * weight;
        }
        PointInfo { x, y }
    }
}
