//! Guard: every `web_sys::Type` / `web_sys::console` use in `src/` must have
//! a matching Cargo feature on the library's `web-sys` dependency.
//!
//! `web-sys` gates each binding behind a Cargo feature of the same name.
//! Using `web_sys::FocusEvent` (or `web_sys::console::…`) without enabling
//! `"FocusEvent"` / `"console"` fails with `E0425` / `E0433` — "cannot find
//! type/module in crate `web_sys`". That looks like "web-sys will not
//! compile" even though the crate itself is fine; the feature list is what
//! is incomplete.
//!
//! A source scan is the right shape: the failure is at the dependency
//! boundary, no component unit test would catch a newly-added type whose
//! feature was forgotten, and `cargo check` only fails after the full
//! sibling path-dep graph resolves.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Feature names enabled on the library's `web-sys` dependency.
fn lib_web_sys_features(cargo_toml: &str) -> BTreeSet<String> {
    let after = cargo_toml
        .split("web-sys = {")
        .nth(1)
        .expect("Cargo.toml must declare a web-sys dependency");
    let block = after
        .split('}')
        .next()
        .expect("web-sys dependency block must close");
    block
        .split('"')
        .enumerate()
        .filter_map(|(i, part)| {
            if i % 2 == 1 && !part.is_empty() && part != "0.3" {
                Some(part.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// `web_sys::TypeName` and `web_sys::console` references in one source file.
fn web_sys_usages(src: &str) -> BTreeSet<String> {
    let mut uses = BTreeSet::new();
    let mut rest = src;
    while let Some(idx) = rest.find("web_sys::") {
        rest = &rest[idx + "web_sys::".len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        // Keep type/module names (`FocusEvent`, `console`); skip free functions
        // like `window` / `window()`.
        let first = name.chars().next().unwrap();
        if first.is_uppercase() || name == "console" {
            uses.insert(name);
        }
    }
    // `use web_sys::{Foo, Bar}` — also count braced imports.
    for chunk in src.split("use web_sys::{").skip(1) {
        let Some(end) = chunk.find('}') else {
            continue;
        };
        for part in chunk[..end].split(',') {
            let name = part.split_whitespace().next().unwrap_or("");
            let name = name.split(" as ").next().unwrap_or(name).trim();
            if name.is_empty() {
                continue;
            }
            let first = name.chars().next().unwrap();
            if first.is_uppercase() || name == "console" {
                uses.insert(name.to_string());
            }
        }
    }
    for chunk in src.split("use web_sys::").skip(1) {
        if chunk.starts_with('{') {
            continue;
        }
        let name: String = chunk
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let first = name.chars().next().unwrap();
        if first.is_uppercase() || name == "console" {
            uses.insert(name);
        }
    }
    uses
}

#[test]
fn every_web_sys_type_used_in_src_has_a_cargo_feature() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo = fs::read_to_string(repo.join("Cargo.toml")).expect("read Cargo.toml");
    let features = lib_web_sys_features(&cargo);
    assert!(
        features.contains("Window"),
        "sanity: expected Window in web-sys features, got {features:?}"
    );

    let mut files = Vec::new();
    rs_files(&repo.join("src"), &mut files);
    assert!(!files.is_empty(), "found no sources to scan");

    let mut used = BTreeSet::new();
    for p in &files {
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        used.extend(web_sys_usages(&src));
    }
    assert!(
        !used.is_empty(),
        "expected at least one web_sys:: usage in src/"
    );

    let missing: Vec<_> = used.difference(&features).cloned().collect();
    assert!(
        missing.is_empty(),
        "src/ uses web_sys types/modules with no matching Cargo feature:\n  {}\n\
         Enable them under the library's web-sys features in Cargo.toml.",
        missing.join("\n  ")
    );
}

#[test]
fn feature_parser_accepts_current_manifest_shape() {
    let sample = r#"
web-sys = { version = "0.3", features = [
    "Window",
    "FocusEvent",
    "console",
] }
"#;
    let feats = lib_web_sys_features(sample);
    assert_eq!(
        feats,
        BTreeSet::from(["Window".into(), "FocusEvent".into(), "console".into(),])
    );
}

#[test]
fn usage_scanner_finds_types_and_console() {
    let src = r#"
        use web_sys::{FocusEvent, HtmlFormElement};
        use web_sys::WheelEvent;
        fn f(e: web_sys::SubmitEvent) {
            web_sys::console::warn_1(&v);
            let _ = web_sys::window();
        }
    "#;
    let uses = web_sys_usages(src);
    assert!(uses.contains("FocusEvent"));
    assert!(uses.contains("HtmlFormElement"));
    assert!(uses.contains("WheelEvent"));
    assert!(uses.contains("SubmitEvent"));
    assert!(uses.contains("console"));
    assert!(!uses.contains("window"));
}
