use std::{collections::HashMap, fs, io::Result};

/*
 * N-dimensional word embeddings derived from LSA.
 *
 * Each word that appears in the commit corpus gets a position in k-dimensional
 * space based on its co-occurrence pattern. Words used in similar commits land
 * near each other.
 */
pub struct WordMap {
    coords: HashMap<String, Vec<f64>>,
    dims: usize,
}

impl WordMap {
    pub fn get(&self, word: &str) -> Option<&Vec<f64>> {
        self.coords.get(word)
    }

    pub fn is_empty(&self) -> bool {
        self.coords.is_empty()
    }

    pub fn dims(&self) -> usize {
        self.dims
    }

    pub fn len(&self) -> usize {
        self.coords.len()
    }

    pub fn from_raw(coords: HashMap<String, Vec<f64>>, dims: usize) -> Self {
        Self { coords, dims }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<f64>)> {
        self.coords.iter()
    }

    pub fn save(&self) -> Result<()> {
        fs::create_dir_all(".vit")?;
        let mut buf: Vec<u8> = Vec::new();

        buf.extend(&(self.dims() as u32).to_le_bytes());
        buf.extend(&(self.len() as u32).to_le_bytes());

        for (word, coords) in self.iter() {
            buf.extend(&(word.len() as u32).to_le_bytes());
            buf.extend(word.as_bytes());
            for &val in coords {
                buf.extend(&val.to_le_bytes());
            }
        }

        fs::write(".vit/wordmap", buf)
    }

    pub fn load() -> Result<WordMap> {
        let buf = fs::read(".vit/wordmap")?;
        let mut pos = 0;

        let dims =
            u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let word_count =
            u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        let mut coords = HashMap::with_capacity(word_count);

        for _ in 0..word_count {
            let word_len =
                u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let word = String::from_utf8(buf[pos..pos + word_len].to_vec()).unwrap();
            pos += word_len;

            let mut vec = Vec::with_capacity(dims);
            for _ in 0..dims {
                let val = f64::from_le_bytes(buf[pos..pos + 8].try_into().unwrap());
                pos += 8;
                vec.push(val);
            }

            coords.insert(word, vec);
        }

        Ok(WordMap::from_raw(coords, dims))
    }
}
