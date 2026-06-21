use super::build_index;

use std::time::Instant;
use vit::commit::{CommitEntry, load_commits, save_commits};
use vit::config::Context;
use vit::git;
use vit::lsa::LsaStats;
use vit::text::{self, load_synonyms};
use vit::vector::VectorInfo;
use vit::verbose;
use vit::word_map::WordMap;

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

    let synonyms = load_synonyms();
    let t_build = Instant::now();
    let (wordmap, entries, stats) = if map {
        let commits = git::read_commits(".", None);

        if commits.is_empty() {
            eprintln!("no commits found");
            return;
        }

        let (wm, stats) = build_index(&commits, ctx, &synonyms);
        let entries: Vec<CommitEntry> = commits
            .iter()
            .map(|c| {
                let clean = text::preprocess(&c.message, &synonyms);
                let info = VectorInfo::from_message(&clean, &wm);
                CommitEntry {
                    hash: c.hash.clone(),
                    message: c.message.clone(),
                    position: info.to_vec(),
                }
            })
            .collect();
        wm.save().ok();
        save_commits(&entries, wm.dims()).ok();
        stats.save().ok();
        (wm, entries, stats)
    } else {
        match (WordMap::load(), load_commits(), LsaStats::load()) {
            (Ok(wm), Ok(entries), Ok(stats)) => {
                verbose!(verbose, "  loaded from .vit/\n");
                (wm, entries, stats)
            }
            _ => {
                eprintln!("no index found, run 'vit map' first");
                return;
            }
        }
    };
    let build_time = t_build.elapsed();

    if wordmap.is_empty() {
        eprintln!("not enough data for LSA");
        return;
    }

    let t_search = Instant::now();
    let clean_query = text::preprocess(&query.text, &synonyms);
    let target = VectorInfo::from_message(&clean_query, &wordmap);
    let mut ranked: Vec<_> = entries
        .iter()
        .map(|c| {
            let dist = target.dist_vec(&c.position);
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
    verbose!(verbose, "  build       {:.2?}", build_time);
    verbose!(verbose, "  search      {:.2?}", search_time);
    verbose!(verbose, "");
}
