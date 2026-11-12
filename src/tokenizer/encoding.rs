use std::{cmp, path::Path};

use crate::{
    config::Config, loader::load_data, log_error, log_info, log_success, log_warning, logger::LoadingBar, tokenizer::{
        bpe::{
            serialization::load_tokenizer,
            training::optimized::update_sequence,
            types::{BpeTokenizer, TokenId},
        },
        preprocessing::normalize_text,
        serialization::save_encoded,
    },
};

pub fn encode(text: &str, tokenizer: &BpeTokenizer) -> Vec<TokenId> {
    log_info!("Started encoding text");
    let normalized_text = normalize_text(text);
    let mut tokens = normalized_text
        .as_bytes()
        .iter()
        .map(|&c| c as TokenId)
        .collect::<Vec<_>>();

    let mut len = tokens.len();
    let merge_seq = &tokenizer.merges;
    let merges_len = merge_seq.len();
    let mut bar = LoadingBar::create(merges_len, 40);

    log_info!("Total number of merges to perform: {merges_len}");

    for (i, merge) in merge_seq.iter().enumerate() {
        let pair = (merge.left, merge.right);
        let new_id = i + 256; // 256 accounting for the initial vocab length due to filling up using all byte combinations.

        bar.show(i);
        update_sequence(pair, new_id, &mut tokens, &mut len);
    }

    bar.clear();
    tokens.truncate(len);
    tokens
}

pub fn encode_and_save(
    text: &str,
    tokenizer: &BpeTokenizer,
    path: impl AsRef<Path>,
) -> Result<Vec<TokenId>, String> {
    let res = encode(text, tokenizer);
    save_encoded(path, &res)?;
    Ok(res)
}

pub fn run_encoder(config: &Config) {
    log_info!("Loading data for encoding.");
    let data = match load_data(&config.paths.dataset) {
        Ok(data) => data,
        Err(err) => {
            log_error!("{err}");
            return;
        }
    };

    log_info!("Loading tokenizer to encode with.");
    let tokenizer = match load_tokenizer(&config.paths.tokenizer) {
        Ok(tok) => tok,
        Err(err) => {
            log_error!(
                "Could not load the tokenizer from memory from path `{}`: {:?}",
                config.paths.tokenizer.display(),
                err
            );
            return;
        }
    };

    let content = data.contents.join("\n\0\n");
    let error = data.errors.join("\n");

    if !error.is_empty() {
        log_warning!("Error(s) occured during loading data:\n{error}");
    }

    if content.is_empty() {
        log_error!("Could not run encoder with empty dataset.");
        return;
    }

    let content_len = content.len();
    let max_bytes = cmp::min(1000, content_len);
    let boundary = content.floor_char_boundary(max_bytes);
    log_info!(
        "Size of the content: {}\nPreview of the text:\n\n{}...\n",
        content_len,
        &content[..boundary]
    );

    let result = encode_and_save(&content, &tokenizer, &config.paths.encoder);
    match result {
        Err(err) => log_error!("Error occured during tokenization of data:\n{err}"),
        Ok(v) => {
            log_success!("Encoding of text successfully completed and data is succesfully saved.");
            log_info!(
                "Few tokens:\n{:#?}...",
                v.iter().take(15).collect::<Vec<_>>()
            );
            log_info!("Encoded token count: {}", v.len());
        }
    }
}
