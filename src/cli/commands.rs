mod internal {
    pub use crate::tokenizer::bpe::trainer::run_tokenizer;
    pub use crate::tokenizer::encoding::run_encoder;
}

use crate::config::Config;

pub fn run_command(config: &Config, command: &str) {
    match command {
        "-tokenizer" => internal::run_tokenizer(config),
        "-encoder" => internal::run_encoder(config),
        _ => eprintln!("Command: `{}` is not implemented", command),
    }
}