use super::save_index;
use std::time::Instant;
use vit::commit::CommitEntry;
use vit::preference::{Preferences};
use vit::git;
use vit::lsa::build_index;
use vit::text::load_synonyms;
use vit::{die, verbose};

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
            _ => die!("unrecognized option \"{}\"", arg),
        }
    }
    flags
}

pub fn map(ctx: &Preferences, args: &[String]) {
    let MapFlags { verbose, list } = map_parse_args(args);

    let t_git = Instant::now();
    let commits = git::read_commits(".", None);
    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    let git_time = t_git.elapsed();
    let t_build = Instant::now();
    let synonyms = load_synonyms();
    let (wordmap, positions, stats) = build_index(&commits, ctx, &synonyms);
    let build_time = t_build.elapsed();

    if wordmap.is_empty() {
        eprintln!("not enough data for LSA");
        return;
    }

    let entries: Vec<CommitEntry> = commits
        .iter()
        .zip(positions.iter())
        .map(|(c, pos)| CommitEntry {
            hash: c.hash.clone(),
            message: c.message.clone(),
            position: pos.clone(),
        })
        .collect();

    if let Err(e) = save_index(&wordmap, &entries, &stats) {
        eprintln!("failed to save index: {}", e);
        return;
    }

    if list {
        for c in &commits {
            println!("  {:.7}  {}", &c.hash[..7], c.message);
        }
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
    verbose!(verbose, "  git log     {:.2?}", git_time);
    verbose!(verbose, "  lsa build   {:.2?}", build_time);
    verbose!(verbose, "");

    eprintln!(
        "  mapped {} commits, {} words, {} dims",
        stats.commit_count, stats.word_count, stats.dimensions
    );
}
