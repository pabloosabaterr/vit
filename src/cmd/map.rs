use super::build_index;
use std::time::Instant;
use vit::commit::CommitEntry;
use vit::commit::save_commits;
use vit::config::Context;
use vit::git;
use vit::text;
use vit::vector::VectorInfo;
use vit::verbose;

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

    let entries: Vec<CommitEntry> = commits
        .iter()
        .map(|c| {
            let clean = text::preprocess(&c.message);
            let info = VectorInfo::from_message(&clean, &wordmap);
            CommitEntry {
                hash: c.hash.clone(),
                message: c.message.clone(),
                position: info.to_vec(),
            }
        })
        .collect();

    if let Err(e) = wordmap.save() {
        eprintln!("failed to save wordmap: {}", e);
        return;
    }
    if let Err(e) = save_commits(&entries, wordmap.dims()) {
        eprintln!("failed to save commits: {}", e);
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
