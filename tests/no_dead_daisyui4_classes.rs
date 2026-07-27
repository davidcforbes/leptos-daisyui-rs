//! Guard against the daisyUI 4 form classes coming back (ldui-mai.3).
//!
//! daisyUI 5 removed `.form-control`, `.label-text` and `.label-text-alt`.
//! Verify for yourself:
//!
//! ```text
//! grep -c 'form-control' demo/node_modules/daisyui/daisyui.css   # -> 0
//! ```
//!
//! They are not merely cosmetic no-ops. `.form-control` supplied
//! `display:flex; flex-direction:column`; without it the wrapper falls back to
//! `display:inline`, `w-full` goes inert, and the label and its input flow
//! **inline instead of stacking**. The repo carried 206 of these.
//!
//! A grep is the right shape of test here: the failure mode is a dead class
//! silently doing nothing, which no unit test of a component's behaviour would
//! catch, and which only shows up as a subtly wrong layout in a browser.

use std::fs;
use std::path::{Path, PathBuf};

/// The daisyUI 4 classes that no longer exist in daisyUI 5.
const DEAD: [&str; 3] = ["form-control", "label-text-alt", "label-text"];

/// Find the dead classes in one line of source, ignoring the doc comments that
/// explain the migration (this file's own guidance included).
fn dead_classes_in(line: &str) -> Vec<&'static str> {
    let t = line.trim_start();
    if t.starts_with("//") || t.starts_with("*") {
        return Vec::new();
    }
    let mut found = Vec::new();
    let mut rest = line.to_string();
    for name in DEAD {
        if rest.contains(name) {
            found.push(name);
            // Remove so `label-text` does not also match inside
            // `label-text-alt` (DEAD is ordered longest-first for this).
            rest = rest.replace(name, "");
        }
    }
    found
}

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

#[test]
fn no_source_file_uses_a_removed_daisyui4_form_class() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_files(&repo.join("src"), &mut files);
    rs_files(&repo.join("demo/src"), &mut files);
    assert!(!files.is_empty(), "found no sources to scan");

    let mut offenders = Vec::new();
    for p in &files {
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        for (i, line) in src.lines().enumerate() {
            for name in dead_classes_in(line) {
                let rel = p.strip_prefix(repo).unwrap_or(p).display();
                offenders.push(format!("{rel}:{} — {name}", i + 1));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "daisyUI 4 classes are no-ops in daisyUI 5 — `form-control` must become \
         `flex flex-col gap-2`, `label-text` -> `text-sm`, `label-text-alt` -> \
         `text-xs`:\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn the_detector_finds_each_dead_class() {
    assert_eq!(
        dead_classes_in(r#"<label class="form-control w-full">"#),
        vec!["form-control"]
    );
    assert_eq!(
        dead_classes_in(r#"<span class="label-text">"#),
        vec!["label-text"]
    );
    assert_eq!(
        dead_classes_in(r#"<span class="label-text-alt text-error">"#),
        vec!["label-text-alt"]
    );
}

#[test]
fn the_detector_does_not_double_count_the_alt_variant() {
    // `label-text-alt` contains `label-text`; it must report once, as the alt.
    let found = dead_classes_in(r#"class="label-text-alt""#);
    assert_eq!(found, vec!["label-text-alt"], "{found:?}");
}

#[test]
fn the_detector_ignores_comments_explaining_the_migration() {
    assert!(dead_classes_in("    // form-control was removed in daisyUI 5").is_empty());
    assert!(dead_classes_in("/// `label-text` -> `text-sm`").is_empty());
}

#[test]
fn the_detector_accepts_the_replacements() {
    assert!(dead_classes_in(r#"<label class="flex flex-col gap-2 w-full">"#).is_empty());
    assert!(dead_classes_in(r#"<span class="text-sm font-semibold">"#).is_empty());
}
