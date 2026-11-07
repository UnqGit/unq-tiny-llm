use std::path::PathBuf;

use crate::cli::types::{ArgValue, Value};

fn get_some_argval(arg: &ArgValue) -> Result<Value, String> {
    match &arg.value {
        Some(value) => Ok(value.clone()),
        _ => Err("Expected a value in argument `ArgValue`, got None.".to_string()),
    }
}

fn unexpected_type_err(config_name: &str, expected_type: &str) -> Result<(), String> {
    Err(format!(
        "Program error: unexpected value received for `{config_name}`, expcted `<{expected_type}>`"
    ))
}

pub fn match_uint(arg: &ArgValue, val: &mut usize) -> Result<(), String> {
    match get_some_argval(arg)? {
        Value::UInt(value) => *val = value,
        _ => unexpected_type_err(&arg.argument, "unsigned integer")?,
    }
    Ok(())
}

pub fn match_path(arg: &ArgValue, val: &mut PathBuf) -> Result<(), String> {
    match get_some_argval(arg)? {
        Value::Path(value) => *val = value,
        _ => unexpected_type_err(&arg.argument, "path")?,
    }
    Ok(())
}

pub fn unimplemented_config(arg: &str, cmd: &str) -> Result<(), String> {
    Err(format!(
        "Configuration `{arg}` not implemented for command `{cmd}`",
    ))
}