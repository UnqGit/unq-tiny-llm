use std::path::Path;

use crate::tokenizer::bpe::serialization::save_tokenizer;
use crate::tokenizer::bpe::training::naive::bpe_training;

pub fn train_and_save(
    text: &str,
    vocab_size: usize,
    min_pair_freq: usize,
    path: impl AsRef<Path>
) -> Result<(), String> {

    let tokenizer = bpe_training(text, vocab_size, min_pair_freq)?;

    save_tokenizer(&tokenizer, path)

}