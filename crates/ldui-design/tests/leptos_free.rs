//! ldui-3u3p: `ldui-design` stays Leptos-free and self-contained.
//!
//! The crate exists so a server-rendered consumer (4iiz-kit) can take the
//! design layer without Leptos, and by git, where `leptos-daisyui-rs`'s
//! `../Rust-DeskApp` path dependencies do not exist. The full check is the
//! `cargo tree` one in this crate's README (every transitive dependency);
//! this test pins the direct half of it in the fast gate (`test-design`), so
//! declaring such a dependency fails before any consumer notices.

/// The crate's own manifest, as committed.
const MANIFEST: &str = include_str!("../Cargo.toml");

/// Dependency names that would pull in a browser/Leptos runtime, or the
/// sibling token crate this one deliberately copies its values from.
const FORBIDDEN: [&str; 7] = [
    "leptos",
    "reactive_graph",
    "tachys",
    "wasm-bindgen",
    "web-sys",
    "js-sys",
    "ui-tokens",
];

/// Every `key = value` line outside comments, as `(key, whole line)`.
fn manifest_entries() -> Vec<(String, String)> {
    MANIFEST
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (key, _) = line.split_once('=')?;
            Some((key.trim().trim_matches('"').to_string(), line.to_string()))
        })
        .collect()
}

#[test]
fn manifest_declares_no_leptos_or_browser_dependency() {
    for (key, line) in manifest_entries() {
        for forbidden in FORBIDDEN {
            assert!(
                !key.starts_with(forbidden),
                "ldui-design must stay Leptos-free and self-contained; found `{line}`"
            );
        }
    }
}

#[test]
fn manifest_declares_no_out_of_crate_path_dependency() {
    for (_, line) in manifest_entries() {
        assert!(
            !line.contains("path = \"..") && !line.contains("path=\".."),
            "a path dependency outside the crate breaks git consumers; found `{line}`"
        );
    }
}

/// The negative control: the scan has to be able to fail.
#[test]
fn the_scan_would_catch_a_leptos_dependency() {
    let bad = "leptos = { version = \"0.8\" }";
    let (key, _) = bad.split_once('=').unwrap();
    assert!(FORBIDDEN.iter().any(|f| key.trim().starts_with(f)));
    assert!(
        !manifest_entries().is_empty(),
        "the manifest scan read nothing"
    );
}
