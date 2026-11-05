use std::fs;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub paths: PathConfig,
    pub tokenizer: TokenizerConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PathConfig {
    pub tokenizer_save: PathBuf,
    pub training_dataset:  PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct TokenizerConfig {
    pub vocab_size: usize,
    pub min_pair_freq: usize,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Could not read config file: {e}"))?;

        let config: Self = toml::from_str(&contents)
            .map_err(|e| format!("Could not read configs: {e}"))?;

        Ok(config)
    }
}