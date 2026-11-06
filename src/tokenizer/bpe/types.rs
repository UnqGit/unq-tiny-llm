use serde::{Deserialize, Serialize};

pub type TokenId = usize;
pub type TokenSeq = Vec<u8>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabWord {
    pub seq: TokenSeq,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Merge {
    pub left: TokenId,
    pub right: TokenId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BpeTokenizer {
    pub merges: Vec<Merge>,
}