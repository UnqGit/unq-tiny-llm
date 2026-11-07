use std::{cmp, path::Path};

use crate::{
    config::Config, loader::load_data, tokenizer::{
        serialization::save_encoded, bpe::{
            serialization::load_tokenizer,
            training::min_alloc::update_sequence,
            types::{
                BpeTokenizer,
                TokenId,
            },
        }, preprocessing::normalize_text,
    }
};

pub fn encode(text: &str, tokenizer: &BpeTokenizer) -> Vec<TokenId> {
    let normalized_text = normalize_text(text);
    let mut tokens = normalized_text.as_bytes().iter()
        .map(|&c| c as TokenId).collect::<Vec<_>>();
    let mut len = tokens.len();
    let merge_seq = &tokenizer.merges;
    
    for (i, merge) in merge_seq.iter().enumerate() {
        let pair = (merge.left, merge.right);
        let new_id = i + 256; // 256 accounting for the initial vocab length due to filling up using all byte combinations.

        update_sequence(pair, new_id, &mut tokens, &mut len);
    }

    tokens.truncate(len);
    tokens
}

pub fn encode_and_save(text: &str, tokenizer: &BpeTokenizer, path: impl AsRef<Path>) -> Result<Vec<TokenId>, String> {
    let res = encode(text, tokenizer);
    save_encoded(path, &res)?;
    Ok(res)
}

pub fn run_encoder(config: &Config) {
    let data = match load_data(&config.paths.dataset) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{err}");
            return;
        },
    };

    let tokenizer = match load_tokenizer(&config.paths.tokenizer) {
        Ok(tok) => tok,
        Err(err) => {
            println!("Could not load the tokenizer from memory from path `{}`: {:?}", config.paths.tokenizer.display(), err);
            return;
        }
    };
    
    let content = data.contents.join("\n\0\n");
    let error = data.errors.join("\n");

    if !error.is_empty() {
        eprintln!("Error(s) occured during loading data:\n{}", error);
    }

    if content.is_empty() {
        eprintln!("Could not run encoder with empty dataset.");
        return;
    }

    let max_bytes = cmp::min(1000, content.len());
    let boundary = content.floor_char_boundary(max_bytes);
    println!("Size of the content: {}\nPreview of the text:\n\n{}...\n", content.len(), &content[..boundary]);

    let result = encode_and_save(&content, &tokenizer, &config.paths.encoder);
    match result {
        Err(err) => eprintln!("Error occured during tokenization of data:\n{}", err),
        Ok(v) => {
            println!("Encoding of text successfully completed and data is succesfully saved.");
            println!("Few tokens:\n{:#?}...", v.iter().take(15).collect::<Vec<_>>());
        },
    }
}