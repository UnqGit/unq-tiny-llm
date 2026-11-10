use std::ops;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::tokenizer::bpe::types::TokenId;

pub type Weight = f32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    weights: Vec<Weight>,
    token_count: usize,
    embedding_dim: usize,
}

impl<> ops::IndexMut<TokenId> for Embedding {
    fn index_mut(&mut self, index: TokenId) -> &mut [Weight] {
        let start = index * self.embedding_dim;
        let end = start + self.embedding_dim;
        &mut self.weights[start..end]
    }
}

impl<> ops::Index<TokenId> for Embedding {
    type Output = [Weight];
    
    fn index(&self, index: TokenId) -> &Self::Output {
        let start = index * self.embedding_dim;
        let end = start + self.embedding_dim;
        &self.weights[start..end]
    }
}

impl Embedding {
    pub fn create(token_count: usize, embedding_dim: usize) -> Self {
        let mut rng = rand::rng();

        let len = token_count * embedding_dim;
        let mut weights = Vec::with_capacity(len);
        for _ in 0..len {
            weights.push(rng.random_range(0.0..1.0));
        }
        Embedding { weights, token_count, embedding_dim }
    }
}