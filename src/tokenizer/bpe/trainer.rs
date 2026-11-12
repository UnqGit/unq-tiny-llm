use std::cmp::min;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::Config;

use crate::loader::load_data;

use crate::{log_error, log_info, log_success, log_warning};
use crate::tokenizer::bpe::serialization::{load_tokenizer, save_tokenizer, save_tokenizer_file};
use crate::tokenizer::bpe::training;
use crate::tokenizer::bpe::types::{BpeTokenizer, Merge};
use crate::tokenizer::preprocessing;

fn create_file_helper(path: &str) -> Option<File> {
    match File::create(path) {
        Ok(handle) => Some(handle),
        Err(err) => {
            log_warning!("Could not create/open file `{path}` due to following errors:\n{err}");
            None
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct MergeDifference {
    idx: usize,
    merge_impl: Merge,
    merge_ref: Merge,
}

struct TokenizerInfo<'a> {
    pretokens: &'a preprocessing::PreTokens,
    vocab_size: usize, 
    min_pair_freq: usize,
}

#[allow(unused)]
fn check_against_naive(tokenizer: &BpeTokenizer, args: Option<TokenizerInfo>) -> bool {
    log_info!("Checking implementation against naive implementation.");
    let naive_tokenizer = match args {
        Some(TokenizerInfo { pretokens, vocab_size, min_pair_freq }) => match training::naive::bpe_training(pretokens, vocab_size, min_pair_freq) {
            Ok(res) => res,
            Err(err) => {
                log_error!("Naive implementation failed to tokenize due to following error:\n{err}");
                return false;
            },
        },
        None => match load_tokenizer("logs/tokenizer/naive.json") {
            Ok(res) => res,
            Err(err) => {
                log_error!("{err}");
                return false;
            },
        },
    };
    log_info!("Difference will be saved in logs/tokenizer/");
    let Some(impl_file) = create_file_helper("logs/tokenizer/impl.json") else { return false; };
    if let Err(err) = save_tokenizer_file(tokenizer, &impl_file) { log_error!("{err}"); };
    if args.is_some() {
        let Some(naive_file) = create_file_helper("logs/tokenizer/naive.json") else { return false; };
        if let Err(err) = save_tokenizer_file(&naive_tokenizer, &naive_file) { log_error!("{err}"); };
    }
    let Some(difference) = create_file_helper("logs/tokenizer/difference.json") else { return false; };
    
    let diff = tokenizer.merges.iter().zip(naive_tokenizer.merges.iter())
        .enumerate()
        .filter(|(_, (ma, mb))| ma != mb)
        .map(|(i, (ma, mb))| MergeDifference { idx: i, merge_impl: *ma, merge_ref: *mb} )
        .collect::<Vec<_>>();

    let writer = BufWriter::new(difference);
    if let Err(err) = serde_json::to_writer_pretty(writer, &diff) {
        log_error!("Could not write difference to `logs/tokenizer/difference.json` due to:\n{err}");
    };

    if *tokenizer != naive_tokenizer {
        log_error!("Trained tokenizer and naive implementation do not match.");
        return false;
    }
    true
}

fn train_and_save(
    pretokens: &preprocessing::PreTokens,
    vocab_size: usize,
    min_pair_freq: usize,
    path: impl AsRef<Path>,
) -> Result<BpeTokenizer, String> {
    let tokenizer = training::optimized::bpe_training(pretokens, vocab_size, min_pair_freq)?;
    // debug_assert!(check_against_naive(&tokenizer, Some(TokenizerInfo {pretokens, vocab_size, min_pair_freq})));
    // In debug mode: comment ^ and uncomment the below line of code if running with the same dataset to not wait for naive to finish.
    // debug_assert!(check_against_naive(&tokenizer, None));
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