use super::build_index;

use crate::Context;
use crate::cmd::Instant;
use crate::git;
use crate::lsa::LsaStats;
use crate::text;
use crate::vector::VectorInfo;
use crate::verbose;
use crate::word_map::WordMap;

#[derive(Default)]
struct NearFlags {
    verbose: bool,
    map: bool,
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
            "-m" | "--map" => query.flags.map = true,

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
    let NearFlags { verbose, map } = query.flags;

    let t_git = Instant::now();
    let commits = git::read_commits(".", None);
    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    let t_build = Instant::now();
    let (wordmap, stats) = if map {
        build_index(&commits, ctx)
    } else {
        match WordMap::load() {
            Ok(wm) => {
                verbose!(verbose, "  loaded index from .vit/index\n");
                (wm, LsaStats::default())
            }
            Err(_) => {
                verbose!(verbose, "  no index found, building...\n");
                build_index(&commits, ctx)
            }
        }
    };

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

    let count = query.limit.min(ranked.len());
    for (c, dist) in &ranked[..count] {
        println!("  {:.7}  {:>5.2}  {}", &c.hash[..7], dist, c.message);
    }

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
}
