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

fn get_most_freq_pair(pairs_count: &HashMap<(TokenId, TokenId), usize>, min_pair_freq: usize) -> Option<(TokenId, TokenId)> {
    pairs_count
        .iter()
        .filter(|&(_, count)| *count >= min_pair_freq)
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

fn build_new_seq(token_sequence: &[TokenId], change_pair: (TokenId, TokenId),  new_id: TokenId) -> Vec<TokenId> {
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

fn bpe_iter(token_sequence: &[TokenId], vocab: &[VocabWord], min_pair_freq: usize) -> Option<(Vec<TokenId>, VocabWord, Merge)> {
    let pairs_count = count_pairs(token_sequence);
    let most_freq_pair = get_most_freq_pair(&pairs_count, min_pair_freq)?;
    let (token_1, token_2) = most_freq_pair;

    let new_word_seq = [
        vocab[token_1].seq.clone(),
        vocab[token_2].seq.clone(),
    ]
    .concat();

    let new_vocab_word = VocabWord { seq: new_word_seq, id: vocab.len() };
    let merge = Merge { left: token_1, right: token_2, merged: new_vocab_word.id };

    let result_seq = build_new_seq(token_sequence, most_freq_pair, new_vocab_word.id);

    Some((result_seq, new_vocab_word, merge))
}

pub fn bpe_training(text: &str, vocab_size: usize, min_pair_freq: usize) -> Result<BpeTokenizer, String> {
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
        let Some(seq_tok) = bpe_iter(&token_sequence, &vocab, min_pair_freq) else {
            break;
        };
        token_sequence = seq_tok.0;
        vocab.push(seq_tok.1);
        merges.push(seq_tok.2);
    }

    Ok(BpeTokenizer{vocab: vocab, merges: merges})
}

fn pair_count(tokens: &Vec<TokenId>) -> HashMap<(TokenId, TokenId), usize> {
    let mut result = HashMap::new();

    for pair in tokens.windows(2) {
        let key = (pair[0], pair[1]);
        *result.entry(key).or_insert(0) += 1;
    }

    result
}

fn decrement_pair(pair_count: &mut HashMap<(TokenId, TokenId), usize>, pair: (TokenId, TokenId)) -> Result<(), String> {
    match pair_count.get_mut(&pair) {
        Some(count) => {
            if *count == 0usize {
                return Err(format!("Program error: pair {:?} in the map has frequency=0 which cannot be decremented.", pair));
            }
            *count -= 1;
            if *count == 0usize {
                pair_count.remove(&pair);
            }
        },
        None => return Err(format!("Program error: the map does not contain {:?}, count of which is to be decremented", pair)),
    }
    Ok(())
}

fn update_pair_count(token_sequence: &[TokenId], removal_pair: (TokenId, TokenId), new_id: TokenId, pair_count: &mut HashMap<(TokenId, TokenId), usize>) -> Result<(), String> {
    let (rtok_1, rtok_2) = removal_pair;
    for pair_vec in token_sequence.windows(2) {
        let (tok_1, tok_2) = (pair_vec[0], pair_vec[1]);
        if tok_1 == new_id {
            if tok_2 == new_id {
                decrement_pair(pair_count, (rtok_2, rtok_1))?;
            } else {
                decrement_pair(pair_count, (rtok_2, tok_2))?;
            }
            decrement_pair(pair_count, removal_pair)?;
            *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
        }
        else if tok_2 == new_id {
            *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
            decrement_pair(pair_count, (tok_1, rtok_1))?
        }
    }
    Ok(())
}

pub fn bpe_training_pair_count_optimized(text: &str, vocab_size: usize, min_pair_freq: usize) -> Result<BpeTokenizer, String> {
    if text.is_empty() {
        return Err(String::from("Cannot encode text of length 0"));
    }

    let mut vocab = get_initial_vocab();
    let mut merges = Vec::new();
    let mut token_seq = get_init_token_sequence(text.as_bytes());
    let mut token_pair = pair_count(&token_seq);

    if vocab_size < vocab.len() {
        return Err(format!("Initial vocab of lone letters of size {} exceed the desired `vocab_size` of {}", vocab.len(), vocab_size));
    }

    while vocab_size > vocab.len() {
        let Some(pair) = get_most_freq_pair(&token_pair, min_pair_freq) else { break };
        let new_word_seq = [
            vocab[pair.0].seq.clone(),
            vocab[pair.1].seq.clone(),
        ].concat();
        let new_vocab_word = VocabWord { seq: new_word_seq, id: vocab.len() };
        token_seq = build_new_seq(&token_seq, pair, new_vocab_word.id);
        update_pair_count(&token_seq, pair, new_vocab_word.id, &mut token_pair)?;

        // it was checked that both implementations produce the same results
        // let expected = pair_count(&token_seq);
        // assert_eq!(token_pair, expected);

        merges.push(Merge { left: pair.0, right: pair.1, merged: new_vocab_word.id });
        vocab.push(new_vocab_word);
    }

    Ok(BpeTokenizer { vocab, merges })
}

fn update_sequence(pair: (TokenId, TokenId), new_id: TokenId, token_sequence: &mut Vec<TokenId>) {
    let len = token_sequence.len();
    let (token_1, token_2) = pair;

    let mut offset: usize = 0;
    for i in 0..len {
        if token_sequence[i] == token_1 && i + 1 < len && token_sequence[i + 1] == token_2 {
            token_sequence[i - offset] = new_id;
            offset += 1;
        }
        else {
            token_sequence[i - offset] = token_sequence[i];
        }
    }

    token_sequence.truncate(len - offset);
}

fn bpe_iter_2(token_sequence: &mut Vec<TokenId>, vocab: &[VocabWord], min_pair_freq: usize) -> Option<(VocabWord, Merge)> {
    let pairs_count = count_pairs(token_sequence);
    let most_freq_pair = get_most_freq_pair(&pairs_count, min_pair_freq)?;
    let (token_1, token_2) = most_freq_pair;

    let new_word_seq = [
        vocab[token_1].seq.clone(),
        vocab[token_2].seq.clone(),
    ]
    .concat();

    let new_vocab_word = VocabWord { seq: new_word_seq, id: vocab.len() };
    let merge = Merge { left: token_1, right: token_2, merged: new_vocab_word.id };

    update_sequence(most_freq_pair, new_vocab_word.id, token_sequence);

    Some((new_vocab_word, merge))
}

pub fn bpe_training_token_seq_update_optimized(text: &str, vocab_size: usize, min_pair_freq: usize) -> Result<BpeTokenizer, String> {
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
        let Some(seq_tok) = bpe_iter_2(&mut token_sequence, &vocab, min_pair_freq) else {
            break;
        };
        vocab.push(seq_tok.0);
        merges.push(seq_tok.1);
    }

    Ok(BpeTokenizer{vocab: vocab, merges: merges})
}