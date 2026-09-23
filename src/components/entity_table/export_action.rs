//! The standard Export toolbar action for [`EntityTable`](super::EntityTable)
//! (ldui-e6x8).
//!
//! Every table that exports used to draw its own button, and each one chose
//! a different glyph, label and disabled behaviour. [`EntityExportAction`] is
//! the one shape: a square ghost icon button with the sprite's download
//! glyph, `"Export to CSV"` as both `aria-label` and [`Tooltip`], the
//! `data-entity-export="true"` hook, and -- when the caller supplies a
//! `rows` signal -- a `disabled_reason` of `"No rows to export"` while that
//! count is zero, so an empty table's Export explains itself instead of
//! silently doing nothing.
//!
//! Render it through the table's `toolbar_actions` slot:
//!
//! ```rust,ignore
//! <EntityTable
//!     toolbar_actions=move || view! {
//!         <EntityExportAction rows=visible_rows on_export=move |()| export.run(()) />
//!     }
//!     ...
//! />
//! ```
//!
//! The framework does not own encoding, authorization, or download policy.
//! `on_export` is a bare trigger; what the consumer writes, how it is
//! encoded, and whether the browser downloads it is theirs, typically fed by
//! [`EntityTable::on_display_projection`](super::EntityTable).

use crate::components::button::{Button, ButtonShape, ButtonSize, ButtonStyle};
use crate::components::icon::IconSize;
use crate::components::tooltip::Tooltip;
use leptos::prelude::*;

/// The sprite symbol id the Export action references.
///
/// ⚠️ `download` is NOT in this crate's [`Icon`](crate::components::Icon)
/// sprite map (`lucide_to_sprite`) as of ldui-e6x8, so the glyph is rendered
/// as a direct `<use href="#download">` rather than through `Icon` -- which
/// would `debug_assert!` on the unmapped name. The host document's inlined
/// sprite must define a `download` symbol for the glyph to paint; a host
/// without one renders an empty box, which is exactly the silent failure
/// ldui-q8bj documents. The `data-entity-export-glyph` attribute on the
/// `<svg>` carries the id so a test can assert it without a sprite.
pub const ENTITY_EXPORT_GLYPH: &str = "download";

/// Consumer-localized copy for [`EntityExportAction`]. English defaults;
/// a Spanish table supplies its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityExportActionTexts {
    /// Accessible name and tooltip. `{rows}` is replaced by the row count
    /// when a `rows` signal is supplied (see [`entity_export_action_label`]).
    /// Default `"Export to CSV"`.
    pub label: String,
    /// The `disabled_reason` while `rows` is zero. Default
    /// `"No rows to export"`.
    pub no_rows: String,
}

impl Default for EntityExportActionTexts {
    fn default() -> Self {
        Self {
            label: "Export to CSV".to_owned(),
            no_rows: "No rows to export".to_owned(),
        }
    }
}

/// Substitutes `{rows}` in an [`EntityExportActionTexts::label`] template
/// with the row count. With no count the template is returned verbatim, so a
/// label that mentions `{rows}` should only be paired with a `rows` signal.
pub fn entity_export_action_label(template: &str, rows: Option<usize>) -> String {
    match rows {
        Some(rows) => template.replace("{rows}", &rows.to_string()),
        None => template.to_owned(),
    }
}

/// The `disabled_reason` an Export action shows for a row count: the
/// `no_rows` text at zero, blank (no reason, enabled) otherwise. With no
/// count the action is never disabled by the framework.
pub fn entity_export_disabled_reason(no_rows: &str, rows: Option<usize>) -> String {
    match rows {
        Some(0) => no_rows.to_owned(),
        _ => String::new(),
    }
}

/// The standard Export toolbar action. See the module doc for placement and
/// for what the framework deliberately does not own.
///
/// ### Add to `input.css`
///
/// Every class this component emits is a literal in this crate's source
/// (`btn btn-ghost btn-xs btn-square` via the `Button` style enums, the
/// Tooltip's `tooltip` family, and `IconSize`'s size classes), so a consumer
/// that already scans `leptos-daisyui-rs/src/**/*.rs` needs nothing more.
#[component]
pub fn EntityExportAction(
    /// Fired on activation. Encoding and download are the caller's.
    #[prop(into)]
    on_export: Callback<()>,

    /// Localized label and empty-state reason. Defaults to English.
    #[prop(optional)]
    texts: EntityExportActionTexts,

    /// The exportable row count. When supplied, zero disables the action
    /// with `texts.no_rows` as the reason and the count is available to the
    /// label as `{rows}`.
    #[prop(optional, into)]
    rows: Option<Signal<usize>>,

    /// Additional classes for the `<button>`.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    let EntityExportActionTexts { label, no_rows } = texts;
    let count = move || rows.map(|rows| rows.get());
    let label = Signal::derive(move || entity_export_action_label(&label, count()));
    let disabled_reason = Signal::derive(move || entity_export_disabled_reason(&no_rows, count()));
    let glyph_class = IconSize::XSmall.as_str();

    view! {
        <Tooltip tip=label>
            <Button
                size=ButtonSize::Xs
                style=ButtonStyle::Ghost
                shape=ButtonShape::Square
                class=class
                disabled_reason=disabled_reason
                on_click=Callback::new(move |_| on_export.run(()))
                attr:aria-label=move || label.get()
                attr:data-entity-export="true"
            >
                <svg
                    class=glyph_class
                    aria-hidden="true"
                    focusable="false"
                    data-entity-export-glyph=ENTITY_EXPORT_GLYPH
                >
                    <use href=format!("#{ENTITY_EXPORT_GLYPH}") />
                </svg>
            </Button>
        </Tooltip>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_texts_are_the_english_copy() {
        let texts = EntityExportActionTexts::default();
        assert_eq!(texts.label, "Export to CSV");
        assert_eq!(texts.no_rows, "No rows to export");
    }

    #[test]
    fn label_substitutes_the_row_count_only_when_one_is_given() {
        assert_eq!(
            entity_export_action_label("Export {rows} rows to CSV", Some(42)),
            "Export 42 rows to CSV"
        );
        assert_eq!(
            entity_export_action_label("Export to CSV", Some(42)),
            "Export to CSV",
            "a template with no placeholder is returned unchanged"
        );
        assert_eq!(
            entity_export_action_label("Export {rows} rows to CSV", None),
            "Export {rows} rows to CSV",
            "with no count the template is verbatim, never a phantom number"
        );
    }

    /// Zero rows is the ONLY count the framework disables on; a table with
    /// no count supplied is never disabled by the framework.
    #[test]
    fn zero_rows_is_the_only_framework_disabled_state() {
        assert_eq!(
            entity_export_disabled_reason("No rows to export", Some(0)),
            "No rows to export"
        );
        assert_eq!(
            entity_export_disabled_reason("No rows to export", Some(1)),
            ""
        );
        assert_eq!(
            entity_export_disabled_reason("No rows to export", Some(9_999)),
            ""
        );
        assert_eq!(entity_export_disabled_reason("No rows to export", None), "");
    }

    /// The glyph id is pinned so a host sprite author knows which symbol to
    /// define, and the rendered `<use>` references exactly that id.
    #[test]
    fn the_export_glyph_is_download_and_is_referenced_directly() {
        assert_eq!(ENTITY_EXPORT_GLYPH, "download");
        let src = include_str!("export_action.rs")
            .split_once("pub fn EntityExportAction(")
            .expect("component source")
            .1;
        assert!(src.contains(r##"<use href=format!("#{ENTITY_EXPORT_GLYPH}") />"##));
        assert!(src.contains("data-entity-export-glyph=ENTITY_EXPORT_GLYPH"));
        assert!(src.contains(r#"attr:data-entity-export="true""#));
        assert!(src.contains("disabled_reason=disabled_reason"));
        assert!(src.contains("<Tooltip tip=label>"));
        assert!(src.contains("size=ButtonSize::Xs"));
        assert!(src.contains("style=ButtonStyle::Ghost"));
        assert!(src.contains("shape=ButtonShape::Square"));
    }
}
