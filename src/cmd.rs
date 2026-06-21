use std::time::Instant;

use crate::config::Context;
use crate::lsa::{self, LsaStats, WordMap};
use crate::vector::VectorInfo;
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

#[derive(Default)]
struct MapFlags {
    verbose: bool,
    list: bool,
}

fn map_parse_args(args: &[String]) -> MapFlags {
    let mut flags = MapFlags::default();
    for arg in args {
        match arg.as_str() {
            "-l" | "--list" => flags.list = true,
            "-v" | "--verbose" => flags.verbose = true,
            _ => {}
        }
    }
    flags
}

pub fn map(ctx: &Context, args: &[String]) {
    let MapFlags { verbose, list } = map_parse_args(args);

    let t_git = Instant::now();
    let commits = git::read_commits(".", None);
    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    let t_build = Instant::now();
    let (wordmap, stats) = build_index(&commits, ctx);
    let build_time = t_build.elapsed();

    if wordmap.is_empty() {
        eprintln!("not enough data for LSA");
        return;
    }

    if list {
        for c in &commits {
            println!("  {:.7}  {}", &c.hash[..7], c.message);
        }
        eprintln!("");
    }

    verbose!(
        verbose,
        "  corpus      {} commits, {} words",
        stats.commit_count,
        stats.word_count
    );
    verbose!(
        verbose,
        "  dims        {} / {} converged (σ₁={:.2}, σₖ={:.2})",
        stats.dimensions,
        ctx.dims,
        stats.sigma_first,
        stats.sigma_last
    );
    verbose!(verbose, "  git log     {:.2?}", t_git.elapsed());
    verbose!(verbose, "  lsa build   {:.2?}", build_time);
    verbose!(verbose, "");

    eprintln!(
        "  mapped {} commits, {} words, {} dims",
        stats.commit_count, stats.word_count, stats.dimensions
    );
}

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
  -N               limit results (default: 10)";

const GENERAL_HELP: &str = "\
usage: vit <command>

commands:
  map              build word map from commits
  near <message>   find commits closest to a message
  help [command]   show help for a command";

pub fn help(args: &[String]) {
    let text = match args.first().map(|s| s.as_str()) {
        Some("map") => MAP_HELP,
        Some("near") => NEAR_HELP,
        _ => GENERAL_HELP,
    };
    eprintln!("{}", text);
}

#[derive(Default)]
struct NearFlags {
    verbose: bool,
}

struct NearQuery {
    text: String,
    limit: usize,
    flags: NearFlags,
}

impl Default for NearQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            limit: 10,
            flags: NearFlags::default(),
        }
    }
}

fn near_parse_args(args: &[String]) -> Option<NearQuery> {
    let mut query = NearQuery::default();
    let mut words = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-v" | "--verbose" => query.flags.verbose = true,

            opt if opt.starts_with('-') => {
                if let Ok(n) = opt[1..].parse::<usize>() {
                    query.limit = n;
                }
            }

            _ => words.push(arg.as_str()),
        }
    }

    query.text = words.join(" ");
    if query.text.is_empty() {
        return None;
    }

    Some(query)
}

pub fn near(ctx: &Context, args: &[String]) {
    let query = match near_parse_args(args) {
        Some(q) => q,
        None => {
            eprintln!("usage: vit near <message>");
            return;
        }
    };
    let NearFlags { verbose } = query.flags;

    let t_git = Instant::now();
    let commits = git::read_commits(".", None);
    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    let t_build = Instant::now();
    let (wordmap, stats) = build_index(&commits, ctx);
    let build_time = t_build.elapsed();

    if wordmap.is_empty() {
        eprintln!("not enough data for LSA");
        return;
    }

    let t_search = Instant::now();
    let clean_query = text::preprocess(&query.text);
    let target = VectorInfo::from_message(&clean_query, &wordmap);
    let mut ranked: Vec<_> = commits
        .iter()
        .map(|c| {
            let clean = text::preprocess(&c.message);
            let info = VectorInfo::from_message(&clean, &wordmap);
            let dist = target.dist(&info);
            (c, dist)
        })
        .collect();

    ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let search_time = t_search.elapsed();

    verbose!(verbose, "");
    verbose!(
        verbose,
        "  query       \"{}\" → \"{}\"",
        query.text,
        clean_query
    );
    verbose!(
        verbose,
        "  corpus      {} commits, {} words",
        stats.commit_count,
        stats.word_count
    );
    verbose!(
        verbose,
        "  dims        {} / {} converged (σ₁={:.2}, σₖ={:.2})",
        stats.dimensions,
        ctx.dims,
        stats.sigma_first,
        stats.sigma_last
    );
    verbose!(verbose, "  git log     {:.2?}", t_git.elapsed());
    verbose!(verbose, "  lsa build   {:.2?}", build_time);
    verbose!(verbose, "  search      {:.2?}", search_time);
    verbose!(verbose, "");

    let count = query.limit.min(ranked.len());
    for (c, dist) in &ranked[..count] {
        println!("  {:.7}  {:>5.2}  {}", &c.hash[..7], dist, c.message);
    }
}
