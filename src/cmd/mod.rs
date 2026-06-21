mod help;
mod map;
mod near;

pub use help::help;
pub use map::map;
pub use near::near;

use std::time::Instant;

use crate::config::Context;
use crate::lsa::{self, LsaStats};
use crate::word_map::WordMap;
use crate::{git, text};

#[macro_export]
macro_rules! verbose {
    ($verbose: expr, $($arg:tt)*) => {
        if $verbose {
            eprintln!($($arg)*)
        }
    };
}

fn build_index(commits: &[git::Commit], ctx: &Context) -> (WordMap, LsaStats) {
    let messages: Vec<String> = commits
        .iter()
        .map(|c| text::preprocess(&c.message))
        .collect();

    lsa::build(&messages, ctx.dims, ctx.scale)
}
