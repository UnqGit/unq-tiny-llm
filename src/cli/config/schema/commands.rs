use crate::cli::types::CommandSpec;
use crate::cli::config::schema::{TOKENIZER_ARGUMENTS, ENCODER_ARGUMENTS, DECODER_ARGUMENTS};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        command: "-tokenizer",
        arguments: Some(TOKENIZER_ARGUMENTS),
    },
    CommandSpec {
        command: "-encoder",
        arguments: Some(ENCODER_ARGUMENTS),
    },
    CommandSpec {
        command: "-decoder",
        arguments: Some(DECODER_ARGUMENTS),
    },
];