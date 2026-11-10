use crate::tokenizer::bpe::types::TokenId;
use crate::tokenizer::preprocessing::{PreTokens, build_text_seq};
use std::ops::Range;

pub fn initial_token_sequence(pretokens: &PreTokens) -> Vec<Vec<TokenId>> {
    let text_tokens = build_text_seq(pretokens);
    text_tokens
        .iter()
        .map(|&b| b.as_bytes().iter().map(|&c| TokenId::from(c)).collect())
        .collect()
}

// a memory optimization inspired by PreTokens
#[derive(Debug, Clone)]
pub struct TokenArena {
    pub arena: Vec<TokenId>,
    pub chunks: Vec<Range<usize>>,
}

pub fn get_token_buffer(pretoken_sequence: &PreTokens) -> Vec<TokenId> {
    pretoken_sequence
        .text
        .as_bytes()
        .iter()
        .map(|&b| TokenId::from(b))
        .collect()
}

pub fn initial_tokens(pretoken_sequence: &PreTokens) -> TokenArena {
    let arena = get_token_buffer(pretoken_sequence);
    TokenArena {
        arena,
        chunks: pretoken_sequence.chunks.clone(),
    }
}

pub fn build_initial_token_sequence(initial_tokens: &mut TokenArena) -> Vec<&mut [TokenId]> {
    let TokenArena {
        arena: data,
        chunks: ranges,
    } = initial_tokens;
    let mut result = Vec::with_capacity(ranges.len());
    let mut remaining = data.as_mut_slice();

    for range in ranges {
        let len = range.end - range.start;
        let (slice, rest) = remaining.split_at_mut(len);
        result.push(slice);
        remaining = rest;
    }

    result
}
