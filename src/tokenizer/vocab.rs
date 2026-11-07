use crate::tokenizer::bpe::types::BpeTokenizer;
use serde::{Deserialize, Serialize};

pub type TokenSeq = Vec<u8>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabWord {
    pub seq: TokenSeq,
}

pub fn initial_vocab() -> Vec<VocabWord> {
    (u8::MIN..=u8::MAX)
        .map(|b| VocabWord { seq: vec![b] })
        .collect()
}

pub fn build_vocab_repr_seq(tokenizer: &BpeTokenizer) -> Vec<VocabWord> {
    let mut res = initial_vocab();
    res.reserve(tokenizer.merges.len());

    for merge in &tokenizer.merges {
        let seq = [
            res[merge.left].seq.as_slice(),
            res[merge.right].seq.as_slice(),
        ]
        .concat();

        res.push(VocabWord { seq });
    }
    res
}
