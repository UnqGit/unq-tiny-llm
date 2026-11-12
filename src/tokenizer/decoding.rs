use std::{cmp, fs::File, io::Write, path::Path};

use crate::{
    config::Config, log_error, log_info, log_success, tokenizer::{
        bpe::{
            serialization::load_tokenizer,
            types::{BpeTokenizer, TokenId},
        },
        serialization::load_encoded,
        vocab::{TokenSeq, build_vocab_repr_seq},
    },
};

pub fn decode_as_bytes(
    token_sequence: &[TokenId],
    tokenizer: &BpeTokenizer,
) -> Result<TokenSeq, String> {
    log_info!("Constructing vocabulary from merge sequence.");
    let vocab = build_vocab_repr_seq(tokenizer);
    let mut res: TokenSeq = Vec::new();

    for &t in token_sequence {
        if t >= vocab.len() {
            return Err(format!(
                "TokenId `{}` is out of bounds of the vocabulary(which is of len={}).\n Wrong pair of encoded sequence and tokenizer.",
                t,
                vocab.len()
            ));
        }
        res.extend(&vocab[t].seq);
    }

    Ok(res)
}

pub fn decode(token_sequence: &[TokenId], tokenizer: &BpeTokenizer) -> Result<String, String> {
    match String::from_utf8(decode_as_bytes(token_sequence, tokenizer)?) {
        Ok(str) => Ok(str),
        Err(err) => Err(format!(
            "Error occured during conversion of byte sequence to String object:\n{err}"
        )),
    }
}

fn save_to_file(sequence: impl AsRef<[u8]>, path: impl AsRef<Path>) -> Result<(), String> {
    let mut file = File::create(path)
        .map_err(|err| format!("Could not create file for saving decoded string:\n{err}"))?;

    file.write_all(sequence.as_ref())
        .map_err(|e| format!("Error occured during writing to the file:\n{e}"))?;

    Ok(())
}

pub fn decode_as_bytes_and_save(
    token_sequence: &[TokenId],
    tokenizer: &BpeTokenizer,
    path: impl AsRef<Path>,
) -> Result<TokenSeq, String> {
    let sequence = decode_as_bytes(token_sequence, tokenizer)?;
    save_to_file(&sequence, path)?;
    Ok(sequence)
}

pub fn decode_as_string_and_save(
    token_sequence: &[TokenId],
    tokenizer: &BpeTokenizer,
    path: impl AsRef<Path>,
) -> Result<String, String> {
    let sequence = decode(token_sequence, tokenizer)?;
    save_to_file(&sequence, path)?;
    Ok(sequence)
}

pub fn run_decoder(config: &Config) {
    log_info!("Loading encoded data to decode.");
    let token_sequence = match load_encoded(&config.paths.encoder) {
        Ok(data) => data,
        Err(err) => {
            log_error!("{err}");
            return;
        }
    };

    log_info!("Loading tokenizer to decode with.");
    let tokenizer = match load_tokenizer(&config.paths.tokenizer) {
        Ok(tok) => tok,
        Err(err) => {
            log_error!(
                "Could not load the tokenizer from memory from path `{}`: {:?}",
                config.paths.tokenizer.display(),
                err
            );
            return;
        }
    };

    log_info!(
        "Number of the encoded tokens: {}\nPreview of the encoded tokens:\n\n{:?}...\n",
        token_sequence.len(),
        token_sequence.iter().take(15).collect::<Vec<_>>()
    );

    if config.decoder.save_as_bytes {
        match decode_as_bytes_and_save(&token_sequence, &tokenizer, &config.paths.decoder) {
            Err(err) => log_error!("Error occured during decoding of tokens:\n{err}"),
            Ok(v) => {
                log_success!(
                    "Decoding of tokens successfully completed and data is succesfully saved."
                );
                log_info!(
                    "Byte Preview:\n{:#?}...",
                    v.iter().take(15).collect::<Vec<_>>()
                );
            }
        }
    } else {
        match decode_as_string_and_save(&token_sequence, &tokenizer, &config.paths.decoder) {
            Err(err) => log_error!("Error occured during decoding of tokens:\n{err}"),
            Ok(v) => {
                let max_bytes = cmp::min(1000, v.len());
                let boundary = v.floor_char_boundary(max_bytes);
                log_success!(
                    "Decoding of tokens successfully completed and data is succesfully saved."
                );
                log_info!("Text Preview:\n{}...", &v[..boundary]);
            }
        }
    }
}
