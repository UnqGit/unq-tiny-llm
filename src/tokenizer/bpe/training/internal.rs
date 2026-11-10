use crate::tokenizer::bpe::types::TokenId;
use std::collections::HashMap;

pub type TokenPair = (TokenId, TokenId);
pub type TokenFreqMap = HashMap<TokenPair, usize>;

pub fn count_pairs(
    token_sequences: &[impl AsRef<[TokenId]>],
) -> TokenFreqMap {
    let mut result = HashMap::new();

    for sequence in token_sequences {
        for pair in sequence.as_ref().windows(2) {
            let key = (pair[0], pair[1]);
            *result.entry(key).or_insert(0) += 1;
        }
    }

    result
}

pub fn get_most_freq_pair(
    pairs_count: &TokenFreqMap,
    min_pair_freq: usize,
) -> Option<TokenPair> {
    pairs_count.iter()
        .filter(|&(_, count)| *count >= min_pair_freq)
        .max_by(|(pair_a, count_a), (pair_b, count_b)| {
            count_a.cmp(count_b).then_with(|| pair_b.cmp(pair_a))
        })
        .map(|(pair, _)| *pair)
}
