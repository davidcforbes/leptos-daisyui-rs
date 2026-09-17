//! Responsive page-section grid: two (or three) equal columns of page
//! sections that collapse to one on a narrow viewport (Office op-89b3h).
//!
//! Thirteen surface sources hand-rolled `grid gap-4 md:grid-cols-2` around a
//! pair of sections -- a table beside a table, a form beside its preview. Per
//! the coding guide (12.2: repetition across three pages is a strong
//! presumption that the framework owns the pattern) this composite owns the
//! grid, its collapse breakpoint, its gap and the equal-height rule, and a
//! surface writes `<SectionGrid>` with the sections as children.
//!
//! What the grid promises its children: every column is the same width
//! (`grid-cols-N`, never `auto-fit`), every cell in a row is the same height
//! (grid's default `align-items: stretch`, restated as `items-stretch` so it
//! survives a caller class), and every child may shrink below its content
//! (`*:min-w-0 *:min-h-0`) so a wide table scrolls inside its cell instead of
//! widening the page. A child that wants to FILL its cell -- so two
//! `EntityTable`s align top and bottom -- lays itself out as
//! `flex flex-col min-h-0` and lets its table take `flex-1`; the grid cannot
//! do that from outside without dictating every child's display type.

use leptos::prelude::*;

/// Canonical grid contract shared by every column count: one column until
/// the collapse breakpoint, equal-height stretched cells, shrinkable children.
pub const SECTION_GRID_BASE_CLASS: &str =
    "grid w-full min-w-0 grid-cols-1 items-stretch gap-4 *:min-w-0 *:min-h-0";

/// The canonical page-section CELL: a column that FILLS its grid cell, so
/// two sections align top and bottom however tall their contents are.
///
/// The grid cannot do this from outside without dictating every child's
/// display type (see the module docs), so the child opts in. This is the
/// class every hand-rolled `<section class="flex min-h-0 flex-col gap-3">`
/// in the portfolio was spelling out by hand.
///
/// `min-h-0`/`min-w-0` repeat the grid's `*:` rules deliberately: a cell is
/// usable inside any flex or grid parent, not only inside [`SectionGrid`].
pub const SECTION_GRID_CELL_CLASS: &str = "flex min-w-0 min-h-0 flex-col gap-3";

/// Merges caller classes with the canonical section-cell contract.
pub fn section_grid_cell_class(class: &str) -> String {
    [SECTION_GRID_CELL_CLASS, class]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// How many equal columns the grid grows to once it is wide enough.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SectionGridColumns {
    /// Two columns from `md` (48rem) up -- the page-section pair.
    #[default]
    Two,
    /// Two columns from `md`, three from `xl` (80rem).
    Three,
}

impl SectionGridColumns {
    /// The responsive column classes this count adds to the base.
    pub const fn as_class(self) -> &'static str {
        match self {
            SectionGridColumns::Two => "md:grid-cols-2",
            SectionGridColumns::Three => "md:grid-cols-2 xl:grid-cols-3",
        }
    }

    /// The count, emitted as `data-section-grid-columns` so a test can read
    /// the intended shape without parsing utility classes.
    pub const fn as_str(self) -> &'static str {
        match self {
            SectionGridColumns::Two => "2",
            SectionGridColumns::Three => "3",
        }
    }
}

/// Merges caller classes with the canonical section-grid contract.
pub fn section_grid_class(columns: SectionGridColumns, class: &str) -> String {
    [SECTION_GRID_BASE_CLASS, columns.as_class(), class]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Two (or three) equal page sections side by side, one above the other on
/// a narrow viewport.
///
/// ```rust,ignore
/// <SectionGrid>
///     <section class="flex min-h-0 flex-col gap-3">/* table A */</section>
///     <section class="flex min-h-0 flex-col gap-3">/* table B */</section>
/// </SectionGrid>
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("grid w-full min-w-0 grid-cols-1 items-stretch gap-4 *:min-w-0 *:min-h-0 md:grid-cols-2 xl:grid-cols-3 flex min-h-0 flex-col gap-3");
/// ```
#[component]
pub fn SectionGrid(
    /// Column count at full width. Default two.
    #[prop(optional)]
    columns: SectionGridColumns,
    /// Additional grid classes (a page-specific gap or margin). Do NOT pass
    /// a `grid-cols-*` here; choose `columns` instead.
    #[prop(optional, into)]
    class: &'static str,
    /// The sections, in reading order.
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=section_grid_class(columns, class)
            data-section-grid="true"
            data-section-grid-columns=columns.as_str()
        >
            {children()}
        </div>
    }
}

/// One page section inside a [`SectionGrid`] -- a `<section>` that fills its
/// cell, so a pair of sections align top and bottom.
///
/// It owns the house column layout and nothing else: no heading, no border,
/// no padding, no background. Compose [`SectionHeading`](super::SectionHeading),
/// [`AsyncDataSection`](super::AsyncDataSection) or an `EntityTable` inside
/// it as usual, and give the table `flex-1` when it should take the slack.
///
/// ```rust,ignore
/// <SectionGrid>
///     <SectionGridCell>/* heading + table A */</SectionGridCell>
///     <SectionGridCell>/* heading + table B */</SectionGridCell>
/// </SectionGrid>
/// ```
#[component]
pub fn SectionGridCell(
    /// Additional section classes (a page-specific gap or padding). Do NOT
    /// pass a `grid-cols-*` or a width here; the grid owns both.
    #[prop(optional, into)]
    class: &'static str,
    /// The section's contents.
    children: Children,
) -> impl IntoView {
    view! {
        <section class=section_grid_cell_class(class) data-section-grid-cell="true">
            {children()}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The contract every hand-rolled `grid md:grid-cols-2` is being replaced
    /// by: one column first, two from `md`, equal heights, shrinkable cells.
    /// BREAK: drop `items-stretch` from the base class; the equal-height
    /// assertion fails.
    #[test]
    fn two_columns_collapse_to_one_and_stretch_equal_heights() {
        let class = section_grid_class(SectionGridColumns::Two, "");
        let parts: Vec<&str> = class.split_whitespace().collect();
        for required in [
            "grid",
            "grid-cols-1",
            "md:grid-cols-2",
            "items-stretch",
            "gap-4",
            "min-w-0",
            "*:min-w-0",
            "*:min-h-0",
        ] {
            assert!(parts.contains(&required), "missing {required}: {class}");
        }
        assert!(
            !class.contains("xl:grid-cols-3"),
            "two columns never grow to three: {class}"
        );
        assert!(
            !class.contains("auto-fit") && !class.contains("auto-fill"),
            "columns are explicit: {class}"
        );
        assert_eq!(SectionGridColumns::default(), SectionGridColumns::Two);
        assert_eq!(SectionGridColumns::Two.as_str(), "2");
    }

    #[test]
    fn three_columns_step_through_two() {
        let class = section_grid_class(SectionGridColumns::Three, "");
        assert!(class.contains("grid-cols-1 "), "{class}");
        assert!(class.contains("md:grid-cols-2"), "{class}");
        assert!(class.contains("xl:grid-cols-3"), "{class}");
        assert_eq!(SectionGridColumns::Three.as_str(), "3");
    }

    #[test]
    fn caller_classes_are_appended_and_blank_ones_dropped() {
        assert_eq!(
            section_grid_class(SectionGridColumns::Two, "mt-6"),
            format!("{SECTION_GRID_BASE_CLASS} md:grid-cols-2 mt-6")
        );
        assert_eq!(
            section_grid_class(SectionGridColumns::Two, "   "),
            format!("{SECTION_GRID_BASE_CLASS} md:grid-cols-2")
        );
    }

    /// The documented `@source inline(...)` line safelists exactly the
    /// classes the component can emit, so a consumer that copies it gets a
    /// working grid.
    #[test]
    fn the_documented_source_inline_line_covers_every_emitted_class() {
        let source = include_str!("section_grid.rs");
        let line = source
            .lines()
            .map(str::trim_start)
            .find(|line| line.starts_with("/// @source inline("))
            .expect("a documented @source inline line");
        for columns in [SectionGridColumns::Two, SectionGridColumns::Three] {
            for class in section_grid_class(columns, "").split_whitespace() {
                assert!(
                    line.contains(class),
                    "@source inline is missing {class}: {line}"
                );
            }
        }
        for class in section_grid_cell_class("").split_whitespace() {
            assert!(
                line.contains(class),
                "@source inline is missing the cell class {class}: {line}"
            );
        }
    }

    /// The cell is the recipe the module docs used to state in PROSE, which
    /// is exactly how four surfaces each ended up writing their own
    /// `flex min-h-0 flex-col gap-3` by hand. It fills its cell (`flex-col`,
    /// so a child may take `flex-1`) and may shrink below its content in
    /// both axes, so a wide table scrolls inside the cell rather than
    /// widening the page.
    ///
    /// BREAK: drop `min-h-0` from [`SECTION_GRID_CELL_CLASS`]; the
    /// shrink assertion fails and a tall table would push its row taller
    /// than the viewport instead of scrolling.
    #[test]
    fn a_cell_fills_its_column_and_may_shrink_in_both_axes() {
        let class = section_grid_cell_class("");
        let parts: Vec<&str> = class.split_whitespace().collect();
        for required in ["flex", "flex-col", "min-w-0", "min-h-0"] {
            assert!(parts.contains(&required), "missing {required}: {class}");
        }
        assert!(
            !parts.iter().any(|c| c.starts_with("grid")),
            "the GRID owns the columns; a cell never declares them: {class}"
        );
        assert!(
            !parts
                .iter()
                .any(|c| c.starts_with("w-") || c.starts_with("h-")),
            "a cell is sized by its grid track, never by itself: {class}"
        );
    }

    /// A cell renders a real `<section>` landmark carrying the machine
    /// readable marker, and appends caller classes the same way the grid
    /// does -- one merge rule for the pattern, not two.
    ///
    /// BREAK: render a `<div>` instead of a `<section>`; the element
    /// assertion fails.
    #[test]
    fn a_cell_is_a_section_element_and_appends_caller_classes() {
        let source = include_str!("section_grid.rs");
        let component = source
            .split_once("pub fn SectionGridCell(")
            .expect("SectionGridCell component source")
            .1;
        assert!(
            component.contains("<section class=section_grid_cell_class(class)"),
            "a page section is a <section> landmark: {component}"
        );
        assert!(
            component.contains(r#"data-section-grid-cell="true""#),
            "the cell must be readable without parsing classes: {component}"
        );
        assert_eq!(
            section_grid_cell_class("p-4"),
            format!("{SECTION_GRID_CELL_CLASS} p-4")
        );
        assert_eq!(section_grid_cell_class("  "), SECTION_GRID_CELL_CLASS);
    }
}
