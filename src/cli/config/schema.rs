use crate::cli::types::{ArgConfig, CommandSpec, ValueType};

pub static TOKENIZER_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig { config: "--vocab-size",     required: false, value: Some(ValueType::UInt) },
    ArgConfig { config: "--dataset",        required: false, value: Some(ValueType::Path) },
    ArgConfig { config: "--min-pair-freq",  required: false, value: Some(ValueType::UInt) },
    ArgConfig { config: "--save",           required: false, value: Some(ValueType::Path) },
];

pub static ENCODER_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig { config: "--tokenizer", required: false, value: Some(ValueType::Path) },
    ArgConfig { config: "--dataset",   required: false, value: Some(ValueType::Path) },
    ArgConfig { config: "--save",      required: false, value: Some(ValueType::Path) },
];


pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec { command: "-tokenizer", arguments: Some(TOKENIZER_ARGUMENTS) },
    CommandSpec { command: "-encoder",   arguments: Some(ENCODER_ARGUMENTS) },
];