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

fn get_most_freq_pair(pairs_count: &HashMap<(TokenId, TokenId), usize>) -> Option<(TokenId, TokenId)> {
    pairs_count
        .iter()
        .filter(|&(_, count)| *count >= 2usize)
        .max_by(|(pair_a, count_a), (pair_b, count_b)|
            count_a
                .cmp(count_b)
                .then_with(|| pair_a.cmp(pair_b))
        ).map(|(pair, _)| *pair)
}

fn get_initial_vocab() -> Vec<VocabWord> {
    (u8::MIN..=u8::MAX)
        .map(|b| VocabWord { seq: vec![b], id: TokenId::from(b) })
        .collect()
}

fn get_init_token_sequence(byte_sequence: &[u8]) -> Vec<TokenId> {
    byte_sequence
        .iter()
        .map(|&b| TokenId::from(b))
        .collect()
}

fn bpe_iter(token_sequence: &[TokenId], vocab: &[VocabWord]) -> Option<(Vec<TokenId>, VocabWord, Merge)> {
    let pairs_count = count_pairs(token_sequence);
    let most_freq_pair = get_most_freq_pair(&pairs_count)?;
    let (token_1, token_2) = most_freq_pair;

    let new_word_seq = [
        vocab[token_1].seq.clone(),
        vocab[token_2].seq.clone(),
    ]
    .concat();

    let new_vocab_word = VocabWord { seq: new_word_seq, id: vocab.len() as TokenId };
    let merge = Merge { left: token_1, right: token_2, merged: new_vocab_word.id };

    let mut result_seq = Vec::with_capacity(token_sequence.len());
    let mut iter = token_sequence.iter().copied().peekable();

    while let Some(token) = iter.next() {
        if token == token_1 && iter.peek() == Some(&token_2) {
            iter.next();
            result_seq.push(new_vocab_word.id);
        } else {
            result_seq.push(token);
        }
    }

    Some((result_seq, new_vocab_word, merge))
}

pub fn bpe_training(text: &str, vocab_size: usize) -> Result<BpeTokenizer, String> {
    if text.is_empty() {
        return Err(String::from("Cannot encode text of length 0"));
    }

    let mut vocab = get_initial_vocab();
    let mut merges = Vec::new();
    let mut token_sequence = get_init_token_sequence(text.as_bytes());

    if (vocab_size) < vocab.len() {
        return Err(format!("Initial vocab of lone letters of size {} exceed the desired 'vocab_size' of {}", vocab.len(), vocab_size));
    }

    while (vocab_size) > vocab.len() {
        let Some(seq_tok) = bpe_iter(&token_sequence, &vocab) else {
            break;
        };
        token_sequence = seq_tok.0;
        vocab.push(seq_tok.1);
        merges.push(seq_tok.2);
    }

    Ok(BpeTokenizer{vocab: vocab, merges: merges})
}
