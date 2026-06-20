/*
 * Porter 2 stemmer algo
 *
 * docs: https://snowballstem.org/algorithms/english/stemmer.html
 *
 * TODO: Add more languages
 */

use std::usize;

fn is_vowel(b: u8) -> bool {
    matches!(b, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
}

fn is_double(word: &[u8]) -> bool {
    let n = word.len();

    if n < 2 {
        return false;
    }

    let (a, b) = (word[n - 2], word[n - 1]);
    a == b
        && matches!(
            a,
            b'b' | b'd' | b'f' | b'g' | b'm' | b'n' | b'p' | b'r' | b't'
        )
}

fn is_short_syllable(word: &[u8], i: usize) -> bool {
    if i + 2 < word.len()
        && !is_vowel(word[i])
        && is_vowel(word[i + 1])
        && !is_vowel(word[i + 2])
        && !matches!(word[i + 2], b'w' | b'x' | b'Y')
    {
        return true;
    }

    if i == 0 && word.len() >= 2 {
        return word.len() >= 2 && is_vowel(word[0]) && !is_vowel(word[1]);
    }
    false
}

fn compute_r(word: &[u8], start: usize) -> usize {
    let mut i = start;

    /* find a vowel */
    while i < word.len() && !is_vowel(word[i]) {
        i += 1;
    }

    i += 1;
    while i < word.len() && is_vowel(word[i]) {
        i += 1;
    }

    if i < word.len() { i + 1 } else { word.len() }
}

fn compute_r1(word: &[u8]) -> usize {
    if word.starts_with(b"gener") || word.starts_with(b"arsen") {
        return 5;
    }

    if word.starts_with(b"commun") {
        return 6;
    }
    compute_r(word, 0)
}

fn compute_r2(word: &[u8], r1_start: usize) -> usize {
    compute_r(word, r1_start)
}

fn ends_with(word: &[u8], suffix: &[u8]) -> bool {
    word.len() >= suffix.len() && &word[word.len() - suffix.len()..] == suffix
}

fn has_vowel(word: &[u8]) -> bool {
    word.iter().any(|&b| is_vowel(b))
}

fn valid_li_ending(b: u8) -> bool {
    matches!(
        b,
        b'c' | b'd' | b'e' | b'g' | b'h' | b'k' | b'm' | b'n' | b'r' | b't'
    )
}

fn is_short_word(word: &[u8], r1: usize) -> bool {
    if r1 < word.len() {
        return false;
    }

    let n = word.len();
    if n < 2 {
        false
    } else if n == 2 {
        is_short_syllable(word, 0)
    } else {
        is_short_syllable(word, n - 3)
    }
}

fn step_0(word: &mut Vec<u8>) {
    if ends_with(&word, b"'s'") {
        word.truncate(word.len() - 3);
    } else if ends_with(&word, b"'s") {
        word.truncate(word.len() - 2);
    } else if ends_with(&word, b"'") {
        word.truncate(word.len() - 1);
    }
}

fn step_1a(word: &mut Vec<u8>) {
    if ends_with(word, b"sses") {
        word.truncate(word.len() - 2);
    } else if ends_with(word, b"ied") || ends_with(word, b"ies") {
        let cut = word.len() - 3;
        if cut > 1 {
            word.truncate(cut + 1);
            word[cut] = b'i';
        } else {
            word.truncate(cut + 2);
            word[cut] = b'i';
            word[cut + 1] = b'e';
        }
    } else if ends_with(word, b"us") || ends_with(word, b"ss") {
        /* do nothing */
    } else if ends_with(word, b"s") {
        let preceding = &word[..word.len() - 2];
        if word.len() > 2 && has_vowel(preceding) {
            word.truncate(word.len() - 1);
        }
    }
}

fn is_exception_post_1a(word: &[u8]) -> bool {
    matches!(
        word,
        b"inning"
            | b"outing"
            | b"canning"
            | b"herring"
            | b"earring"
            | b"proceed"
            | b"exceed"
            | b"succeed"
    )
}

fn mark_consonant_y(word: &mut Vec<u8>) {
    if word[0] == b'y' {
        word[0] = b'Y';
    }
    for i in 1..word.len() {
        if word[i] == b'y' && is_vowel(word[i - 1]) {
            word[i] = b'Y';
        }
    }
}

fn restore_y(word: &mut Vec<u8>) {
    for b in word.iter_mut() {
        if *b == b'Y' {
            *b = b'y';
        }
    }
}

fn step_1b(word: &mut Vec<u8>, r1: usize) {
    if ends_with(&word, b"eedly") {
        if word.len() - 5 >= r1 {
            word.truncate(word.len() - 3); /* -> ee */
        }
    } else if ends_with(&word, b"eed") {
        if word.len() - 3 >= r1 {
            word.truncate(word.len() - 1); /* -> ee */
        }
    } else {
        let mut found = false;
        let suffixes: &[&[u8]] = &[b"ingly", b"edly", b"ing", b"ed"];
        for &suf in suffixes {
            if ends_with(&word, suf) {
                let preceding = &word[..word.len() - suf.len()];
                if has_vowel(preceding) {
                    word.truncate(word.len() - suf.len());
                    found = true;
                }
                break;
            }
        }
        if found {
            if ends_with(&word, b"at")
                || ends_with(&word, b"bl")
                || ends_with(&word, b"iz")
            {
                word.push(b'e');
            } else if is_double(&word) {
                word.truncate(word.len() - 1);
            } else if is_short_word(&word, compute_r1(&word)) {
                word.push(b'e');
            }
        }
    }
}

fn step_1c(word: &mut Vec<u8>) {
    if word.len() > 2 {
        let last = word.len() - 1;
        if (word[last] == b'y' || word[last] == b'Y') && !is_vowel(word[last - 1]) {
            word[last] = b'i';
        }
    }
}

fn step_2(word: &mut Vec<u8>, r1: usize) {
    let suffixes: &[(&[u8], &[u8], Option<u8>)] = &[
        (b"ational", b"ate", None),
        (b"fulness", b"ful", None),
        (b"iveness", b"ive", None),
        (b"ization", b"ize", None),
        (b"ousness", b"ous", None),
        (b"biliti", b"ble", None),
        (b"lessli", b"less", None),
        (b"tional", b"tion", None),
        (b"ation", b"ate", None),
        (b"alism", b"al", None),
        (b"aliti", b"al", None),
        (b"ousli", b"ous", None),
        (b"iviti", b"ive", None),
        (b"fulli", b"ful", None),
        (b"entli", b"ent", None),
        (b"enci", b"ence", None),
        (b"anci", b"ance", None),
        (b"abli", b"able", None),
        (b"izer", b"ize", None),
        (b"ator", b"ate", None),
        (b"alli", b"al", None),
        (b"bli", b"ble", None),
        (b"ogi", b"og", Some(b'l')),
        (b"li", b"", None), /* valid li-ending check below */
    ];

    for &(suf, rep, guard) in suffixes {
        if ends_with(&word, suf) {
            let cut = word.len() - suf.len();
            if cut >= r1 {
                if suf == b"li" {
                    if cut > 0 && valid_li_ending(word[cut - 1]) {
                        word.truncate(cut);
                    }
                } else if let Some(g) = guard {
                    if cut > 0 && word[cut - 1] == g {
                        word.truncate(cut);
                        word.extend_from_slice(rep);
                    }
                } else {
                    word.truncate(cut);
                    word.extend_from_slice(rep);
                }
            }
            break;
        }
    }
}

fn step_3(word: &mut Vec<u8>, r1: usize, r2: usize) {
    let suffixes: &[(&[u8], &[u8], bool)] = &[
        (b"ational", b"ate", false),
        (b"tional", b"tion", false),
        (b"alize", b"al", false),
        (b"icate", b"ic", false),
        (b"iciti", b"ic", false),
        (b"ative", b"", true), /* must be in R2 */
        (b"ical", b"ic", false),
        (b"ness", b"", false),
        (b"ful", b"", false),
    ];
    for &(suf, rep, need_r2) in suffixes {
        if ends_with(&word, suf) {
            let cut = word.len() - suf.len();
            let region = if need_r2 { r2 } else { r1 };
            if cut >= region {
                word.truncate(cut);
                word.extend_from_slice(rep);
            }
            break;
        }
    }
}

fn step_4(word: &mut Vec<u8>, r2: usize) {
    let suffixes: &[&[u8]] = &[
        b"ement", b"ment", b"ance", b"ence", b"able", b"ible", b"ant", b"ent",
        b"ism", b"ate", b"iti", b"ous", b"ive", b"ize", b"ion", b"al", b"er", b"ic",
    ];

    for &suf in suffixes {
        if ends_with(&word, suf) {
            let cut = word.len() - suf.len();
            if cut >= r2 {
                if suf == b"ion" {
                    /* delete only if preceded by s or t */
                    if cut > 0 && matches!(word[cut - 1], b's' | b't') {
                        word.truncate(cut);
                    }
                } else {
                    word.truncate(cut);
                }
            }
            break;
        }
    }
}

fn step_5(word: &mut Vec<u8>, r1: usize, r2: usize) {
    if let Some(&last) = word.last() {
        let n = word.len();
        if last == b'e' {
            if n - 1 >= r2 {
                word.truncate(n - 1);
            } else if n - 1 >= r1 {
                /* delete unless preceded by a short syllable */
                if n >= 3 && !is_short_syllable(&word, n - 3) {
                    word.truncate(n - 1);
                }
            }
        } else if last == b'l' && n - 1 >= r2 && n >= 2 && word[n - 2] == b'l' {
            word.truncate(n - 1);
        }
    }
}

pub fn stem(input: &str) -> String {
    let input = input.to_lowercase();
    let bytes = input.as_bytes();

    /* Words of 1-2 characters are left unchanged. */
    if bytes.len() <= 2 {
        return input;
    }

    let mut word: Vec<u8> = bytes.to_vec();
    mark_consonant_y(&mut word);

    let mut r1 = compute_r1(&word);

    step_0(&mut word);
    step_1a(&mut word);

    if is_exception_post_1a(&word) {
        restore_y(&mut word);
        return String::from_utf8(word).unwrap_or(input);
    }

    step_1b(&mut word, r1);
    step_1c(&mut word);

    /* Recompute R1/R2 since the word may have changed. */
    r1 = compute_r1(&word);
    let mut r2 = compute_r2(&word, r1);

    step_2(&mut word, r1);
    step_3(&mut word, r1, r2);
    step_4(&mut word, r2);

    /* Recompute R1/R2 for step 5. */
    r1 = compute_r1(&word);
    r2 = compute_r2(&word, r1);

    step_5(&mut word, r1, r2);
    restore_y(&mut word);

    String::from_utf8(word).unwrap_or(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_running() {
        assert_eq!(stem("running"), "run");
    }

    #[test]
    fn stem_replace_and_replacement_return_the_same() {
        assert_eq!(stem("replace"), "replac");
        assert_eq!(stem("replacement"), "replac");
        assert_eq!(stem("replacement"), stem("replace"));
    }
}
