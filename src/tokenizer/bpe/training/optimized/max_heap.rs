use std::{
    cmp::Reverse, collections::BinaryHeap,
};
use rayon::prelude::*;

use crate::tokenizer::{
    bpe::{
        training::{
            internal::{self, TokenFreqMap, TokenPair}, optimized::internal::{ThreadTokenArena, TokenDeltaMap, calculate_chunk_count, create_thread_arenas, update_thread},
        }, types::{
            BpeTokenizer, Merge, TokenId,
        }
    }, initial::{self, TokenArena}, preprocessing::PreTokens
};

fn update_freq_map(freq_map: &mut TokenFreqMap, delta_map: &TokenDeltaMap, token_queue: &mut BinaryHeap<(usize, Reverse<TokenPair>)>) -> Result<(), String> {
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

fn combine_delta_maps(cummulative_map: &mut TokenDeltaMap, delta_map: &TokenDeltaMap) {
    for (&pair, &delta) in delta_map {
        *cummulative_map.entry(pair).or_insert(0) += delta;
    }
}

fn update(arenas: &mut [ThreadTokenArena], freq_map: &mut TokenFreqMap, token_queue: &mut BinaryHeap<(usize, Reverse<TokenPair>)>, removal_pair: TokenPair, new_token_id: TokenId) -> Result<(), String> {
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

fn create_priority_queue(freq_map: &TokenFreqMap) -> BinaryHeap<(usize, Reverse<TokenPair>)> {
    freq_map
        .iter()
        .map(|(&k, &v)| (v, Reverse(k)))
        .collect()
}

fn get_most_freq_pair(freq_map: &TokenFreqMap, token_queue: &mut BinaryHeap<(usize, Reverse<TokenPair>)>, min_pair_freq: usize) -> Option<TokenPair> {
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