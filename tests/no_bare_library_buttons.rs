//! Guard against a new bare `<button>` reintroducing `ldui-zl58`/`ldui-2e7a`'s
//! `button-without-btn` drift into every consumer's `ldui-audit` run
//! (`audit/src/drift.js`):
//!
//! ```js
//! if (tag === 'BUTTON' && !el.classList.contains('btn')) {
//!   if (!el.closest(EXEMPT_CLOSEST) && !el.hasAttribute('data-pressable')) {
//!     push(path(el), 'button-without-btn: raw <button> lacks .btn');
//!   }
//! }
//! ```
//!
//! A `<button>` this crate emits must satisfy one of the rule's three
//! exemptions: carry `.btn`, carry `data-pressable="true"` (the marker
//! [`Pressable`](../src/components/pressable/component.rs) defines for a
//! deliberately unstyled action -- `ldui-zl58`, `ldui-2e7a`), or sit inside an
//! `EXEMPT_CLOSEST` ancestor (`.menu, .tabs, .dropdown, .modal-backdrop,
//! [data-ld-audit-exempt]`). Every button that is none of those is markup a
//! consumer did not write showing up in *their* audit as if it were.
//!
//! ## What this scan can and cannot see
//!
//! `.btn` and `data-pressable` are checked textually: does a `class="..."`
//! (static or in a `move ||` closure literal) containing the word `btn`, or a
//! literal `data-pressable`, appear within a short window after the opening
//! `<button`. That is a real, if approximate, stand-in for
//! `classList.contains('btn')` / `hasAttribute('data-pressable')`.
//!
//! The `EXEMPT_CLOSEST` ancestor check is **not** reproduced -- this is a text
//! scan over `.rs` source, not a DOM walk, and "is this button a descendant of
//! an element with one of five classes" is not decidable from surrounding
//! lines in general. Rather than approximate it (and risk a false exemption
//! hiding a real violation), every case in this codebase that relies on it is
//! named explicitly in [`SCAN_BLIND_SPOT_ALLOWLIST`] with the reason that
//! makes it exempt, checked once by a human, same discipline as
//! `svg_paint_routing.rs`'s router allowlist. A new ancestor-exempt (or
//! otherwise scan-invisible) button must be added there deliberately, with a
//! reason -- it cannot silently pass by resembling an existing one.
//!
//! ## What is out of scope
//!
//! `entity_table/`, `data_table/`, `modal/` and `patterns/search_picker_dialog.rs`
//! carry their own unmarked bare buttons as of this writing and are excluded
//! from the scan (`ldui-2e7a`'s scope explicitly stayed out of concurrent work
//! there). Widening this guard to cover them is follow-up work for whoever
//! closes those gaps, not a reason to leave the rest of the crate unguarded
//! meanwhile -- same reasoning as `svg_paint_routing.rs`'s "a scanner scoped to
//! less than the defect class is scoped to less than the defect class."

use std::fs;
use std::path::{Path, PathBuf};

/// Directories/files excluded from this scan because they are out of
/// `ldui-2e7a`'s scope (concurrent work elsewhere, or a documented separate
/// gap). Relative to `src/`.
const EXCLUDED: [&str; 4] = [
    "components/entity_table",
    "components/data_table",
    "components/modal",
    "patterns/search_picker_dialog.rs",
];

/// Buttons this text scan cannot correctly judge: the exemption is ancestry
/// (`EXEMPT_CLOSEST`), a dynamic class function the scanner cannot evaluate,
/// or the button is `#[cfg(test)]`-only fixture code that never ships to a
/// consumer. Each entry is `(file suffix relative to src/, a substring
/// unique to that button's opening tag, the reason it is exempt)`.
const SCAN_BLIND_SPOT_ALLOWLIST: [(&str, &str, &str); 3] = [
    (
        "components/toolbar/component.rs",
        "class:menu-active=checked",
        "renders inside <DropdownContent>, itself inside <Dropdown> (root class \
         `dropdown`) -- EXEMPT_CLOSEST's `.dropdown` covers it",
    ),
    (
        "components/login_screen/provider.rs",
        "class=btn_class",
        "class comes from LoginProvider::button_class(), which always includes \
         \"btn btn-block\" -- a dynamic fn call this text scan cannot evaluate",
    ),
    (
        "patterns/snapshot_table_page.rs",
        "with_toolbar_actions(|| view! { <button>",
        "inside #[cfg(test)] mod tests as a placeholder toolbar_actions fixture \
         -- never compiled into the shipped library, so never reaches a consumer",
    ),
];

fn rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn is_excluded(rel: &Path) -> bool {
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    EXCLUDED.iter().any(|ex| rel_str.starts_with(ex))
}

/// True if `window` (the opening tag plus a short lookahead) demonstrates
/// one of the two textually-visible exemptions.
fn has_visible_marker(window: &str) -> bool {
    if window.contains("data-pressable") {
        return true;
    }
    // A `class="..."` (`class=move || "..."`, or a `merge_classes!("a",
    // "btn", ...)` call with `btn` as one of several string-literal args)
    // whose string literal(s) contain the whole word `btn`. Scan every
    // quoted literal from the first `class` keyword onward -- not just the
    // first quoted pair -- since `btn` is often not the first argument.
    let Some(class_at) = window.find("class") else {
        return false;
    };
    let mut in_literal = false;
    for part in window[class_at..].split('"') {
        if in_literal && part.split_whitespace().any(|w| w == "btn") {
            return true;
        }
        in_literal = !in_literal;
    }
    false
}

fn allowlisted(rel: &Path, window: &str) -> Option<&'static str> {
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    SCAN_BLIND_SPOT_ALLOWLIST
        .iter()
        .find(|(file, marker, _)| rel_str.ends_with(file) && window.contains(marker))
        .map(|(_, _, reason)| *reason)
}

#[test]
fn every_library_button_carries_btn_or_pressable_or_is_allowlisted() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_files(&repo.join("src"), &mut files);
    assert!(!files.is_empty(), "found no sources to scan");

    let mut offenders = Vec::new();
    for p in &files {
        let rel = p.strip_prefix(repo.join("src")).unwrap_or(p);
        if is_excluded(rel) {
            continue;
        }
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        let lines: Vec<&str> = src.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.contains("<button") {
                continue;
            }
            // Skip doc comments / plain comments (documentation showing the
            // migration or usage, not emitted markup).
            if trimmed.starts_with("///") || trimmed.starts_with("//") {
                continue;
            }
            // Skip a `<button` that is itself a quoted string literal, e.g.
            // `.find("<button")` in a test helper -- real JSX-like markup is
            // never immediately followed by a closing quote.
            if let Some(after) = trimmed.split("<button").nth(1)
                && after.starts_with('"')
            {
                continue;
            }
            let end = (i + 20).min(lines.len());
            let window = lines[i..end].join("\n");
            if has_visible_marker(&window) {
                continue;
            }
            if let Some(_reason) = allowlisted(rel, &window) {
                continue;
            }
            let display_rel = rel.display();
            offenders.push(format!("src/{display_rel}:{}", i + 1));
        }
    }

    assert!(
        offenders.is_empty(),
        "found a <button> with none of the ldui-audit button-without-btn \
         exemptions (.btn class, data-pressable=\"true\", or an allowlisted \
         EXEMPT_CLOSEST ancestor) -- either give it .btn, mark it \
         data-pressable=\"true\" if it is a deliberate unstyled action \
         (ldui-zl58 / ldui-2e7a), or add it to SCAN_BLIND_SPOT_ALLOWLIST in \
         this file with the reason that exempts it:\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn the_marker_detector_recognizes_btn_class() {
    assert!(has_visible_marker(
        r#"<button class="btn btn-ghost btn-xs">"#
    ));
    assert!(has_visible_marker(
        "<button\n  class=move || if x { \"btn btn-primary\" } else { \"btn btn-ghost\" }\n>"
    ));
}

#[test]
fn the_marker_detector_recognizes_data_pressable() {
    assert!(has_visible_marker(
        "<button\n  type=\"button\"\n  data-pressable=\"true\"\n  class=\"shrink-0\"\n>"
    ));
}

#[test]
fn the_marker_detector_rejects_a_genuinely_bare_button() {
    assert!(!has_visible_marker(
        r#"<button class="shrink-0 w-4 h-4 flex items-center">"#
    ));
    assert!(!has_visible_marker(
        "<button type=\"button\" style=ITEM_STYLE>"
    ));
}

#[test]
fn the_marker_detector_does_not_false_positive_on_a_substring_of_btn() {
    // `class="button-outer"` contains "btn"-adjacent text but not the whole
    // word `btn` -- must not be treated as the daisyUI .btn class.
    assert!(!has_visible_marker(r#"<button class="button-outer">"#));
}
