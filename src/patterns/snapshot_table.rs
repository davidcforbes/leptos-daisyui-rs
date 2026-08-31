//! Atomic runtime state for opinionated client-snapshot table pages.
//!
//! The pure reducer (generations, revisions, requests, unmintable handles,
//! keyed action feedback, and the generation-preserving displayed-snapshot
//! delta API) lives in the shared `snapshot-core` crate so the desktop
//! (d2d-ui), web (this crate), and terminal (tui-daisy) faces consume one
//! implementation instead of three copies. This module re-exports its
//! public surface unchanged, so every existing `leptos_daisyui_rs::patterns`
//! call site keeps compiling without change. See `ldui-gzmf` and
//! `../Rust-DeskApp/crates/snapshot-core/PROVENANCE.md` for the fidelity
//! proof against this crate's prior local implementation.
//!
//! Render helpers coupled to this face (`SnapshotTablePage` and friends in
//! `snapshot_table_page.rs`, plus the keyed-action renderer in
//! `action_feedback.rs`) stay in this crate: `snapshot-core` is render-free
//! and serde-free by construction, which is `tui-daisy`'s condition as a
//! third consumer.

pub use snapshot_core::{
    LocalResultSummary, PageStatePanelKind, SnapshotAccess, SnapshotActionDisposition,
    SnapshotActionHandle, SnapshotActionStartError, SnapshotData, SnapshotDataError,
    SnapshotDeltaDisposition, SnapshotDeltaHandle, SnapshotDeltaStartError, SnapshotGeneration,
    SnapshotLocalRowProjection, SnapshotRenderDecision, SnapshotRequestError,
    SnapshotRequestHandle, SnapshotTablePhase, SnapshotTableState, SnapshotTableView,
    SnapshotTransitionDisposition,
};
