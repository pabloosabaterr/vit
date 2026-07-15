use std::{collections::HashMap, fs};

use crate::read::Reader;

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
    idf: HashMap<String, f64>,
}

impl WordMap {
    pub fn get(&self, word: &str) -> Option<&Vec<f64>> {
        self.coords.get(word)
    }

    pub fn idf(&self, word: &str) -> Option<f64> {
        self.idf.get(word).copied()
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

    pub fn from_raw(
        coords: HashMap<String, Vec<f64>>,
        idf: HashMap<String, f64>,
        dims: usize,
    ) -> Self {
        Self { coords, idf, dims }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<f64>)> {
        self.coords.iter()
    }

    pub fn save(&self) -> crate::error::Result<()> {
        fs::create_dir_all(".vit")?;
        let mut buf: Vec<u8> = Vec::new();

        crate::read::write_version(&mut buf, crate::VERSION);

        buf.extend(&(self.dims() as u32).to_le_bytes());
        buf.extend(&(self.len() as u32).to_le_bytes());

        for (word, coords) in self.iter() {
            buf.extend(&(word.len() as u32).to_le_bytes());
            buf.extend(word.as_bytes());

            let idf = self.idf.get(word).copied().unwrap_or(0.0);
            buf.extend(&idf.to_le_bytes());

            for &val in coords {
                buf.extend(&val.to_le_bytes());
            }
        }

        Ok(fs::write(".vit/wordmap", buf)?)
    }

    pub fn load() -> crate::error::Result<WordMap> {
        let buf = fs::read(".vit/wordmap")?;
        let mut reader = Reader::new(&buf, "word-map");

        reader.expect_version(crate::VERSION)?;
        let dims = reader.read_u32()? as usize;

        let word_count = reader.read_u32()? as usize;

        let mut coords = HashMap::with_capacity(word_count);
        let mut idf_map = HashMap::with_capacity(word_count);

        for _ in 0..word_count {
            let word_len = reader.read_u32()? as usize;

            let word = reader.read_string(word_len)?;
            let idf = reader.read_f64()?;

            let mut vec = Vec::with_capacity(dims);
            for _ in 0..dims {
                let val = reader.read_f64()?;
                vec.push(val);
            }

            idf_map.insert(word.clone(), idf);
            coords.insert(word, vec);
        }

        Ok(WordMap::from_raw(coords, idf_map, dims))
    }
}
