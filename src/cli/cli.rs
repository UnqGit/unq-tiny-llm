use crate::config::Config;
use crate::cli::handle::handle_args;

pub fn run(args: &[String], config: &mut Config) -> Result<(), String> {
    handle_args(args, config)
}