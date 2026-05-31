//! Text helpers that are safe for multibyte UTF-8 content.

/// Truncate a string to at most `max_chars` *characters* (not bytes).
///
/// Slicing a `str` by a raw byte index panics when the index falls inside a
/// multibyte UTF-8 sequence (e.g. Chinese, Japanese, emoji). This helper
/// truncates on a character boundary, so it is always safe.
///
/// Returns a borrowed slice when no truncation is needed, otherwise an owned
/// `String` cut at the nearest character boundary at or before `max_chars`
/// characters.
pub fn truncate_chars(text: &str, max_chars: usize) -> &str {
    if text.len() <= max_chars {
        // Fast path: byte length is an upper bound on char count, so if the
        // byte length already fits there is nothing to truncate.
        return text;
    }
    match text.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => &text[..byte_idx],
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_truncation_when_short() {
        assert_eq!(truncate_chars("hello", 10), "hello");
    }

    #[test]
    fn truncates_ascii_by_chars() {
        assert_eq!(truncate_chars("hello world", 5), "hello");
    }

    #[test]
    fn truncates_multibyte_on_boundary() {
        // Each Chinese char is 3 bytes in UTF-8.
        let s = "你好世界朋友";
        let out = truncate_chars(s, 3);
        assert_eq!(out, "你好世");
        // Must be valid UTF-8 and exactly 3 chars.
        assert_eq!(out.chars().count(), 3);
    }

    #[test]
    fn does_not_panic_on_emoji() {
        let s = "a😀b😀c";
        let out = truncate_chars(s, 2);
        assert_eq!(out, "a😀");
    }
}
