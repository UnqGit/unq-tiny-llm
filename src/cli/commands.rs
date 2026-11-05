mod internal {
    pub use crate::tokenizer::bpe::trainer::run_tokenizer;
}

use crate::config::Config;

pub fn run_command(config: &Config, command: &str) {
    match command {
        "-tokenizer" => internal::run_tokenizer(config),
        _ => eprintln!("Command: `{}` is not implemented", command),
    }
}