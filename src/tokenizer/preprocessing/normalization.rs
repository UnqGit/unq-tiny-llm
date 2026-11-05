use unicode_normalization::UnicodeNormalization;

pub fn normalize_text(text: &str) -> String {
    text.nfc().collect()
}