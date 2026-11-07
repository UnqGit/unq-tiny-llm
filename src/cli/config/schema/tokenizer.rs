use crate::cli::types::{ArgConfig, ValueType};

pub static TOKENIZER_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig {
        config: "--vocab-size",
        required: false,
        value: Some(ValueType::UInt),
    },
    ArgConfig {
        config: "--dataset",
        required: false,
        value: Some(ValueType::Path),
    },
    ArgConfig {
        config: "--min-pair-freq",
        required: false,
        value: Some(ValueType::UInt),
    },
    ArgConfig {
        config: "--save",
        required: false,
        value: Some(ValueType::Path),
    },
];