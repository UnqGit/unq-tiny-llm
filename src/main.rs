mod cli;
mod config;
mod loader;

use config::Config;
use cli::{handle::parse_arguments, config::update_config};
use std::{env, process, path::Path};

fn main() {
    let config_path: &Path = Path::new("config/default.toml");
    let mut config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not load configs: \n\n{e}");
            process::exit(1);
        }
    };
    println!("Default configurations: \n\n{:#?}\n", config);

    let args: Vec<String> = env::args().skip(1).collect();
    println!("Passed arguments: {:#?}\n", args);
    
    let result = parse_arguments(&args);
    match result {
        Err(err) => {
            eprintln!("Could not run: \n\n{err}");
            process::exit(1);
        },
        Ok(res) => {
            println!("Parsed command: \n\n{:#?}\n", res);
            match update_config(&mut config, &res) {
                Err(err) => {
                    eprintln!("Could not update the configs: \n\n{err}");
                    process::exit(1);
                },
                Ok(_) => (),
            }
        }
    }


    println!("Final configurations: \n\n{:#?}\n", config);
}