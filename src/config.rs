pub struct Context {
    pub scale: f64,
    pub word_decay: f64,
    pub char_decay: f64,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            scale: 1.0,
            word_decay: 1.0,
            char_decay: 1.0,
        }
    }
}

impl Context {
    fn update(&mut self, key: &str, value: &str) -> bool {
        let field = match key {
            "scale" => &mut self.scale,
            "word-decay" => &mut self.word_decay,
            "char-decay" => &mut self.char_decay,
            _ => return false,
        };
        *field = value.parse().unwrap_or(*field);
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
