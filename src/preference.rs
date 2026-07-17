use crate::die;
use crate::lsa;

pub const PREFERENCES_SET_DIMS: u32 = 1 << 0;
pub const PREFERENCES_SET_MIN_FREQ: u32 = 1 << 1;

pub struct Preferences {
    pub dims: usize,
    pub min_freq: usize,
    set: u32,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            dims: lsa::DEFAULT_DIMS,
            min_freq: 0,
            set: 0,
        }
    }
}

impl Preferences {
    fn update(&mut self, key: &str, value: &str) {
        match key {
            "dims" => {
                if let Ok(v) = value.parse() {
                    self.dims = v;
                    self.set |= PREFERENCES_SET_DIMS;
                }
            }
            "min-freq" => {
                if let Ok(v) = value.parse() {
                    self.min_freq = v;
                    self.set |= PREFERENCES_SET_MIN_FREQ;
                }
            }
            /*
             * On loading ignore unknonw settings.
             */
            _ => (),
        }
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

    pub fn is_set(self, flag: u32) -> bool {
        self.set & flag != 0
    }

    /*
     * #TODO: only write the modified preference not the whole file again.
     */
    pub fn write_preferences(&self) {
        let mut pref = String::new();

        pref.push_str(&format!("dims={}\n", self.dims));
        pref.push_str(&format!("min_freq={}\n", self.min_freq));

        std::fs::write(".vitrc", pref).unwrap_or_else(|e| die!("{}", e));
    }
}
