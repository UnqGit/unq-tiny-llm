use crate::cli::types::{ArgConfig, ValueType};

pub static ENCODER_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig {
        config: "--tokenizer",
        required: false,
        value: Some(ValueType::Path),
    },
    ArgConfig {
        config: "--dataset",
        required: false,
        value: Some(ValueType::Path),
    },
    ArgConfig {
        config: "--save",
        required: false,
        value: Some(ValueType::Path),
    },
];