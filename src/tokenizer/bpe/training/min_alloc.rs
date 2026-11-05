use std::collections::HashMap;
use crate::tokenizer::bpe::types::*;
use crate::tokenizer::bpe::training::internal::*;

fn update_sequence(pair: (TokenId, TokenId), new_id: TokenId, token_sequence: &mut Vec<TokenId>) {
    let len = token_sequence.len();
    let (token_1, token_2) = pair;
    
    let mut read: usize = 0;
    let mut write: usize = 0;

    while read < len {
        if token_sequence[read] == token_1 && read + 1 < len && token_sequence[read + 1] == token_2 {
            token_sequence[write] = new_id;
            read += 2;
        } else {
            token_sequence[write] = token_sequence[read];
            read += 1;
        }
        write += 1;
    }

    token_sequence.truncate(write);
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

    // if the sequence ends with two consecutive new tokens: (a,a,a,a) -> (x,x)
    // here frequency should only be 1 or 0, so we try to decrement it and assert the final condition.
    if pair_count.contains_key(&removal_pair) {
        decrement_pair(pair_count, removal_pair)?;
    }
    assert!(pair_count.get(&removal_pair).is_none());
    Ok(())
}

pub fn bpe_training(text: &str, vocab_size: usize, min_pair_freq: usize) -> Result<BpeTokenizer, String> {
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
        update_sequence(pair, new_vocab_word.id, &mut token_seq);
        update_pair_count(&token_seq, pair, new_vocab_word.id, &mut token_pair)?;
                
        merges.push(Merge { left: pair.0, right: pair.1, merged: new_vocab_word.id });
        vocab.push(new_vocab_word);
    }
    
    Ok(BpeTokenizer { vocab, merges })
}