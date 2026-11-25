//! Pure search helpers.
use std::ops::Range;

/// All non-overlapping matches. Case-insensitive mode is ASCII-only so byte offsets stay valid.
// ponytail: ASCII case folding only; unicode folding can change byte lengths
pub fn find_all(text: &str, query: &str, case_sensitive: bool) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    let (hay, needle) = if case_sensitive {
        (text.to_string(), query.to_string())
    } else {
        (text.to_ascii_lowercase(), query.to_ascii_lowercase())
    };
    hay.match_indices(&needle).map(|(i, m)| i..i + m.len()).collect()
}

/// Index of the first match starting at or after `from`, wrapping around.
pub fn next_index(matches: &[Range<usize>], from: usize) -> Option<usize> {
    matches.iter().position(|m| m.start >= from).or(if matches.is_empty() { None } else { Some(0) })
}

/// Index of the last match starting before `from`, wrapping around.
pub fn prev_index(matches: &[Range<usize>], from: usize) -> Option<usize> {
    matches.iter().rposition(|m| m.start < from).or(matches.len().checked_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_and_wraps() {
        let m = find_all("Foo foo FOO", "foo", false);
        assert_eq!(m, vec![0..3, 4..7, 8..11]);
        assert_eq!(find_all("Foo foo", "foo", true), vec![4..7]);
        assert_eq!(next_index(&m, 5), Some(2));
        assert_eq!(next_index(&m, 9), Some(0));
        assert_eq!(prev_index(&m, 4), Some(0));
        assert_eq!(prev_index(&m, 0), Some(2));
    }
}
