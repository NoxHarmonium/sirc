#[must_use]
pub fn words_to_bytes(words: &[u16]) -> Vec<u8> {
    words
        .iter()
        .flat_map(|word| u16::to_be_bytes(*word))
        .collect()
}

///
/// # Panics
/// Will panic if an odd number of bytes are provided
///
#[must_use]
pub fn bytes_to_words(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()))
        .collect()
}
