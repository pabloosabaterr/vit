mod cmd;
mod config;
mod git;
mod lin_alg;
mod lsa;
mod sparse_matrix;
mod stemmer;
mod text;
mod vector;

use config::Context;

fn main() {
    let ctx = Context::load();
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(|s| s.as_str()) {
        Some("map") => cmd::map(&ctx),
        Some("help") | Some("--help") | Some("-h") => cmd::help(),
        Some("near") => cmd::near(&ctx, &args[1..]),
        Some(other) => {
            eprintln!("unknown command: {}", other);
        }
        None => cmd::help(),
    }
}
