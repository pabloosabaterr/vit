mod config;
mod git;
mod hash;
mod stemmer;
mod text;
mod vector;

use config::Context;
use vector::VectorInfo;

fn main() {
    let ctx = Context::load();
    let commits = git::read_commits(".");

    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    for c in &commits {
        let clean = text::preprocess(&c.message);
        let info = VectorInfo::from_message(&clean, &ctx);
        println!(
            "{:.7}  ({:>10.2}, {:>10.2})  {}",
            &c.hash[..7], info.x, info.y, c.message
        );
    }
}
