mod normalization;
mod pre_tokenization;

pub use pre_tokenization::PreTokens;

pub fn preprocess_text<'a>(text: &'a str) -> PreTokens {
    let normalized_text = normalization::normalize_text(text);
    pre_tokenization::pretokenize_text(normalized_text)
}