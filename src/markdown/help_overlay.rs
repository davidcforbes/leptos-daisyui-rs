//! Keyboard-shortcut discovery overlay.
//!
//! Opened via the "?" toolbar button or `Ctrl+/` (`Cmd+/` on macOS).
//! Lists every shortcut the editor responds to.  Closes on Escape, on a
//! click outside the panel, or by clicking the close button.

use leptos::ev;
use leptos::prelude::*;

/// One row in the shortcut table.
struct ShortcutRow {
    keys: &'static str,
    description: &'static str,
}

const FORMATTING: &[ShortcutRow] = &[
    ShortcutRow {
        keys: "Ctrl/⌘ + B",
        description: "Bold",
    },
    ShortcutRow {
        keys: "Ctrl/⌘ + I",
        description: "Italic",
    },
    ShortcutRow {
        keys: "Ctrl/⌘ + K",
        description: "Link",
    },
];

const EDITING: &[ShortcutRow] = &[
    ShortcutRow {
        keys: "Tab",
        description: "Indent (or indent selection)",
    },
    ShortcutRow {
        keys: "Shift + Tab",
        description: "Dedent (or dedent selection)",
    },
    ShortcutRow {
        keys: "Enter",
        description: "Continue list (empty item exits)",
    },
];

const FIND_REPLACE: &[ShortcutRow] = &[
    ShortcutRow {
        keys: "Ctrl/⌘ + F",
        description: "Find",
    },
    ShortcutRow {
        keys: "Ctrl/⌘ + H",
        description: "Find and Replace",
    },
    ShortcutRow {
        keys: "Enter (in find)",
        description: "Next match",
    },
    ShortcutRow {
        keys: "Shift + Enter (in find)",
        description: "Previous match",
    },
];

const DOCUMENT: &[ShortcutRow] = &[ShortcutRow {
    keys: "Ctrl/⌘ + S",
    description: "Save (fires on_save callback)",
}];

const IMAGES: &[ShortcutRow] = &[
    ShortcutRow {
        keys: "Paste image",
        description: "Uploads via on_asset_upload and inserts",
    },
    ShortcutRow {
        keys: "Drag-drop image",
        description: "Same as paste — drop onto the editor",
    },
];

const GLOBAL: &[ShortcutRow] = &[
    ShortcutRow {
        keys: "Ctrl/⌘ + /",
        description: "Show this overlay",
    },
    ShortcutRow {
        keys: "Esc",
        description: "Close overlay / find bar",
    },
];

/// Keyboard-shortcut help overlay.  Bound to a single boolean signal so
/// the parent editor can toggle it via the toolbar button and Ctrl+/.
#[component]
pub fn HelpOverlay(open: RwSignal<bool>) -> impl IntoView {
    let close = move || open.set(false);
    let on_backdrop_click = move |_ev: ev::MouseEvent| close();
    let stop_propagation = move |ev: ev::MouseEvent| ev.stop_propagation();
    let show = move || open.get();

    view! {
        <Show when=show>
            <div class="lds-help-backdrop" on:click=on_backdrop_click>
                <div class="lds-help-dialog" on:click=stop_propagation>
                    <div class="lds-help-header">
                        <span class="lds-help-title">"Keyboard Shortcuts"</span>
                        <button
                            class="btn btn-xs btn-ghost"
                            title="Close (Esc)"
                            on:click=move |_| close()
                        >
                            "✕"
                        </button>
                    </div>
                    <div class="lds-help-body">
                        {section("Formatting", FORMATTING)}
                        {section("Editing", EDITING)}
                        {section("Find / Replace", FIND_REPLACE)}
                        {section("Document", DOCUMENT)}
                        {section("Images", IMAGES)}
                        {section("Help", GLOBAL)}
                    </div>
                </div>
            </div>
        </Show>
    }
}

fn section(title: &'static str, rows: &'static [ShortcutRow]) -> impl IntoView {
    view! {
        <div class="lds-help-section">
            <div class="lds-help-section-title">{title}</div>
            <table class="lds-help-table">
                {rows.iter().map(|row| view! {
                    <tr>
                        <td class="lds-help-keys"><kbd>{row.keys}</kbd></td>
                        <td class="lds-help-desc">{row.description}</td>
                    </tr>
                }).collect::<Vec<_>>()}
            </table>
        </div>
    }
}
