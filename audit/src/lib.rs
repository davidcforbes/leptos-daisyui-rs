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
    AuditReport, Ceiling, FamilyReport, RatchetOutcome, ShadowSpec, StyleProfile, Violation,
    check_ceilings, family,
};
pub use pixelproof_web::{Harness, HarnessConfig, ViewportSize};
