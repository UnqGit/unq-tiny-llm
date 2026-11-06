use crate::tokenizer::bpe::types::*;
use crate::tokenizer::bpe::training::internal::*;
use crate::tokenizer::initial::*;
use crate::tokenizer::preprocessing::PreTokens;

fn build_new_seq(token_sequences: &[Vec<TokenId>], change_pair: (TokenId, TokenId),  new_id: TokenId) -> Vec<Vec<TokenId>> {
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

fn bpe_iter(token_sequences: &[Vec<TokenId>], vocab: &[VocabWord], min_pair_freq: usize) -> Option<(Vec<Vec<TokenId>>, VocabWord, Merge)> {
    let pairs_count = count_pairs(token_sequences);
    let most_freq_pair = get_most_freq_pair(&pairs_count, min_pair_freq)?;
    let (token_1, token_2) = most_freq_pair;

    let new_word_seq = [
        vocab[token_1].seq.clone(),
        vocab[token_2].seq.clone(),
    ]
    .concat();

    let new_vocab_word = VocabWord { seq: new_word_seq, id: vocab.len() };
    let merge = Merge { left: token_1, right: token_2, merged: new_vocab_word.id };

    let result_seq = build_new_seq(token_sequences, most_freq_pair, new_vocab_word.id);

    Some((result_seq, new_vocab_word, merge))
}

#[allow(unused)]
pub fn bpe_training(pretokens: &PreTokens, vocab_size: usize, min_pair_freq: usize) -> Result<BpeTokenizer, String> {
    let mut vocab = initial_vocab();
    let mut merges = Vec::new();
    let mut token_sequence = initial_token_sequence(pretokens);

    if (vocab_size) < vocab.len() {
        return Err(format!("Initial vocab of lone letters of size {} exceed the desired 'vocab_size' of {}", vocab.len(), vocab_size));
    }

    while (vocab_size) > vocab.len() {
        let Some(seq_tok) = bpe_iter(&token_sequence, &vocab, min_pair_freq) else {
            break;
        };
        token_sequence = seq_tok.0;
        vocab.push(seq_tok.1);
        merges.push(seq_tok.2);
    }

    Ok(BpeTokenizer{vocab: vocab, merges: merges})
}