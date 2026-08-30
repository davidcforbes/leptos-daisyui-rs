//! Framework-owned semantic emphasis for one classified row in
//! [`EntityTable`](super::EntityTable).
//!
//! Mirrors `selection.rs`'s shape: a narrow, framework-owned enum plus pure
//! functions that turn a classified row into `EntityTable`'s own tokens. The
//! caller supplies only the classification predicate (an
//! [`EntityRowEmphasisClassifier`]) -- never a class string -- so
//! `EntityTable` retains complete ownership of every token, stroke width,
//! and forced-colors rule a variant applies, and applies them identically in
//! the wide and compact presentations that share one `<tr>`.

use std::rc::Rc;

/// Framework-owned semantic emphasis for one classified row.
///
/// [`EntityTable`](super::EntityTable)'s optional `row_emphasis` callback
/// classifies each row into this narrow enum -- never an unrestricted
/// class-string hook -- so the framework keeps complete ownership of the
/// resulting tokens and responsive rules; the caller owns only the
/// classification predicate. `Standard` (the default returned when a table
/// has no `row_emphasis` callback at all) renders identically to a table
/// that predates this prop: no extra class, no extra attribute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityRowEmphasis {
    /// The table's ordinary row presentation.
    #[default]
    Standard,
    /// A totals/subtotal row: bold text plus a top rule that sets it apart
    /// from the data rows above it, in both presentations.
    Summary,
    /// A de-emphasized row: reduced-contrast text held at the framework's
    /// audited AA-safe ratio rather than a lower, axe-failing opacity.
    Muted,
    /// A row that needs the reader's attention: warning-toned, bold text.
    Attention,
}

impl EntityRowEmphasis {
    /// Stable runtime marker emitted as `data-entity-row-emphasis`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Summary => "summary",
            Self::Muted => "muted",
            Self::Attention => "attention",
        }
    }
}

/// A callback that classifies a borrowed row into a narrow, framework-owned
/// emphasis variant. See [`EntityRowEmphasis`].
pub type EntityRowEmphasisClassifier<T> = Rc<dyn Fn(&T) -> EntityRowEmphasis>;

/// `<tr>`-level classes for one emphasis variant.
///
/// Text-only (font weight and color): no variant sets `background-color`,
/// so this composes with the selected-row background (`bg-base-200`,
/// applied independently in `render_keyed_row`) and with `table-zebra`
/// striping without two authors racing for the same CSS property.
pub(crate) const fn entity_row_emphasis_row_class(emphasis: EntityRowEmphasis) -> &'static str {
    match emphasis {
        EntityRowEmphasis::Standard => "",
        EntityRowEmphasis::Summary => "font-semibold",
        // `text-base-content/75`, not a lower opacity: the `test-style` axe
        // gate fails `opacity-60`/`opacity-50` text (the same AA-contrast
        // rule `TableViewport`'s muted text already carries).
        EntityRowEmphasis::Muted => "text-base-content/75",
        EntityRowEmphasis::Attention => "font-semibold text-warning",
    }
}

/// Per-`<td>` classes for one emphasis variant, applied identically to
/// every wide-layout cell and to the compact single-cell wrapper so a
/// totals rule reads the same in both presentations (they share one
/// `<tr>`; only the cells differ). Neither `background-color` nor `color`
/// appears here -- only a top border, layered safely atop the table's own
/// `border-collapse` grid -- keeping selection and zebra untouched.
pub(crate) const fn entity_row_emphasis_cell_class(emphasis: EntityRowEmphasis) -> &'static str {
    match emphasis {
        EntityRowEmphasis::Summary => {
            "border-t-(--border-width-accent) border-t-base-content forced-colors:border-t-[CanvasText]"
        }
        EntityRowEmphasis::Standard | EntityRowEmphasis::Muted | EntityRowEmphasis::Attention => "",
    }
}

/// Resolves the emphasis for one rendered row.
///
/// `Standard` when the table has no `row_emphasis` classifier at all, or
/// when `row` is absent -- mirroring `selection`'s fail-safe: a row that
/// sorted, filtered, or paged away (or was removed) never carries a stale
/// classification computed against different source data. Classification
/// is otherwise a pure function of the row's own content, so it is
/// unaffected by sort order, paging, or column visibility.
pub(crate) fn entity_row_emphasis_for<T>(
    classifier: Option<&EntityRowEmphasisClassifier<T>>,
    row: Option<&T>,
) -> EntityRowEmphasis {
    match (classifier, row) {
        (Some(classify), Some(row)) => classify(row),
        _ => EntityRowEmphasis::Standard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Row {
        id: &'static str,
    }

    // ── entity_row_emphasis_row_class / entity_row_emphasis_cell_class ──

    #[test]
    fn standard_emits_no_row_or_cell_class() {
        assert_eq!(
            entity_row_emphasis_row_class(EntityRowEmphasis::Standard),
            ""
        );
        assert_eq!(
            entity_row_emphasis_cell_class(EntityRowEmphasis::Standard),
            ""
        );
    }

    #[test]
    fn summary_is_bold_with_a_top_rule_and_no_background() {
        let row_class = entity_row_emphasis_row_class(EntityRowEmphasis::Summary);
        let cell_class = entity_row_emphasis_cell_class(EntityRowEmphasis::Summary);
        assert!(row_class.contains("font-semibold"));
        assert!(!row_class.contains("bg-"));
        assert!(cell_class.contains("border-t"));
        assert!(!cell_class.contains("bg-"));
    }

    #[test]
    fn muted_uses_the_axe_safe_seventy_five_percent_ratio_not_a_lower_opacity() {
        let class = entity_row_emphasis_row_class(EntityRowEmphasis::Muted);
        assert!(class.contains("text-base-content/75"));
        assert!(!class.contains("opacity-60"));
        assert!(!class.contains("opacity-50"));
    }

    #[test]
    fn attention_is_warning_toned_and_bold_with_no_background() {
        let class = entity_row_emphasis_row_class(EntityRowEmphasis::Attention);
        assert!(class.contains("text-warning"));
        assert!(class.contains("font-semibold"));
        assert!(!class.contains("bg-"));
    }

    #[test]
    fn no_variant_sets_a_background_color() {
        // The load-bearing composition rule: selection paints
        // `bg-base-200` on the `<tr>` and `table-zebra` paints alternating
        // row backgrounds via its own CSS. If any emphasis variant also set
        // `background-color`, the two authors would race for the same
        // property. None do, at either the row or cell level.
        for emphasis in [
            EntityRowEmphasis::Standard,
            EntityRowEmphasis::Summary,
            EntityRowEmphasis::Muted,
            EntityRowEmphasis::Attention,
        ] {
            assert!(!entity_row_emphasis_row_class(emphasis).contains("bg-"));
            assert!(!entity_row_emphasis_cell_class(emphasis).contains("bg-"));
        }
    }

    #[test]
    fn merged_row_class_keeps_emphasis_alongside_interactive_and_selected_classes() {
        // Structural complement to the browser fixture's zebra/selection
        // composition proof: builds the same `merge_classes!` call
        // `render_keyed_row` performs for an interactive, selected `Summary`
        // row and confirms the merged string carries all three families --
        // interactive, selected, and emphasis -- rather than one crowding
        // out another. `table-zebra` can never appear in this string at
        // all: it is a class on the ancestor `<table>`, never on the row,
        // so it has nothing here to collide with regardless of how the row
        // classes merge.
        let merged = crate::merge_classes!(
            "cursor-pointer ld-focus-ring",
            "bg-base-200",
            entity_row_emphasis_row_class(EntityRowEmphasis::Summary)
        )
        .to_class();
        assert!(merged.contains("cursor-pointer"));
        assert!(merged.contains("bg-base-200"));
        assert!(merged.contains("font-semibold"));
    }

    #[test]
    fn as_str_is_stable_and_distinct() {
        assert_eq!(EntityRowEmphasis::Standard.as_str(), "standard");
        assert_eq!(EntityRowEmphasis::Summary.as_str(), "summary");
        assert_eq!(EntityRowEmphasis::Muted.as_str(), "muted");
        assert_eq!(EntityRowEmphasis::Attention.as_str(), "attention");
    }

    // ── entity_row_emphasis_for ──

    #[test]
    fn no_classifier_is_always_standard() {
        let row = Row { id: "r1" };
        assert_eq!(
            entity_row_emphasis_for::<Row>(None, Some(&row)),
            EntityRowEmphasis::Standard
        );
        assert_eq!(
            entity_row_emphasis_for::<Row>(None, None),
            EntityRowEmphasis::Standard
        );
    }

    #[test]
    fn classifier_result_is_forwarded_verbatim() {
        let classifier: EntityRowEmphasisClassifier<Row> = Rc::new(|row: &Row| {
            if row.id == "total" {
                EntityRowEmphasis::Summary
            } else {
                EntityRowEmphasis::Standard
            }
        });
        let total = Row { id: "total" };
        let plain = Row { id: "r1" };
        assert_eq!(
            entity_row_emphasis_for(Some(&classifier), Some(&total)),
            EntityRowEmphasis::Summary
        );
        assert_eq!(
            entity_row_emphasis_for(Some(&classifier), Some(&plain)),
            EntityRowEmphasis::Standard
        );
    }

    #[test]
    fn a_missing_row_fails_safe_to_standard() {
        // Mirrors `selection`'s "a selected key absent from the visible
        // page selects nothing" fail-safe: a row that sorted, filtered, or
        // paged away, or was removed, must never carry a stale
        // classification computed against a different row's data.
        let classifier: EntityRowEmphasisClassifier<Row> =
            Rc::new(|_: &Row| EntityRowEmphasis::Attention);
        assert_eq!(
            entity_row_emphasis_for(Some(&classifier), None),
            EntityRowEmphasis::Standard
        );
    }
}
