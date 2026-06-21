use std::time::Instant;

use crate::config::Context;
use crate::lsa::{self, WordMap};
use crate::vector::VectorInfo;
use crate::{git, text};

fn log_config(ctx: &Context) {
    eprintln!("config:     dims={}, scale={}", ctx.dims, ctx.scale,);
}

fn build_index(commits: &[git::Commit], ctx: &Context) -> WordMap {
    let t0 = Instant::now();
    let messages: Vec<String> = commits
        .iter()
        .map(|c| text::preprocess(&c.message))
        .collect();
    eprintln!("preprocess:      {:.2?}", t0.elapsed());

    let t1 = Instant::now();
    let wm = lsa::build(&messages, ctx.dims, ctx.scale);
    eprintln!("lsa build:  {:.2?}", t1.elapsed());
    eprintln!("total:      {:.2?}", t0.elapsed());

    wm
}

pub fn map(ctx: &Context) {
    log_config(ctx);
    let t_git = Instant::now();
    let commits = git::read_commits(".", None);
    eprintln!("git log:    {:.2?}", t_git.elapsed());
    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    let wm = build_index(&commits, ctx);
    if wm.is_empty() {
        eprintln!("not enough data for LSA");
        return;
    }

    for c in &commits {
        println!("{:.7} : {}", &c.hash[..7], c.message
        );
    }
}

pub fn help() {
    eprintln!(
        "usage: vit <command>\n\
         \n\
         commands:\n\
         \x20 map            \
         show all commits with coordinates\n\
         \x20 near <message> \
         find commits closest to a message\n\
         \x20 help           \
         show this help"
    );
}

fn parse_limit(args: &[String]) -> Option<usize> {
    args.iter()
        .find(|a| a.starts_with('-') && a[1..].parse::<usize>().is_ok())
        .and_then(|a| a[1..].parse().ok())
}

pub fn near(ctx: &Context, args: &[String]) {
    let limit = parse_limit(args);

    let query: String = args
        .iter()
        .filter(|a| !a.starts_with('-') || a[1..].parse::<usize>().is_err())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    if query.is_empty() {
        eprintln!("usage: vit near <message>");
        return;
    }

    log_config(ctx);
    eprintln!("query:      \"{}\"", query);

    let t_git = Instant::now();
    let commits = git::read_commits(".", None);
    eprintln!("git log:    {:.2?}", t_git.elapsed());

    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    let wordmap = build_index(&commits, ctx);
    if wordmap.is_empty() {
        eprintln!("not enough data for LSA");
        return;
    }

    let t_search = Instant::now();
    let clean_query = text::preprocess(&query);
    eprintln!("stemmed:    \"{}\"", clean_query);
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
    eprintln!("search:     {:.2?}", t_search.elapsed());

    let display = match limit {
        Some(n) => &ranked[..n.min(ranked.len())],
        None => &ranked[..10.min(ranked.len())],
    };

    for (c, dist) in display {
        println!("{:.7}  {:>8.2}  {}", &c.hash[..7], dist, c.message);
    }
}
