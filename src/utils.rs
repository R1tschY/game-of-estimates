pub fn char_len(name: &str) -> usize {
    name.encode_utf16().count()
}

pub fn char_trim(name: &str, n: usize) -> String {
    let trimmed: Vec<u16> = name.encode_utf16().take(n).collect();
    String::from_utf16_lossy(trimmed.as_ref())
}
