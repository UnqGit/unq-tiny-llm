use crate::cli::config::internal::{match_path, match_uint, unimplemented_config};
use crate::cli::types::{ArgValue};
use crate::config::Config;

pub fn tokenizer(config: &mut Config, update_args: &Vec<ArgValue>) -> Result<(), String> {
    for arg in update_args {
        match arg.argument.as_str() {
            "--vocab-size"    => match_uint(arg, &mut config.tokenizer.vocab_size)?,
            "--dataset"       => match_path(arg, &mut config.paths.dataset)?,
            "--min-pair-freq" => match_uint(arg, &mut config.tokenizer.min_pair_freq)?,
            "--save"          => match_path(arg, &mut config.paths.tokenizer)?,
            _                 => unimplemented_config(&arg.argument, "-tokenizer")?,
        }
    }
    Ok(())
}

pub fn encoder(config: &mut Config, update_args: &Vec<ArgValue>) -> Result<(), String> {
    for arg in update_args {
        match arg.argument.as_str() {
            "--dataset"   => match_path(arg, &mut config.paths.dataset)?,
            "--tokenizer" => match_path(arg, &mut config.paths.tokenizer)?,
            "--save"      => match_path(arg, &mut config.paths.encoder)?,
            _             => unimplemented_config(&arg.argument, "-encoder")?,
        }
    }
    Ok(())
}

pub fn decoder(config: &mut Config, update_args: &Vec<ArgValue>) -> Result<(), String> {
    for arg in update_args {
        match arg.argument.as_str() {
            "--encoded"   => match_path(arg, &mut config.paths.encoder)?,
            "--save"      => match_path(arg, &mut config.paths.decoder)?,
            "--tokenizer" => match_path(arg, &mut config.paths.tokenizer)?,
            "--as-bytes"  => config.decoder.save_as_bytes = true,
            _             => unimplemented_config(&arg.argument, "-decoder")?,
        }
    }
    Ok(())
}
