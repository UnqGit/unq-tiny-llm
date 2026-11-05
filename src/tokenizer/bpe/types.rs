use serde::{Deserialize, Serialize};

pub type TokenSeq = Vec<u8>;
pub type TokenId = usize;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabWord {
    pub seq: TokenSeq,
    pub id: TokenId,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Merge {
    pub left: TokenId,
    pub right: TokenId,
    pub merged: TokenId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BpeTokenizer {
    pub vocab: Vec<VocabWord>,
    pub merges: Vec<Merge>,
}