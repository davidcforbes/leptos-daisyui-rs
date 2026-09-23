//! Canonical per-row icon actions for [`EntityTable`](super::EntityTable)
//! (ldui-bmqj).
//!
//! Every Office table used to hand-roll its row buttons: some ghost, some
//! outlined, some `btn-sm`, some with a Tooltip and some without, and the
//! accessible name -- when there was one -- rarely said *which* row the
//! action acted on. [`RowActionButton`] is the one shape: a square ghost
//! `btn-xs` icon button, a sprite glyph, an `aria-label` built from an
//! [`EntityRowActionTexts`] template with the row's `{name}` substituted, the
//! same text as a daisyUI [`Tooltip`], and a `data-row-action="<kind>"` hook
//! so a test or an audit can find every Delete on a page without parsing
//! class lists.
//!
//! **Icon actions only in rows; text buttons live in the quick-action row.**
//! A row is dense and repeats; a labelled text button per row is what
//! overflowed the originating consumer's tables. Page-level actions with
//! visible labels belong in
//! [`PageQuickActions`](crate::patterns::PageQuickActions).
//!
//! Place the button inside the table's existing
//! [`EntityRowAction`](super::EntityRowAction) slot so framework-owned focus
//! recovery keeps working after a row is removed:
//!
//! ```rust,ignore
//! <EntityRowAction action_id="delete">
//!     <RowActionButton
//!         kind=RowActionKind::Delete
//!         name=row.name.clone()
//!         on_click=move |()| on_delete.run(row.id)
//!     />
//! </EntityRowAction>
//! ```
//!
//! The EN defaults are the framework's; Spanish (or any other) copy is the
//! consumer's to supply through [`EntityRowActionTexts`], exactly as
//! [`EntityAutoFilterTexts`](super::EntityAutoFilterTexts) is localized.

use crate::components::button::{Button, ButtonShape, ButtonSize, ButtonStyle};
use crate::components::icon::{Icon, IconSize};
use crate::components::tooltip::Tooltip;
use leptos::prelude::*;

/// The six canonical row actions.
///
/// The variant order is the recommended left-to-right order inside a row
/// (read/navigate first, destructive last is deliberately NOT the rule here:
/// Delete sits second so its position is stable across tables that only
/// offer Open and Delete, the most common pair).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RowActionKind {
    /// Open / view the row's record (`eye`).
    Open,
    /// Delete the row's record (`trash`).
    Delete,
    /// Place a phone call to the row's contact (`phone`).
    Call,
    /// Send a text message to the row's contact (`message`).
    Text,
    /// Send an email to the row's contact (`envelope`).
    Email,
    /// Mark the row's item complete (`circle-check`).
    Complete,
}

impl RowActionKind {
    /// Every kind, in the recommended row order.
    pub const ALL: [RowActionKind; 6] = [
        RowActionKind::Open,
        RowActionKind::Delete,
        RowActionKind::Call,
        RowActionKind::Text,
        RowActionKind::Email,
        RowActionKind::Complete,
    ];

    /// Stable lower-case marker, emitted as `data-row-action`.
    pub const fn as_str(self) -> &'static str {
        match self {
            RowActionKind::Open => "open",
            RowActionKind::Delete => "delete",
            RowActionKind::Call => "call",
            RowActionKind::Text => "text",
            RowActionKind::Email => "email",
            RowActionKind::Complete => "complete",
        }
    }

    /// The Lucide name handed to [`Icon`], which translates it through the
    /// shared sprite map. Every name here is pinned by a test as resolving to
    /// a real symbol, never the `blank` fallback.
    pub const fn icon(self) -> &'static str {
        match self {
            RowActionKind::Open => "eye",
            RowActionKind::Delete => "trash",
            RowActionKind::Call => "phone",
            RowActionKind::Text => "message-square",
            RowActionKind::Email => "envelope",
            RowActionKind::Complete => "check",
        }
    }

    /// The label template for this kind from a texts table.
    pub fn label_template(self, texts: &EntityRowActionTexts) -> &str {
        match self {
            RowActionKind::Open => &texts.open,
            RowActionKind::Delete => &texts.delete,
            RowActionKind::Call => &texts.call,
            RowActionKind::Text => &texts.text,
            RowActionKind::Email => &texts.email,
            RowActionKind::Complete => &texts.complete,
        }
    }
}

/// Consumer-localized accessible names for the row-action presets. Every
/// string is a template in which `{name}` is replaced by the row's display
/// name (see [`entity_row_action_label`]), so a screen reader hears
/// "Delete Ana Ruiz", never a bare "Delete" repeated forty times down a
/// column. The defaults are English; a Spanish table supplies its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityRowActionTexts {
    /// [`RowActionKind::Open`]. Default `"Open {name}"`.
    pub open: String,
    /// [`RowActionKind::Delete`]. Default `"Delete {name}"`.
    pub delete: String,
    /// [`RowActionKind::Call`]. Default `"Call {name}"`.
    pub call: String,
    /// [`RowActionKind::Text`]. Default `"Text {name}"`.
    pub text: String,
    /// [`RowActionKind::Email`]. Default `"Email {name}"`.
    pub email: String,
    /// [`RowActionKind::Complete`]. Default `"Complete {name}"`.
    pub complete: String,
}

impl Default for EntityRowActionTexts {
    fn default() -> Self {
        Self {
            open: "Open {name}".to_owned(),
            delete: "Delete {name}".to_owned(),
            call: "Call {name}".to_owned(),
            text: "Text {name}".to_owned(),
            email: "Email {name}".to_owned(),
            complete: "Complete {name}".to_owned(),
        }
    }
}

/// Substitutes `{name}` in an [`EntityRowActionTexts`] template with the
/// row's display name. A template with no placeholder is returned unchanged,
/// which is how a consumer opts out of per-row naming for one kind.
pub fn entity_row_action_label(template: &str, name: &str) -> String {
    template.replace("{name}", name)
}

/// One canonical row action: a square ghost `btn-xs` icon button with a
/// sprite glyph, a per-row `aria-label`, a matching [`Tooltip`], and the
/// `data-row-action` hook. See the module doc for placement rules.
///
/// `disabled_reason` is forwarded to [`Button`] unchanged: non-blank text
/// disables the action and explains why through `aria-describedby` and the
/// native `title` (ldui-p82h). A per-row "why not" -- "Already complete",
/// "No phone number on file" -- is exactly the case that prop exists for.
///
/// ### Add to `input.css`
///
/// Every class this component emits is a literal in this crate's source
/// (`btn btn-ghost btn-xs btn-square` via the `Button` style enums, the
/// Tooltip's `tooltip` family, and `Icon`'s size classes), so a consumer
/// that already scans `leptos-daisyui-rs/src/**/*.rs` needs nothing more.
///
/// ## Node References
/// - The rendered `<button>` is reachable through the `Tooltip` root's
///   `data-row-action` descendant; the component exposes no `node_ref` of
///   its own because `EntityRowAction` locates it by marker, not by handle.
#[component]
pub fn RowActionButton(
    /// Which canonical action this is; picks the glyph, the label template
    /// and the `data-row-action` marker.
    kind: RowActionKind,

    /// The row's display name, substituted for `{name}` in the label.
    #[prop(into)]
    name: Signal<String>,

    /// Fired on activation. The row identity is the caller's to capture --
    /// the button carries none.
    #[prop(into)]
    on_click: Callback<()>,

    /// Why this action is unavailable for this row; forwarded to
    /// [`Button::disabled_reason`](Button). Blank (the default) is "no
    /// reason", so the action stays enabled.
    #[prop(optional, into)]
    disabled_reason: Signal<String>,

    /// Localized label templates. Defaults to the English table.
    #[prop(optional)]
    texts: EntityRowActionTexts,

    /// Additional classes for the `<button>`.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    let template = kind.label_template(&texts).to_owned();
    let label = Signal::derive(move || entity_row_action_label(&template, &name.get()));

    view! {
        <Tooltip tip=label>
            <Button
                size=ButtonSize::Xs
                style=ButtonStyle::Ghost
                shape=ButtonShape::Square
                class=class
                disabled_reason=disabled_reason
                on_click=Callback::new(move |_| on_click.run(()))
                attr:aria-label=move || label.get()
                attr:data-row-action=kind.as_str()
            >
                <Icon name=kind.icon() size=IconSize::XSmall />
            </Button>
        </Tooltip>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::icon::lucide_sprite_lookup;

    #[test]
    fn every_kind_has_a_distinct_stable_marker() {
        let markers: Vec<&str> = RowActionKind::ALL.iter().map(|k| k.as_str()).collect();
        assert_eq!(
            markers,
            ["open", "delete", "call", "text", "email", "complete"]
        );
    }

    /// The whole point of a preset is that its glyph is real. An alias that
    /// silently degrades to `blank` is indistinguishable from success at
    /// runtime (an empty `<svg>`, no error), so every kind's icon is pinned
    /// here as resolving to the symbol the browser test expects.
    #[test]
    fn every_kind_resolves_to_a_real_sprite_symbol() {
        let expected = [
            (RowActionKind::Open, "eye"),
            (RowActionKind::Delete, "trash"),
            (RowActionKind::Call, "phone"),
            (RowActionKind::Text, "message"),
            (RowActionKind::Email, "envelope"),
            (RowActionKind::Complete, "circle-check"),
        ];
        for (kind, symbol) in expected {
            assert_eq!(
                lucide_sprite_lookup(kind.icon()),
                Some(symbol),
                "{kind:?} icon {:?} must resolve to {symbol:?}",
                kind.icon()
            );
        }
    }

    #[test]
    fn default_texts_are_the_english_templates() {
        let texts = EntityRowActionTexts::default();
        assert_eq!(texts.open, "Open {name}");
        assert_eq!(texts.delete, "Delete {name}");
        assert_eq!(texts.call, "Call {name}");
        assert_eq!(texts.text, "Text {name}");
        assert_eq!(texts.email, "Email {name}");
        assert_eq!(texts.complete, "Complete {name}");
        for kind in RowActionKind::ALL {
            assert!(
                kind.label_template(&texts).contains("{name}"),
                "{kind:?} default must name the row"
            );
        }
    }

    #[test]
    fn label_substitutes_the_row_name() {
        assert_eq!(
            entity_row_action_label("Delete {name}", "Ana Ruiz"),
            "Delete Ana Ruiz"
        );
        assert_eq!(
            entity_row_action_label("Eliminar {name}", "Ana Ruiz"),
            "Eliminar Ana Ruiz",
            "a consumer's own template substitutes the same way"
        );
        assert_eq!(
            entity_row_action_label("Delete", "Ana Ruiz"),
            "Delete",
            "a template with no placeholder is returned unchanged"
        );
    }

    /// The rendered shape is the contract: square ghost xs, the marker, the
    /// per-row name, the reason forwarded, the tooltip carrying the label.
    #[test]
    fn the_button_renders_the_canonical_shape() {
        let src = include_str!("row_actions.rs")
            .split_once("pub fn RowActionButton(")
            .expect("component source")
            .1;
        assert!(src.contains("size=ButtonSize::Xs"));
        assert!(src.contains("style=ButtonStyle::Ghost"));
        assert!(src.contains("shape=ButtonShape::Square"));
        assert!(src.contains("disabled_reason=disabled_reason"));
        assert!(src.contains("attr:aria-label=move || label.get()"));
        assert!(src.contains("attr:data-row-action=kind.as_str()"));
        assert!(src.contains("<Tooltip tip=label>"));
        assert!(src.contains("<Icon name=kind.icon() size=IconSize::XSmall />"));
    }
}
