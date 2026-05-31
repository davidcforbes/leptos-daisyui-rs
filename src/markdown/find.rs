//! Pure find/replace search functions.
//!
//! No DOM, no signals — just `&str` in, results out.  The editor uses these
//! to drive the FindOverlay's match list and to apply Replace / Replace All.

use std::ops::Range;

/// Return the byte ranges in `source` where `query` matches.
///
/// Matches do not overlap — after a match, scanning resumes at `match.end`.
/// When `query` is empty, returns an empty vec.
pub fn find_all_matches(source: &str, query: &str, case_sensitive: bool) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    if case_sensitive {
        find_all_case_sensitive(source, query)
    } else {
        find_all_case_insensitive(source, query)
    }
}

fn find_all_case_sensitive(source: &str, query: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = source[start..].find(query) {
        let abs = start + pos;
        let end = abs + query.len();
        out.push(abs..end);
        start = end;
    }
    out
}

fn find_all_case_insensitive(source: &str, query: &str) -> Vec<Range<usize>> {
    // Two-pass scan: lowercase both sides, then map indices back via
    // byte-by-byte tracking.  ASCII-only fast path avoids allocating when
    // both sides are pure ASCII.
    if source.is_ascii() && query.is_ascii() {
        let lower_source = source.to_ascii_lowercase();
        let lower_query = query.to_ascii_lowercase();
        return find_all_case_sensitive(&lower_source, &lower_query)
            .into_iter()
            .map(|r| r.start..r.start + query.len())
            .collect();
    }
    // Unicode-aware: lowercase via to_lowercase().  Match positions in the
    // lowercased source map back to the original via the running byte
    // offset of each pre-lowercase char.
    let lower_query: String = query.chars().flat_map(|c| c.to_lowercase()).collect();
    let mut lower_source = String::with_capacity(source.len());
    let mut idx_map: Vec<usize> = Vec::with_capacity(source.len() + 1);
    let mut orig_pos = 0usize;
    for ch in source.chars() {
        for lc in ch.to_lowercase() {
            for _ in 0..lc.len_utf8() {
                idx_map.push(orig_pos);
            }
            lower_source.push(lc);
        }
        orig_pos += ch.len_utf8();
    }
    idx_map.push(orig_pos);

    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = lower_source[start..].find(&lower_query) {
        let lower_abs = start + pos;
        let lower_end = lower_abs + lower_query.len();
        let orig_start = idx_map.get(lower_abs).copied().unwrap_or(source.len());
        let orig_end = idx_map.get(lower_end).copied().unwrap_or(source.len());
        out.push(orig_start..orig_end);
        start = lower_end;
    }
    out
}

/// Result of a replace-all operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceResult {
    pub new_source: String,
    pub count: usize,
}

/// Replace every match of `query` in `source` with `replacement`.
///
/// When `query` is empty, returns the source unchanged with count 0.
pub fn replace_all(
    source: &str,
    query: &str,
    replacement: &str,
    case_sensitive: bool,
) -> ReplaceResult {
    if query.is_empty() {
        return ReplaceResult {
            new_source: source.to_string(),
            count: 0,
        };
    }
    let matches = find_all_matches(source, query, case_sensitive);
    if matches.is_empty() {
        return ReplaceResult {
            new_source: source.to_string(),
            count: 0,
        };
    }
    let count = matches.len();
    let mut out =
        String::with_capacity(source.len() + replacement.len().saturating_sub(query.len()) * count);
    let mut cursor = 0;
    for m in matches {
        out.push_str(&source[cursor..m.start]);
        out.push_str(replacement);
        cursor = m.end;
    }
    out.push_str(&source[cursor..]);
    ReplaceResult {
        new_source: out,
        count,
    }
}

/// Pick the next match index `>= cursor`.  Falls back to the first match
/// when the cursor is past all matches (wrap-around).
pub fn next_match_index(matches: &[Range<usize>], cursor: usize) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }
    matches.iter().position(|m| m.start >= cursor).or(Some(0))
}

/// Pick the previous match index `< cursor`.  Falls back to the last match
/// when the cursor is before all matches (wrap-around).
pub fn prev_match_index(matches: &[Range<usize>], cursor: usize) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }
    matches
        .iter()
        .rposition(|m| m.start < cursor)
        .or(Some(matches.len() - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_returns_no_matches() {
        assert!(find_all_matches("hello", "", true).is_empty());
        assert!(find_all_matches("hello", "", false).is_empty());
    }

    #[test]
    fn basic_case_sensitive() {
        let m = find_all_matches("hello world hello", "hello", true);
        assert_eq!(m, vec![0..5, 12..17]);
    }

    #[test]
    fn case_insensitive_ascii() {
        let m = find_all_matches("Hello WORLD hello", "hello", false);
        assert_eq!(m, vec![0..5, 12..17]);
    }

    #[test]
    fn case_insensitive_unicode() {
        // German ß lowercases to "ss" (length grows) — case-insensitive
        // match should still locate the original byte range.
        let src = "Straße";
        let m = find_all_matches(src, "straße", false);
        assert_eq!(m.len(), 1);
        assert_eq!(&src[m[0].clone()], "Straße");
    }

    #[test]
    fn non_overlapping_matches() {
        // "aaa" matched against "aa" finds two matches, not three.
        let m = find_all_matches("aaaa", "aa", true);
        assert_eq!(m, vec![0..2, 2..4]);
    }

    #[test]
    fn replace_all_basic() {
        let r = replace_all("foo bar foo", "foo", "baz", true);
        assert_eq!(r.new_source, "baz bar baz");
        assert_eq!(r.count, 2);
    }

    #[test]
    fn replace_all_case_insensitive() {
        let r = replace_all("Foo bar FOO", "foo", "baz", false);
        assert_eq!(r.new_source, "baz bar baz");
        assert_eq!(r.count, 2);
    }

    #[test]
    fn replace_all_empty_query_is_noop() {
        let r = replace_all("hello", "", "x", true);
        assert_eq!(r.new_source, "hello");
        assert_eq!(r.count, 0);
    }

    #[test]
    fn replace_all_no_matches() {
        let r = replace_all("hello", "world", "x", true);
        assert_eq!(r.new_source, "hello");
        assert_eq!(r.count, 0);
    }

    #[test]
    fn next_match_index_at_or_after_cursor() {
        let m = vec![0..3, 10..13, 20..23];
        assert_eq!(next_match_index(&m, 0), Some(0));
        assert_eq!(next_match_index(&m, 5), Some(1));
        assert_eq!(next_match_index(&m, 15), Some(2));
        // Past all matches → wrap to first.
        assert_eq!(next_match_index(&m, 100), Some(0));
    }

    #[test]
    fn prev_match_index_strictly_before_cursor() {
        let m = vec![0..3, 10..13, 20..23];
        // Cursor at 0 → no match strictly before → wrap to last.
        assert_eq!(prev_match_index(&m, 0), Some(2));
        assert_eq!(prev_match_index(&m, 5), Some(0));
        assert_eq!(prev_match_index(&m, 15), Some(1));
        assert_eq!(prev_match_index(&m, 100), Some(2));
    }

    #[test]
    fn next_prev_empty_returns_none() {
        let m: Vec<Range<usize>> = vec![];
        assert_eq!(next_match_index(&m, 0), None);
        assert_eq!(prev_match_index(&m, 0), None);
    }
}
