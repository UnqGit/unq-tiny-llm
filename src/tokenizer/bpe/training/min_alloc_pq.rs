use std::{
    cmp::{self, Reverse}, collections::{BinaryHeap, HashMap}, iter::zip,
};
use rayon::prelude::*;

use crate::tokenizer::{
    bpe::{
        training::internal,
        types::{
            BpeTokenizer, Merge, TokenId,
        }
    }, initial::{self, TokenArena}, preprocessing::PreTokens
};

type TokenPair = (TokenId, TokenId);

#[derive(Debug)]
struct ThreadTokenArena<'a> {
    arena: &'a mut [&'a mut[TokenId]],
    lens: Vec<usize>,
    change_buf: Vec<usize>,
    pair_delta: HashMap<TokenPair, isize>,
}

fn update_sequence(
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

fn update_pair_count(
    token_sequence: &[TokenId],
    removal_pair: TokenPair,
    pair_count: &mut HashMap<TokenPair, isize>,
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

fn update_thread(
    arena: &mut [&mut [TokenId]],
    lens: &mut [usize],
    change_buf: &mut [usize],
    pair_detla: &mut HashMap<TokenPair, isize>,
    removal_pair: TokenPair,
    new_token_id: TokenId
) {
    pair_detla.clear();
    for (token_sequence, len) in zip(arena, lens) {
        let buf_len = update_sequence(removal_pair, new_token_id, token_sequence, len, change_buf);
        if buf_len != 0 {
            let tok_seq = &token_sequence[..*len];
            let pos_buf = &change_buf[..buf_len];
            update_pair_count(tok_seq, removal_pair, pair_detla, pos_buf);
        }
    }
}

fn update_freq_map(freq_map: &mut HashMap<TokenPair, usize>, delta_map: &HashMap<TokenPair, isize>, token_queue: &mut BinaryHeap<(usize, Reverse<TokenPair>)>) -> Result<(), String> {
    for (&pair, &delta) in delta_map {
        if delta < 0 && freq_map.get(&pair).is_none() {
            return Err(format!("Program Error: pair {pair:?} not found in the map, frequency of which is to be decremented by {delta}"))
        }
        let entry = freq_map.entry(pair).or_insert(0);
        if delta < 0 {
            let neg_delta = (-delta) as usize;
            if *entry < neg_delta {
                return Err(format!("Program Error: pair {pair:?} is to be decremented by frequence {neg_delta}, but the frequency is {}.", *entry));
            }
            *entry -= neg_delta;
        } else {
            *entry += delta as usize;
        }

        let new_freq = *entry;
        if new_freq == 0usize {
            freq_map.remove(&pair);
        } else {
            token_queue.push((new_freq, Reverse(pair)));
        }
    }
    Ok(())
}

fn combine_delta_maps(cummulative_map: &mut HashMap<TokenPair, isize>, delta_map: &HashMap<TokenPair, isize>) {
    for (&pair, &delta) in delta_map {
        *cummulative_map.entry(pair).or_insert(0) += delta;
    }
}

fn update(arenas: &mut [ThreadTokenArena], freq_map: &mut HashMap<TokenPair, usize>, token_queue: &mut BinaryHeap<(usize, Reverse<TokenPair>)>, removal_pair: TokenPair, new_token_id: TokenId) -> Result<(), String> {
    arenas
        .into_par_iter()
        .for_each(|arena| {
            update_thread(arena.arena, &mut arena.lens, &mut arena.change_buf, &mut arena.pair_delta, removal_pair, new_token_id)
        });
    let (cummulative_map, rest) = arenas.split_at_mut(1);
    for arena in rest {
        combine_delta_maps(&mut cummulative_map[0].pair_delta, &arena.pair_delta);
    }
    update_freq_map(freq_map, &cummulative_map[0].pair_delta, token_queue)?;
    freq_map.remove(&removal_pair);
    Ok(())
}

fn create_thread_arenas<'a>(arena: &'a mut [&'a mut [TokenId]], max_len: usize) -> Vec<ThreadTokenArena<'a>> {
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
                max_token_sequence_len = cmp::max(max_token_sequence_len, len);
                len
            }).collect();
        let mut change_buf = Vec::new();
        change_buf.resize(max_token_sequence_len / 2, 0);
        arenas.push(ThreadTokenArena { arena: thread_arena, lens: thread_arena_lens, change_buf:  change_buf, pair_delta: HashMap::new()});
        remaining = rest;
    }

    arenas
}

fn calculate_chunk_count(size: usize) -> usize {
    assert!(size != 0);
    static CONTROL_FACTOR: f64 = 3.5;
    static CONTROL_BIAS: f64 = -50.0;
    static SHIFT_BIAS: usize = 1;

    let log_factor = (size as f64).log2();
    let chunk_count = cmp::max(0, (log_factor * CONTROL_FACTOR + CONTROL_BIAS).floor() as usize);
    chunk_count + SHIFT_BIAS
}

fn create_priority_queue(freq_map: &HashMap<TokenPair, usize>) -> BinaryHeap<(usize, Reverse<TokenPair>)> {
    freq_map
        .iter()
        .map(|(&k, &v)| (v, Reverse(k)))
        .collect()
}

fn get_most_freq_pair(freq_map: &HashMap<TokenPair, usize>, token_queue: &mut BinaryHeap<(usize, Reverse<TokenPair>)>, min_pair_freq: usize) -> Option<TokenPair> {
    while let Some((freq, rev_token)) = token_queue.pop() {
        if freq < min_pair_freq {
            return None;
        }
        if let Some(&curr_freq) = freq_map.get(&(rev_token.0)) {
            if curr_freq == freq {
                return Some(rev_token.0);
            }
        }
    }
    None
}

pub fn bpe_training(pretokens: &PreTokens, vocab_size: usize, min_pair_freq: usize) -> Result<BpeTokenizer, String> {
    let total_token_count = pretokens.text.len();

    let mut curr_vocab_size = 256;
    if vocab_size < curr_vocab_size {
        return Err(format!(
            "Initial vocab of lone letters of size {curr_vocab_size} exceed the desired `vocab_size` of {vocab_size}"
        ));
    }
    
    let mut initial_tokens: TokenArena = initial::initial_tokens(pretokens);

    let target_chunk_count = calculate_chunk_count(total_token_count);
    let target_count = total_token_count.div_ceil(target_chunk_count);

    let mut arena = initial::build_initial_token_sequence(&mut initial_tokens);
    let arena_len = arena.len();
    
    let mut merges = Vec::new();
    if arena_len == 0 {
        return Ok(BpeTokenizer { merges });
    }

    let mut pair_count = internal::count_pairs(&arena);
    let mut token_queue = create_priority_queue(&pair_count);
    let mut arenas = create_thread_arenas(&mut arena, target_count);
    
    while curr_vocab_size < vocab_size {
        let Some(best_pair) = get_most_freq_pair(&pair_count, &mut token_queue, min_pair_freq) else { break; };
        update(&mut arenas, &mut pair_count, &mut token_queue, best_pair, curr_vocab_size)?;
        curr_vocab_size += 1;
        merges.push(Merge {
            left: best_pair.0,
            right: best_pair.1
        });
    }

    Ok(BpeTokenizer { merges })
}