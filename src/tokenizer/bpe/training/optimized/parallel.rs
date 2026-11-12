use rayon::prelude::*;

use crate::{log_info, log_success, logger::LoadingBar, tokenizer::{
    bpe::{
        training::{
            initial::{self, TokenArena}, internal::{self, TokenFreqMap, TokenPair}, optimized::internal::{ThreadTokenArena, TokenDeltaMap, calculate_chunk_count, create_thread_arenas, update_thread},
        }, types::{
            BpeTokenizer, Merge, TokenId,
        },
    }, preprocessing::PreTokens
}};

fn update_freq_map(freq_map: &mut TokenFreqMap, delta_map: &TokenDeltaMap) -> Result<(), String> {
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
            *entry -= (-delta) as usize;
        } else {
            *entry += delta as usize;
        }

        if *entry == 0usize {
            freq_map.remove(&pair);
        }
    }
    Ok(())
}

fn update(arenas: &mut [ThreadTokenArena], freq_map: &mut TokenFreqMap, removal_pair: TokenPair, new_token_id: TokenId) -> Result<(), String> {
    arenas
        .into_par_iter()
        .for_each(|arena| {
            update_thread(arena.arena, &mut arena.lens, &mut arena.change_buf, &mut arena.pair_delta, removal_pair, new_token_id)
        });
    for arena in arenas {
        update_freq_map(freq_map, &arena.pair_delta)?;
    }
    freq_map.remove(&removal_pair);
    Ok(())
}

#[allow(unused)]
pub fn bpe_training(pretokens: &PreTokens, vocab_size: usize, min_pair_freq: usize) -> Result<BpeTokenizer, String> {
    log_info!("BPE Training initialized.");
    log_info!("Method of training: Parallel\n");

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
    let mut arenas = create_thread_arenas(&mut arena, target_count);
    
    let mut bar = LoadingBar::create(vocab_size, 40);
    
    while curr_vocab_size < vocab_size {
        let Some(best_pair) = internal::get_most_freq_pair(&pair_count, min_pair_freq) else { break; };
        update(&mut arenas, &mut pair_count, best_pair, curr_vocab_size)?;
        curr_vocab_size += 1;
        bar.show(curr_vocab_size);
        merges.push(Merge {
            left: best_pair.0,
            right: best_pair.1
        });
    }
    bar.clear();
    log_success!("BPE Training complete with {} merges.\n", merges.len());

    Ok(BpeTokenizer { merges })
}