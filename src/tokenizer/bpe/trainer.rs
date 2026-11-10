use std::cmp::min;
use std::path::Path;

use crate::config::Config;

use crate::loader::load_data;

use crate::tokenizer::bpe::serialization::save_tokenizer;
use crate::tokenizer::bpe::training;
use crate::tokenizer::bpe::types::BpeTokenizer;
use crate::tokenizer::preprocessing;

fn train_and_save(
    pretokens: &preprocessing::PreTokens,
    vocab_size: usize,
    min_pair_freq: usize,
    path: impl AsRef<Path>,
) -> Result<BpeTokenizer, String> {
    let tokenizer = training::optimized::bpe_training(pretokens, vocab_size, min_pair_freq)?;

    debug_assert!({
        let res = training::naive::bpe_training(pretokens, vocab_size, min_pair_freq)?;

        assert!(
            tokenizer == res,
            "Optimized tokenizer differed from the naive implementation:\n\
            Optimized:\n{tokenizer:#?}\n\
            Naive:\n{res:#?}"
        );

        true
    });
    save_tokenizer(&tokenizer, path)?;
    Ok(tokenizer)
}

pub fn run_tokenizer(config: &Config) {
    let data = match load_data(&config.paths.dataset) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let content = data.contents.join("\n\0\n");
    let error = data.errors.join("\n");

    if !error.is_empty() {
        eprintln!("Error(s) occured during loading data:\n{error}");
    }

    if content.is_empty() {
        eprintln!("Could not run tokenizer with empty dataset.");
        return;
    }

    let max_bytes = min(1000, content.len());
    let boundary = content.floor_char_boundary(max_bytes);
    println!(
        "Size of the content: {}\nPreview of the text:\n\n{}...\n",
        content.len(),
        &content[..boundary]
    );

    let pretokens = preprocessing::preprocess_text(&content);

    println!(
        "Pretokenized string: {:#?}\n...\n",
        pretokens
            .chunks
            .iter()
            .take(10)
            .map(|r| &pretokens.text[r.clone()])
            .collect::<Vec<_>>()
    );

    let result = train_and_save(
        &pretokens,
        config.tokenizer.vocab_size,
        config.tokenizer.min_pair_freq,
        &config.paths.tokenizer,
    );
    match result {
        Err(err) => eprintln!("Error occured during tokenization of data:\n{err}"),
        Ok(tok) => {
            println!("Tokenizer training successfully completed and data is succesfully saved.");
            println!("Total number of learnt merges: {}", tok.merges.len());
            println!(
                "Few merged tokens:\n{:#?}...",
                tok.merges.iter().take(5).collect::<Vec<_>>()
            );
        }
    }
}
