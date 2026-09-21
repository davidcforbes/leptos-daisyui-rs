//! Guard against muted body text below WCAG AA contrast (`ldui-zy37`).
//!
//! On the light themes this crate's consumers ship, `text-base-content/N` with
//! N below 75 fails the AA 4.5:1 minimum for normal text at the sizes this
//! library uses (`text-xs`, `text-sm`), computed against 4iiz-Office's
//! base-content `#3E3F42` over white:
//!
//! | step | ratio  |
//! |------|--------|
//! | /40  | 2.12:1 |
//! | /50  | 2.65:1 |
//! | /60  | 3.37:1 |
//! | /65  | 3.84:1 |
//! | /70  | 4.36:1 |
//! | /75  | 5.03:1 |
//!
//! Every earlier fix was one site and the lesson never travelled: `ldui-usqz`
//! fixed a `/65`, a test banned `/60` on compact labels only, and `3c35912`
//! fixed a caption a CONSUMER's axe found. This scans all of `src/`, so the
//! rule applies everywhere at once. A genuinely exempt site (disabled
//! control, purely decorative, large text) goes in [`ALLOWLIST`] with its
//! reason; the list starts empty because the 2026-09-21 sweep needed none.

use std::fs;
use std::path::{Path, PathBuf};

/// The smallest `text-base-content/N` step that clears AA at 14px.
const FLOOR: u32 = 75;

/// `(file suffix relative to src/, a substring unique to the line, reason)`.
const ALLOWLIST: [(&str, &str, &str); 0] = [];

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

/// Every `text-base-content/N` step on `line` whose N is below [`FLOOR`].
/// Matches with any variant prefix (`hover:`, `placeholder:`), because that
/// text must be readable too.
fn sub_floor_steps(line: &str) -> Vec<u32> {
    const NEEDLE: &str = "text-base-content/";
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find(NEEDLE) {
        let after = &rest[at + NEEDLE.len()..];
        let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(n) = digits.parse::<u32>()
            && n < FLOOR
        {
            found.push(n);
        }
        rest = after;
    }
    found
}

#[test]
fn no_library_text_uses_a_muted_step_below_aa() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_files(&repo.join("src"), &mut files);
    assert!(!files.is_empty(), "found no sources to scan");

    let mut offenders = Vec::new();
    for p in &files {
        let rel = p.strip_prefix(repo.join("src")).unwrap_or(p);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        for (i, line) in src.lines().enumerate() {
            // Comments -- including `@source inline(...)` declarations --
            // are not rendered text.
            if line.trim_start().starts_with("//") {
                continue;
            }
            let steps = sub_floor_steps(line);
            if steps.is_empty() {
                continue;
            }
            if ALLOWLIST
                .iter()
                .any(|(file, marker, _)| rel_str.ends_with(file) && line.contains(marker))
            {
                continue;
            }
            offenders.push(format!("src/{rel_str}:{} uses /{steps:?}", i + 1));
        }
    }

    assert!(
        offenders.is_empty(),
        "muted text below /{FLOOR} fails WCAG AA at the sizes this library uses \
         (see this file's table) -- use text-base-content/{FLOOR} or darker, or add \
         a genuinely exempt site (disabled, decorative, large text) to ALLOWLIST \
         with its reason:\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn the_detector_flags_every_failing_step_and_passes_the_floor() {
    assert_eq!(
        sub_floor_steps(r#"class="text-xs text-base-content/60""#),
        vec![60]
    );
    assert_eq!(
        sub_floor_steps("text-base-content/70 text-base-content/50"),
        vec![70, 50]
    );
    assert_eq!(
        sub_floor_steps("hover:text-base-content/40"),
        vec![40],
        "variants count"
    );
    assert!(
        sub_floor_steps("text-base-content/75").is_empty(),
        "the floor passes"
    );
    assert!(sub_floor_steps("text-base-content/80").is_empty());
    assert!(
        sub_floor_steps("text-base-content").is_empty(),
        "full content passes"
    );
}

#[test]
fn the_detector_ignores_non_text_utilities() {
    // A border or background at /40 is not text contrast (WCAG 1.4.3).
    assert!(sub_floor_steps("border border-base-content/40").is_empty());
    assert!(sub_floor_steps("bg-base-content/10").is_empty());
}
