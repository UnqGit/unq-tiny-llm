use std::collections::HashMap;
use crate::tokenizer::bpe::types::*;

pub fn count_pairs(sequence: &[TokenId]) -> HashMap<(TokenId, TokenId), usize> {
    let mut result = HashMap::new();

    for pair in sequence.windows(2) {
        let key = (pair[0], pair[1]);
        *result.entry(key).or_insert(0) += 1;
    }

    result
}

pub fn get_most_freq_pair(pairs_count: &HashMap<(TokenId, TokenId), usize>, min_pair_freq: usize) -> Option<(TokenId, TokenId)> {
    pairs_count
        .iter()
        .filter(|&(_, count)| *count >= min_pair_freq)
        .max_by(|(pair_a, count_a), (pair_b, count_b)|
            count_a
                .cmp(count_b)
                .then_with(|| pair_b.cmp(pair_a))
        ).map(|(pair, _)| *pair)
}

pub fn get_initial_vocab() -> Vec<VocabWord> {
    (u8::MIN..=u8::MAX)
        .map(|b| VocabWord { seq: vec![b], id: TokenId::from(b) })
        .collect()
}

pub fn get_init_token_sequence(byte_sequence: &[u8]) -> Vec<TokenId> {
    byte_sequence
        .iter()
        .map(|&b| TokenId::from(b))
        .collect()
}

pub fn build_new_seq(token_sequence: &[TokenId], change_pair: (TokenId, TokenId),  new_id: TokenId) -> Vec<TokenId> {
    let (token_1, token_2) = change_pair;
    let mut result_seq = Vec::with_capacity(token_sequence.len());
    let mut iter = token_sequence.iter().copied().peekable();
    
    while let Some(token) = iter.next() {
        if token == token_1 && iter.peek() == Some(&token_2) {
            iter.next();
            result_seq.push(new_id);
        } else {
            result_seq.push(token);
        }
    }

    result_seq
}