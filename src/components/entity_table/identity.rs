//! Stable DOM identity for the framework-owned controls `EntityTable` renders
//! (`ldui-izkq`).
//!
//! ## Why this module exists
//!
//! An accessible *name* is not a DOM *identity*. The generated select-all and
//! per-row checkboxes of
//! [`EntityTableMultiSelection`](super::EntityTableMultiSelection) carried a
//! localized `aria-label` and no `id`/`name`, so Chrome reported form controls
//! without an id or name and a consuming page could neither reference them from
//! a `label[for]`/`aria-controls` nor address them from a form submission or a
//! deterministic automation selector. Repairing that from the consuming page
//! means reaching into markup this crate owns, which is the ownership violation
//! the bead forbids.
//!
//! ## Deliberately a restatement, not an import
//!
//! `DataTable` solved the same problem first (`ldui-j6sh`,
//! `data_table::identity`), and that module is private to `data_table`. This is
//! the same contract, restated here rather than reached across a module
//! boundary, so the two tables agree by construction and neither owns the
//! other's DOM vocabulary.
//!
//! ## The contract
//!
//! One resolved **table control prefix** per mounted table (caller-supplied
//! wins; otherwise a process-unique minted one), and every framework-owned
//! control derives its `id` from that prefix plus a fixed **role** segment
//! plus, where the control is per-row, an **encoded stable token** — the row
//! key. Never the visible index: an index re-points at a different row the
//! moment the table pages, sorts, filters, groups or collapses, and an id that
//! silently aliases to a different row is worse than no id.
//!
//! `name` is set alongside `id` on every control, because `name` is what makes
//! an input a real form control; an `id` alone leaves Chrome's "form field
//! without an id or name" report standing for the submission path.
//!
//! ## Why the token is escape-encoded rather than slugified
//!
//! Row keys are arbitrary consumer strings — a UUID, an email, a composite
//! `office/2026-09-01`, a name with a space. The usual "replace anything
//! non-alphanumeric with a dash" slug is **not injective**: `a b`, `a-b` and
//! `a_b` all collapse to `a-b`, so three distinct rows would share one id.
//! [`encode_entity_id_token`] instead escapes every byte outside `[A-Za-z0-9]`
//! as `_` plus two lowercase hex digits. `_` itself is escaped, so `_` in the
//! output is *always* an escape marker followed by exactly two hex digits and
//! the encoding is decodable — hence injective, which is what proves two
//! distinct row keys can never mint the same id. See
//! [`tests::decode_entity_id_token`], which round-trips the encoding as the
//! proof.
//!
//! A second property falls out of it and is load-bearing: an encoded token can
//! never contain `-`, so the `-`-joined segments of an id are unambiguous and a
//! per-row id can never collide with a fixed-role id such as the select-all
//! checkbox.

use std::sync::atomic::{AtomicU64, Ordering};

/// Process-wide sequence behind [`next_entity_control_id`].
static ENTITY_CONTROL_ID: AtomicU64 = AtomicU64::new(0);

/// The reserved prefix namespace this crate mints into. A caller-supplied table
/// control prefix should not start with it.
pub(crate) const RESERVED_ENTITY_CONTROL_NAMESPACE: &str = "ldui-";

/// A process-unique control prefix for one mounted table, minted only when the
/// caller omits `control_id`.
///
/// A monotonic counter rather than randomness, mirroring
/// `next_entity_page_size_id`: unique across every table mounted in one page's
/// lifetime, which is all `id`/`name` association needs. It is deliberately
/// *not* offered as the public identity — the counter depends on mount order,
/// so a consumer that wants a prefix stable across builds supplies its own.
pub(crate) fn next_entity_control_id() -> String {
    format!(
        "{RESERVED_ENTITY_CONTROL_NAMESPACE}entity-table-{}",
        ENTITY_CONTROL_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// Escapes `raw` into `[A-Za-z0-9_]`, injectively.
///
/// Every byte outside `[A-Za-z0-9]` becomes `_` plus two lowercase hex digits,
/// `_` included, so the result is decodable and two distinct inputs can never
/// produce the same output. The output contains no `-`, which is what keeps the
/// `-`-joined id segments unambiguous.
pub(crate) fn encode_entity_id_token(raw: &str) -> String {
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
/// `[A-Za-z0-9_-]` survives verbatim so a readable `activities-table` stays
/// readable in the DOM; anything else (a space, a slash, non-ASCII) is escaped
/// the same way [`encode_entity_id_token`] escapes it, because an `id`
/// containing whitespace is invalid and one containing `.`/`#`/`:` is a
/// CSS-selector trap. An all-whitespace or empty value is treated as absent.
pub(crate) fn normalize_entity_control_id(supplied: &str) -> Option<String> {
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

/// Resolves the one table control prefix: a usable caller value wins, otherwise
/// the already-minted per-instance `fallback`.
pub(crate) fn resolve_entity_control_id(supplied: Option<String>, fallback: &str) -> String {
    supplied
        .as_deref()
        .and_then(normalize_entity_control_id)
        .unwrap_or_else(|| fallback.to_owned())
}

/// `id`/`name` for the rows-per-page select derived from a table prefix.
///
/// Only consulted when the caller supplies a table `control_id` and no explicit
/// `page_size_control_id`: the dedicated prop predates this module
/// (`ldui-kl55`) and still wins outright, and a table that supplies neither
/// keeps its own minted `ldui-entity-page-size-N` exactly as before.
pub(crate) fn entity_page_size_control_id(prefix: &str) -> String {
    format!("{prefix}-page-size")
}

/// `id`/`name` for the select-all ("select everything displayed") checkbox. A
/// fixed role segment: there is exactly one per table.
pub(crate) fn entity_selection_header_control_id(prefix: &str) -> String {
    format!("{prefix}-select-all")
}

/// `id`/`name` for one row's selection checkbox, derived from the **stable row
/// key** the selection contract already requires — never from the row's
/// position on the current page.
pub(crate) fn entity_selection_row_control_id(prefix: &str, row_key: &str) -> String {
    format!("{prefix}-select-row-{}", encode_entity_id_token(row_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// The inverse of [`encode_entity_id_token`]. Only a test needs it — its
    /// existence is the proof that the encoding is injective, because a
    /// function with a left inverse cannot map two inputs to one output.
    fn decode_entity_id_token(encoded: &str) -> Option<String> {
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
            let encoded = encode_entity_id_token(key);
            assert_eq!(
                decode_entity_id_token(&encoded).as_deref(),
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
                seen.insert(encode_entity_id_token(key)),
                "{key:?} aliased onto an id already minted for another key"
            );
        }
    }

    #[test]
    fn encoded_tokens_are_id_safe_and_contain_no_segment_separator() {
        for key in AMBIGUOUS_KEYS {
            let encoded = encode_entity_id_token(key);
            assert!(
                encoded
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
                "{key:?} encoded to {encoded:?}, which is not id-safe"
            );
            assert!(
                !encoded.contains('-'),
                "{key:?} encoded to {encoded:?}; a '-' would make the id segments ambiguous"
            );
        }
    }

    /// Every framework-owned id one table can mint, for a fixed prefix.
    fn all_ids(prefix: &str, row_keys: &[&str]) -> Vec<String> {
        let mut ids = vec![
            entity_page_size_control_id(prefix),
            entity_selection_header_control_id(prefix),
        ];
        ids.extend(
            row_keys
                .iter()
                .map(|key| entity_selection_row_control_id(prefix, key)),
        );
        ids
    }

    #[test]
    fn one_table_mints_no_duplicate_ids_even_for_adversarial_row_keys() {
        let mut adversarial = AMBIGUOUS_KEYS.to_vec();
        // Deliberately shaped like the fixed role segments.
        adversarial.extend(["select-all", "page-size", "select-row"]);
        let ids = all_ids("t", &adversarial);
        let unique: HashSet<&String> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "duplicate id within one table");
        assert!(ids.iter().all(|id| !id.is_empty()));
        assert!(ids.iter().all(|id| !id.contains(char::is_whitespace)));
    }

    #[test]
    fn a_row_checkbox_never_collides_with_the_select_all_checkbox() {
        let header = entity_selection_header_control_id("t");
        for key in AMBIGUOUS_KEYS.iter().chain(["all", "row", ""].iter()) {
            assert_ne!(entity_selection_row_control_id("t", key), header);
        }
    }

    #[test]
    fn two_tables_on_one_page_share_no_id() {
        let first = next_entity_control_id();
        let second = next_entity_control_id();
        assert_ne!(first, second);

        let a: HashSet<String> = all_ids(&first, AMBIGUOUS_KEYS).into_iter().collect();
        let b: HashSet<String> = all_ids(&second, AMBIGUOUS_KEYS).into_iter().collect();
        assert!(
            a.is_disjoint(&b),
            "two mounted tables minted a shared id: {:?}",
            a.intersection(&b).collect::<Vec<_>>()
        );

        let named_a: HashSet<String> = all_ids("alpha", AMBIGUOUS_KEYS).into_iter().collect();
        let named_b: HashSet<String> = all_ids("beta", AMBIGUOUS_KEYS).into_iter().collect();
        assert!(named_a.is_disjoint(&named_b));
    }

    #[test]
    fn a_row_identity_is_the_key_not_the_position() {
        // The same three keys in three different page orders -- a sort, a
        // filter that drops a row, and a page that starts at a different
        // offset. Identity must follow the key.
        let first_page = ["k-9", "k-2", "k-7"];
        let sorted = ["k-2", "k-7", "k-9"];
        let after_filter = ["k-7"];

        let ids_of = |slice: &[&str]| -> Vec<String> {
            slice
                .iter()
                .map(|key| entity_selection_row_control_id("t", key))
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
        // Page 1 and page 2 of the same dataset. An index-derived id would give
        // both pages `row-0..row-2`; a key-derived one must not.
        let page_one = ["a", "b", "c"];
        let page_two = ["d", "e", "f"];
        let one: HashSet<String> = page_one
            .iter()
            .map(|key| entity_selection_row_control_id("t", key))
            .collect();
        let two: HashSet<String> = page_two
            .iter()
            .map(|key| entity_selection_row_control_id("t", key))
            .collect();
        assert!(one.is_disjoint(&two));
        assert_eq!(one.len(), 3);
    }

    #[test]
    fn a_caller_supplied_prefix_wins_and_an_unusable_one_falls_back() {
        assert_eq!(
            resolve_entity_control_id(Some("activities-table".into()), "minted"),
            "activities-table"
        );
        assert_eq!(resolve_entity_control_id(None, "minted"), "minted");
        assert_eq!(
            resolve_entity_control_id(Some(String::new()), "minted"),
            "minted"
        );
        assert_eq!(
            resolve_entity_control_id(Some("   ".into()), "minted"),
            "minted"
        );
        // Trimmed, then escaped: an id may not contain whitespace, and a `.` or
        // `#` would break every CSS selector built from it.
        assert_eq!(
            resolve_entity_control_id(Some("  office perf.table  ".into()), "minted"),
            "office_20perf_2etable"
        );
    }

    #[test]
    fn a_resolved_prefix_is_always_id_safe() {
        for supplied in ["a b", "a.b", "a#b", "a:b", "тбл", "tab\tle", "ok-name_1"] {
            let resolved = resolve_entity_control_id(Some(supplied.into()), "minted");
            assert!(!resolved.is_empty());
            assert!(
                resolved
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'),
                "{supplied:?} resolved to {resolved:?}, which is not id-safe"
            );
        }
    }

    #[test]
    fn the_minted_namespace_is_the_reserved_one() {
        assert!(next_entity_control_id().starts_with(RESERVED_ENTITY_CONTROL_NAMESPACE));
    }

    // ── The rendered markup is actually wired to this contract ──
    //
    // The functions above can be perfect while no control uses them. This scans
    // the view source, which is what makes dropping a `name=` a native failure
    // rather than something only an Office satellite notices again.

    const COMPONENT_SRC: &str = include_str!("component.rs");

    #[test]
    fn entity_table_exposes_the_caller_supplied_prefix_once_per_instance() {
        assert!(
            COMPONENT_SRC.contains("control_id: MaybeProp<String>"),
            "EntityTable must let a caller supply one stable control prefix"
        );
        assert!(
            COMPONENT_SRC.contains("let minted_control_id = next_entity_control_id();"),
            "the fallback prefix must be minted ONCE per instance -- minting it inside a \
             reactive closure would hand every control a new id on each re-render"
        );
        assert!(
            COMPONENT_SRC
                .contains("resolve_entity_control_id(control_id.get(), &minted_control_id)"),
            "EntityTable must resolve the caller's prefix against the per-instance minted fallback"
        );
    }

    #[test]
    fn both_selection_checkboxes_emit_an_id_and_a_name_from_the_row_key() {
        assert!(
            COMPONENT_SRC.contains("attr:id=move || selection_header_id.get()")
                && COMPONENT_SRC.contains("attr:name=move || selection_header_id.get()"),
            "the select-all checkbox lost its identity"
        );
        assert!(
            COMPONENT_SRC.contains("attr:id=move || row_control_id.get()")
                && COMPONENT_SRC.contains("attr:name=move || row_control_id.get()"),
            "the row checkbox lost its identity"
        );
        assert!(
            COMPONENT_SRC.contains(
                "entity_selection_row_control_id(&table_control_id.get(), &identity_key)"
            ),
            "a row checkbox's identity must derive from the stable row key -- an index-derived \
             one re-points at a different row the moment the table sorts, filters or pages"
        );
        assert!(
            COMPONENT_SRC.contains("entity_selection_header_control_id(&table_control_id.get())"),
            "the select-all checkbox's identity must come from the resolved table prefix"
        );
    }

    #[test]
    fn the_selection_indeterminate_property_survived_gaining_an_identity() {
        // `indeterminate` has no HTML attribute; a rewrite that turned these
        // controls into plain attribute soup would silently drop it.
        assert!(
            COMPONENT_SRC.contains(
                "prop:indeterminate=move || displayed_page_state.get().is_indeterminate()"
            ),
            "the select-all checkbox must keep writing `indeterminate` as a DOM property"
        );
    }
}
