//! Floating find/replace bar that anchors over the editor.
//!
//! Driven by a single `RwSignal<FindState>` so the parent editor can
//! open/close it via keyboard shortcuts (Ctrl+F / Ctrl+H / Escape).
//!
//! Textareas can't display per-match highlighting (no per-character
//! styling), so the UX mirrors the standard browser-editor convention:
//! the current match is shown via the textarea's native selection, and
//! Enter / Shift-Enter advance through matches.

use leptos::ev;
use leptos::prelude::*;
use web_sys::HtmlTextAreaElement;

use super::find::{find_all_matches, replace_all};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FindMode {
    #[default]
    FindOnly,
    FindReplace,
}

#[derive(Debug, Clone, Default)]
pub struct FindState {
    pub open: bool,
    pub mode: FindMode,
    pub query: String,
    pub replace: String,
    pub case_sensitive: bool,
    /// 0-based index into the currently-derived `matches` vec.
    pub current_index: Option<usize>,
}

#[component]
pub fn FindOverlay(
    state: RwSignal<FindState>,
    source: RwSignal<String>,
    textarea: NodeRef<leptos::html::Textarea>,
) -> impl IntoView {
    // Recompute matches whenever source or query or case-sensitivity changes.
    let matches = Signal::derive(move || {
        let s = state.get();
        if s.query.is_empty() {
            return Vec::new();
        }
        find_all_matches(&source.get(), &s.query, s.case_sensitive)
    });

    // Jump the textarea selection to the match at `idx`.
    let goto = move |idx: usize| {
        let m = matches.get_untracked();
        if let Some(range) = m.get(idx).cloned() {
            state.update(|s| s.current_index = Some(idx));
            if let Some(el) = textarea.get_untracked() {
                let ta: &HtmlTextAreaElement = el.as_ref();
                let _ = ta.focus();
                let _ = ta.set_selection_range(range.start as u32, range.end as u32);
            }
        }
    };

    let on_next = move || {
        let m = matches.get_untracked();
        if m.is_empty() {
            return;
        }
        let cur = state.with_untracked(|s| s.current_index.unwrap_or(usize::MAX));
        let next = if cur == usize::MAX {
            0
        } else {
            (cur + 1) % m.len()
        };
        goto(next);
    };
    let on_prev = move || {
        let m = matches.get_untracked();
        if m.is_empty() {
            return;
        }
        let cur = state.with_untracked(|s| s.current_index.unwrap_or(0));
        let prev = if cur == 0 { m.len() - 1 } else { cur - 1 };
        goto(prev);
    };

    let on_replace_one = move || {
        let m = matches.get_untracked();
        let snapshot = state.get_untracked();
        if m.is_empty() {
            return;
        }
        let idx = snapshot.current_index.unwrap_or(0);
        let Some(range) = m.get(idx) else { return };
        let value = source.get_untracked();
        let new_value = format!(
            "{}{}{}",
            &value[..range.start],
            snapshot.replace,
            &value[range.end..]
        );
        source.set(new_value.clone());
        if let Some(el) = textarea.get_untracked() {
            let ta: &HtmlTextAreaElement = el.as_ref();
            ta.set_value(&new_value);
        }
        // After replacement, the slot that was `idx` no longer has a
        // match; matches[idx+1..] shifted to matches[idx..].  So `idx`
        // already points to the next match — except when idx == new_len,
        // in which case wrap to 0.  Wait until the derived signal updates
        // before jumping.
        let new_matches = find_all_matches(&new_value, &snapshot.query, snapshot.case_sensitive);
        if new_matches.is_empty() {
            state.update(|s| s.current_index = None);
        } else {
            let next = if idx >= new_matches.len() { 0 } else { idx };
            state.update(|s| s.current_index = Some(next));
            if let Some(el) = textarea.get_untracked() {
                let ta: &HtmlTextAreaElement = el.as_ref();
                let r = &new_matches[next];
                let _ = ta.set_selection_range(r.start as u32, r.end as u32);
            }
        }
    };

    let on_replace_all = move || {
        let snapshot = state.get_untracked();
        if snapshot.query.is_empty() {
            return;
        }
        let result = replace_all(
            &source.get_untracked(),
            &snapshot.query,
            &snapshot.replace,
            snapshot.case_sensitive,
        );
        source.set(result.new_source.clone());
        if let Some(el) = textarea.get_untracked() {
            let ta: &HtmlTextAreaElement = el.as_ref();
            ta.set_value(&result.new_source);
        }
        state.update(|s| s.current_index = None);
    };

    let on_close = move || {
        state.update(|s| {
            s.open = false;
            s.current_index = None;
        });
        if let Some(el) = textarea.get_untracked() {
            let ta: &HtmlTextAreaElement = el.as_ref();
            let _ = ta.focus();
        }
    };

    // Re-find when the user types in the search box.
    let on_query_input = move |ev: ev::Event| {
        let v = event_target_value(&ev);
        state.update(|s| {
            s.query = v;
            s.current_index = None;
        });
        // Auto-select first match.
        let m = matches.get_untracked();
        if let Some(first) = m.first() {
            if let Some(el) = textarea.get_untracked() {
                let ta: &HtmlTextAreaElement = el.as_ref();
                let _ = ta.set_selection_range(first.start as u32, first.end as u32);
            }
            state.update(|s| s.current_index = Some(0));
        }
    };

    let on_replace_input = move |ev: ev::Event| {
        let v = event_target_value(&ev);
        state.update(|s| s.replace = v);
    };

    // Enter in the find box → next, Shift+Enter → prev, Esc → close.
    let on_find_keydown = move |ev: ev::KeyboardEvent| match ev.key().as_str() {
        "Enter" => {
            ev.prevent_default();
            if ev.shift_key() {
                on_prev();
            } else {
                on_next();
            }
        }
        "Escape" => {
            ev.prevent_default();
            on_close();
        }
        _ => {}
    };

    let show = move || state.with(|s| s.open);
    let show_replace = move || state.with(|s| matches!(s.mode, FindMode::FindReplace));

    let count_label = move || {
        let m = matches.get();
        if m.is_empty() {
            "0 of 0".to_string()
        } else {
            let cur = state.with(|s| s.current_index.unwrap_or(0));
            format!("{} of {}", cur + 1, m.len())
        }
    };

    let case_sensitive = move || state.with(|s| s.case_sensitive);

    view! {
        <Show when=show>
            <div class="lds-find-overlay">
                <div class="lds-find-row">
                    <input
                        type="text"
                        class="lds-find-input"
                        placeholder="Find"
                        prop:value=move || state.with(|s| s.query.clone())
                        on:input=on_query_input
                        on:keydown=on_find_keydown
                    />
                    <span class="lds-find-count">{count_label}</span>
                    <button
                        class="btn btn-xs btn-ghost"
                        title="Previous match (Shift+Enter)"
                        on:click=move |_| on_prev()
                    >
                        "↑"
                    </button>
                    <button
                        class="btn btn-xs btn-ghost"
                        title="Next match (Enter)"
                        on:click=move |_| on_next()
                    >
                        "↓"
                    </button>
                    <button
                        class="btn btn-xs btn-ghost"
                        title="Case sensitive"
                        class:em-find-toggle-on=case_sensitive
                        on:click=move |_| {
                            state.update(|s| s.case_sensitive = !s.case_sensitive);
                        }
                    >
                        "Aa"
                    </button>
                    <button
                        class="btn btn-xs btn-ghost"
                        title="Close (Esc)"
                        on:click=move |_| on_close()
                    >
                        "✕"
                    </button>
                </div>
                <Show when=show_replace>
                    <div class="lds-find-row">
                        <input
                            type="text"
                            class="lds-find-input"
                            placeholder="Replace"
                            prop:value=move || state.with(|s| s.replace.clone())
                            on:input=on_replace_input
                        />
                        <button
                            class="btn btn-xs btn-ghost"
                            title="Replace current match"
                            on:click=move |_| on_replace_one()
                        >
                            "Replace"
                        </button>
                        <button
                            class="btn btn-xs btn-ghost"
                            title="Replace all matches"
                            on:click=move |_| on_replace_all()
                        >
                            "Replace All"
                        </button>
                    </div>
                </Show>
            </div>
        </Show>
    }
}
