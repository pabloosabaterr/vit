use std::fs;

use crate::read::Reader;

const COMMIT_FILE: &str = ".vit/commits";

pub struct CommitEntry {
    pub hash: String,
    pub message: String,
    pub position: Vec<f32>,
}

pub fn save_commits(
    commits: &[CommitEntry],
    dims: usize,
) -> crate::error::Result<()> {
    fs::create_dir_all(".vit")?;
    let mut buf: Vec<u8> = Vec::new();

    crate::read::write_version(&mut buf, crate::VERSION);
    buf.extend(&(dims as u32).to_le_bytes());
    buf.extend(&(commits.len() as u32).to_le_bytes());

    for c in commits {
        buf.extend(&(c.hash.len() as u32).to_le_bytes());
        buf.extend(c.hash.as_bytes());
        buf.extend(&(c.message.len() as u32).to_le_bytes());
        buf.extend(c.message.as_bytes());
        for &val in &c.position {
            buf.extend(&val.to_le_bytes());
        }
    }

    Ok(fs::write(COMMIT_FILE, buf)?)
}

pub fn load_commits() -> crate::error::Result<Vec<CommitEntry>> {
    let buf = fs::read(COMMIT_FILE)?;
    let mut reader = Reader::new(&buf, "commit");

    reader.expect_version(crate::VERSION)?;

    let dims = reader.read_u32()? as usize;
    let count = reader.read_u32()? as usize;

    let mut commits = Vec::with_capacity(count);

    for _ in 0..count {
        let hash_len = reader.read_u32()? as usize;
        let hash = reader.read_string(hash_len)?;

        let msg_len = reader.read_u32()? as usize;
        let message = reader.read_string(msg_len)?;

        let mut position = Vec::with_capacity(dims);
        for _ in 0..dims {
            let val = reader.read_f32()?;
            position.push(val);
        }

        commits.push(CommitEntry {
            hash,
            message,
            position,
        });
    }

    Ok(commits)
}
