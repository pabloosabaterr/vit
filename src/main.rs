mod cmd;

use vit::config::Context;

/*
 * Vit is a search engine for Git.
 *
 * Lets you search through commits in a semantic way, topics or things that are
 * related. All of this in a deterministic way and _hopefully_ also fast.
 *
 * Sometimes Vit from vectors + Git... and sometimes a Very Irritating Tool.
 *
 * <3
 */
fn main() {
    let ctx = Context::load();
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(|s| s.as_str()) {
        Some("map") => cmd::map(&ctx, &args[1..]),
        Some("help") | Some("--help") | Some("-h") => cmd::help(&args[1..]),
        Some("near") => cmd::near(&ctx, &args[1..]),
        Some("--version") | Some("-V") => {
            println!("vit {}", env!("CARGO_PKG_VERSION"));
        }
        Some(other) => {
            eprintln!("unknown command: {}", other);
        }
        None => cmd::help(&args[1..]),
    }
}
