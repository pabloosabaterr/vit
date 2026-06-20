use crate::config::Context;
use crate::vector::VectorInfo;
use crate::{git, text};

pub fn map(ctx: &Context) {
    let commits = git::read_commits(".", None);
    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }
    for c in &commits {
        let clean = text::preprocess(&c.message);
        let info = VectorInfo::from_message(&clean, ctx);
        println!(
            "{:.7}  ({:>10.2}, {:>10.2})  {}",
            &c.hash[..7], info.x, info.y, c.message
        );
    }
}

pub fn help() {
   eprintln!(
        "usage: vit <command>\n\
         \n\
         commands:\n\
         \x20 map            show all commits with coordinates\n\
         \x20 help           show this help"
    );
}

fn parse_limit(args: &[String]) -> Option<usize> {
    args.iter()
        .find(|a| {
            a.starts_with('-')
                && a[1..].parse::<usize>().is_ok()
        })
        .and_then(|a| a[1..].parse().ok())
}

pub fn near(ctx: &Context, args: &[String]) {
    let limit = parse_limit(args);

    let query: String = args.iter()
        .filter(|a| {
            !a.starts_with('-')
                || a[1..].parse::<usize>().is_err()
        })
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    if query.is_empty() {
        eprintln!("usage: vit near <message>");
        return;
    }

    let commits = git::read_commits(".", None);
    if commits.is_empty() {
        eprintln!("no commits found");
        return;
    }

    let clean = text::preprocess(&query);
    let target = VectorInfo::from_message(&clean, ctx);

    let mut ranked: Vec<_> = commits
        .iter()
        .map(|c| {
            let clean = text::preprocess(&c.message);
            let info = VectorInfo::from_message(&clean, ctx);
            let dist = ((info.x - target.x).powi(2)
                + (info.y - target.y).powi(2))
                .sqrt();
            (c, info, dist)
        })
        .collect();

    ranked.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    let display = match limit {
        Some(n) => &ranked[..n.min(ranked.len())],
        None => &ranked[..],
    };

    for (c, info, dist) in display {
        println!(
            "{:.7}  ({:>10.2}, {:>10.2})  {:>8.2}  {}",
            &c.hash[..7], info.x, info.y, dist, c.message
        );
    }
}
