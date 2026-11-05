use crate::tokenizer::bpe::types::*;
use crate::tokenizer::bpe::training::internal::*;

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

#[allow(unused)]
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