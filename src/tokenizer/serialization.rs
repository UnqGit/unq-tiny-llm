use std::{fs::File, io::{BufReader, BufWriter}, path::Path};

use crate::tokenizer::bpe::types::TokenId;

pub fn save_encoded(path: impl AsRef<Path>, encoded: &Vec<TokenId>) -> Result<(), String> {
    let file = File::create(path)
        .map_err(|e| format!("Failed to create encoder file: {e}"))?;
    
    let writer = BufWriter::new(file);
    
    serde_json::to_writer(writer, &encoded)
        .map_err(|e| format!("Failed to serialize encoded tokens: {e}"))?;

    Ok(())
}

pub fn load_encoded(path: impl AsRef<Path>) -> Result<Vec<TokenId>, String> {
    let file = File::open(&path)
        .map_err(|e| format!("Could not open encoded file path `{}`.\n{}", &path.as_ref().display(), e))?;

    let reader = BufReader::new(file);

    let tokens = serde_json::from_reader(reader)
        .map_err(|e| format!("Could not read encoded token sequence from file `{}`.\n{}", &path.as_ref().display(), e))?;

    Ok(tokens)
}