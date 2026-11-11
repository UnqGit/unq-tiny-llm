mod cli;
mod config;
mod loader;
mod tokenizer;
mod logger;

use cli::{commands::run_command, config::update_config, handle::parse_arguments};
use config::Config;
use std::{env, path::Path, process};

fn main() {
    let config_path: &Path = Path::new("config/default.toml");
    let mut config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            log_error!("Could not load configs: \n\n{e}");
            process::exit(1);
        }
    };
    log_info!("Default configurations: \n\n{config:#?}\n");

    let args: Vec<String> = env::args().skip(1).collect();
    log_info!("Passed arguments: {args:#?}\n");

    let result = parse_arguments(&args);
    match result {
        Err(err) => {
            log_error!("Error occured during parsing: \n\n{err}");
            process::exit(1);
        }
        Ok(res) => {
            log_info!("Parsed command: \n\n{res:#?}\n");
            if let Err(err) = update_config(&mut config, &res) {
                log_error!("Could not update the configs: \n\n{err}");
                process::exit(1);
            }
        }
    }

    log_info!("Final configurations: \n\n{config:#?}\n");
    print_colored!(logger::GREEN, "==================================Running commands==================================\n\n");

    run_command(&config, &args[0]);
}
