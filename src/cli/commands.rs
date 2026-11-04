use crate::config::Config;

pub mod internal {
    use crate::config::Config;
    use crate::tokenizer::bpe::trainer::train_and_save;
    use crate::loader::load_data;
    use std::cmp::min;

    pub fn run_preprocessing(config: &Config) {
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
    
        let Err(err) = train_and_save(&content, config.pre_processing.vocab_size, config.pre_processing.min_pair_freq, config.paths.tokenizer.clone())
        else {
            println!("Pre processing successfully completed and data is succesfully saved.");
            return;
        };
        eprintln!("Error occured during preprocessing of data:\n{}", err);
    }
}

pub fn run_command(config: &Config, command: &str) {
    match command {
        "-pre-processing" => internal::run_preprocessing(config),
        _ => eprintln!("Command: `{}` is not implemented", command),
    }
}