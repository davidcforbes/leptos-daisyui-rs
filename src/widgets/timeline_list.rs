//! Reusable chronological event list (UT2-30 / EUC-rqw6).
//!
//! Renders a vertical timeline using the leptos-daisyui-rs `Timeline` library
//! component, with one row per event: a colored dot, a title, and an optional
//! subtitle / description. Used by:
//!   - Alerts page right rail (UT2-30)
//!   - Investigations page right rail (UT2-32)
//!   - Network Logs page right rail (UT2-28)
//!
//! Composes library components only — no hand-coded daisyUI HTML.
//!
//! Entries are read-only by default. An entry that carries an [`id`] on a
//! list given an `on_activate` callback renders as ONE real `<button>`
//! ([`Pressable`]) whose accessible name is the entry's `action_label` (or
//! its title), so a notes list can open the edit dialog from the entry
//! itself instead of being rebuilt from bare buttons (Office op-j3ffb).
//!
//! [`id`]: TimelineListEntry::id

use leptos::prelude::*;

use crate::components::{
    Pressable, Timeline, TimelineItem, TimelineItemEnd, TimelineItemMiddle, TimelineItemPosition,
};

/// One row in a `TimelineList`.
#[derive(Clone, Debug)]
pub struct TimelineListEntry {
    /// Primary event text — required.
    pub title: String,
    /// Optional secondary line (e.g. timestamp, actor).
    pub subtitle: Option<String>,
    /// Optional third line (e.g. event detail / context).
    pub description: Option<String>,
    /// Free-text severity hint mapped to dot color: `error` | `warn`/`warning`
    /// | `info` (default) | `success` | `muted`.
    pub severity_code: Option<String>,
    /// Stable identity handed to `TimelineList`'s `on_activate` when this
    /// entry is pressed (Office op-j3ffb). `None` -- the default -- renders
    /// the entry read-only even when the list has a callback, so a mixed
    /// list can make only some entries activatable.
    pub id: Option<String>,
    /// Accessible name of the activation control, e.g. `"Edit note from 12
    /// May"`. Falls back to `title` when `None`. Consumer-localized like
    /// every other string here; the framework never mints a verb.
    pub action_label: Option<String>,
}

impl TimelineListEntry {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            description: None,
            severity_code: None,
            id: None,
            action_label: None,
        }
    }

    /// Makes the entry activatable (given a list `on_activate`), handing
    /// `id` to the callback.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Names the activation control for assistive technology; the title is
    /// the fallback.
    pub fn with_action_label(mut self, label: impl Into<String>) -> Self {
        self.action_label = Some(label.into());
        self
    }

    pub fn with_subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    pub fn with_description(mut self, s: impl Into<String>) -> Self {
        self.description = Some(s.into());
        self
    }

    pub fn with_severity(mut self, s: impl Into<String>) -> Self {
        self.severity_code = Some(s.into());
        self
    }
}

fn dot_class(severity_code: Option<&str>) -> &'static str {
    match severity_code {
        Some("error") | Some("critical") => "bg-red-500",
        Some("warn") | Some("warning") => "bg-amber-500",
        Some("success") | Some("ok") => "bg-emerald-500",
        Some("muted") => "bg-base-300",
        _ => "bg-blue-500",
    }
}

/// Whether one entry renders as a control: it needs BOTH an identity to hand
/// back and a list-level callback to hand it to. Either half alone renders
/// the read-only entry every existing caller had (Office op-j3ffb).
pub fn timeline_entry_is_activatable(has_id: bool, has_callback: bool) -> bool {
    has_id && has_callback
}

/// The activation control's accessible name: the entry's `action_label`, or
/// its title when none was given. Never empty for a titled entry.
pub fn timeline_entry_action_name(action_label: Option<&str>, title: &str) -> String {
    match action_label.map(str::trim) {
        Some(label) if !label.is_empty() => label.to_owned(),
        _ => title.trim().to_owned(),
    }
}

/// Classes of the per-entry activation control: a full-width, left-aligned
/// block that keeps the entry's text layout and gains the framework focus
/// ring; the text colour is the entry's own, so an activatable entry reads
/// like its read-only neighbour until it is hovered or focused.
const TIMELINE_ENTRY_ACTION_CLASS: &str =
    "block w-full cursor-pointer rounded-field text-left hover:bg-base-200/60";

/// Vertical chronological timeline. Items are rendered in the order supplied
/// (caller controls newest-first vs. oldest-first). Compact = true to match
/// the right-rail aesthetic.
///
/// With `on_activate`, every entry that carries an [`TimelineListEntry::id`]
/// becomes ONE `<button>` whose accessible name is its `action_label` (or
/// title) and which hands the id to the callback when pressed; its lines
/// render as block `<span>`s because a button takes phrasing content only.
/// Entries without an id, and every entry of a list without the callback,
/// render exactly as before (Office op-j3ffb).
///
/// ### Add to `input.css`
/// ```css
/// @source inline("block w-full cursor-pointer rounded-field text-left hover:bg-base-200/60");
/// ```
#[component]
pub fn TimelineList(
    /// Events to render, top-to-bottom.
    entries: Vec<TimelineListEntry>,
    /// If true (default), use compact spacing suitable for a right rail.
    #[prop(default = true)]
    compact: bool,
    /// Receives the pressed entry's [`TimelineListEntry::id`]. Without it no
    /// entry is a control (Office op-j3ffb).
    #[prop(optional)]
    on_activate: Option<Callback<String>>,
) -> impl IntoView {
    let len = entries.len();
    view! {
        <Timeline compact=compact>
            {entries
                .into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let position = if i == 0 {
                        TimelineItemPosition::Start
                    } else if i + 1 == len {
                        TimelineItemPosition::End
                    } else {
                        TimelineItemPosition::Between
                    };
                    let dot = dot_class(e.severity_code.as_deref());
                    let activation = timeline_entry_is_activatable(e.id.is_some(), on_activate.is_some())
                        .then(|| e.id.clone().zip(on_activate))
                        .flatten();
                    let body = match activation {
                        Some((id, on_activate)) => {
                            let name = timeline_entry_action_name(e.action_label.as_deref(), &e.title);
                            let pressed_id = id.clone();
                            view! {
                                <Pressable
                                    class=TIMELINE_ENTRY_ACTION_CLASS
                                    on_click=Callback::new(move |_| on_activate.run(pressed_id.clone()))
                                    attr:aria-label=name
                                    attr:data-timeline-entry-action=id
                                >
                                    <span class="block text-sm leading-tight">{e.title}</span>
                                    {e.subtitle.map(|s| view! {
                                        <span class="block text-xs text-base-content/60">{s}</span>
                                    })}
                                    {e.description.map(|d| view! {
                                        <span class="block text-xs text-base-content/50">{d}</span>
                                    })}
                                </Pressable>
                            }
                                .into_any()
                        }
                        None => view! {
                            <p class="text-sm leading-tight">{e.title}</p>
                            {e.subtitle.map(|s| view! {
                                <p class="text-xs text-base-content/60">{s}</p>
                            })}
                            {e.description.map(|d| view! {
                                <p class="text-xs text-base-content/50">{d}</p>
                            })}
                        }
                            .into_any(),
                    };
                    view! {
                        <TimelineItem position=position>
                            <TimelineItemMiddle>
                                <div class=format!("w-2 h-2 rounded-full {dot}")></div>
                            </TimelineItemMiddle>
                            <TimelineItemEnd>{body}</TimelineItemEnd>
                        </TimelineItem>
                    }
                })
                .collect::<Vec<_>>()}
        </Timeline>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Office op-j3ffb: an entry is a control only with BOTH an id and a
    /// list callback. BREAK: make `timeline_entry_is_activatable` return
    /// `has_callback` alone; the id-less assertion fails.
    #[test]
    fn an_entry_is_activatable_only_with_an_id_and_a_callback() {
        assert!(timeline_entry_is_activatable(true, true));
        assert!(
            !timeline_entry_is_activatable(false, true),
            "no id: read-only"
        );
        assert!(
            !timeline_entry_is_activatable(true, false),
            "no callback: read-only"
        );
        assert!(!timeline_entry_is_activatable(false, false));
        let plain = TimelineListEntry::new("Note");
        assert!(
            plain.id.is_none() && plain.action_label.is_none(),
            "opt-in, never the default"
        );
        let named = plain
            .with_id("note-7")
            .with_action_label("Edit note from 12 May");
        assert_eq!(named.id.as_deref(), Some("note-7"));
        assert_eq!(named.action_label.as_deref(), Some("Edit note from 12 May"));
    }

    /// The control's accessible name is the action label, falling back to
    /// the title -- never empty for a titled entry.
    #[test]
    fn the_action_name_is_the_label_or_the_title() {
        assert_eq!(
            timeline_entry_action_name(Some("Edit note from 12 May"), "Called client"),
            "Edit note from 12 May"
        );
        assert_eq!(
            timeline_entry_action_name(None, " Called client "),
            "Called client"
        );
        assert_eq!(
            timeline_entry_action_name(Some("   "), "Called client"),
            "Called client"
        );
    }

    /// The activatable branch renders ONE `Pressable` carrying the accessible
    /// name and the stable hook, and its lines are block spans (a button
    /// takes phrasing content, so no `<p>` inside it). BREAK: drop the
    /// `attr:aria-label=name` line; the assertion names it.
    #[test]
    fn the_activatable_branch_is_one_named_button_with_phrasing_content() {
        let source = include_str!("timeline_list.rs");
        let (_, component) = source
            .split_once("pub fn TimelineList(")
            .expect("component");
        let (component, _) = component.split_once("#[cfg(test)]").expect("tests follow");
        assert_eq!(
            component.matches("<Pressable").count(),
            1,
            "one control per entry"
        );
        assert!(
            component.contains("attr:aria-label=name"),
            "the control needs an accessible name"
        );
        assert!(
            component.contains("attr:data-timeline-entry-action=id"),
            "a stable hook per entry"
        );
        let (_, branch) = component.split_once("<Pressable").expect("branch");
        let (branch, _) = branch.split_once("</Pressable>").expect("branch end");
        assert!(
            !branch.contains("<p"),
            "no block elements inside a button: {branch}"
        );
        assert_eq!(
            branch.matches("<span class=\"block").count(),
            3,
            "title, subtitle, description"
        );
    }

    #[test]
    fn dot_class_maps_severity_codes() {
        assert_eq!(dot_class(Some("error")), "bg-red-500");
        assert_eq!(dot_class(Some("warning")), "bg-amber-500");
        assert_eq!(dot_class(Some("ok")), "bg-emerald-500");
        assert_eq!(dot_class(Some("muted")), "bg-base-300");
        assert_eq!(dot_class(None), "bg-blue-500");
    }
}
