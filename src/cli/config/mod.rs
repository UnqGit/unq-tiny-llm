pub mod schema;
mod internal;
mod update;

use crate::cli::types::ParsedCommand;
use crate::config::Config;

pub fn update_config(config: &mut Config, update_values: &ParsedCommand) -> Result<(), String> {
    match update_values.command.as_str() {
        "-tokenizer" => update::tokenizer(config, &update_values.updated_args),
        "-encoder" => update::encoder(config, &update_values.updated_args),
        _ => Err(format!("Command: {} not implemented.", update_values.command)),
    }
}