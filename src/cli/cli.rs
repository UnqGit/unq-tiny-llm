use crate::config::Config;
use crate::cli::handle::handle_args;

pub fn run(args: &[String], config: &Config) -> Result<(), String> {
    if let Err(err) = handle_args(args, config) {
        return Err(err);
    }
    Ok(())
}