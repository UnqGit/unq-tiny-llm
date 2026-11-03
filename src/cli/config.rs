use crate::cli::types::{Value, ArgValue, ParsedCommand};
use crate::config::Config;

pub mod schema {
    use crate::cli::types::{ArgConfig, CommandSpec, ValueType};

    pub static PREPROCESSING_ARGUMENTS: &[ArgConfig] = &[
        ArgConfig { config: "--vocab-size",     required: true,  value: Some(ValueType::UInt) },
        ArgConfig { config: "--dataset",        required: false, value: Some(ValueType::Path) },
        ArgConfig { config: "--min-pair-freq",  required: false, value: Some(ValueType::UInt) },
    ];
    
    pub static COMMANDS: &[CommandSpec] = &[
        CommandSpec { command: "-pre-processing", arguments: Some(PREPROCESSING_ARGUMENTS) },
    ];
}

fn get_some_argval(arg: &ArgValue) -> Result<Value, String> {
    match &arg.value {
        Some(value) => Ok(value.clone()),
        _ => Err(format!("Expected a value in argument `ArgValue`, got None."))
    }
}

fn unexpected_type_err(config_name: &str, expected_type: &str) -> String {
    format!("Program error: unexpected value received for `{}`, expcted `<{}>`", config_name, expected_type)
}

fn update_preprocessing(config: &mut Config, update_args: &Vec<ArgValue>) -> Result<(), String> {
    for arg in update_args {
        match arg.argument.as_str() {
            "--vocab-size" => match get_some_argval(arg)? {
                Value::UInt(value) => config.pre_processing.vocab_size = value,
                _ => return Err(unexpected_type_err(&arg.argument, "unsigned integer")),
            },
            "--dataset" => match get_some_argval(arg)? {
                Value::Path(path) => config.paths.training_dataset = path,
                _ => return Err(unexpected_type_err(&arg.argument, "path")),
            },
            "--min-pair-freq" => match get_some_argval(arg)? {
                Value::UInt(value) => config.pre_processing.min_pair_freq = value,
                _ => return Err(unexpected_type_err(&arg.argument, "unsigned integer")),
            }
            _ => return Err(format!("Configuration `{}` not implemented for command `-pre-processing`", arg.argument)),
        }
    }
    Ok(())
}

pub fn update_config(config: &mut Config, update_values: &ParsedCommand) -> Result<(), String> {
    match update_values.command.as_str() {
        "-pre-processing" => update_preprocessing(config, &update_values.updated_args),
        _ => Err(format!("Command: {} not implemented.", update_values.command)),
    }
}