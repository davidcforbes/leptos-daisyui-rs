# Visual Quality Checks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A reusable visual-quality audit (`ldui-audit`) any consumer app runs
against its rendered pages, plus a growing rulebook + skill, piloted in
office-perf-web.

**Architecture:** The shared rule model (`StyleProfile`, `AuditReport`,
`Violation`, `Ceiling`, `check_ceilings`) **already landed** in
`PixelProof/crates/pixelproof-style-audit` (commit `5bba7dc`; bead
`PixelProof-5af`). This plan builds everything downstream of it: the
`ldui-audit` web layer in this repo (CDP harness config, in-page sweep JS,
box-shadow parsing, `from_ui_tokens()`, daisyUI component-drift heuristics,
`audit_page()`), migrates this repo's own suites onto it, moves `test_mode`
into the library behind a feature, seeds the `doc/visual-quality/` rulebook +
user skill, and pilots in `office-perf-web`.

**Tech Stack:** Rust 2024, `pixelproof-web` (CDP/chromiumoxide harness),
`pixelproof-style-audit` (rule model), `ui-tokens` (profile defaults), tokio,
serde. All sibling path-deps under `C:\dev`.

**Spec:** `docs/superpowers/specs/2026-08-08-visual-quality-checks-design.md`.
One deviation from spec §1a, already landed in PixelProof and adopted here:
the engine crate holds the **rule model only**; the sweep JS and harness
plumbing live in `ldui-audit` (this repo), not in PixelProof.

## Global Constraints

- `cargo fmt` is **per-package** in this repo — never `cargo fmt --all` (it
  reaches into sibling repos). Every new crate must be added to xtask's
  fmt/clippy/test step lists (`xtask/src/main.rs` `steps()`).
- `cargo clippy` is **per-crate** — `--workspace` fails on leptos `csr`
  feature unification.
- Doc comments: every inline code span stays on ONE `///` line (a wrapped
  backtick span ICEs clippy 1.95 and silently disables linting).
- Run `cargo xtask` from the repo root only.
- Never reference a `ui_tokens` item not on `../Rust-DeskApp`'s DEFAULT
  branch (`master`). Items used here — `typography::{RAMP, Weight}`,
  `radius::{CONTROL, CARD, BADGE, PILL}`, `elevation::{Shadow, LEVELS}`,
  `spacing::SCALE` — verified on `master` at plan time.
- After touching `../Rust-DeskApp/crates/ui-tokens`, run `cargo fmt -p
  ui-tokens` in THAT repo (this plan does not touch it).
- This repo is `publish = false`; the new crate is too.
- Browser-suite tests are `#[ignore]`d and need the demo dev server
  (`npm install` in `demo/` once, then trunk serve via xtask orchestration).
  Budget ~8 min for a cold wasm build; run browser suites in the background.
- Commit messages end with the `Co-Authored-By: Claude Fable 5` trailer.

## Execution order

Phase 1 (Tasks 1–7, this repo) → Phase 2 (Task 8, this repo) → Phase 3
(Tasks 9–10, docs + skill) → Phase 4 (Tasks 11–12, `C:\dev\4iiz-office`).
Task 8 can run any time after Task 1. Tasks 9–10 have no code dependency and
can run in parallel with Phase 1. Phase 4 needs Tasks 1–8 done.

---

### Task 1: Scaffold the `ldui-audit` crate and wire it into the workspace + gate

**Files:**
- Create: `audit/Cargo.toml`
- Create: `audit/src/lib.rs`
- Modify: `Cargo.toml:17` (workspace members)
- Modify: `xtask/src/main.rs` (`steps()` — fmt list, new clippy step, new test step)
- Test: xtask's own `steps_for` unit tests (`xtask/src/main.rs`, bottom)

**Interfaces:**
- Produces: workspace member `ldui-audit` (lib name `ldui_audit`) that later
  tasks fill in; gate steps `clippy-audit`, `test-audit`.

- [ ] **Step 1: Create `audit/Cargo.toml`**

```toml
[package]
name = "ldui-audit"
version = "0.1.0"
edition = "2024"
publish = false
description = "Reusable visual-quality audit for leptos-daisyui-rs consumer apps: CDP harness config, style/layout sweep, daisyUI drift heuristics, ui-tokens profile defaults"

[dependencies]
pixelproof-style-audit = { path = "../../PixelProof/crates/pixelproof-style-audit" }
pixelproof-web = { path = "../../PixelProof/crates/pixelproof-web" }
ui-tokens = { path = "../../Rust-DeskApp/crates/ui-tokens" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
```

- [ ] **Step 2: Create `audit/src/lib.rs`** (skeleton; modules land in later tasks)

```rust
//! # ldui-audit
//!
//! The web surface of the shared visual-quality rule model
//! (`pixelproof-style-audit`): CDP harness plumbing, the in-page sweep over
//! computed styles, daisyUI component-drift heuristics, and `ui-tokens`
//! profile defaults. Consumer apps add this as a dev-dependency, declare a
//! `StyleProfile`, list pages, and assert ratcheted ceilings.
//!
//! Design: `docs/superpowers/specs/2026-08-08-visual-quality-checks-design.md`.

pub use pixelproof_style_audit::{
    check_ceilings, family, AuditReport, Ceiling, FamilyReport, RatchetOutcome, ShadowSpec,
    StyleProfile, Violation,
};
pub use pixelproof_web::{Harness, HarnessConfig, ViewportSize};
```

- [ ] **Step 3: Add `"audit"` to workspace members** in root `Cargo.toml`:
      `members = [".", "xtask", "demo", "audit"]`

- [ ] **Step 4: Wire the gate.** In `xtask/src/main.rs` `steps()`:
      add `"-p", "ldui-audit"` to the `fmt-check` arg list (after the
      `xtask` pair); add two steps after `clippy-demo` / `test-xtask`
      respectively, copying their shape exactly:

```rust
cmd(
    "clippy-audit",
    "cargo",
    &["clippy", "-p", "ldui-audit", "--all-targets", "--", "-D", "warnings"],
    None,
),
// ...and next to test-xtask:
cmd("test-audit", "cargo", &["test", "-p", "ldui-audit", "--lib"], None),
```

Update the `steps_for` matcher so `"clippy"` still selects by prefix (it
already does: `s.name.starts_with("clippy")`) and extend the
`clippy_subcommand_runs_both_crate_steps` unit test's expected vector to
`vec!["clippy-lib", "clippy-demo", "clippy-audit"]`.

- [ ] **Step 5: Verify** — `cargo check -p ldui-audit` passes;
      `cargo test -p xtask` passes (the updated steps test).

- [ ] **Step 6: Commit** — `feat(audit): scaffold ldui-audit crate and gate wiring`

---

### Task 2: `from_ui_tokens()` profile defaults

**Files:**
- Create: `audit/src/profile.rs`
- Modify: `audit/src/lib.rs` (add `pub mod profile; pub use profile::from_ui_tokens;`)

**Interfaces:**
- Produces: `pub fn from_ui_tokens(font_family: impl Into<String>) -> StyleProfile`
- Consumes: `ui_tokens::{typography, radius, elevation, spacing}` (all on
  the sibling's `master`).

- [ ] **Step 1: Write the failing tests** (in `audit/src/profile.rs` `#[cfg(test)]`)

```rust
#[test]
fn defaults_pin_to_the_token_crate() {
    let p = from_ui_tokens("Inter");
    assert_eq!(p.font_family, "Inter");
    assert_eq!(p.body_weight, 400);
    // Pinned element-wise so a token change here is a loud, reviewed event.
    assert_eq!(p.type_ramp, vec![28.0, 20.0, 16.0, 14.0, 12.0, 11.0]);
    assert_eq!(p.radii, vec![4.0, 8.0, 12.0, 9999.0]);
    assert_eq!(p.spacing_scale, vec![4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0]);
    assert_eq!(p.shadows.len(), 5);
    assert!(p.shadow_ok(&ShadowSpec::new(0.0, 2.0, 4.0, 0.14)), "LEVEL_4");
}

#[test]
fn overrides_replace_not_append() {
    let p = from_ui_tokens("Manrope").radii([15.0, 8.0, 999.0]);
    assert!(p.radius_ok(15.0));
    assert!(!p.radius_ok(12.0), "token BADGE radius replaced by override");
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p ldui-audit --lib` →
      FAIL (`from_ui_tokens` not defined).

- [ ] **Step 3: Implement**

```rust
use pixelproof_style_audit::{ShadowSpec, StyleProfile};

/// Profile defaults derived from `ui_tokens` at compile time, so a profile
/// cannot drift from the token crate. Apps chain builder overrides for their
/// deliberate deviations only. `font_family` is a parameter because the token
/// crate does not own a web font-family name.
pub fn from_ui_tokens(font_family: impl Into<String>) -> StyleProfile {
    StyleProfile::new(font_family)
        .body_weight(ui_tokens::typography::Weight::Regular.value())
        .type_ramp(ui_tokens::typography::RAMP.iter().map(|&v| v as f64))
        .radii(
            [
                ui_tokens::radius::CONTROL,
                ui_tokens::radius::CARD,
                ui_tokens::radius::BADGE,
                ui_tokens::radius::PILL,
            ]
            .iter()
            .map(|&v| v as f64),
        )
        .shadows(ui_tokens::elevation::LEVELS.iter().map(|s| {
            ShadowSpec::new(
                s.offset_x as f64,
                s.offset_y as f64,
                s.blur as f64,
                s.opacity as f64,
            )
        }))
        .spacing_scale(ui_tokens::spacing::SCALE.iter().map(|&v| v as f64))
}
```

- [ ] **Step 4: Run to verify pass**, then run
      `cargo xtask check-sibling-tokens` (must stay green — all referenced
      items are on `master`).

- [ ] **Step 5: Commit** — `feat(audit): StyleProfile::from_ui_tokens defaults`

---

### Task 3: Harness plumbing generalized from `tests/common/mod.rs`

**Files:**
- Create: `audit/src/config.rs`
- Create: `audit/src/harness.rs`
- Modify: `audit/src/lib.rs` (add modules + re-exports)

**Interfaces:**
- Produces:

```rust
pub struct AuditConfig {
    pub base_url: String,        // default "http://127.0.0.1:3010"
    pub mount_selector: String,  // default "main"
    pub settle_ms: u64,          // default 500
    pub viewport: ViewportSize,  // default ViewportSize::SMALL
    pub baseline_root: Option<std::path::PathBuf>, // None = no SSIM use
}
impl AuditConfig { pub fn new(base_url: impl Into<String>) -> Self; /* builder setters for each field */ }

pub async fn harness_at(cfg: &AuditConfig, path: &str) -> Harness;
pub async fn wait_for_selector(h: &Harness, sel: &str);
pub async fn click(h: &Harness, selector: &str);
pub async fn oracle(h: &Harness) -> serde_json::Value;
```

- Consumes: `pixelproof_web::{Harness, HarnessConfig, ViewportSize}`.

- [ ] **Step 1: Port the code.** Copy `tests/common/mod.rs` bodies of
      `config()`, `harness_at()`, `wait_for_selector()`, `click()`,
      `oracle()` into `audit/src/harness.rs`, replacing every hard-coded
      value with the `AuditConfig` field: `DEFAULT_BASE_URL` →
      `cfg.base_url`; the `"main"` selector in `harness_at` →
      `cfg.mount_selector`; `with_settle_ms(500)` → `cfg.settle_ms`;
      `VIEWPORT` → `cfg.viewport`. Keep verbatim: the `?pp-freeze=1` append,
      the `style[data-pixelproof="freeze"]` wait, `with_isolated_profile()`,
      the 60 s / 100 ms poll loop, and the panic messages (update the "start
      it with" hint to name the consumer's own server generically). The
      `VISUAL_TEST_BASE_URL` env override moves into `AuditConfig::new`'s
      default handling exactly as `config()` does today. `baseline_root:
      None` skips `with_baseline_root` (rule audits don't need SSIM
      baselines; this repo's visual suite passes `Some`).

- [ ] **Step 2: Unit-test the config defaults** (native, no browser):

```rust
#[test]
fn defaults_mirror_the_proven_demo_values() {
    let c = AuditConfig::new("http://127.0.0.1:3010");
    assert_eq!(c.mount_selector, "main");
    assert_eq!(c.settle_ms, 500);
}
```

- [ ] **Step 3: Verify** — `cargo test -p ldui-audit --lib` and
      `cargo clippy -p ldui-audit --all-targets -- -D warnings` pass.

- [ ] **Step 4: Commit** — `feat(audit): configurable CDP harness plumbing`

---

### Task 4: Box-shadow parser

**Files:**
- Create: `audit/src/shadow.rs`
- Modify: `audit/src/lib.rs`

**Interfaces:**
- Produces: `pub fn parse_box_shadow(computed: &str) -> Vec<ShadowSpec>`
  (empty vec for `"none"`; one entry per comma-separated shadow).

- [ ] **Step 1: Write failing tests with real Chrome serializations**

```rust
#[test]
fn parses_chrome_computed_single_shadow() {
    // Chrome getComputedStyle: color first, then x y blur [spread].
    let v = parse_box_shadow("rgba(0, 0, 0, 0.05) 0px 1px 15px 0px");
    assert_eq!(v.len(), 1);
    assert!(v[0].matches(&ShadowSpec::new(0.0, 1.0, 15.0, 0.05), 0.5));
}

#[test]
fn parses_multiple_shadows_split_on_top_level_commas_only() {
    let v = parse_box_shadow(
        "rgba(0, 0, 0, 0.1) 0px 4px 6px 0px, rgba(0, 0, 0, 0.05) 0px 2px 4px 0px",
    );
    assert_eq!(v.len(), 2);
    assert!(v[1].matches(&ShadowSpec::new(0.0, 2.0, 4.0, 0.05), 0.5));
}

#[test]
fn none_and_empty_yield_no_shadows() {
    assert!(parse_box_shadow("none").is_empty());
    assert!(parse_box_shadow("").is_empty());
}

#[test]
fn opaque_rgb_color_reads_as_alpha_one() {
    let v = parse_box_shadow("rgb(255, 0, 0) 1px 2px 3px 0px");
    assert!(v[0].matches(&ShadowSpec::new(1.0, 2.0, 3.0, 1.0), 0.5));
}
```

- [ ] **Step 2: Run to verify failure.**

- [ ] **Step 3: Implement**

```rust
use pixelproof_style_audit::ShadowSpec;

/// Parse a Chrome computed `box-shadow` into [`ShadowSpec`]s. Chrome
/// serializes as `<color> <x>px <y>px <blur>px [<spread>px] [inset]` per
/// shadow, comma-separated — but commas also appear INSIDE `rgb(a)()`, so we
/// split only at paren depth zero. Spread and inset are ignored: the rule
/// model matches on offset/blur/opacity.
pub fn parse_box_shadow(computed: &str) -> Vec<ShadowSpec> {
    let computed = computed.trim();
    if computed.is_empty() || computed == "none" {
        return Vec::new();
    }
    split_top_level_commas(computed)
        .iter()
        .filter_map(|part| parse_single(part))
        .collect()
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let (mut out, mut depth, mut cur) = (Vec::new(), 0i32, String::new());
    for ch in s.chars() {
        match ch {
            '(' => { depth += 1; cur.push(ch); }
            ')' => { depth -= 1; cur.push(ch); }
            ',' if depth == 0 => { out.push(cur.trim().to_string()); cur.clear(); }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() { out.push(cur.trim().to_string()); }
    out
}

fn parse_single(part: &str) -> Option<ShadowSpec> {
    // Alpha: 4th argument of rgba(...); plain rgb(...) is fully opaque.
    let opacity = if let Some(open) = part.find("rgba(") {
        let inner = &part[open + 5..part[open..].find(')')? + open];
        inner.split(',').nth(3)?.trim().parse::<f64>().ok()?
    } else {
        1.0
    };
    // Lengths: every "<number>px" token OUTSIDE the color function, in order
    // x, y, blur[, spread].
    let after_color = match part.find(')') {
        Some(i) => &part[i + 1..],
        None => part,
    };
    let px: Vec<f64> = after_color
        .split_whitespace()
        .filter_map(|t| t.strip_suffix("px"))
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();
    if px.len() < 3 {
        return None;
    }
    Some(ShadowSpec::new(px[0], px[1], px[2], opacity))
}
```

- [ ] **Step 4: Run to verify pass.**
- [ ] **Step 5: Commit** — `feat(audit): computed box-shadow parser`

---

### Task 5: The style sweep — in-page JS + Rust judging

**Files:**
- Create: `audit/src/sweep.rs` (Rust: `RawElement`, `RawSweep`, `judge`)
- Create: `audit/src/sweep.js` (included via `include_str!`)
- Modify: `audit/src/lib.rs`

**Interfaces:**
- Produces:

```rust
/// One element's raw computed values, as collected in-page.
#[derive(Debug, Deserialize)]
pub struct RawElement {
    pub selector: String,
    pub font_family: String,   // getComputedStyle .fontFamily (first family, unquoted)
    pub font_size: f64,        // px
    pub font_weight: u16,
    pub border_radius: f64,    // px, top-left corner
    pub box_shadow: String,    // raw computed string; "" if none
    pub has_text: bool,        // direct text content (typography rules apply)
}

#[derive(Debug, Deserialize)]
pub struct RawSweep {
    pub elements: Vec<RawElement>,
    pub drift: Vec<Violation>,          // component-drift, decided in-page (Task 6)
    pub layout: LayoutRaw,              // overlap/grid/internal arrays (Task 7 migration)
    pub scanned: usize,
    pub truncated: bool,
}

pub fn judge(raw: &RawSweep, profile: &StyleProfile) -> AuditReport;
pub const STYLE_SWEEP_JS: &str = include_str!("sweep.js");
```

- Consumes: `parse_box_shadow` (Task 4), engine matchers
  (`on_ramp`/`radius_ok`/`shadow_ok`), `family::*` names. Family strings
  used for layout sub-checks (surface-specific extensions the engine
  explicitly allows): `"spacing-overlap"`, `"spacing-grid"`,
  `"spacing-internal"`.

- [ ] **Step 1: Write failing tests for `judge`** (pure Rust, no browser):

```rust
fn raw(sel: &str) -> RawElement {
    RawElement {
        selector: sel.into(), font_family: "Inter".into(), font_size: 14.0,
        font_weight: 400, border_radius: 8.0, box_shadow: String::new(),
        has_text: true,
    }
}

#[test]
fn on_profile_elements_produce_no_violations() {
    let p = from_ui_tokens("Inter");
    let sweep = RawSweep { elements: vec![raw("p#1")], drift: vec![],
        layout: LayoutRaw::default(), scanned: 1, truncated: false };
    let r = judge(&sweep, &p);
    assert_eq!(r.total(), 0);
    assert_eq!(r.scanned, 1);
}

#[test]
fn off_ramp_size_wrong_family_rogue_shadow_and_radius_are_each_reported() {
    let p = from_ui_tokens("Inter");
    let mut bad = raw("div#bad");
    bad.font_size = 13.0;                       // off ramp
    bad.font_family = "Segoe UI".into();        // fallback family
    bad.border_radius = 15.0;                   // undeclared radius
    bad.box_shadow = "rgba(0, 0, 0, 0.5) 0px 9px 9px 0px".into(); // rogue
    let sweep = RawSweep { elements: vec![bad], drift: vec![],
        layout: LayoutRaw::default(), scanned: 1, truncated: false };
    let r = judge(&sweep, &p);
    assert_eq!(r.count(family::TYPOGRAPHY), 2, "size + family");
    assert_eq!(r.count(family::SHAPE), 1);
    assert_eq!(r.count(family::DEPTH), 1);
}

#[test]
fn textless_elements_skip_typography_rules() {
    let p = from_ui_tokens("Inter");
    let mut icon = raw("svg#i");
    icon.has_text = false;
    icon.font_size = 13.0; // inherited junk on a non-text node: not a violation
    let sweep = RawSweep { elements: vec![icon], drift: vec![],
        layout: LayoutRaw::default(), scanned: 1, truncated: false };
    assert_eq!(judge(&sweep, &p).count(family::TYPOGRAPHY), 0);
}
```

- [ ] **Step 2: Run to verify failure.**

- [ ] **Step 3: Implement `judge`** — for each element: if `has_text`, check
      `font_family` (compare the first computed family, case-insensitive,
      against `profile.font_family`) and `profile.on_ramp(font_size)`;
      check `profile.radius_ok(border_radius)` for every element with a
      non-zero radius; parse `box_shadow` and require every parsed shadow
      to satisfy `profile.shadow_ok`. Push violations with the engine
      `family::*` constants; copy `drift` violations into
      `family::COMPONENT_DRIFT`; fold `layout` arrays into
      `"spacing-overlap"` / `"spacing-grid"` / `"spacing-internal"`;
      set `scanned`/`truncated` from the raw sweep. Each `Violation.detail`
      names the expected set, e.g. `"font-size 13 not on ramp [28, 20, 16, 14, 12, 11]"`.

- [ ] **Step 4: Write `sweep.js`.** Start from
      `tests/common/layout_audit.rs`'s `SWEEP_JS` (it already walks visible
      elements, builds CSS-ish selector paths, caps the walk, and returns a
      JSON **string**). Extend the per-element record with
      `fontFamily.split(',')[0].replace(/["']/g,'').trim()`, parsed
      `fontSize`/`fontWeight`, `borderTopLeftRadius` parsed to px,
      `boxShadow` (raw string, `'none'` → `''`), and `has_text` = element
      has a non-whitespace direct text node. Keep returning
      `JSON.stringify(...)` (CDP nested-array marshalling is inconsistent;
      the string convention is proven). Body-weight check: emit one
      synthetic element for `document.body` so `judge` can compare
      `font_weight` to `profile.body_weight` (only for body — child weights
      legitimately vary).

- [ ] **Step 5: Verify** — `cargo test -p ldui-audit --lib` passes;
      clippy-audit clean.

- [ ] **Step 6: Commit** — `feat(audit): style sweep JS and profile judging`

---

### Task 6: Component-drift heuristics (in `sweep.js`)

**Files:**
- Modify: `audit/src/sweep.js`
- Modify: `audit/src/sweep.rs` (tests only)

**Interfaces:**
- Produces: the sweep's `drift` array. Rules (each violation's `detail`
  names the rule id so the rulebook can reference it):
  1. `button-without-btn` — a `<button>` whose classList lacks `btn`, unless
     it is inside `.menu`, `.tabs`, `.dropdown`, `.modal-backdrop`, or
     carries `data-ld-audit-exempt`.
  2. `table-without-table-class` — a `<table>` whose classList lacks `table`.
  3. `badge-lookalike` — a `<span>`/`<div>` with computed
     `border-radius >= 8px`, rendered height `<= 24px`, a non-transparent
     background, direct text, and neither `badge` nor `btn` in classList.
  4. `input-outside-field` — an `<input>` (not checkbox/radio/range/hidden),
     `<select>`, or `<textarea>` with no `<fieldset>` ancestor, no
     wrapping `<label>` ancestor, and no `label[for]` pointing at its id.

- [ ] **Step 1: Implement the four rules in `sweep.js`**, pushing
      `{selector, value, detail}` objects (value = 1.0; detail =
      `"<rule-id>: <human sentence>"`).

- [ ] **Step 2: Native test that judged drift lands in the right family**
      (extend the Task 5 test module):

```rust
#[test]
fn drift_violations_pass_through_to_their_family() {
    let p = from_ui_tokens("Inter");
    let sweep = RawSweep {
        elements: vec![], layout: LayoutRaw::default(), scanned: 1, truncated: false,
        drift: vec![Violation {
            selector: "button#save".into(), value: 1.0,
            detail: "button-without-btn: raw <button> lacks .btn".into(),
        }],
    };
    assert_eq!(judge(&sweep, &p).count(family::COMPONENT_DRIFT), 1);
}
```

Browser-side proof of the JS itself is Task 7's negative controls — do not
try to unit-test DOM heuristics natively.

- [ ] **Step 3: Verify + commit** — `feat(audit): daisyUI component-drift heuristics`

---

### Task 7: `audit_page()`, migrate this repo's suites, `test-style` gate step

**Files:**
- Create: `audit/src/page.rs` (`audit_page`)
- Create: `tests/style_audit_smoke.rs`
- Modify: `tests/common/mod.rs` (delegate to `ldui-audit`)
- Delete: `tests/common/layout_audit.rs` (its JS + types now live in `audit/`)
- Modify: `tests/layout_audit_smoke.rs` (imports only — assertions unchanged)
- Modify: root `Cargo.toml` (`[dev-dependencies] ldui-audit = { path = "audit" }`)
- Modify: `xtask/src/main.rs` (new `test-style` subcommand)

**Interfaces:**
- Produces: `pub async fn audit_page(h: &Harness, profile: &StyleProfile) -> AuditReport`
  (evaluates `STYLE_SWEEP_JS`, deserializes `RawSweep`, returns
  `judge(&raw, profile)`; panics with the raw payload on parse failure,
  mirroring today's `layout_report`).
- Consumes: everything from Tasks 3–6.

- [ ] **Step 1: Implement `audit_page`** (port `layout_report`'s
      evaluate-string-then-parse shape from `tests/common/mod.rs:161-171`).

- [ ] **Step 2: Slim `tests/common/mod.rs`** to keep its public names but
      delegate: `pub use ldui_audit::{wait_for_selector, click, oracle};`
      plus a `harness_at(path)` wrapper that builds the demo's
      `AuditConfig` (base URL `http://127.0.0.1:3010`, baseline root
      `tests/visual/baselines`, `.review/` render/diff roots) and calls
      `ldui_audit::harness_at`. `layout_report` becomes a thin call to
      `audit_page` + a shim exposing the three spacing families so
      `layout_audit_smoke.rs` assertions keep their current shape.

- [ ] **Step 3: Write `tests/style_audit_smoke.rs`** — the demo's own style
      audit plus the negative controls that prove each new family:

```rust
mod common;
use common::harness_at;
use ldui_audit::{audit_page, family, from_ui_tokens, Ceiling, check_ceilings};

/// (path, ceilings) — initial ceilings COME FROM THE FIRST RUN's counts;
/// fill the numbers in during this step, then they only ratchet down.
const PAGES: &[(&str, &[Ceiling])] = &[ /* button, card, data-table, kanban */ ];

// One #[ignore]d test per page, macro'd like layout_audit_smoke.rs:
// let report = audit_page(&h, &from_ui_tokens("<demo font family>")).await;
// report.sanity().unwrap();
// assert_eq!(report.count("spacing-overlap"), 0);  // overlap stays hard-fail
// let out = check_ceilings(&report, ceilings);
// assert!(out.is_pass(), "{}\n{}", out.over.join("\n"), report.describe(path));

/// Negative control: inject one violation per family, assert each is caught.
#[tokio::test]
#[ignore = "needs the demo dev server (cargo xtask test-style)"]
async fn sweep_detects_injected_style_violations() {
    // Inject via evaluate(): (1) a <p style="font-size:13px"> (typography),
    // (2) a <div style="border-radius:17px"> (shape),
    // (3) a <div style="box-shadow:0 9px 9px rgba(0,0,0,.5)"> (depth),
    // (4) a bare <button>Save</button> (component-drift).
    // Assert each family's count rises by >= 1, remove the nodes, assert clean.
}
```

The demo's font family: read the computed `font-family` daisyUI applies
(inspect once via the harness during implementation) and pin it in the test —
that pin is itself a regression check.

- [ ] **Step 4: Add the `test-style` xtask subcommand** — copy the
      `test-layout` arm in `xtask/src/main.rs` (search `"test-layout"`),
      changing only the `--test` target to `style_audit_smoke` and the
      usage string. Update the usage string in `main()` to include
      `test-style`.

- [ ] **Step 5: Run the suites** (background, ~8 min cold):
      `cargo xtask test-layout` (must stay green after the migration) and
      `cargo xtask test-style`; record first-run counts into `PAGES`
      ceilings; re-run to green.

- [ ] **Step 6: Commit** — `feat(audit): audit_page + demo style audit; migrate suites to ldui-audit`

---

### Task 8: `test-mode` feature in the library

**Files:**
- Create: `src/test_mode.rs` (moved from `demo/src/test_mode.rs`, plus `install_test_mode`)
- Modify: `Cargo.toml` (`[features] test-mode = []`; ensure `web-sys` in scope for the module)
- Modify: `src/lib.rs` (`#[cfg(feature = "test-mode")] pub mod test_mode;`)
- Modify: `demo/Cargo.toml` (enable the feature on the path dep)
- Delete: `demo/src/test_mode.rs`
- Modify: `demo/src/main.rs` (import from the library)

**Interfaces:**
- Produces: `leptos_daisyui_rs::test_mode::{QUERY_PARAM, is_enabled, is_test_mode, install_style_kill_switch, install_test_mode, FREEZE_CSS}`.
  New convenience:

```rust
/// One-call setup: returns whether test mode is active, installing the
/// freeze stylesheet when it is. Apps still install their own debug bridge
/// (app state is app-specific) inside the returned-true branch.
pub fn install_test_mode() -> bool {
    let active = is_test_mode();
    if active {
        install_style_kill_switch();
    }
    active
}
```

- [ ] **Step 1: Move the module verbatim** (file content is
      `demo/src/test_mode.rs` — pure functions, unit tests included), add
      `install_test_mode`, gate with the feature. If `web_sys` items
      (`Window`, `Document`, `Element`, `HtmlHeadElement`, `Location`) are
      missing from the library's `web-sys` feature list, add them under the
      `test-mode` feature via `[features] test-mode = []` + the dep features
      unconditionally (they are tiny).
- [ ] **Step 2: Demo consumes it** — `demo/Cargo.toml`:
      `leptos-daisyui-rs = { path = "..", features = ["test-mode"] }`
      (adjust to the existing dep line's shape); `demo/src/main.rs`: replace
      `mod test_mode;` + local calls with
      `use leptos_daisyui_rs::test_mode;` (call sites unchanged).
- [ ] **Step 3: Verify** — `cargo test -p leptos-daisyui-rs --lib`
      (the moved unit tests run), `cargo check -p leptos-daisyui-showcase`,
      and a quick `cargo xtask test-reactivity` in the background (the
      freeze switch is what that suite depends on).
- [ ] **Step 4: Commit** — `feat(test-mode): freeze/oracle seam moves into the library behind a feature`

---

### Task 9: Seed the rulebook — `doc/visual-quality/`

**Files:**
- Create: `doc/visual-quality/README.md`
- Create: seven entry files (below)

**Entry template** (every file follows it):

```markdown
# <defect name>

**Status:** automated (<family>, since 2026-08-08) | rulebook-only
**Seen in:** <app(s)>

## What it looks like
## Root cause
## How to check (manual)
## Automation
<which ldui-audit family/rule catches it, or why it cannot be automated>
```

- [ ] **Step 1: Write `README.md`** — purpose (one paragraph: the growing
      defect-pattern index; new patterns land here first and graduate to
      `ldui-audit` when mechanically checkable; cites
      `PixelProof/docs/methodology/principles.md` for the layer model),
      the lifecycle (rulebook-only → automated), the entry template, and a
      table indexing the entries below.
- [ ] **Step 2: Write the seven seed entries** (status / seen-in / substance):
  1. `fallback-font.md` — automated (typography). Manrope silently falls
     back to Segoe UI when `@font-face`, the font file, or the Trunk copy
     directive disagree; looks "merely slightly wrong", nobody files it
     (office-perf-web, op-edag.1).
  2. `off-ramp-font-size.md` — automated (typography). Ad-hoc `text-[13px]`
     style sizes off the 6-step ramp; breaks rhythm and WCAG 1.4.4 scaling
     when px-pinned.
  3. `ad-hoc-shadow.md` — automated (depth). A `shadow-md` grabbed from
     Tailwind instead of the declared elevation set; cards stop reading as
     one product (4Ease parity work).
  4. `undeclared-radius.md` — automated (shape). 8px vs 15px card corners is
     most of what makes two apps look unrelated (office-perf-web,
     op-edag.2).
  5. `hand-rolled-button.md` — automated (component-drift,
     `button-without-btn`). A raw `<button>` styled inline instead of
     `Button`; loses states, focus ring, theme.
  6. `dead-daisyui4-classes.md` — automated (this repo's `test-daisyui5`
     gate, not ldui-audit). `.form-control`/`.label-text` are no-ops in
     daisyUI 5; layout silently collapses to inline (ldui-mai.3, 206 sites).
  7. `default-component-not-specced.md` — **rulebook-only** (judgment). A
     screen ships with default variants because nobody selected the
     intended color/size/style props against the design spec; checkable
     only against the design intent, not computed styles. Manual check:
     compare against the design reference (mockup/Figma) per screen.
- [ ] **Step 3: Commit** — `docs(visual-quality): seed the defect-pattern rulebook`

---

### Task 10: The `ldui-visual-quality` user skill

**Files:**
- Create: `C:\Users\david\.claude\skills\ldui-visual-quality\SKILL.md`

- [ ] **Step 1: Write the skill** (shape mirrors `app-testing-methodology`:
      thin router to the canonical docs, not a restatement):

```markdown
---
name: ldui-visual-quality
description: Use when building, changing, or reviewing UI in any app that consumes leptos-daisyui-rs (EUC, office-perf-web, LLM-Wiki, Rust-DeskApp web faces) — routes to the shared visual-quality rulebook and the ldui-audit suite instead of ad-hoc visual checking. Triggers on "visual quality", "UI consistency", "why does this page look off", new-page reviews, and pre-merge UI review.
---

# ldui-visual-quality

The visual-quality rulebook lives at
`C:\dev\leptos-daisyui-rs\doc\visual-quality\` (README indexes the defect
patterns). The automated half is the `ldui-audit` crate (same repo,
`audit/`).

## Steps
1. Read the rulebook README + any entry matching the symptom.
2. If the app has a style-audit test wired (search its tests/ for
   `audit_page`), run it and read the report before eyeballing.
3. For rulebook-only patterns (e.g. default-component-not-specced), check
   manually against the design reference.
4. NEW defect pattern found? Add a rulebook entry (template in README)
   in the same session — that is how the check set grows. If it is
   mechanically checkable, file a bead in leptos-daisyui-rs to graduate
   it into a sweep family.
5. An app not yet wired? Follow the pilot recipe in the plan
   (`docs/superpowers/plans/2026-08-08-visual-quality-checks.md`, Task 11).

## Hard rules
- Never hand-roll a new visual check in an app repo — extend the rulebook
  / ldui-audit instead (framework-purity rule).
- Ratchet ceilings only ever go down without justification.
- A new sweep rule needs a negative control (inject → caught → revert).
```

- [ ] **Step 2: Commit** (the skill file lives outside this repo — no repo
      commit; just verify the file exists and the skill loads with
      `/ldui-visual-quality` next session).

---

### Task 11: Pilot — wire office-perf-web (`C:\dev\4iiz-office`)

**Files (all in `C:\dev\4iiz-office`):**
- Modify: `crates/office-perf-web/Cargo.toml` (dev-deps + feature)
- Modify: `crates/office-perf-web/src/main.rs` (install_test_mode)
- Create: `crates/office-perf-web/tests/style_audit.rs`
- Modify: `Makefile.toml` (a `test-style` task serving trunk + running the test)

**Interfaces:**
- Consumes: `ldui_audit::{AuditConfig, harness_at, audit_page, from_ui_tokens, ShadowSpec, Ceiling, check_ceilings}`, `leptos_daisyui_rs` feature `test-mode`.

- [ ] **Step 1: Dev-deps.** In `crates/office-perf-web/Cargo.toml`:

```toml
[dev-dependencies]
ldui-audit = { path = "../../../leptos-daisyui-rs/audit" }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
```

and enable the feature on the existing `leptos-daisyui-rs` dependency line:
`features = [..., "test-mode"]`.

- [ ] **Step 2: Test-mode in `main()`** — before mount:

```rust
if leptos_daisyui_rs::test_mode::install_test_mode() {
    // office-perf's own debug bridge, if/when it grows one, installs here.
}
```

- [ ] **Step 3: The profile + pages.** In `tests/style_audit.rs`:

```rust
use ldui_audit::{audit_page, from_ui_tokens, AuditConfig, Ceiling, ShadowSpec, check_ceilings};

/// 4Ease parity values, measured in the running 4Ease page (op-edag):
/// these are the DELIBERATE deviations; everything else inherits ui-tokens.
fn office_profile() -> ldui_audit::StyleProfile {
    from_ui_tokens("Manrope")
        .body_weight(500)
        .radii([15.0, 8.0, 999.0, 6.0]) // card, field, pill, global-search
        .shadows([
            ShadowSpec::new(0.0, 1.0, 15.0, 0.05),  // card
            ShadowSpec::new(0.0, 4.0, 12.0, 0.04),  // raised
            ShadowSpec::new(15.0, 4.0, 24.0, 0.03), // panel
        ])
}

fn cfg() -> AuditConfig {
    // Port from crates/office-perf-web/Trunk.toml [serve] (verified 8081).
    AuditConfig::new("http://127.0.0.1:8081")
}
```

Enumerate routes (`rg "path=" crates/office-perf-web/src` or its router
module), pick 4 spanning complexity (a login/landing page, the work queue
table, a detail/form screen, the busiest dashboard), and write one
`#[ignore]`d `#[tokio::test]` per page in the Task 7 shape (sanity →
overlap hard-fail → `check_ceilings`).

- [ ] **Step 4: First run + ceilings.** `trunk serve` in
      `crates/office-perf-web` (background), run
      `cargo test -p office-perf-web --test style_audit -- --ignored`,
      copy each page's per-family counts into its `Ceiling` list, re-run to
      green. Component-drift counts on a real app WILL be non-zero — that
      is the ratchet's baseline, not a blocker.
- [ ] **Step 5: `Makefile.toml` task** — add a `test-style` task mirroring
      the repo's existing serve-then-test orchestration pattern (search
      `Makefile.toml` for the existing trunk-serving test task and copy its
      shape, swapping in the `--test style_audit` run).
- [ ] **Step 6: Commit in 4iiz-office** —
      `test(web): ldui-audit style audit wired (4Ease profile, 4 pages, ratcheted)`

---

### Task 12: Pilot proof — break-and-revert

**Files:** none kept (temporary edits, reverted)

- [ ] **Step 1: Typography control.** In office-perf-web's `input.css`,
      comment out the Manrope `@font-face` block, re-run the style audit:
      the typography family must light up on every text element (fallback
      family ≠ "Manrope"). This is the exact regression `font_pipeline.rs`
      warns cannot be caught statically. Restore, re-run, green.
- [ ] **Step 2: Depth control.** Change `--shadow-card` in `input.css` to
      `0 9px 9px rgb(0 0 0 / 0.5)`, re-run: depth family fires on cards.
      Restore, green.
- [ ] **Step 3: Record the evidence** — paste both failing `describe()`
      outputs into the bead tracking the pilot (comment), so the "seen
      failing" proof survives.
- [ ] **Step 4: Commit** (only if any test/ceiling text was improved by the
      exercise; the CSS edits themselves are reverted).

---

## Amendment (2026-08-08, mid-execution)

The concurrent PixelProof session closed `PixelProof-5af` having landed far
more than the rule model (commits `088519f..d771f66`): a **generated
computed-style sweep** (`sweep_js(profile, &SweepOptions)`, ported and
generalized from this repo's `tests/common/layout_audit.rs`), engine family
constants `family::{OVERLAP, GRID, INTERNAL}`, a `verify()`
overlap-hard-fail + ceilings gate, `StyleProfile::line_ramp`,
`ShadowSpec::with_spread`, and a `web` module (default `web` feature:
`WebAuditConfig`, `harness_at`, `wait_for_selector`, `run_sweep`, all
Result-based) — with 8 browser negative controls proving the generic
families in the engine's own test suite. Consequences:

- **Task 3 (harness plumbing): SUPERSEDED** by `web::WebAuditConfig` +
  `harness_at` (env-var override, query suffix, ready selectors, isolated
  profile, 60 s selector budget — all present with the proven defaults).
- **Task 4 (box-shadow parser): SUPERSEDED** — the generated sweep judges
  shadows in-page; no Rust-side computed-string parsing is needed.
- **Task 5 (sweep + judge): SUPERSEDED** — `sweep_js` covers typography /
  shape / depth / spacing / overlap / grid / internal, judged in-page
  against the embedded profile JSON.
- **Task 2 (amended):** additionally sets `.line_ramp(LINE_RAMP)` from
  `ui_tokens`; absorbs the ldui-flavored web defaults
  (`ldui_web_config(base_url)` — `?pp-freeze=1` suffix +
  `style[data-pixelproof="freeze"]` ready selector) and extends the crate
  re-exports (`verify`, `sweep_js`, `SweepOptions`, `web::*`).
- **Task 6 (amended):** the drift heuristics become a standalone ldui JS
  sweep (`DRIFT_JS` + `run_drift(&Harness) -> Vec<Violation>`) merged into
  the engine report; the four rules are unchanged.
- **Task 7 (amended):** `audit_page(h, profile, opts)` composes
  `web::run_sweep` + `run_drift` via `AuditReport::push`; the demo suite
  migration re-points `tests/common` at the engine web module and engine
  family constants; negative controls narrow to component-drift plus one
  engine-family sanity injection (the engine already proves its families).
- Tasks 1, 8–12 unchanged (1 and 8 were complete before the amendment).

Amended briefs live beside the originals in the SDD workspace as
`task-{2,6,7}-brief-amended.md` and govern over the sections above.

## Self-review notes

- Spec coverage: §1a landed externally (noted in header); §1b Tasks 1–7;
  §2 families Tasks 5–6 (+ layout via migration in 7); §3 Task 2; §4 Task 8;
  §5 Tasks 9–10; §6 Tasks 11; §7 negative controls Task 7 + break-and-revert
  Task 12; §8 ratchet posture in Tasks 7 & 11 (overlap hard-fail preserved).
- Open questions from the spec resolved here: ceilings live in the
  consumer's test file (proven pattern); shadow comparison via
  `ShadowSpec::matches` epsilon (engine, landed); drift heuristics tuned on
  the demo in Task 7 before the pilot in Task 11.
