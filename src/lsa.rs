/*
 * Latent Semantic Analysis via Power Iteration.
 *
 * Builds:
 *
 *   TF-IDF weighted term×document sparse matrix from preprocessed commit
 *   Note that docoment refers to commit and term to word.
 *
 * Extracts the top-k left singular vectors using power iteration on A*A^t
 * Words that co-occur across commits end up with similar coordinates.
 *
 * Uses 32 dimensions by defaults but the user can choose how many on .vitrc
 */

use std::collections::{HashMap, HashSet};

use crate::lin_alg::power_iteration;
use crate::sparse_matrix::SparseMatrix;
use crate::verbose;

pub const DEFAULT_DIMS: usize = 32;

/*
 * Build the vocabulary excluding the words that does not add any correlation value
 * (they only appear once in a commit).
 *
 * Sends also the frequencies after computing them so build_tfidf doesn't have to
 * build them again.
 */
fn build_vocab(
    messages: &[String],
) -> (Vec<String>, HashMap<String, usize>, Vec<usize>) {
    let mut raw_frequency: HashMap<String, usize> = HashMap::new();
    let mut seen = HashSet::new();

    /*
     * Count how much words appear in commits messages.
     */
    for msg in messages {
        seen.clear();

        for word in msg.split_whitespace() {
            if seen.insert(word) {
                *raw_frequency.entry(word.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut words: HashMap<String, usize> = HashMap::new();
    let mut vocab: Vec<String> = Vec::new();
    let mut word_frequency: Vec<usize> = Vec::new();

    /*
     * Iterate over the raw frequencies and exclude the ones that appear less than
     * the minimum required.
     *
     * The minimum required is two because a word that only appears once cannot
     * correlate with other commits.
     */
    for (word, &freq) in &raw_frequency {
        if freq < 2 {
            continue;
        }

        words.insert(word.clone(), vocab.len());
        vocab.push(word.clone());
        word_frequency.push(freq);
    }

    (vocab, words, word_frequency)
}

/*
 * Build a wordsxcommits matrix weighted by:
 *
 *   term-frequency - inverse-document-frequency
 *
 * Each entry measures how important that word is to a specific commit.
 * Note that a commit is a document and a term is a word.
 */
fn build_tfidf(
    messages: &[String],
    words: &HashMap<String, usize>,
    word_nr: usize,
    word_freq: &[usize],
) -> SparseMatrix {
    let doc_nr = messages.len();
    let log_docs = (doc_nr as f64).ln();
    let mut triplets: Vec<(usize, usize, f64)> = Vec::new();

    for (doc, msg) in messages.iter().enumerate() {
        let tokens: Vec<&str> = msg.split_whitespace().collect();

        if tokens.is_empty() {
            continue;
        }

        /*
         * term frequency within this commit.
         */
        let mut counts: HashMap<usize, usize> = HashMap::new();
        for word in &tokens {
            if let Some(&id) = words.get(*word) {
                *counts.entry(id).or_insert(0) += 1;
            }
        }

        /*
         * Check how rare the word is against all the commits.
         */
        for (&word_id, &count) in &counts {
            /*
             * Note: the number of times that a word appears in the same commit
             *       matters, it would be more relevant if we care about the commit
             *       bodies as well.
             */
            let term_freq = count as f64 / tokens.len() as f64;
            let idf = log_docs - (word_freq[word_id] as f64).ln();

            if idf > 0.0 {
                triplets.push((word_id, doc, term_freq * idf));
            }
        }
    }

    SparseMatrix::from_triplets(word_nr, doc_nr, &mut triplets)
}

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
}

pub fn build(
    messages: &[String],
    dims: usize,
    scale: f64,
    verbose: bool,
) -> WordMap {
    if messages.len() < 2 {
        return WordMap {
            coords: HashMap::new(),
            dims,
        };
    }

    let (vocab, words, word_freq) = build_vocab(messages);
    let word_nr = vocab.len();

    if word_nr < 2 {
        return WordMap {
            coords: HashMap::new(),
            dims,
        };
    }

    verbose!(verbose, "  corpus      {} commits, {} words", messages.len(), word_nr);

    let importance_matrix = build_tfidf(messages, &words, word_nr, &word_freq);
    let (vectors, sigmas) = power_iteration(&importance_matrix, dims);
    let real_dimensions = vectors.len();

    verbose!(
        verbose,
        "  dims        {} / {} converged (σ₁={:.2}, σₖ={:.2})",
        real_dimensions,
        dims,
        sigmas[0],
        sigmas[real_dimensions - 1]
    );
    /*
     * Build raw word vectors.
     *
     * Scaling by sigma preserves the relative importance of each dimension, the
     * first axis captures the most variance, the last the least.
     */
    let target = 100.0 * scale;
    let mut max_abs = 0.0_f64;

    let raw: Vec<Vec<f64>> = (0..word_nr)
        .map(|i| {
            let v: Vec<f64> = (0..real_dimensions)
                .map(|d| {
                    let val = sigmas[d] * vectors[d][i];
                    max_abs = max_abs.max(val.abs());
                    val
                })
                .collect();
            v
        })
        .collect();

    let factor = if max_abs > 0.0 { target / max_abs } else { 1.0 };

    let mut coords = HashMap::with_capacity(word_nr);
    for (i, word) in vocab.iter().enumerate() {
        let scaled: Vec<f64> = raw[i].iter().map(|v| v * factor).collect();
        coords.insert(word.clone(), scaled);
    }

    WordMap {
        coords,
        dims: real_dimensions,
    }
}

