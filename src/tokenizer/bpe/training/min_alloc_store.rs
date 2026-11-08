use crate::tokenizer::bpe::training::internal::{count_pairs, get_most_freq_pair};
use crate::tokenizer::bpe::types::{BpeTokenizer, Merge, TokenId};
use crate::tokenizer::initial::{build_initial_token_sequence, initial_tokens};
use crate::tokenizer::preprocessing::PreTokens;
use std::collections::HashMap;
use std::iter::zip;

fn update_sequence(
    pair: (TokenId, TokenId),
    new_id: TokenId,
    token_sequence: &mut [TokenId],
    len: &mut usize,
    pos_buffer: &mut [usize]
) -> usize {
    let size = *len;
    if size == 1 {
        return 0;
    }
    
    let mut read: usize = 0;
    let mut write: usize = 0;
    
    let (token_1, token_2) = pair;


    while read < size {
        if token_sequence[read] == token_1 && read + 1 < size && token_sequence[read + 1] == token_2
        {
            pos_buffer[read-write] = write;
            token_sequence[write] = new_id;
            read += 2;
        } else {
            token_sequence[write] = token_sequence[read];
            read += 1;
        }
        write += 1;
    }

    *len = write;
    read-write
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

fn update_pair_count(
    token_sequence: &[TokenId],
    removal_pair: (TokenId, TokenId),
    pair_count: &mut HashMap<(TokenId, TokenId), usize>,
    pos_buffer: &[usize]
) -> Result<(), String> {
    let len = token_sequence.len();
    let (rtok_1, rtok_2) = removal_pair;

    for &pos in pos_buffer {
        if pos > 0 {
            let (tok_1, tok_2) = (token_sequence[pos - 1], token_sequence[pos]);
            if tok_1 != tok_2 {
                *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
                decrement_pair(pair_count, (tok_1, rtok_1)).map_err(|err| {
                    format!(
                        "{err}\ntok_1 != new_id && tok_2 == new_id({tok_2})\nremoval pair:{removal_pair:?}"
                    )
                })?;
            }
        }
        if pos + 1 < len {
            let (tok_1, tok_2) = (token_sequence[pos], token_sequence[pos + 1]);
            if tok_2 == tok_1 {
                decrement_pair(pair_count, (rtok_2, rtok_1)).map_err(|err| {
                    format!(
                        "{err}\ntok_1 == tok_2 == new_id({tok_1})\nremoval pair:{removal_pair:?}"
                    )
                })?;
            } else {
                decrement_pair(pair_count, (rtok_2, tok_2)).map_err(|err| {
                    format!(
                        "{err}\ntok_1 == new_id && tok_2 != new_id({tok_1})\nremoval pair:{removal_pair:?}"
                    )
                })?;
            }
            *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
        }
    }
    Ok(())
}

fn update(
    token_sequences: &mut [&mut [TokenId]],
    lens: &mut [usize],
    removal_pair: (TokenId, TokenId),
    new_id: TokenId,
    pair_count: &mut HashMap<(TokenId, TokenId), usize>,
    pos_buffer: &mut [usize],
) -> Result<(), String> {
    for (token_sequence, len) in zip(token_sequences, lens) {
        let buf_len = update_sequence(removal_pair, new_id, token_sequence, len, pos_buffer);
        if buf_len != 0 {
            let token_seq = &token_sequence[..*len];
            update_pair_count(token_seq, removal_pair, pair_count, &pos_buffer[..buf_len])?;
        }
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
    if vocab_size < curr_vocab_size {
        return Err(format!(
            "Initial vocab of lone letters of size {curr_vocab_size} exceed the desired `vocab_size` of {vocab_size}"
        ));
    }

    let mut merges = Vec::new();
    let mut initial_tokens = initial_tokens(pretokens);
    let mut token_seq = build_initial_token_sequence(&mut initial_tokens);

    if token_seq.is_empty() {
        return Ok(BpeTokenizer { merges });
    }

    let mut max_len = token_seq[0].len();
    let mut lens = token_seq.iter().map(|s| {
        let len = s.len();
        max_len = max_len.max(len);
        s.len()
    }).collect::<Vec<_>>();

    let mut pos_vec: Vec<usize> = Vec::new();
    pos_vec.resize(max_len / 2, 0);
    let mut token_pair = count_pairs(&token_seq);

    while vocab_size > curr_vocab_size {
        let Some(pair) = get_most_freq_pair(&token_pair, min_pair_freq) else {
            break;
        };
        update(&mut token_seq, &mut lens, pair, curr_vocab_size, &mut token_pair, &mut pos_vec)?;
        curr_vocab_size += 1;

        merges.push(Merge {
            left: pair.0,
            right: pair.1,
        });
    }

    Ok(BpeTokenizer { merges })
}