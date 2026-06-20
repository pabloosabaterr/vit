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
    let message = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let clean = text::preprocess(&message);
    let info = VectorInfo::from_message(&clean, &ctx);
    println!(
        "\"{}\" :\n  x = {:.4},\n  y = {:.4}",
        message, info.x, info.y
    );
}
