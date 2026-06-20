/*
 * Git integration.
 *
 * Reads commit data from .git directory.
 *
 * Reads commit data by calling git log.
 * Uses the plumbing-friendly format with null byte separators
 * to avoid ambiguity from commit messages containing spaces.
 */

use std::process::Command;

pub struct Commit {
    pub hash: String,
    pub timestamp: u64,
    pub message: String,
}

/*
 * Reads commits from the git repository at the given path.
 *
 * Calls `git log` with a null-separated format:
 *   %H  - full commit hash
 *   %at - author timestamp (unix epoch)
 *   %s  - subject line (first line of message)
 *
 * Returns commits in reverse chronological order (newest first).
 */
pub fn read_commits(path: &str, limit: Option<usize>) -> Vec<Commit> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(path)
        .arg("log")
        .arg("--format=%H%x00%at%x00%s");

    if let Some(n) = limit {
        cmd.arg(format!("-{}", n));
    }

    let output = match cmd.output() {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stdout
        .lines()
        .filter_map(|line| parse_commit(line))
        .collect()
}

fn parse_commit(line: &str) -> Option<Commit> {
    let mut parts = line.splitn(3, '\0');
    let hash = parts.next()?;
    let timestamp = parts.next()?.parse::<u64>().ok()?;
    let message = parts.next()?;

    Some(Commit {
        hash: hash.to_string(),
        timestamp,
        message: message.to_string(),
    })
}
