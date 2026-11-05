use regex::Regex;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct PreTokens {
    pub text: String,
    pub chunks: Vec<Range<usize>>,
}

pub fn pretokenize_text(text: String) -> PreTokens {
    let pattern = Regex::new(r"\s+|[a-zA-Z]+|\d+|[^\w\s]+").unwrap();

    let chunks = pattern
        .find_iter(&text)
        .map(|mat| mat.range())
        .collect();

    PreTokens { text, chunks }
}