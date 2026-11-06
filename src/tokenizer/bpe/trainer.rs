use std::path::Path;
use std::cmp::min;

use crate::config::Config;

use crate::loader::load_data;

use crate::tokenizer::preprocessing;
use crate::tokenizer::bpe::training;
use crate::tokenizer::bpe::serialization::save_tokenizer;

fn train_and_save(
    text: &[&str],
    vocab_size: usize,
    min_pair_freq: usize,
    path: impl AsRef<Path>
) -> Result<(), String> {
    let tokenizer = training::naive::bpe_training(text, vocab_size, min_pair_freq)?;
    save_tokenizer(&tokenizer, path)
}

pub fn run_tokenizer(config: &Config) {
    let data = match load_data(&config.paths.training_dataset) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{err}");
            return;
        },
    };
    
    let content = data.contents.join("\n\0\n");
    let error = data.errors.join("\n");

    if !error.is_empty() {
        eprintln!("Error(s) occured during loading data:\n{}", error);
    }

    if content.is_empty() {
        eprintln!("Could not run tokenizer with empty dataset.");
        return;
    }

    let max_bytes = min(1000, content.len());
    let boundary = content.floor_char_boundary(max_bytes);
    println!("Size of the content: {}\nPreview of the text:\n\n{}...\n", content.len(), &content[..boundary]);

    let preprocessing::PreTokens { text, chunks } = preprocessing::preprocess_text(&content);
    let chunked_slices = chunks.iter().map(|r| &text[r.clone()]).collect::<Vec<_>>();

    println!("Pretokenized string: {:#?}\n...\n", chunked_slices.iter().take(10).collect::<Vec<_>>());

    let Err(err) = train_and_save(&chunked_slices, config.tokenizer.vocab_size, config.tokenizer.min_pair_freq, &config.paths.tokenizer_save)
    else {
        println!("Tokenizer training successfully completed and data is succesfully saved.");
        return;
    };
    eprintln!("Error occured during tokenization of data:\n{}", err);
}