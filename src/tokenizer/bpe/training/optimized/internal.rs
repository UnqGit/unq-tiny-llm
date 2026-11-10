use std::collections::HashMap;
use std::iter::zip;

use crate::tokenizer::bpe::{
    types::TokenId,
    training::internal::{TokenPair, TokenFreqMap},
};

// SERIAL optimized version common functions

pub const fn update_sequence(
    pair: TokenPair,
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

pub fn decrement_pair(
    pair_count: &mut TokenFreqMap,
    pair: TokenPair,
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

// Also used internally by parallel versions
pub fn update_sequence_discrete(
    pair: TokenPair,
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

// PARALLEL optimized version common functions

pub type TokenDeltaMap = HashMap<TokenPair, isize>;

#[derive(Debug)]
pub struct ThreadTokenArena<'a> {
    pub arena: &'a mut [&'a mut[TokenId]],
    pub lens: Vec<usize>,
    pub change_buf: Vec<usize>,
    pub pair_delta: HashMap<TokenPair, isize>,
}

fn update_delta_aggregate(
    token_sequence: &[TokenId],
    removal_pair: TokenPair,
    pair_count: &mut TokenDeltaMap,
    pos_buffer: &[usize]
) {
    let len = token_sequence.len();
    let (rtok_1, rtok_2) = removal_pair;

    for &pos in pos_buffer {
        if pos > 0 {
            let (tok_1, tok_2) = (token_sequence[pos - 1], token_sequence[pos]);
            if tok_1 != tok_2 {
                *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
                *pair_count.entry((tok_1, rtok_1)).or_insert(0) -= 1;
            }
        }
        if pos + 1 < len {
            let (tok_1, tok_2) = (token_sequence[pos], token_sequence[pos + 1]);
            if tok_2 == tok_1 {
                *pair_count.entry((rtok_2, rtok_1)).or_insert(0) -= 1;
            } else {
                *pair_count.entry((rtok_2, tok_2)).or_insert(0) -= 1;
            }
            *pair_count.entry((tok_1, tok_2)).or_insert(0) += 1;
        }
    }
}

pub fn update_thread(
    arena: &mut [&mut [TokenId]],
    lens: &mut [usize],
    change_buf: &mut [usize],
    pair_detla: &mut TokenDeltaMap,
    removal_pair: TokenPair,
    new_token_id: TokenId
) {
    pair_detla.clear();
    for (token_sequence, len) in zip(arena, lens) {
        let buf_len = update_sequence_discrete(removal_pair, new_token_id, token_sequence, len, change_buf);
        if buf_len != 0 {
            let tok_seq = &token_sequence[..*len];
            let pos_buf = &change_buf[..buf_len];
            update_delta_aggregate(tok_seq, removal_pair, pair_detla, pos_buf);
        }
    }
}

pub fn create_thread_arenas<'a>(arena: &'a mut [&'a mut [TokenId]], max_len: usize) -> Vec<ThreadTokenArena<'a>> {
    let mut remaining = arena;
    let mut arenas: Vec<ThreadTokenArena> = Vec::new();

    while remaining.len() != 0 {
        let remaining_len = remaining.len();
        let mut split_dist = 0;
        let mut running_len = 0;
        while running_len < max_len && split_dist < remaining_len {
            running_len += remaining[split_dist].len();
            split_dist += 1;
        }
        let (thread_arena, rest) = remaining.split_at_mut(split_dist);
        let mut max_token_sequence_len = thread_arena[0].len();
        let thread_arena_lens = thread_arena.iter()
            .map(|token_sequence| {
                let len = token_sequence.len();
                max_token_sequence_len = max_token_sequence_len.max(len);
                len
            }).collect();
        let mut change_buf = Vec::new();
        change_buf.resize(max_token_sequence_len / 2, 0);
        arenas.push(ThreadTokenArena { arena: thread_arena, lens: thread_arena_lens, change_buf:  change_buf, pair_delta: HashMap::new()});
        remaining = rest;
    }

    arenas
}

pub fn calculate_chunk_count(size: usize) -> usize {
    assert!(size != 0);
    static CONTROL_FACTOR: f64 = 3.5;
    static CONTROL_BIAS: f64 = -50.0;
    static SHIFT_BIAS: usize = 1;

    let log_factor = (size as f64).log2();
    let chunk_count = 0.max((log_factor * CONTROL_FACTOR + CONTROL_BIAS).floor() as usize);
    chunk_count + SHIFT_BIAS
}

// pub fn calculate_chunk_count(size: usize) -> usize {
//     static CONTROL_FACTOR: f64 = 50.0;

//     let sqrt_factor = (size as f64).sqrt();
//     let chunk_count = (sqrt_factor / CONTROL_FACTOR) as usize;
//     chunk_count.max(1)
// }