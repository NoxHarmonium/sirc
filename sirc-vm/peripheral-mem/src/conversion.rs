///
/// Converts a slice of words into a vector of bytes.
///
/// Usually used to convert something that is stored in words (e.g. `MemoryPeripheral`)
/// into a format that can be written to, for example, a file.
/// In other words, moving data from inside the VM to outside it (the environment).
///
/// ```
/// use peripheral_mem::conversion::words_to_bytes;
///
/// let words = &[0x7A79, 0x7877];
/// let bytes = words_to_bytes(words);
///
/// assert_eq!(vec![0x7A, 0x79, 0x78, 0x77], bytes);
///
/// ```
///
#[must_use]
pub fn words_to_bytes(words: &[u16]) -> Vec<u8> {
    words
        .iter()
        .flat_map(|word| u16::to_be_bytes(*word))
        .collect()
}

///
/// Converts a slice of bytes into a vector of words.
///
/// Usually used to convert something that is stored in bytes (e.g. a file read from disk)
/// into a format that can be written to a `MemoryPeripheral` which has a 16 bit data bus
/// and does not deal with bytes.
///
/// In other words, moving data from outside the VM (the environment).to inside it.
///
/// ```
/// use peripheral_mem::conversion::bytes_to_words;
///
/// let bytes = "zyxw".as_bytes();
/// let words = bytes_to_words(bytes);
///
/// assert_eq!(vec![0x7A79, 0x7877], words);
///
/// ```
///
/// # Panics
/// Will panic if an odd number of bytes are provided
///
#[must_use]
pub fn bytes_to_words(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks(2)
        .map(|chunk| match chunk {
            [lonely_byte] => u16::from_be_bytes([0, *lonely_byte]),
            [high_byte, low_byte] => u16::from_be_bytes([*high_byte, *low_byte]),
            _ => panic!("chunk must be either one or two bytes long"),
        })
        .collect()
}
