use crate::config::Config;

pub mod internal {
    use crate::config::Config;
    use crate::tokenizer::bpe::trainer::train_and_save;
    use crate::loader::load_data;
    use crate::tokenizer::bpe::training;
    use std::cmp::min;

    pub fn run_tokenizer(config: &Config) {
        let data = match load_data(config.paths.training_dataset.clone()) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("{err}");
                return;
            },
        };
        
        let content = data.contents.join("\n\0\n");
        let error = data.errors.join("\n");
        let is_content_empty = content.is_empty();
        let is_error_empty = error.is_empty();
    
        if !is_error_empty {
            eprintln!("Error(s) occured during loading data:\n{}", error);
        }
    
        if is_content_empty {
            eprintln!("Could not run pre-processing with empty dataset.");
            return;
        }
    
        let max_bytes = min(1000, content.len());
        let boundary = content.floor_char_boundary(max_bytes);
        println!("Size of the content: {}\nPreview of the text:\n{}\n", content.len(), &content[..boundary]);
    
        let impl1 = training::bpe_training(&content, config.tokenizer.vocab_size, config.tokenizer.min_pair_freq);
        match impl1 {
            Ok(_) => println!("Tokenization using naive implementation was successfully completed."),
            Err(ref err) => eprintln!("Error occured during tokenization of data by the naive implementation:\n{}", err),
        }
        let impl2 = training::bpe_training_pair_count_optimized(&content, config.tokenizer.vocab_size, config.tokenizer.min_pair_freq);
        match impl2 {
            Ok(_) => println!("Tokenization using pair count optimized implementation was successfully completed."),
            Err(ref err) => eprintln!("Error occured during tokenization of data by the pair count optimized implementation:\n{}", err),
        }

        if impl1.is_ok() && impl2.is_ok() {
            let tokenizer1 = impl1.unwrap();
            let tokenizer2 = impl2.unwrap();
            if tokenizer1.vocab != tokenizer2.vocab || tokenizer1.merges != tokenizer2.merges {
                eprintln!("Error: The two implementations produced different results.");
            } else {
                println!("Both implementations produced the same result.");
            }
        }
        // let Err(err) = train_and_save(&content, config.tokenizer.vocab_size, config.tokenizer.min_pair_freq, config.paths.tokenizer_save.clone())
        // else {
        //     println!("Tokenizer training successfully completed and data is succesfully saved.");
        //     return;
        // };
        // eprintln!("Error occured during tokenization of data:\n{}", err);
    }
}

pub fn run_command(config: &Config, command: &str) {
    match command {
        "-tokenizer" => internal::run_tokenizer(config),
        _ => eprintln!("Command: `{}` is not implemented", command),
    }
}