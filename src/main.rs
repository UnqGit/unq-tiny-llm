mod cli;
mod config;

use config::Config;
use cli::cli::run;
use std::{env, process, path::Path};

fn main() {
    let config_path: &Path = Path::new("config/default.toml");
    let mut config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not load configs: {e}");
            process::exit(1);
        }
    };

    let args: Vec<String> = env::args().skip(1).collect();
    let result = run(&args, &mut config);
    if let Err(e) = result {
        eprintln!("Could not run the program: \n\n{e}");
        process::exit(1);
    }
}