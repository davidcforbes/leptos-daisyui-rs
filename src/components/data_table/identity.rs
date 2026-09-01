//! Stable DOM identity for the framework-owned controls both DataTable
//! variants render (ldui-j6sh).
//!
//! ## Why this module exists
//!
//! An accessible *name* is not a DOM *identity*. `DataTableTextFilter`, the
//! `<select>`s in `DataTableFilterRow` and both halves of
//! `ServerTableMultiSelection` all carried an `aria-label` and no `id`/`name`,
//! so a consuming page could name them for a screen reader but could not
//! reference them from a `label[for]`, an `aria-controls`, a form submission,
//! or a deterministic browser-automation selector. Repairing that from the
//! consuming page means reaching into markup this crate owns, which is exactly
//! the ownership violation the bead forbids.
//!
//! ## The contract
//!
//! One resolved **table control prefix** per mounted table (caller-supplied
//! wins; otherwise a process-unique minted one), and every framework-owned
//! control derives its `id` from that prefix plus a fixed **role** segment
//! plus, where the control is per-column or per-row, an **encoded stable
//! token** — the column id or the row key. Never the visual index: an index
//! re-points at a different row the moment the slice pages, sorts or filters,
//! and an id that silently aliases to a different row is worse than no id
//! (ldui-nz6d, ldui-px06).
//!
//! ## Why the token is escape-encoded rather than slugified
//!
//! Row keys are arbitrary consumer strings — a UUID, an email, a composite
//! `office/2026-09-01`, a name with a space. The usual "replace anything
//! non-alphanumeric with a dash" slug is **not injective**: `a b`, `a-b` and
//! `a_b` all collapse to `a-b`, so three distinct rows would share one id.
//! [`encode_id_token`] instead escapes every byte outside `[A-Za-z0-9]` as
//! `_` plus two lowercase hex digits. `_` itself is escaped, so `_` in the
//! output is *always* an escape marker followed by exactly two hex digits and
//! the encoding is decodable — hence injective, which is what proves two
//! distinct row keys can never mint the same id. See
//! [`tests::decode_id_token`], which round-trips the encoding as the proof.
//!
//! A second property falls out of it and is load-bearing: an encoded token can
//! never contain `-`, so the `-`-joined segments of an id are unambiguous and
//! a per-column id can never collide with a per-row id or with a fixed-role id
//! such as the current-slice checkbox.

use std::sync::atomic::{AtomicU64, Ordering};

/// Process-wide sequence behind [`next_data_table_control_id`].
static DATA_TABLE_CONTROL_ID: AtomicU64 = AtomicU64::new(0);

/// The reserved prefix namespace this crate mints into. A caller-supplied
/// table control prefix should not start with it.
pub(crate) const RESERVED_CONTROL_ID_NAMESPACE: &str = "ldui-";

/// A process-unique control prefix for one mounted table, minted only when the
/// caller omits `control_id`.
///
/// A monotonic counter rather than randomness, mirroring
/// `next_entity_page_size_id`: unique across every table mounted in one page's
/// lifetime, which is all `id`/`name` association needs. It is deliberately
/// *not* offered as the public identity — the counter depends on mount order,
/// so a consumer that wants a prefix stable across builds supplies its own.
pub(crate) fn next_data_table_control_id() -> String {
    format!(
        "{RESERVED_CONTROL_ID_NAMESPACE}data-table-{}",
        DATA_TABLE_CONTROL_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// Escapes `raw` into `[A-Za-z0-9_]`, injectively.
///
/// Every byte outside `[A-Za-z0-9]` becomes `_` plus two lowercase hex digits,
/// `_` included, so the result is decodable and two distinct inputs can never
/// produce the same output. The output contains no `-`, which is what keeps
/// the `-`-joined id segments unambiguous.
pub(crate) fn encode_id_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        if byte.is_ascii_alphanumeric() {
            out.push(*byte as char);
        } else {
            out.push('_');
            out.push_str(&format!("{byte:02x}"));
        }
    }
    out
}

/// Normalizes a caller-supplied table control prefix into something usable as
/// an HTML `id` stem, or `None` when the caller supplied nothing usable.
///
/// `[A-Za-z0-9_-]` survives verbatim so a readable `conversations-table` stays
/// readable in the DOM; anything else (a space, a slash, non-ASCII) is escaped
/// the same way [`encode_id_token`] escapes it, because an `id` containing
/// whitespace is invalid and one containing `.`/`#`/`:` is a CSS-selector trap.
/// An all-whitespace or empty value is treated as absent.
pub(crate) fn normalize_control_id(supplied: &str) -> Option<String> {
    let trimmed = supplied.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(trimmed.len());
    for byte in trimmed.as_bytes() {
        if byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_' {
            out.push(*byte as char);
        } else {
            out.push('_');
            out.push_str(&format!("{byte:02x}"));
        }
    }
    Some(out)
}

/// Resolves the one table control prefix: a usable caller value wins,
/// otherwise the already-minted per-instance `fallback`.
pub(crate) fn resolve_control_id(supplied: Option<String>, fallback: &str) -> String {
    supplied
        .as_deref()
        .and_then(normalize_control_id)
        .unwrap_or_else(|| fallback.to_owned())
}

/// `id`/`name` for the free-text search box above the table.
pub(crate) fn search_control_id(prefix: &str) -> String {
    format!("{prefix}-search")
}

/// `id`/`name` for the rows-per-page select.
pub(crate) fn page_size_control_id(prefix: &str) -> String {
    format!("{prefix}-page-size")
}

/// `id` for the column-tools popover (an `aria-controls` target, not a form
/// control, so it takes no `name`).
pub(crate) fn column_tools_control_id(prefix: &str) -> String {
    format!("{prefix}-column-tools")
}

/// `id`/`name` for one column's filter control — the exact `<select>` and the
/// substring `<input>` share the role segment because a column renders exactly
/// one of the two.
pub(crate) fn filter_control_id(prefix: &str, column_id: &str) -> String {
    format!("{prefix}-filter-{}", encode_id_token(column_id))
}

/// `id`/`name` for the current-slice ("select everything on this page")
/// checkbox. A fixed role segment: there is exactly one per table.
pub(crate) fn selection_header_control_id(prefix: &str) -> String {
    format!("{prefix}-select-all")
}

/// `id`/`name` for one row's selection checkbox, derived from the **stable row
/// key** the multi-selection contract already requires — never from the row's
/// position in the current slice.
pub(crate) fn selection_row_control_id(prefix: &str, row_key: &str) -> String {
    format!("{prefix}-select-row-{}", encode_id_token(row_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// The inverse of [`encode_id_token`]. Only a test needs it — its
    /// existence is the proof that the encoding is injective, because a
    /// function with a left inverse cannot map two inputs to one output.
    fn decode_id_token(encoded: &str) -> Option<String> {
        let bytes = encoded.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'_' {
                let hex = encoded.get(index + 1..index + 3)?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                index += 3;
            } else {
                if !bytes[index].is_ascii_alphanumeric() {
                    return None;
                }
                out.push(bytes[index]);
                index += 1;
            }
        }
        String::from_utf8(out).ok()
    }

    /// Keys chosen for the ways a slug would collapse them together.
    const AMBIGUOUS_KEYS: &[&str] = &[
        "a b",
        "a-b",
        "a_b",
        "a.b",
        "a/b",
        "a:b",
        "a#b",
        "ab",
        "A B",
        "office/2026-09-01",
        "office-2026-09-01",
        "user@example.com",
        "user.example.com",
        "",
        " ",
        "ключ",
        "鍵",
        "row 1",
        "row  1",
        "row1",
    ];

    #[test]
    fn encoded_tokens_round_trip_so_the_encoding_is_injective() {
        for key in AMBIGUOUS_KEYS {
            let encoded = encode_id_token(key);
            assert_eq!(
                decode_id_token(&encoded).as_deref(),
                Some(*key),
                "{key:?} did not round-trip through {encoded:?}"
            );
        }
    }

    #[test]
    fn distinct_keys_that_a_slug_would_collapse_stay_distinct() {
        let mut seen = HashSet::new();
        for key in AMBIGUOUS_KEYS {
            assert!(
                seen.insert(encode_id_token(key)),
                "{key:?} aliased onto an id already minted for another key"
            );
        }
    }

    #[test]
    fn encoded_tokens_are_id_safe_and_contain_no_segment_separator() {
        for key in AMBIGUOUS_KEYS {
            let encoded = encode_id_token(key);
            assert!(
                encoded
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_'),
                "{key:?} encoded to {encoded:?}, which is not id-safe"
            );
            assert!(
                !encoded.contains('-'),
                "{key:?} encoded to {encoded:?}; a '-' would make the id segments ambiguous"
            );
        }
    }

    /// Every framework-owned id one table can mint, for a fixed prefix.
    fn all_ids(prefix: &str, columns: &[&str], row_keys: &[&str]) -> Vec<String> {
        let mut ids = vec![
            search_control_id(prefix),
            page_size_control_id(prefix),
            column_tools_control_id(prefix),
            selection_header_control_id(prefix),
        ];
        ids.extend(columns.iter().map(|c| filter_control_id(prefix, c)));
        ids.extend(row_keys.iter().map(|k| selection_row_control_id(prefix, k)));
        ids
    }

    const COLUMNS: &[&str] = &[
        "client",
        "channel",
        "office",
        "coordinator",
        "status",
        "last_update",
        // Deliberately named to look like a fixed role segment.
        "search",
        "page-size",
        "select-all",
    ];

    #[test]
    fn one_table_mints_no_duplicate_ids_even_for_adversarial_column_names() {
        let ids = all_ids("t", COLUMNS, AMBIGUOUS_KEYS);
        let unique: HashSet<&String> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "duplicate id within one table");
        assert!(ids.iter().all(|id| !id.is_empty()));
        assert!(ids.iter().all(|id| !id.contains(char::is_whitespace)));
    }

    #[test]
    fn a_row_checkbox_never_collides_with_the_current_slice_checkbox() {
        let header = selection_header_control_id("t");
        for key in AMBIGUOUS_KEYS.iter().chain(["all", "row", ""].iter()) {
            assert_ne!(selection_row_control_id("t", key), header);
        }
    }

    #[test]
    fn two_tables_on_one_page_share_no_id() {
        let first = next_data_table_control_id();
        let second = next_data_table_control_id();
        assert_ne!(first, second);

        let a: HashSet<String> = all_ids(&first, COLUMNS, AMBIGUOUS_KEYS)
            .into_iter()
            .collect();
        let b: HashSet<String> = all_ids(&second, COLUMNS, AMBIGUOUS_KEYS)
            .into_iter()
            .collect();
        assert!(
            a.is_disjoint(&b),
            "two mounted tables minted a shared id: {:?}",
            a.intersection(&b).collect::<Vec<_>>()
        );

        let named_a: HashSet<String> = all_ids("alpha", COLUMNS, AMBIGUOUS_KEYS)
            .into_iter()
            .collect();
        let named_b: HashSet<String> = all_ids("beta", COLUMNS, AMBIGUOUS_KEYS)
            .into_iter()
            .collect();
        assert!(named_a.is_disjoint(&named_b));
    }

    #[test]
    fn a_row_identity_is_the_key_not_the_position() {
        // The same three keys in three different slice orders -- a sort, a
        // filter that drops a row, and a page that starts at a different
        // offset. Identity must follow the key.
        let first_page = ["k-9", "k-2", "k-7"];
        let sorted = ["k-2", "k-7", "k-9"];
        let after_filter = ["k-7"];

        let ids_of = |slice: &[&str]| -> Vec<String> {
            slice
                .iter()
                .map(|k| selection_row_control_id("t", k))
                .collect()
        };

        let baseline = ids_of(&first_page);
        assert_eq!(ids_of(&sorted)[2], baseline[0], "k-9 changed identity");
        assert_eq!(ids_of(&sorted)[0], baseline[1], "k-2 changed identity");
        assert_eq!(
            ids_of(&after_filter)[0],
            baseline[2],
            "k-7 changed identity"
        );

        // And the converse: position alone never determines the id, so a
        // *different* key at the same index gets a different id.
        assert_ne!(ids_of(&sorted)[0], baseline[0]);
    }

    #[test]
    fn a_paged_slice_never_reuses_an_id_for_a_different_key() {
        // Page 1 and page 2 of the same dataset. An index-derived id would
        // give both pages `row-0..row-2`; a key-derived one must not.
        let page_one = ["a", "b", "c"];
        let page_two = ["d", "e", "f"];
        let one: HashSet<String> = page_one
            .iter()
            .map(|k| selection_row_control_id("t", k))
            .collect();
        let two: HashSet<String> = page_two
            .iter()
            .map(|k| selection_row_control_id("t", k))
            .collect();
        assert!(one.is_disjoint(&two));
        assert_eq!(one.len(), 3);
    }

    #[test]
    fn a_caller_supplied_prefix_wins_and_an_unusable_one_falls_back() {
        assert_eq!(
            resolve_control_id(Some("conversations-table".into()), "minted"),
            "conversations-table"
        );
        assert_eq!(resolve_control_id(None, "minted"), "minted");
        assert_eq!(resolve_control_id(Some(String::new()), "minted"), "minted");
        assert_eq!(resolve_control_id(Some("   ".into()), "minted"), "minted");
        // Trimmed, then escaped: an id may not contain whitespace, and a `.`
        // or `#` would break every CSS selector built from it.
        assert_eq!(
            resolve_control_id(Some("  office perf.table  ".into()), "minted"),
            "office_20perf_2etable"
        );
    }

    #[test]
    fn a_resolved_prefix_is_always_id_safe() {
        for supplied in ["a b", "a.b", "a#b", "a:b", "тбл", "tab\tle", "ok-name_1"] {
            let resolved = resolve_control_id(Some(supplied.into()), "minted");
            assert!(!resolved.is_empty());
            assert!(
                resolved
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
                "{supplied:?} resolved to {resolved:?}, which is not id-safe"
            );
        }
    }

    #[test]
    fn the_minted_namespace_is_the_reserved_one() {
        assert!(next_data_table_control_id().starts_with(RESERVED_CONTROL_ID_NAMESPACE));
    }

    // ── The rendered markup is actually wired to this contract ──
    //
    // The functions above can be perfect while no control uses them. These
    // scan the view source, which is the same idiom `filter_sidebar`'s tests
    // use, and is what makes dropping a `name=` a native failure rather than
    // something only the Office satellite notices again.

    const FILTER_SRC: &str = include_str!("filter.rs");
    const CLIENT_SRC: &str = include_str!("component.rs");
    const SERVER_SRC: &str = include_str!("server_component.rs");

    #[test]
    fn both_public_tables_expose_the_caller_supplied_prefix() {
        for (name, source) in [("DataTable", CLIENT_SRC), ("ServerDataTable", SERVER_SRC)] {
            assert!(
                source.contains("control_id: MaybeProp<String>"),
                "{name} must let a caller supply one stable control prefix"
            );
            assert!(
                source.contains("resolve_control_id(control_id.get(), &minted_control_id)"),
                "{name} must resolve the caller's prefix against a per-instance minted fallback"
            );
            assert!(
                source.contains("let minted_control_id = next_data_table_control_id();"),
                "{name}'s fallback must be minted ONCE per instance -- minting it inside a \
                 reactive closure would hand the control a new id on every re-render"
            );
        }
    }

    #[test]
    fn every_filter_control_emits_both_an_id_and_a_name() {
        // Two controls, one per `ColumnFilterKind`; each needs id + name and
        // an explicit `for` on its visually-hidden label.
        assert_eq!(
            FILTER_SRC.matches("id=move || control_id.get()").count(),
            1,
            "the substring filter input lost its id"
        );
        assert_eq!(
            FILTER_SRC.matches("name=move || control_id.get()").count(),
            1,
            "the substring filter input lost its name"
        );
        assert_eq!(
            FILTER_SRC.matches("id=move || filter_id.get()").count(),
            1,
            "the exact filter select lost its id"
        );
        assert_eq!(
            FILTER_SRC.matches("name=move || filter_id.get()").count(),
            1,
            "the exact filter select lost its name"
        );
        assert_eq!(
            FILTER_SRC.matches("r#for=move || ").count(),
            2,
            "both filter controls need their sr-only label associated by `for`"
        );
        assert!(
            FILTER_SRC.contains("filter_control_id(&control_id.get(), col_id)"),
            "a filter's identity must come from the column's STABLE id"
        );
    }

    #[test]
    fn both_selection_checkboxes_emit_an_id_and_a_name_from_the_row_key() {
        assert!(
            SERVER_SRC.contains("attr:id=move || selection_header_id.get()")
                && SERVER_SRC.contains("attr:name=move || selection_header_id.get()"),
            "the current-slice checkbox lost its identity"
        );
        assert!(
            SERVER_SRC.contains("attr:id=move || row_control_id.get()")
                && SERVER_SRC.contains("attr:name=move || row_control_id.get()"),
            "the row checkbox lost its identity"
        );
        assert!(
            SERVER_SRC.contains("selection_row_control_id(&table_control_id.get(), &identity_key)"),
            "a row checkbox's identity must derive from the stable row key -- an index-derived \
             one re-points at a different row the moment the slice sorts, filters or pages"
        );
        // The row identity closure must not be able to see a slice index at
        // all: the leading cell is built per KEY, so there is no index in
        // scope to accidentally reach for.
        assert!(
            SERVER_SRC.contains("Callback::new(move |key: String| {"),
            "the selection cell must continue to be built from the row key"
        );
    }

    #[test]
    fn the_selection_indeterminate_property_survived_gaining_an_identity() {
        // `indeterminate` has no HTML attribute; a rewrite that turned these
        // controls into plain attribute soup would silently drop it.
        assert!(
            SERVER_SRC.contains("prop:indeterminate=move || slice_state.get().is_indeterminate()"),
            "the current-slice checkbox must keep writing `indeterminate` as a DOM property"
        );
    }

    #[test]
    fn named_table_controls_opt_out_of_browser_autofill() {
        // A `name` is what makes a control a form field, which is also what
        // invites the saved-value dropdown over a table filter.
        assert_eq!(
            FILTER_SRC.matches("autocomplete=\"off\"").count(),
            2,
            "both filter controls must opt out of autofill"
        );
        for (name, source) in [("DataTable", CLIENT_SRC), ("ServerDataTable", SERVER_SRC)] {
            assert!(
                source.contains("data-table-search-control=\"true\""),
                "{name}'s search box needs a stable hook that does not depend on its id shape"
            );
        }
    }
}
