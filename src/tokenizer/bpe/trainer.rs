use std::cmp::min;
use std::path::Path;

use crate::config::Config;

use crate::loader::load_data;

use crate::{log_error, log_info, log_success, log_warning};
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
            log_error!("{}", err);
            return;
        }
    };

    let content = data.contents.join("\n\0\n");
    let error = data.errors.join("\n");
    let content_len = content.len();

    if !error.is_empty() {
        log_warning!("Error(s) occured during loading data:\n{error}");
    }

    if content_len == 0 {
        log_error!("Could not run tokenizer with empty dataset.");
        return;
    }

    let max_bytes = min(1000, content_len);
    let boundary = content.floor_char_boundary(max_bytes);
    log_info!("Size of the content: {content_len}");
    log_info!("Preview of the text:\n\n{}...\n", &content[..boundary]);

    let pretokens = preprocessing::preprocess_text(&content);

    log_info!(
        "Preview of pretokenized string: {:#?}\n...\n",
        pretokens
            .chunks.iter().take(10)
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
        Err(err) => log_error!("Error occured during tokenization of data:\n{err}"),
        Ok(tok) => {
            log_success!("Tokenizer training successfully completed and data is succesfully saved.");
            log_info!("Total number of learnt merges: {}", tok.merges.len());
            log_info!("Few merged tokens:\n{:#?}...",tok.merges.iter().take(5).collect::<Vec<_>>());
        }
    }
}