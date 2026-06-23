use crate::lsa;

pub struct Context {
    pub dims: usize,
    pub min_freq: usize,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            dims: lsa::DEFAULT_DIMS,
            min_freq: 0,
        }
    }
}

impl Context {
    fn update(&mut self, key: &str, value: &str) -> bool {
        match key {
            "dims" => {
                self.dims = value.parse().unwrap_or(self.dims);
            }
            "min-freq" => {
                self.min_freq = value.parse().unwrap_or(self.min_freq);
            }
            _ => return false,
        }
        true
    }

    fn try_load() -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(".vitrc")?;
        let mut ctx = Self::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                ctx.update(key.trim(), value.trim());
            };
        }
        Ok(ctx)
    }

    pub fn load() -> Self {
        Self::try_load().unwrap_or_default()
    }
}
