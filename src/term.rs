/*
 * Lsa should should only need to call a function expecting to get a sparse matrix
 * with the word distribution, Lsa should not be aware of which algorithm/function is
 * being used.
 *
 * Here are the functions that the Lsa will consume
 */

use std::collections::{HashMap, HashSet};

pub struct TermData {
    pub matrix: SparseMatrix,
    pub vocab: Vec<String>,
    pub weights: Vec<f64>,
}

use crate::sparse_matrix::SparseMatrix;

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

fn get_idf(word_freq: &[usize], doc_nr: usize) -> Vec<f64> {
    let log_docs = (doc_nr as f64).ln();

    word_freq
        .iter()
        .map(|&freq| {
            let idf = log_docs - (freq as f64).ln();
            if idf > 0.0 { idf } else { 0.0 }
        })
        .collect()
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
    idf: &[f64],
) -> SparseMatrix {
    let doc_nr = messages.len();
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
            let weight = term_freq * idf[word_id];

            if weight > 0.0 {
                triplets.push((word_id, doc, weight));
            }
        }
    }

    SparseMatrix::from_triplets(word_nr, doc_nr, &mut triplets)
}

pub fn get_sparse_matrix(messages: &[String]) -> Option<TermData> {
    let (vocab, words, word_freq) = build_vocab(messages);

    if vocab.len() < 2 {
        return None;
    }

    let idf = get_idf(&word_freq, messages.len());
    let matrix = build_tfidf(messages, &words, vocab.len(), &idf);

    Some(TermData {
        matrix,
        vocab,
        weights: idf,
    })
}
