use crate::cli::config::internal::*;
use crate::cli::types::{Value, ArgValue};
use crate::config::Config;

pub fn tokenizer(config: &mut Config, update_args: &Vec<ArgValue>) -> Result<(), String> {
    for arg in update_args {
        match arg.argument.as_str() {
            "--vocab-size" => match get_some_argval(arg)? {
                Value::UInt(value) => config.tokenizer.vocab_size = value,
                _ => return Err(unexpected_type_err(&arg.argument, "unsigned integer")),
            },
            "--dataset" => match get_some_argval(arg)? {
                Value::Path(path) => config.paths.dataset = path,
                _ => return Err(unexpected_type_err(&arg.argument, "path")),
            },
            "--min-pair-freq" => match get_some_argval(arg)? {
                Value::UInt(value) => config.tokenizer.min_pair_freq = value,
                _ => return Err(unexpected_type_err(&arg.argument, "unsigned integer")),
            },
            "--save" => match get_some_argval(arg)? {
                Value::Path(path) => config.paths.tokenizer = path,
                _ => return Err(unexpected_type_err(&arg.argument, "path")),
            },
            _ => return Err(format!("Configuration `{}` not implemented for command `-tokenizer`", arg.argument)),
        }
    }
    Ok(())
}

pub fn encoder(config: &mut Config, update_args: &Vec<ArgValue>) -> Result<(), String> {
    for arg in update_args {
        match arg.argument.as_str() {
            "--dataset" => match get_some_argval(arg)? {
                Value::Path(path) => config.paths.dataset = path,
                _ => return Err(unexpected_type_err(&arg.argument, "path")),
            },
            "--tokenizer" => match get_some_argval(arg)? {
                Value::Path(path) => config.paths.tokenizer = path,
                _ => return Err(unexpected_type_err(&arg.argument, "path")),
            },
            "--save" => match get_some_argval(arg)? {
                Value::Path(path) => config.paths.encoder = path,
                _ => return Err(unexpected_type_err(&arg.argument, "path")),
            },
            _ => return Err(format!("Configuration `{}` not implemented for command `-tokenizer`", arg.argument)),
        }
    }
    Ok(())
}