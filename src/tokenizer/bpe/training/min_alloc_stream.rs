use crate::tokenizer::bpe::training::internal::{count_pairs, get_most_freq_pair};
use crate::tokenizer::bpe::types::{BpeTokenizer, Merge, TokenId};
use crate::tokenizer::initial::{build_initial_token_sequence, initial_tokens};
use crate::tokenizer::preprocessing::PreTokens;
use std::collections::HashMap;
use std::iter::zip;

pub const fn update_sequence(
    pair: (TokenId, TokenId),
    new_id: TokenId,
    token_sequence: &mut [TokenId],
    len: &mut usize,
) {
    let (token_1, token_2) = pair;

    let mut read: usize = 0;
    let mut write: usize = 0;

    while read < *len {
        if token_sequence[read] == token_1 && read + 1 < *len && token_sequence[read + 1] == token_2
        {
            token_sequence[write] = new_id;
            read += 2;
        } else {
            token_sequence[write] = token_sequence[read];
            read += 1;
        }
        write += 1;
    }

    *len = write;
}

fn decrement_pair(
    pair_count: &mut HashMap<(TokenId, TokenId), usize>,
    pair: (TokenId, TokenId),
) -> Result<(), String> {
    match pair_count.get_mut(&pair) {
        Some(count) => {
            if *count == 0usize {
                return Err(format!(
                    "Program error: pair {pair:?} in the map has frequency=0 which cannot be decremented."
                ));
            }
            *count -= 1;
            if *count == 0usize {
                pair_count.remove(&pair);
            }
        }
        None => {
            return Err(format!(
                "Program error: the map does not contain {pair:?}, count of which is to be decremented"
            ));
        }
    }
    Ok(())
}

fn update_pair_count(token_sequence: &[TokenId], removal_pair: (TokenId, TokenId), new_id: TokenId, pair_count: &mut HashMap<(TokenId, TokenId), usize>) -> Result<(), String> {
    let (rtok_1, rtok_2) = removal_pair;

    for pair_vec in token_sequence.windows(2) {
        let (tok_1, tok_2) = (pair_vec[0], pair_vec[1]);
        if tok_1 == new_id {
            if tok_2 == new_id {
                decrement_pair(pair_count, (rtok_2, rtok_1)).map_err(|err| {
                    format!(
                        "{err}\ntok_1 == tok_2 == new_id({new_id})\nremoval pair:{removal_pair:?}"
                    )
                })?;
            } else {
                decrement_pair(pair_count, (rtok_2, tok_2)).map_err(|err| {
                    format!(
                        "{err}\ntok_1 == new_id && tok_2 != new_id({new_id})\nremoval pair:{removal_pair:?}"
                    )
                })?;
            }
            *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
        } else if tok_2 == new_id {
            *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
            decrement_pair(pair_count, (tok_1, rtok_1)).map_err(|err| {
                format!(
                    "{err}\ntok_1 != new_id && tok_2 == new_id({new_id})\nremoval pair:{removal_pair:?}"
                )
            })?;
        }
    }
    Ok(())
}

fn update(token_sequences: &mut [&mut [TokenId]], lens: &mut [usize], removal_pair: (TokenId, TokenId), new_id: TokenId, pair_count: &mut HashMap<(TokenId, TokenId), usize>) -> Result<(), String> {
    for (token_sequence, len) in zip(token_sequences, lens) {
        update_sequence(removal_pair, new_id, token_sequence, len);
        let token_seq = &token_sequence[..*len];
        update_pair_count(token_seq, removal_pair, new_id, pair_count)?;
    }
    pair_count.remove(&removal_pair);
    Ok(())
}

pub fn bpe_training(
    pretokens: &PreTokens,
    vocab_size: usize,
    min_pair_freq: usize,
) -> Result<BpeTokenizer, String> {
    let mut curr_vocab_size: TokenId = 256; // initial vocab consisting of all possible bytes
    let mut merges = Vec::new();

    let mut initial_tokens = initial_tokens(pretokens);
    let mut token_seq = build_initial_token_sequence(&mut initial_tokens);
    let mut lens = token_seq.iter().map(|s| s.len()).collect::<Vec<_>>();

    let mut token_pair = count_pairs(&token_seq);

    if vocab_size < curr_vocab_size {
        return Err(format!(
            "Initial vocab of lone letters of size {curr_vocab_size} exceed the desired `vocab_size` of {vocab_size}"
        ));
    }

    while vocab_size > curr_vocab_size {
        let Some(pair) = get_most_freq_pair(&token_pair, min_pair_freq) else {
            break;
        };
        update(&mut token_seq, &mut lens, pair, curr_vocab_size, &mut token_pair)?;
        curr_vocab_size += 1;

        merges.push(Merge {
            left: pair.0,
            right: pair.1,
        });
    }

    Ok(BpeTokenizer { merges })
}