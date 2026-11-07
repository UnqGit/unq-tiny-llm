use crate::cli::types::{ArgConfig, ValueType};

pub static DECODER_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig {
        config: "--as-bytes",
        required: false,
        value: None,
    },
    ArgConfig {
        config: "--encoded",
        required: false,
        value: Some(ValueType::Path),
    },
    ArgConfig {
        config: "--tokenizer",
        required: false,
        value: Some(ValueType::Path),
    },
    ArgConfig {
        config: "--save",
        required: false,
        value: Some(ValueType::Path),
    },
];