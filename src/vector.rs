use std::collections::HashMap;

use crate::word_map::WordMap;

pub struct VectorInfo {
    pub coords: Vec<f64>,
}

impl VectorInfo {
    pub fn from_message(message: &str, word_map: &WordMap) -> Self {
        let dims = word_map.dims();
        let tokens: Vec<&str> = message.split_whitespace().collect();

        if tokens.is_empty() {
            return VectorInfo {
                coords: vec![0.0; dims],
            };
        }

        let mut counts: HashMap<&str, usize> = HashMap::new();
        for &w in &tokens {
            *counts.entry(w).or_insert(0) += 1;
        }

        let total = tokens.len() as f64;
        let mut acc = vec![0.0; dims];

        for (&word, &count) in &counts {
            let wv = match word_map.get(word) {
                Some(v) => v,
                None => continue,
            };
            let idf = match word_map.idf(word) {
                Some(v) => v,
                None => continue,
            };

            let weight = (count as f64 / total) * idf;

            for (j, &val) in wv.iter().enumerate() {
                acc[j] += weight * val;
            }
        }

        VectorInfo { coords: acc }
    }

    pub fn dist(&self, other: &VectorInfo) -> f64 {
        self.coords
            .iter()
            .zip(other.coords.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    pub fn cosine_vec(&self, other: &[f64]) -> f64 {
        let dot: f64 = self
            .coords
            .iter()
            .zip(other.iter())
            .map(|(a, b)| a * b)
            .sum();
        let na: f64 = self.coords.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = other.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            0.0
        } else {
            dot / (na * nb)
        }
    }
}
