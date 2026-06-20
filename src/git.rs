/*
 * Git integration.
 *
 * Reads commit data from .git directory.
 *
 * TODO:
 *   - parse .git/objects or shell out to `git log`
 *   - extract: hash, timestamp, message
 */

pub struct Commit {
    pub hash: String,
    pub timestamp: u64,
    pub message: String,
}

/// Reads commits from the git repository at the given path.
/// Returns an empty vec for now.
pub fn read_commits(_path: &str) -> Vec<Commit> {
    Vec::new()
}
