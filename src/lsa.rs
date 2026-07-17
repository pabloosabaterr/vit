/*
 * Latent Semantic Analysis.
 *
 * Builds:
 *
 *   TF-IDF weighted term×document sparse matrix from preprocessed commit
 *   Note that docoment refers to commit and term to word.
 *
 * Extracts the top-k left singular vectors via truncated SVD (block
 * subspace iteration, see lin_alg.rs). Words that co-occur across
 * commits end up with similar coordinates.
 *
 * Uses 32 dimensions by defaults but the user can choose how many on .vitrc
 */

use std::collections::HashMap;

use crate::lin_alg::truncated_svd;
use crate::preference::Preferences;
use crate::term::get_sparse_matrix;
use crate::word_map::WordMap;

pub const DEFAULT_DIMS: usize = 32;

#[derive(Default)]
pub struct LsaStats {
    pub word_count: usize,
    pub commit_count: usize,
    pub dimensions: usize,
    pub sigma_first: f64,
    pub sigma_last: f64,
}

impl LsaStats {
    pub fn save(&self) -> crate::error::Result<()> {
        std::fs::create_dir_all(".vit")?;
        let content = [
            format!("word_count={}", self.word_count),
            format!("commit_count={}", self.commit_count),
            format!("dimensions={}", self.dimensions),
            format!("sigma_first={}", self.sigma_first),
            format!("sigma_last={}", self.sigma_last),
        ]
        .join("\n");
        Ok(std::fs::write(".vit/stats", content)?)
    }

    pub fn load() -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(".vit/stats")?;
        let mut stats = LsaStats::default();
        for line in content.lines() {
            if let Some((key, val)) = line.split_once('=') {
                match key {
                    "word_count" => stats.word_count = val.parse().unwrap_or(0),
                    "commit_count" => stats.commit_count = val.parse().unwrap_or(0),
                    "dimensions" => stats.dimensions = val.parse().unwrap_or(0),
                    "sigma_first" => stats.sigma_first = val.parse().unwrap_or(0.0),
                    "sigma_last" => stats.sigma_last = val.parse().unwrap_or(0.0),
                    _ => {}
                }
            }
        }
        Ok(stats)
    }
}

/*
 * What build() returns when the corpus is too small or degenerate to
 * extract anything meaningful from it.
 */
fn empty_result(dims: usize) -> (WordMap, Vec<Vec<f64>>, LsaStats) {
    (
        WordMap::from_raw(HashMap::new(), HashMap::new(), dims),
        Vec::new(),
        LsaStats::default(),
    )
}

pub fn build(
    messages: &[String],
    ctx: &Preferences,
) -> (WordMap, Vec<Vec<f64>>, LsaStats) {
    let &Preferences { dims, min_freq, .. } = ctx;

    if messages.len() < 2 {
        return empty_result(dims);
    }

    /*
     * Build A (words x commit matrix)
     *
     * Build the vocabulary and computes BM25 weighted entries.
     * Each column is a commit and each row is a word. A[word][commit] is how
     * important is that word for that commit.
     *
     * A is the matrix to decompose in A = U * Sigma * V^t.
     */
    let term = match get_sparse_matrix(messages, min_freq) {
        Some(m) => m,
        None => return empty_result(dims),
    };

    let word_nr = term.vocab.len();

    /*
     * Extract U and Sigma via eigen value problem A * A^t * u = sigma^2 * u
     *
     * V is unitary so A * A^t cancels V, leaving: U * Sigma^2 * U^t. The eigen
     * vectors of A * A^t are the columns of U and the eigen values the sigma^2.
     *
     * truncated_svd finds the top-k of them by iterating a whole block of
     * deterministic vectors against A * A^t, see lin_alg.rs.
     */
    let (vectors, sigmas) = truncated_svd(&term.matrix, dims);
    let real_dimensions = vectors.len();

    /*
     * Degenerate corpus (e.g. every message empty after preprocessing),
     * nothing useful got extracted.
     */
    if real_dimensions == 0 {
        return empty_result(dims);
    }

    let stats = LsaStats {
        word_count: word_nr,
        commit_count: messages.len(),
        dimensions: real_dimensions,
        sigma_first: sigmas[0],
        sigma_last: sigmas[real_dimensions - 1],
    };

    /*
     * truncated_svd returns an array per dimension. example:
     *
     *   vectors[0] = [a, b, c, d, e, ...] <- first dimension
     *   vectors[1] = [f, g, h, i, j, ...] <- second dimension
     *
     * Here, flip this to be an array per word:
     *
     *   word[0] = [a, f]
     *   word[1] = [b, g]
     *   ...
     */
    let mut coords = HashMap::with_capacity(word_nr);
    let mut weight_map = HashMap::with_capacity(word_nr);

    for (i, word) in term.vocab.iter().enumerate() {
        let v: Vec<f64> = (0..real_dimensions).map(|d| vectors[d][i]).collect();
        coords.insert(word.clone(), v);
        weight_map.insert(word.clone(), term.weights[i]);
    }

    /*
     * Get each commit position, we already have the word coords and we use
     * them to calculate the commits positions:
     *
     * A commit is a group of words, to place it in each dimension we sum
     * each word's weight in the commit times its score in that dimension,
     * which is A^t * u for every dimension at once.
     *
     * mul_block_t computes all dimensions in a single matrix pass. Its
     * output is row-major, so the k coordinates of commit j come out
     * already contiguous at positions[j * k .. (j + 1) * k], which is
     * exactly the per-commit shape we want.
     */
    let k = real_dimensions;
    let mut word_block = vec![0.0; word_nr * k];

    for (d, vector) in vectors.iter().enumerate() {
        for (r, &val) in vector.iter().enumerate() {
            word_block[r * k + d] = val;
        }
    }

    let commit_nr = messages.len();
    let mut position_block = vec![0.0; commit_nr * k];
    term.matrix.mul_block_t(&word_block, &mut position_block, k);

    let commit_positions: Vec<Vec<f64>> = position_block
        .chunks_exact(k)
        .map(|row| row.to_vec())
        .collect();

    (
        WordMap::from_raw(coords, weight_map, real_dimensions),
        commit_positions,
        stats,
    )
}

pub fn build_index(
    commits: &[crate::git::Commit],
    ctx: &crate::preference::Preferences,
    syn: &HashMap<String, String>,
) -> (WordMap, Vec<Vec<f64>>, LsaStats) {
    let messages: Vec<String> = commits
        .iter()
        .map(|c| crate::text::preprocess(&c.message, syn))
        .collect();

    build(&messages, ctx)
}
