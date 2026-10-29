use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::tokenizer::bpe::types::BpeTokenizer;

pub fn save_tokenizer(tokenizer: &BpeTokenizer, path: impl AsRef<Path>) -> Result<(), String> {
    let file = File::create(path)
        .map_err(|e| format!("Failed to create tokenizer file: {e}"))?;

    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, tokenizer)
        .map_err(|e| format!("Failed to serialize tokenizer: {e}"))?;

    Ok(())
}

pub fn load_tokenizer(path: impl AsRef<Path>) -> Result<BpeTokenizer, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open the tokenizer file: {e}"))?;

    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to deserialize tokenizer: {e}"))
}