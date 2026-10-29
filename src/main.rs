mod tokenizer;
mod cli;

use std::env;
use std::process;
use tokenizer::bpe::trainer::train_and_save;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: cargo run -- <text> [vocab_size]");
        process::exit(1);
    }

    let text = &args[0];

    let vocab_size: usize = match args.get(1) {
        Some(value) => match value.parse() {
            Ok(size) => size,
            Err(_) => {
                eprintln!("Error: vocab_size must be a valid unsigned integer.");
                process::exit(1);
            }
        },
        None => 300,
    };

    let _ = train_and_save(text, vocab_size, "/encoding.json")
        .map_err(|e| println!("Could not train and save the text: {e}"));
}