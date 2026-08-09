# Framework-Level Visual Quality Checks — Design

**Date:** 2026-08-08
**Status:** Approved design, pre-implementation
**Approach:** Hybrid — reusable audit crate + growing rulebook/skill (approach A)

## Problem

Across 5+ web applications consuming this framework, the same classes of visual
inconsistency keep appearing: wrong or fallback fonts, hand-rolled badges and
labels, off-spec buttons, ad-hoc shadows, divergent DataTable styling, and
design specs simply not applied (a page ships with default component styling
because nobody selected the intended variant). Each app has been reinventing
its own partial checks — `office-perf-web` alone carries five bespoke static
tests (`font_pipeline.rs`, `shape_depth_tokens.rs`, `theme_contract.rs`,
`focus_ring.rs`, `workspace_layout_guard.rs`) — and those static checks
explicitly cannot prove a declaration *reaches* an element through the CSS
cascade. Only a browser can.

Meanwhile this repo already owns the browser half: a proven
`pixelproof-web`-based CDP harness (`tests/common/`) running three suites
against the demo (SSIM visual baselines, the ratcheted layout audit,
reactivity). It is not reusable by consumer apps today because it is a private
test-support module hard-wired to the demo.

## Goals

1. Consumer apps get a standard visual quality audit by adding a dev-dependency
   and writing one small test file — no per-app harness reinvention.
2. Legitimate per-app divergence (e.g. office-perf's 4Ease parity: Manrope,
   15px card radius, three named shadows) is *declared*, and the audit verifies
   the rendered app against the declaration. Undeclared defaults fail.
3. The check set grows: new defect patterns land as written rules first and
   graduate to automated checks when mechanically checkable.
4. Adoption never blocks an app on day one — ratcheted ceilings block getting
   *worse*, not being imperfect.

## Non-goals

- Publishing to crates.io (this repo is path-dep-only; the audit crate follows).
- Auditing non-Leptos / non-sibling apps (a standalone CLI was considered and
  rejected — it loses the typed link to `ui-tokens`).
- Replacing app-local static CSS tests. They are the fast no-browser half and
  stay; the audit is the "did it reach the element" half.
- SSIM baseline management for consumer apps (pixelproof-web already offers
  that directly; this crate is about rule checks, not screenshot baselines).

## Deliverables

### 1. `ldui-audit` crate (new workspace member, `audit/`)

Generalizes `tests/common/` into a public library, consumed as a path
dependency by C:\dev siblings (same convention as `pixelproof-web` itself).

- **Harness plumbing, made configurable.** `harness_at`, `wait_for_selector`,
  isolated Chrome profile, settle/mount-poll logic move in from
  `tests/common/mod.rs`. Base URL, mount selector (currently hard-coded
  `main`), and env-var names become `AuditConfig` fields with the current
  values as defaults.
- **The sweep + report types.** The in-page layout sweep JS and
  `AuditReport`/`Violation` types (`tests/common/layout_audit.rs`) move in
  unchanged and are extended with the new families below. The report keeps the
  `selector / value / detail` violation shape, `describe()` formatting,
  `scanned > 0` sanity check, and explicit `truncated` flag.
- **Entry point.** `audit_page(&harness, &profile) -> QualityReport`, grouping
  violations by rule family. This repo's own `tests/` become the first
  consumer of the crate, so the demo suites and the public library are the
  same code — nothing maintained twice.

### 2. Rule families (v1)

All checks read **computed styles over the rendered DOM**, compared against the
app's `StyleProfile`:

| Family | Checks |
|---|---|
| Typography | `font-family` resolves to the declared family (catches silent fallback, e.g. to Segoe UI); `font-size` ∈ declared type ramp; body `font-weight` explicit and expected |
| Shape | `border-radius` on cards / buttons / fields ∈ declared radius set |
| Depth | `box-shadow` ∈ declared shadow set — no ad-hoc elevations |
| Spacing/layout | the existing overlap / off-grid / internal≤external sweep, unchanged |
| Component drift | heuristics for "hand-rolled or default where a framework component was intended": `<button>` without `.btn`; badge/label look-alikes (small rounded colored inline boxes) not using `.badge`/`.label`; `<table>` without framework table classes; inputs outside a `Field`/fieldset structure |

Component drift is heuristic by nature: it **reports against ratcheted
ceilings** rather than hard-failing, so a legitimate exception cannot block a
gate. Defects a computed-style check cannot see (a page that *chose* a
wrong-but-valid component; look-and-feel judgment calls) live in the rulebook
(deliverable 5) as reviewer instructions, not in code.

### 3. `StyleProfile`

The reconciliation between "shared design system" and "apps legitimately
diverge":

```rust
let profile = StyleProfile::from_ui_tokens()   // ramp, spacing, radii, shadows, family
    .font_family("Manrope")
    .radii([15.0, 8.0, 999.0])
    .shadows([CARD_4EASE, RAISED_4EASE, PANEL_4EASE]);
```

- Defaults come from `ui_tokens` **at compile time**, so a profile cannot
  drift from the token crate. An app writes down only its deliberate
  deviations.
- An app that forgot to apply its design specs renders framework/daisyUI
  defaults; defaults contradicting the declared profile are exactly what
  fails. This encodes the "got the default component / forgot the design
  spec" failure mode directly.
- `ui-tokens` lives on the sibling's DEFAULT branch rule (see CLAUDE.md's
  `sibling-tokens` gate) — profile defaults must only reference items on that
  branch.

### 4. `test-mode` feature in `leptos-daisyui-rs`

The harness depends on `?pp-freeze=1` (CSS animation kill-switch + the
`window.__APP_DEBUG__` state oracle), currently implemented privately in
`demo/src/test_mode.rs`. That module moves into the library behind a
`test-mode` cargo feature. A consumer app enables the feature in its dev/test
build and calls `install_test_mode()` in `main()` before mount — no per-app
copy of the freeze logic. The demo migrates to the library version.

### 5. Growing rulebook + skill

- **`doc/visual-quality/`** in this repo: `README.md` index plus one file per
  defect pattern. Each entry records: what the defect looks like (screenshots
  welcome), which app it was observed in, root cause, the manual check, and
  automation status — `rulebook-only` or `automated (family X, since <date>)`.
  New patterns land here first; mechanically-checkable ones graduate into a
  sweep family and the entry is updated to point at the rule. Entries are
  never deleted on automation — the prose is the "why" behind the rule.
- **User skill** `~/.claude/skills/ldui-visual-quality/` in the style of
  `app-testing-methodology`: triggers when building or reviewing UI in any
  ldui-consuming app; instructs the agent to (a) run `ldui-audit` where wired,
  (b) apply rulebook-only checks by eye/browser tools, (c) file newly observed
  patterns back into `doc/visual-quality/`.
- Stays consistent with the PixelProof methodology: this is a Layer A/B
  instantiation for the ldui ecosystem; the rulebook cites
  `PixelProof/docs/methodology/principles.md` rather than restating it.

### 6. Pilot: `office-perf-web` (C:\dev\4iiz-office)

- Add `ldui-audit` + `pixelproof-web` dev-dependencies (sibling-path
  convention), enable `test-mode`, call `install_test_mode()` in test builds.
- Declare its profile: Manrope (weight 500 body), radii {15, 8, 999}, the
  three measured 4Ease shadows.
- Pick ~4 representative pages; set initial ratchet ceilings to the counts
  observed on the first run.
- Its existing static CSS tests stay unchanged.
- Success criterion for the pilot: the audit catches a deliberately-introduced
  violation of each family on a real office-perf page (see §7), and runs in
  that repo's gate.

## Proving the checker

Per the PixelProof methodology ("a test never seen failing is not a test"):

- **Negative controls per family**, extending the pattern already in
  `layout_audit_smoke.rs::sweep_detects_injected_violations`: inject a wrong
  font-size, a rogue shadow, and a classless button into a known-clean page;
  assert each family catches its injection; remove; assert clean again.
- **Break-and-revert on the pilot**: temporarily disable office-perf's Manrope
  `@font-face`, watch the typography family fail (this is the exact "silently
  falls back to Segoe UI" regression its own test file warns about), restore.
- `scanned > 0` and `truncated` carry over so an empty or capped sweep cannot
  read as a pass.

## Failure posture

Identical to the existing layout audit: **overlap remains a hard failure**;
every other family is **ratcheted per page** — committed ceilings, lowered
freely, raised only with justification in the commit message. New rules ship
with ceilings at current-count so adoption is never blocked on existing debt.

## Open questions deferred to implementation planning

- Exact heuristic thresholds for badge/label look-alike detection (tune on the
  demo's 109 components to keep false positives near zero before piloting).
- Whether `QualityReport` ceilings live in the consumer's test file (like
  `PAGES` today) or a committed TOML — start with the test-file constant, the
  proven pattern.
- Shadow matching tolerance (browsers normalize `box-shadow` serialization;
  compare parsed components with an epsilon, not strings — decided in
  principle, exact epsilon at implementation).
