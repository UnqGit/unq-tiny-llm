use regex::Regex;
use std::ops::Range;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct PreTokens {
    pub text: String,
    pub chunks: Vec<Range<usize>>,
}

pub fn normalize_text(text: &str) -> String {
    text.nfc().collect()
}

pub fn pretokenize_text(text: String) -> PreTokens {
    let pattern = Regex::new(r"\s+|[a-zA-Z]+|\d+|[^\w\s]+").unwrap();

    let chunks = pattern
        .find_iter(&text)
        .map(|mat| mat.range())
        .collect();

    PreTokens { text, chunks }
}

pub fn preprocess_text(text: &str) -> PreTokens {
    let normalized_text = normalize_text(text);
    pretokenize_text(normalized_text)
}

pub fn build_text_seq(pretokens: &PreTokens) -> Vec<&str> {
    pretokens.chunks
        .iter()
        .map(|r| &pretokens.text[r.clone()])
        .collect()
}