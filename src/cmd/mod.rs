mod help;
mod map;
mod near;

pub use help::help;
pub use map::map;
pub use near::near;

use vit::config::Context;
use vit::lsa::{self, LsaStats};
use vit::word_map::WordMap;
use vit::{git, text};

fn build_index(commits: &[git::Commit], ctx: &Context) -> (WordMap, LsaStats) {
    let messages: Vec<String> = commits
        .iter()
        .map(|c| text::preprocess(&c.message))
        .collect();

    lsa::build(&messages, ctx.dims, ctx.scale)
}
