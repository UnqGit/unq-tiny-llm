mod internal {
    pub use crate::tokenizer::bpe::trainer::run_tokenizer;
    pub use crate::tokenizer::decoding::run_decoder;
    pub use crate::tokenizer::encoding::run_encoder;
}

use crate::{config::Config, log_warning};

pub fn run_command(config: &Config, command: &str) {
    match command {
        "-tokenizer" => internal::run_tokenizer(config),
        "-encoder"   => internal::run_encoder(config),
        "-decoder"   => internal::run_decoder(config),
        _            => log_warning!("Command: `{command}` is not implemented"),
    }
}
