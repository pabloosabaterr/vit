use crate::lsa::WordMap;

pub struct VectorInfo {
    pub coords: Vec<f64>,
    #[allow(unused)]
    pub z: u64,
}

impl VectorInfo {
    /*
     * Calculates the centroid of the word vectors of a message in N-dimensional
     * LSA space.
     *
     * All words contribute equally. The LSA embeddings already encode semantic
     * weight through co-occurrence
     */
    pub fn from_message(message: &str, word_map: &WordMap) -> Self {
        let dims = word_map.dims();
        let mut acc = vec![0.0; dims];
        let mut count = 0usize;

        for word in message.split_whitespace() {
            let wv = match word_map.get(word) {
                Some(v) => v,
                None => continue,
            };

            for (j, &val) in wv.iter().enumerate() {
                acc[j] += val;
            }
            count += 1;
        }

        if count == 0 {
            return VectorInfo {
                coords: vec![0.0; dims],
                z: 0,
            };
        }

        let n = count as f64;
        for v in acc.iter_mut() {
            *v /= n;
        }

        VectorInfo { coords: acc, z: 0 }
    }

    pub fn dist(&self, other: &VectorInfo) -> f64 {
        self.coords
            .iter()
            .zip(other.coords.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /*
     * Alternative, I need to test this.
     */
    #[allow(unused)]
    pub fn cosine(&self, other: &VectorInfo) -> f64 {
        let dot: f64 = self
            .coords
            .iter()
            .zip(other.coords.iter())
            .map(|(a, b)| a * b)
            .sum();
        let na: f64 = self.coords.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = other.coords.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            return 0.0;
        }
        dot / (na * nb)
    }
}
