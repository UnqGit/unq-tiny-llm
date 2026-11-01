use crate::config::Config;
use crate::cli::handle::handle_args;

pub fn run(args: &[String], _config: &Config) -> Result<(), String> {
    if let Err(err) = handle_args(args) {
        return Err(err);
    }
    Ok(())
}