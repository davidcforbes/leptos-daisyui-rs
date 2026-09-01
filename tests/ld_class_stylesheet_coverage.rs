//! Guard: every literal `ld-*` class a component or demo page emits must be
//! defined by SOME stylesheet this crate ships (ldui-h7tw).
//!
//! ldui-h7tw's actual bug was narrower than "no stylesheet defines it": the
//! six `.ld-text-*` steps (`SectionHeading`, `KpiStrip`, `PageHeader`, ...)
//! WERE defined — but only by the runtime `UiTokensPreamble` component
//! (`src/tokens/preamble.rs`'s `ui_tokens_css()`), never by the generated
//! static `styles/tokens.css`. A consumer that used those patterns without
//! also mounting the preamble silently lost the size step: weight and colour
//! still applied, but with Tailwind preflight resetting heading sizes an H2
//! rendered at body size. This file carries two checks:
//!
//! 1. [`type_ramp_classes_are_defined_in_the_static_stylesheet`] pins the
//!    actual fix: the six `.ld-text-*` steps must resolve from
//!    `styles/tokens.css` alone, with no dependency on any runtime preamble.
//!    (`xtask`'s own test suite separately pins that the generator derives
//!    those rules from `ui_tokens`, not a second hand-copied set of numbers —
//!    see `gen_tokens_tests::ld_text_classes_reference_the_theme_vars_rather_than_re_deriving_numbers`
//!    in `xtask/src/main.rs`.)
//! 2. [`card_elevation_resolves_from_the_static_stylesheet_alone`] and
//!    [`static_and_runtime_elevation_declarations_agree`] make the same two
//!    demands for `.ld-card-depth`, `KpiCard`'s resting elevation
//!    (ldui-k4fn). That class replaced a stock `shadow-sm`, so a
//!    runtime-only definition would have been a strict regression — no
//!    shadow at all — rather than a fix, which is why it is held to the
//!    stricter bar rather than the general backstop below.
//! 3. [`every_ld_class_literal_is_defined_somewhere`] is the general backstop
//!    the bead's acceptance criteria asked for: it scans `src/` and
//!    `demo/src/` for literal `ld-*` string tokens and asserts each is
//!    defined by AT LEAST ONE of the three stylesheets this crate ships
//!    (static or runtime). It is deliberately a *lower* bar than check 1 for
//!    classes outside the type ramp (`ld-eased`, `ld-focus-ring`,
//!    `ld-elevated`, `ld-ripple-host`, `ld-aichat-msg-in`, `ld-vstep-rail`,
//!    ...): those are documented as requiring `UiAnimationsPreamble` /
//!    `UiTokensPreamble` to be mounted (see their own doc comments and
//!    `src/tokens/animations.rs`), so "defined only at runtime" is their
//!    intended, advertised contract — not the silent-dependency bug this
//!    bead is about. Promoting the whole animation family into the static
//!    file too is a separate, larger decision this bead does not make.
//!
//! **Not covered**: `--ld-space-*` / `--ld-stroke-*` / `--ld-radius-*`
//! custom properties, which a component would reach via `var(--ld-space-m)`
//! rather than a `.class` selector, and which are ALSO currently emitted
//! only by the runtime preamble (never `styles/tokens.css`) — the same shape
//! of gap this bead fixes for the type ramp. Verified 2026-08-31: no
//! component references any of them today
//! (`rg "var\(--ld-(space|stroke|radius)-" src demo/src` is empty), so there
//! is nothing yet for a guard to protect against a false failure on. Extend
//! this file with a `var(--ld-...)` scanner the day a component starts
//! referencing one directly — the ldui-h7tw comment thread already flags
//! this as the next place the same defect can recur.
//!
//! A source scan is the right shape of test here for the same reason the
//! daisyUI-4 class guard (`tests/no_dead_daisyui4_classes.rs`) and the SVG
//! paint-routing guard (`tests/svg_paint_routing.rs`) are: the property is
//! "no component in this crate can *ever* emit a class no stylesheet
//! defines", which is a statement about every possible caller, not about one
//! rendered tree.

use leptos_daisyui_rs::tokens::{TYPE_STEPS, ui_animations_css, ui_tokens_css};
use std::collections::HashSet;
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

/// Extract every maximal run of `[a-z0-9-]` that starts with `"ld-"` from one
/// non-comment source line.
///
/// Tokens ending in `-` are dropped: that shape is always a `format!`/id
/// builder prefix cut off at an interpolation brace (`ld-field-{}`,
/// `ld-menu-{instance}-item-{index}`, `ld-roster-{instance}-cell-{row}-{col}`)
/// — a DOM id template, never a complete class name a stylesheet could
/// define. Comment lines are skipped (doc comments legitimately mention class
/// names as prose, e.g. `` `ld-field-0` `` as an example id), matching the
/// convention `tests/no_dead_daisyui4_classes.rs` already uses.
fn ld_class_tokens_in(line: &str) -> Vec<&str> {
    let t = line.trim_start();
    if t.starts_with("//") || t.starts_with('*') {
        return Vec::new();
    }
    let is_tok_char = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in line.char_indices() {
        if is_tok_char(c) {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            push_if_class(&line[s..i], &mut out);
        }
    }
    if let Some(s) = start {
        push_if_class(&line[s..], &mut out);
    }
    out
}

fn push_if_class<'a>(tok: &'a str, out: &mut Vec<&'a str>) {
    if tok.starts_with("ld-") && !tok.ends_with('-') {
        out.push(tok);
    }
}

/// Whether `css` defines `class` — either as a `.class` selector (any
/// combinator or pseudo-class may follow, e.g. `.ld-pressable:active`,
/// `.ld-eased,`) or as an `@keyframes class` animation name (the motion
/// primitives reference these by name in an `animation:` declaration, never
/// a plain class attribute — e.g. `ld-dropdown-in`, `ld-ripple`).
fn css_defines(css: &str, class: &str) -> bool {
    let boundary = |c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    for pat in [format!(".{class}"), format!("@keyframes {class}")] {
        let mut base = 0usize;
        while let Some(rel) = css[base..].find(pat.as_str()) {
            let abs = base + rel;
            let before_ok = css[..abs].chars().next_back().is_none_or(boundary);
            let after = abs + pat.len();
            let after_ok = css[after..].chars().next().is_none_or(boundary);
            if before_ok && after_ok {
                return true;
            }
            base = after;
        }
    }
    false
}

fn read_static_tokens_css() -> String {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(repo.join("styles/tokens.css"))
        .expect("styles/tokens.css must exist — run `cargo xtask gen-tokens`")
}

#[test]
fn type_ramp_classes_are_defined_in_the_static_stylesheet() {
    let css = read_static_tokens_css();
    for (name, _, _) in TYPE_STEPS {
        assert!(
            css_defines(&css, &format!("ld-text-{name}")),
            "styles/tokens.css does not define .ld-text-{name} — the type \
             ramp is not usable by a consumer who never mounts \
             UiTokensPreamble (ldui-h7tw). Run `cargo xtask gen-tokens` \
             after checking xtask/src/main.rs's tokens_css()."
        );
    }
}

/// ldui-k4fn, exactly the check [`type_ramp_classes_are_defined_in_the_static_stylesheet`]
/// makes for the type ramp: `KpiCard`'s resting elevation must resolve from
/// `styles/tokens.css` **alone**.
///
/// The class replaces a stock `shadow-sm`, so a runtime-only definition would
/// be a strict regression rather than a fix -- a consumer who never mounts
/// `UiTokensPreamble` would go from a slightly-wrong shadow to no shadow at
/// all, with no error anywhere. Both halves matter: the rule, and the
/// `--ld-elevation-4` custom property its `var()` fallback resolves to.
#[test]
fn card_elevation_resolves_from_the_static_stylesheet_alone() {
    let css = read_static_tokens_css();
    assert!(
        css_defines(&css, "ld-card-depth"),
        "styles/tokens.css does not define .ld-card-depth -- KpiCard would \
         render with NO shadow for a consumer who never mounts \
         UiTokensPreamble, which is worse than the shadow-sm it replaced \
         (ldui-k4fn). Run `cargo xtask gen-tokens`."
    );
    assert!(
        css.contains("--ld-elevation-4:"),
        "styles/tokens.css defines .ld-card-depth but not the \
         --ld-elevation-4 it falls back to, so the rule resolves to nothing."
    );
    assert!(
        !css.contains("--ld-card-shadow:"),
        "the framework must not DECLARE --ld-card-shadow: leaving it \
         undefined is what makes the var() fallback paint and what lets a \
         product theme override it with no specificity contest."
    );
}

/// The two delivery paths must emit the same elevation values, not merely
/// both emit something.
///
/// `styles/tokens.css` is generated by `xtask` (which cannot depend on this
/// crate) and `ui_tokens_css()` is built here, so the formatting is
/// duplicated by necessity. Both read `ui_tokens::elevation`, and this
/// asserts they agree declaration-for-declaration -- otherwise
/// `.ld-card-depth` would paint one depth or another depending on which
/// stylesheet a consumer happened to load.
#[test]
fn static_and_runtime_elevation_declarations_agree() {
    let static_css = read_static_tokens_css();
    let runtime_css = ui_tokens_css();
    for level in ["2", "4", "8", "16", "64"] {
        let key = format!("--ld-elevation-{level}:");
        let pick = |css: &str| -> String {
            css.lines()
                .find(|l| l.trim_start().starts_with(&key))
                .unwrap_or_else(|| panic!("no {key} declaration"))
                .trim()
                .to_owned()
        };
        assert_eq!(
            pick(&static_css),
            pick(&runtime_css),
            "styles/tokens.css and UiTokensPreamble disagree on {key}"
        );
    }
}

#[test]
fn every_ld_class_literal_is_defined_somewhere() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_files(&repo.join("src"), &mut files);
    rs_files(&repo.join("demo/src"), &mut files);
    assert!(!files.is_empty(), "found no sources to scan");

    let static_css = read_static_tokens_css();
    let runtime_css = format!("{}\n{}", ui_tokens_css(), ui_animations_css());

    let mut offenders = Vec::new();
    let mut seen = HashSet::new();
    for p in &files {
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        let rel = p.strip_prefix(repo).unwrap_or(p).display().to_string();
        for (i, line) in src.lines().enumerate() {
            for class in ld_class_tokens_in(line) {
                if !seen.insert(class.to_string()) {
                    continue; // report each undefined class once, at its first sighting
                }
                if !css_defines(&static_css, class) && !css_defines(&runtime_css, class) {
                    offenders.push(format!("{rel}:{} — .{class}", i + 1));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "these ld-* classes are emitted by a component or demo page but \
         defined by NO stylesheet this crate ships — neither \
         styles/tokens.css nor UiTokensPreamble's ui_tokens_css() nor \
         UiAnimationsPreamble's ui_animations_css(). This is the ldui-h7tw \
         defect class: an ld-* class that resolves to nothing, so its \
         weight/colour apply but its size/animation/whatever the class was \
         for does not:\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn the_tokenizer_extracts_a_bare_class_and_drops_a_template_prefix() {
    assert_eq!(
        ld_class_tokens_in(r#"                    class="ld-text-body block min-w-0""#),
        vec!["ld-text-body"]
    );
    assert_eq!(
        ld_class_tokens_in(r#"    format!("ld-menu-{instance}-item-{index}")"#),
        Vec::<&str>::new(),
        "an id template prefix cut off at `{{` must not be treated as a class name"
    );
}

#[test]
fn the_tokenizer_skips_comment_lines() {
    assert!(
        ld_class_tokens_in(
            "/// A process-unique id base for one `Field` instance (`ld-field-0`, ...)."
        )
        .is_empty()
    );
    assert!(ld_class_tokens_in("    // ld-eased is applied below").is_empty());
}

#[test]
fn css_defines_finds_a_plain_selector_and_a_pseudo_class_variant() {
    let css =
        ".ld-eased {\n  transition: none;\n}\n.ld-pressable:active {\n  transform: none;\n}\n";
    assert!(css_defines(css, "ld-eased"));
    assert!(css_defines(css, "ld-pressable"));
    assert!(!css_defines(css, "ld-elevated"));
}

#[test]
fn css_defines_finds_a_keyframes_name_but_not_a_prefix_of_one() {
    let css = "@keyframes ld-dropdown-in {\n  from { opacity: 0; }\n}\n";
    assert!(css_defines(css, "ld-dropdown-in"));
    assert!(
        !css_defines(css, "ld-dropdown"),
        "a shorter class must not match as a substring of a longer selector/keyframes name"
    );
}
