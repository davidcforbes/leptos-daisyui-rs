//! Read-only PropertyList — a simple `(label, value)` table styled with
//! DaisyUI tokens. Used by the Audit page detail section to render the
//! lock-icon-flagged immutable fields of an audit row.
//!
//! The component intentionally accepts owned `String` values (not
//! `&'static str`) so it can render dynamic, per-row data fetched from
//! GraphQL. Pass `locked = true` for the audit page to surface a small
//! lock glyph next to each value reinforcing the append-only contract.

use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct PropertyEntry {
    pub label: String,
    pub value: String,
    /// Optional badge class to wrap the value (e.g. for action chips).
    pub badge_class: Option<String>,
    /// Render the value in a monospace style (good for UUIDs / timestamps).
    pub mono: bool,
}

impl PropertyEntry {
    pub fn new<L: Into<String>, V: Into<String>>(label: L, value: V) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            badge_class: None,
            mono: false,
        }
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn with_badge<S: Into<String>>(mut self, cls: S) -> Self {
        self.badge_class = Some(cls.into());
        self
    }
}

/// Render a list of `(label, value)` rows. When `locked` is true, each
/// value cell is suffixed with a lock glyph and the row is given a
/// muted border to communicate immutability.
#[component]
pub fn PropertyList(
    entries: Vec<PropertyEntry>,
    /// Suffix every value with a lock icon (used on the Audit page).
    #[prop(default = false)]
    locked: bool,
) -> impl IntoView {
    view! {
        <div class="divide-y divide-base-200 rounded-lg border border-base-200 bg-base-100">
            {entries.into_iter().map(|p| {
                let label = p.label;
                let value = p.value;
                let badge_cls = p.badge_class;
                let mono = p.mono;
                view! {
                    <div class="flex items-start justify-between gap-4 px-3 py-2">
                        <span class="text-xs text-base-content/60 shrink-0 mt-0.5">{label}</span>
                        <span class="text-right break-all flex items-center gap-1.5 justify-end">
                            {match badge_cls {
                                Some(cls) => view! { <span class=cls>{value.clone()}</span> }.into_any(),
                                None if mono => view! {
                                    <span class="text-sm font-mono text-base-content/80">{value.clone()}</span>
                                }.into_any(),
                                None => view! {
                                    <span class="text-sm font-medium">{value.clone()}</span>
                                }.into_any(),
                            }}
                            {locked.then(|| view! {
                                <span class="text-[10px] text-base-content/40" title="Append-only field">"\u{1F512}"</span>
                            })}
                        </span>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
