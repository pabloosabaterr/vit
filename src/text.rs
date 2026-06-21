/*
 * Text preprocessing pipeline.
 *
 * Normalizes commit messages before hashing:
 *   1. lowercase
 *   2. strip punctuation
 *   3. remove stopwords
 *   4. resolve synonyms
 *   5. stem (Porter2)
 */

use std::collections::HashMap;

use crate::stemmer;

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "do", "for", "from",
    "had", "has", "have", "he", "her", "his", "how", "i", "if", "in", "into", "is",
    "it", "its", "my", "no", "not", "of", "on", "or", "our", "out", "she", "so",
    "than", "that", "the", "their", "them", "then", "there", "these", "they",
    "this", "to", "up", "us", "was", "we", "were", "what", "when", "which", "who",
    "will", "with", "would", "you", "your",
];

fn is_stopword(word: &str) -> bool {
    STOPWORDS.iter().any(|&s| s == word)
}

pub fn load_synonyms() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let content = match std::fs::read_to_string(".vitsynonyms") {
        Ok(c) => c,
        Err(_) => return map,
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((from, to)) = line.split_once('=') {
            map.insert(from.trim().to_lowercase(), to.trim().to_lowercase());
        }
    }
    map
}

fn resolve_synonym(word: &str, synonyms: &HashMap<String, String>) -> String {
    synonyms
        .get(word)
        .cloned()
        .unwrap_or_else(|| word.to_string())
}

fn strip_punctuation(word: &str) -> String {
    word.chars()
        .map(|c| if c == '-' { ' ' } else { c })
        .filter(|&c| c.is_alphabetic() || c == ' ')
        .collect()
}

/*
 * Cleans and normalizes a commit message for hashing.
 * Applies: lowercase, strip punctuation, stem each word.
 *
 * It is caller duty to load the synonyms.
 */
pub fn preprocess(message: &str, syn: &HashMap<String, String>) -> String {
    message
        .split_whitespace()
        .flat_map(|w| {
            strip_punctuation(&w.to_lowercase())
                .split_whitespace()
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .filter(|w| !w.is_empty() && !is_stopword(w))
        .map(|w| resolve_synonym(&w, syn))
        .map(|w| stemmer::stem(&w))
        .collect::<Vec<_>>()
        .join(" ")
}
