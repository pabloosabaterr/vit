mod help;
mod map;
mod near;
mod config;

pub use help::help;
pub use map::map;
pub use near::near;
pub use config::config;

use vit::commit::{CommitEntry, save_commits};
use vit::lsa::LsaStats;
use vit::word_map::WordMap;

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
