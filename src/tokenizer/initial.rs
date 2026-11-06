use crate::tokenizer::bpe::types::{VocabWord, TokenId};

pub fn initial_vocab() -> Vec<VocabWord> {
    (u8::MIN..=u8::MAX)
        .map(|b| VocabWord { seq: vec![b], id: TokenId::from(b) })
        .collect()
}

pub fn initial_token_sequence(byte_sequence: &[&str]) -> Vec<Vec<TokenId>> {
    byte_sequence
        .iter()
        .map(|&b| b.as_bytes()
            .iter()
            .map(|&c| TokenId::from(c))
            .collect()
        )
        .collect()
}