mod cmd;
mod config;
mod git;
mod hash;
mod stemmer;
mod text;
mod vector;

use config::Context;

fn main() {
    let ctx = Context::load();
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(|s| s.as_str()) {
        Some("map") => cmd::map(&ctx),
        Some(other) => {
            eprintln!("unknown command: {}", other);
        }
        None => eprintln!("usage: vit <command>")
    }
}
