mod tokenizer;
mod cli;

use std::env;
use std::process;
use tokenizer::bpe::training::bpe_training;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: cargo run -- <text> [vocab_size]");
        process::exit(1);
    }

    let text = &args[0];

    let vocab_size: u32 = match args.get(1) {
        Some(value) => match value.parse() {
            Ok(size) => size,
            Err(_) => {
                eprintln!("Error: vocab_size must be a valid unsigned integer.");
                process::exit(1);
            }
        },
        None => 300,
    };

    match bpe_training(text, vocab_size) {
        Ok(tokenizer) => {
            println!("BPE training completed successfully!");
            println!("{:#?}", tokenizer);
        }
        Err(error) => {
            eprintln!("Error: {error}");
            process::exit(1);
        }
    }
}