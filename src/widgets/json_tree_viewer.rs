//! Recursive collapsible JSON tree viewer (~50 LOC).
//!
//! Used by the Audit page to render the `before_value` / `after_value`
//! snapshots without committing to a typed schema. Accepts a `String`
//! payload that is parsed once on mount; on parse failure falls back to a
//! pre-formatted code block so the operator still sees something.

use leptos::prelude::*;
use serde_json::Value;

/// Renders a JSON string as a nested, collapsible tree. Pass `collapsed`
/// = true to render the root closed by default.
#[component]
pub fn JsonTreeViewer(
    /// Raw JSON payload as a string. May be empty — empty strings render
    /// as `(empty)`.
    json: String,
    #[prop(default = false)] collapsed: bool,
) -> impl IntoView {
    if json.trim().is_empty() {
        return view! {
            <span class="text-xs text-base-content/50">"(empty)"</span>
        }
        .into_any();
    }

    match serde_json::from_str::<Value>(&json) {
        Ok(v) => view! { <JsonNode value=v depth=0 collapsed=collapsed/> }.into_any(),
        Err(_) => view! {
            <pre class="text-xs font-mono whitespace-pre-wrap break-all bg-base-200 rounded p-2">
                {json}
            </pre>
        }
        .into_any(),
    }
}

/// The array/object disclosure toggle is deliberately unstyled (no `.btn` --
/// it is inline text, not a button-shaped control), so it carries
/// `data-pressable="true"` for the `ldui-audit` `button-without-btn` drift
/// rule (`ldui-2e7a`).
#[component]
fn JsonNode(value: Value, depth: usize, #[prop(default = false)] collapsed: bool) -> impl IntoView {
    let open = RwSignal::new(!collapsed || depth == 0);
    let indent = format!("padding-left: {}px", depth * 12);
    match value {
        Value::Null => view! {
            <span class="text-xs font-mono text-base-content/50" style=indent>"null"</span>
        }
        .into_any(),
        Value::Bool(b) => view! {
            <span class="text-xs font-mono text-info" style=indent>{b.to_string()}</span>
        }
        .into_any(),
        Value::Number(n) => view! {
            <span class="text-xs font-mono text-warning" style=indent>{n.to_string()}</span>
        }
        .into_any(),
        Value::String(s) => view! {
            <span class="text-xs font-mono text-success break-all" style=indent>
                {format!("\"{}\"", s)}
            </span>
        }
        .into_any(),
        Value::Array(arr) => {
            let len = arr.len();
            view! {
                <div style=indent.clone()>
                    <button
                        type="button"
                        data-pressable="true"
                        class="text-xs font-mono cursor-pointer hover:underline"
                        on:click=move |_| open.update(|o| *o = !*o)
                    >
                        {move || if open.get() { "\u{25BC}" } else { "\u{25B6}" }.to_string()}
                        {format!(" [{} items]", len)}
                    </button>
                    <Show when=move || open.get()>
                        <div>
                            {arr.iter().enumerate().map(|(i, v)| {
                                view! {
                                    <div class="flex gap-1">
                                        <span class="text-xs font-mono text-base-content/50">{format!("{}:", i)}</span>
                                        <JsonNode value=v.clone() depth=depth+1 collapsed=true/>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Show>
                </div>
            }.into_any()
        }
        Value::Object(map) => {
            let len = map.len();
            view! {
                <div style=indent.clone()>
                    <button
                        type="button"
                        data-pressable="true"
                        class="text-xs font-mono cursor-pointer hover:underline"
                        on:click=move |_| open.update(|o| *o = !*o)
                    >
                        {move || if open.get() { "\u{25BC}" } else { "\u{25B6}" }.to_string()}
                        {format!(" {{{} keys}}", len)}
                    </button>
                    <Show when=move || open.get()>
                        <div>
                            {map.iter().map(|(k, v)| {
                                let key = k.clone();
                                view! {
                                    <div class="flex gap-1 items-start">
                                        <span class="text-xs font-mono text-primary">{format!("\"{}\":", key)}</span>
                                        <JsonNode value=v.clone() depth=depth+1 collapsed=true/>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Show>
                </div>
            }.into_any()
        }
    }
}
