use std::fs;

pub struct CommitEntry {
    pub hash: String,
    pub message: String,
    pub position: Vec<f64>,
}

pub fn save_commits(commits: &[CommitEntry], dims: usize) -> std::io::Result<()> {
    fs::create_dir_all(".vit")?;
    let mut buf: Vec<u8> = Vec::new();

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

    fs::write(".vit/commits", buf)
}

pub fn load_commits() -> std::io::Result<Vec<CommitEntry>> {
    let buf = fs::read(".vit/commits")?;
    let mut pos = 0;

    let dims = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let count = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    let mut commits = Vec::with_capacity(count);

    for _ in 0..count {
        let hash_len =
            u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let hash = String::from_utf8(buf[pos..pos + hash_len].to_vec()).unwrap();
        pos += hash_len;

        let msg_len =
            u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let message = String::from_utf8(buf[pos..pos + msg_len].to_vec()).unwrap();
        pos += msg_len;

        let mut position = Vec::with_capacity(dims);
        for _ in 0..dims {
            let val = f64::from_le_bytes(buf[pos..pos + 8].try_into().unwrap());
            pos += 8;
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
