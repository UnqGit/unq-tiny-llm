use std::{fs::File, io::{BufReader, BufWriter}, path::Path};

use crate::embedding::embedding::Embedding;
use serde_json;

pub fn save_embedding(embedding: &Embedding, path: impl AsRef<Path>) -> Result<(), String> {
    let file = File::create(path)
        .map_err(|e| format!("Could not open the file for saving the embedding:\n{e}"))?;

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, embedding)
        .map_err(|e| format!("There was an error serializing the embeddings:\n{e}"))
}

pub fn load_embedding(path: impl AsRef<Path>) -> Result<Embedding, String> {
    let file = File::open(path)
        .map_err(|e| format!("Unable to open the file to deserialize embeddings.\n{e}"))?;

    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .map_err(|e| format!("Could not deserialize the embeddings:\n{e}"))
}