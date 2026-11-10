use crate::tokenizer::bpe::training::internal::{TokenPair, count_pairs, get_most_freq_pair};
use crate::tokenizer::bpe::types::{BpeTokenizer, Merge, TokenId};
use crate::tokenizer::bpe::training::initial::initial_token_sequence;
use crate::tokenizer::preprocessing::PreTokens;

fn build_new_seq(
    token_sequences: &[Vec<TokenId>],
    change_pair: TokenPair,
    new_id: TokenId,
) -> Vec<Vec<TokenId>> {
    let (token_1, token_2) = change_pair;

    let mut result_seq = Vec::with_capacity(token_sequences.len());

    for token_sequence in token_sequences {
        let mut updated_sequence = Vec::with_capacity(token_sequence.len());
        let mut iter = token_sequence.iter().copied().peekable();

        while let Some(token) = iter.next() {
            if token == token_1 && iter.peek() == Some(&token_2) {
                iter.next();
                updated_sequence.push(new_id);
            } else {
                updated_sequence.push(token);
            }
        }

        result_seq.push(updated_sequence);
    }

    result_seq
}

fn bpe_iter(
    token_sequences: &[Vec<TokenId>],
    min_pair_freq: usize,
    new_id: TokenId,
) -> Option<(Vec<Vec<TokenId>>, Merge)> {
    let pairs_count = count_pairs(token_sequences);
    let most_freq_pair = get_most_freq_pair(&pairs_count, min_pair_freq)?;
    let (token_1, token_2) = most_freq_pair;

    let merge = Merge {
        left: token_1,
        right: token_2,
    };

    let result_seq = build_new_seq(token_sequences, most_freq_pair, new_id);

    Some((result_seq, merge))
}

#[allow(unused)]
pub fn bpe_training(
    pretokens: &PreTokens,
    vocab_size: usize,
    min_pair_freq: usize,
) -> Result<BpeTokenizer, String> {
    let mut curr_vocab_size: TokenId = 256; // initial vocab consisting of all possible bytes
    let mut merges = Vec::new();
    let mut token_sequence = initial_token_sequence(pretokens);

    if vocab_size < curr_vocab_size {
        return Err(format!(
            "Initial vocab of lone letters of size {curr_vocab_size} exceed the desired 'vocab_size' of {vocab_size}"
        ));
    }

    while vocab_size > curr_vocab_size {
        let Some(seq_tok) = bpe_iter(&token_sequence, min_pair_freq, curr_vocab_size) else {
            break;
        };
        token_sequence = seq_tok.0;
        curr_vocab_size += 1;
        merges.push(seq_tok.1);
    }

    Ok(BpeTokenizer { merges })
}
