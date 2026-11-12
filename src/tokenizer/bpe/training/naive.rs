use crate::{log_info, log_success};
use crate::logger::LoadingBar;
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

#[allow(unused)]
pub fn bpe_training(
    pretokens: &PreTokens,
    vocab_size: usize,
    min_pair_freq: usize,
) -> Result<BpeTokenizer, String> {
    log_info!("BPE Training initialized.");
    log_info!("Method of training: Naive\n");

    let mut curr_vocab_size: TokenId = 256; // initial vocab consisting of all possible bytes
    let mut merges = Vec::new();
    let mut token_sequence = initial_token_sequence(pretokens);

    if vocab_size < curr_vocab_size {
        return Err(format!(
            "Initial vocab of lone letters of size {curr_vocab_size} exceed the desired 'vocab_size' of {vocab_size}"
        ));
    }

    let mut bar = LoadingBar::create(vocab_size, 40);

    while vocab_size > curr_vocab_size {
        let pair_count = count_pairs(&token_sequence);
        let Some(best_pair) = get_most_freq_pair(&pair_count, min_pair_freq) else { break; };
        token_sequence = build_new_seq(&token_sequence, best_pair, curr_vocab_size);
        merges.push(Merge { left: best_pair.0, right: best_pair.1 });
        curr_vocab_size += 1;
        bar.show(curr_vocab_size);
    }
    bar.clear();
    
    log_success!("BPE Training complete with {} merges.\n", merges.len());

    Ok(BpeTokenizer { merges })
}
