const MAP_HELP: &str = "\
usage: vit map [options]

Build word map from commits.

options:
  -v, --verbose    show timing and debug info
  -l, --list       list all commits";

const NEAR_HELP: &str = "\
usage: vit near <message> [options]

Find commits closest to a message.

options:
  -v, --verbose    show timing and debug info
  -m, --map        rebuild the word map before searching
  -N               limit results (default: 10)";

const GENERAL_HELP: &str = "\
usage: vit <command>

correlation search for git commits.

commands:
  map              build word map from commits
  near <message>   find commits closest to a message
  help [command]   show help for a command
  -V, --version    vit's version";

const CONFIG_HELP: &str = "\
usage: vit config <cfg> <value>

Set a value for a config variable in '.vitrc' file.

\x1b[33m#NEEDSWORK\x1b[0m: Rewrites the whole '.vitrc' file loosing comments in the
proccess.
";

pub fn help(args: &[String]) {
    let text = match args.first().map(|s| s.as_str()) {
        Some("map") => MAP_HELP,
        Some("near") => NEAR_HELP,
        Some("config") => CONFIG_HELP,
        _ => GENERAL_HELP,
    };
    eprintln!("{}", text);
}
