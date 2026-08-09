//! # ldui-audit
//!
//! The web surface of the shared visual-quality rule model
//! (`pixelproof-style-audit`): CDP harness plumbing, the in-page sweep over
//! computed styles, daisyUI component-drift heuristics, and `ui-tokens`
//! profile defaults. Consumer apps add this as a dev-dependency, declare a
//! `StyleProfile`, list pages, and assert ratcheted ceilings.
//!
//! Design: `docs/superpowers/specs/2026-08-08-visual-quality-checks-design.md`.

pub mod drift;
pub mod profile;
pub mod web_config;

pub use drift::{DriftReport, drift_js, run_drift};
pub use pixelproof_style_audit::web;
pub use pixelproof_style_audit::{
    AuditReport, Ceiling, FamilyReport, RatchetOutcome, ShadowSpec, StyleProfile, SweepOptions,
    Violation, check_ceilings, family, sweep_js, verify,
};
pub use pixelproof_web::{Harness, HarnessConfig, ViewportSize};
pub use profile::from_ui_tokens;
pub use web_config::ldui_web_config;
