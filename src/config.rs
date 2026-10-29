use std::fs;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: PathConfig,
    pub training: TrainingConfig,
}

#[derive(Debug, Deserialize)]
pub struct PathConfig {
    pub tokenizer: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct TrainingConfig {
    pub vocab_size: usize
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