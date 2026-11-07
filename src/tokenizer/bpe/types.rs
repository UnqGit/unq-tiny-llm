use serde::{Deserialize, Serialize};

pub type TokenId = usize;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Merge {
    pub left: TokenId,
    pub right: TokenId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BpeTokenizer {
    pub merges: Vec<Merge>,
}
