use crate::config::Context;
use crate::vector::VectorInfo;
use crate::{git, text};

pub fn map(ctx: &Context) {
    let commits = git::read_commits(".");
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
