mod config;
mod help;
mod map;
mod near;

pub use config::config;
pub use help::help;
pub use map::map;
pub use near::near;

use vit::commit::{CommitEntry, save_commits};
use vit::lsa::LsaStats;
use vit::word_map::WordMap;
use vit::{RESET, YELLOW};

fn save_index(
    wordmap: &WordMap,
    entries: &[CommitEntry],
    stats: &LsaStats,
) -> vit::error::Result<()> {
    wordmap.save()?;
    save_commits(entries, wordmap.dims())?;
    stats.save()?;
    Ok(())
}

fn dims_hint(stats: &LsaStats, requested: usize) {
    if stats.dimensions == 0 || stats.sigma_first <= 0.0 {
        return;
    }

    if stats.dimensions < requested {
        eprintln!(
            "{}  hint: only {} of {} dims converged, corpus may be too small, \
             consider lowering the dimensions{}",
            YELLOW, stats.dimensions, requested, RESET
        );
        return;
    }

    let ratio = stats.sigma_last / stats.sigma_first;

    if ratio < 0.05 {
        eprintln!(
            "{}  hint: last dims carry little signal (σₖ/σ₁ = {:.3}), \
             lowering dims would shrink the index{}",
            YELLOW, ratio, RESET
        );
    } else if ratio > 0.3 {
        eprintln!(
            "{}  hint: all dims still carry signal (σₖ/σ₁ = {:.3}), \
             raising dims may improve results{}",
            YELLOW, ratio, RESET
        );
    }
}
