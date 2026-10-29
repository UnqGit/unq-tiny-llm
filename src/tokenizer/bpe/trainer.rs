use std::path::Path;

use crate::tokenizer::bpe::serialization::save_tokenizer;
use crate::tokenizer::bpe::training::bpe_training;

pub fn train_and_save(
    text: &str,
    vocab_size: usize,
    path: impl AsRef<Path>
) -> Result<(), String> {

    let tokenizer = bpe_training(text, vocab_size)?;

    save_tokenizer(&tokenizer, path)

}