use crate::config::Context;
use crate::hash::PointInfo;

pub struct VectorInfo {
    pub x: f64,
    pub y: f64,
    #[allow(unused)]
    pub z: u64,
}

impl VectorInfo {
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
    pub fn from_message(message: &str, ctx: &Context) -> Self {
        let mut x = 0.0;
        let mut y = 0.0;
        /*
         * Accumulated weight to normalize the final coords, making the
         * length of a message less relevant to the final position.
         */
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
            let w =
                1.0 / (1.0 + ctx.word_decay * i as f64).powi(2);
            let p = PointInfo::from_word(word, ctx);
            x += p.x * w;
            y += p.y * w;
            weight += w;
        }

        /* Message vector is the center of mass */
        VectorInfo {
            x: x / weight,
            y: y / weight,
            z: 0,
        }
    }
}
