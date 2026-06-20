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
        Some("help") | Some("--help") | Some("-h") => cmd::help(),
        Some("near") => {
            let query = args[1..].join(" ");
            if query.is_empty() {
                eprintln!("usage: vit near <message>");
                return;
            }
            cmd::near(&ctx, &args[1..]);
        }        Some(other) => {
            eprintln!("unknown command: {}", other);
        }
        None => cmd::help()
    }
}
