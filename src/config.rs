use crate::lsa;

pub struct Context {
    pub scale: f64,
    pub dims: usize,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            scale: 1.0,
            dims: lsa::DEFAULT_DIMS,
        }
    }
}

impl Context {
    fn update(&mut self, key: &str, value: &str) -> bool {
        match key {
            "scale" => {
                self.scale = value.parse().unwrap_or(self.scale);
            }
            "dims" => {
                self.dims = value.parse().unwrap_or(self.dims);
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
