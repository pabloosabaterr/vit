/*
 * Text preprocessing pipeline.
 *
 * TODO:
 *   - stopword removal
 *   - synonym resolution
 */

use crate::stemmer;

/// Cleans and normalizes a commit message for hashing.
/// Applies: lowercase, strip punctuation, stem each word.
pub fn preprocess(message: &str) -> String {
    message
        .split_whitespace()
        .map(|w| {
            let clean: String = w
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect();
            stemmer::stem(&clean)
        })
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
