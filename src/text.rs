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
use std::collections::HashSet;
use std::sync::LazyLock;

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "do", "for", "from",
    "had", "has", "have", "he", "her", "his", "how", "i", "if", "in", "into", "is",
    "it", "its", "my", "no", "not", "of", "on", "or", "our", "out", "she", "so",
    "than", "that", "the", "their", "them", "then", "there", "these", "they",
    "this", "to", "up", "us", "was", "we", "were", "what", "when", "which", "who",
    "will", "with", "would", "you", "your",
];

static STOPWORD_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| STOPWORDS.iter().copied().collect());

fn is_stopword(word: &str) -> bool {
    STOPWORD_SET.contains(word)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lowercase_and_get_stem() {
        let res = preprocess("RunNiNg", &HashMap::new());
        assert_eq!(res, "run");
    }

    #[test]
    fn hyphens_become_spaces() {
        let res = preprocess("compile-time", &HashMap::new());
        assert_eq!(res, "compil tim");
    }

    #[test]
    fn removes_punctuation() {
        let res = preprocess("fix(parser): expresions!?", &HashMap::new());
        assert_eq!(res, "fixpars expres");
    }

    #[test]
    fn stopwords_get_removed() {
        let res =
            preprocess("fix the bug in the assembly generation", &HashMap::new());
        assert_eq!(res, "fix bug assembl generat");
    }

    #[test]
    fn synonims_get_replaced() {
        let mut synonyms = HashMap::new();
        synonyms.insert("redo".to_string(), "refactor".to_string());
        let res = preprocess("redo the whole lexer phase", &synonyms);
        assert_eq!(res, "refactor whol lexer phas");
    }
}
