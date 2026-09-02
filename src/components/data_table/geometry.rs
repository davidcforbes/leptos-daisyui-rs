//! Stable column-track geometry for opinionated tables.
//!
//! The track model is deliberately derived only from column declarations and
//! controlled resize widths. Body cells and the currently rendered page never
//! participate, so sorting/filtering/paging cannot move the table shell.

use leptos::prelude::*;

/// Floor (px) an UNDECLARED column contributes to the scroll viewport's
/// `min-width`. A column with no resize preference and no explicit
/// minimum/initial width does not get a pixel track at all (ldui-qsqz): in
/// the fixed table layout it shares whatever width the declared tracks
/// leave, so a ten-column table with nothing declared still fits `w-full`
/// exactly as it did before stable geometry landed. The floor only exists
/// so that a narrow container makes the wrapper scroll (its `overflow-x:
/// auto` is the affordance) instead of squeezing ten columns into unusable
/// slivers. It was 160px per column before -- 1600px for ten columns, wider
/// than a 1440px viewport, with the last two columns clipped by the host.
pub(crate) const UNDECLARED_COLUMN_FLOOR_WIDTH: u32 = 96;

/// One stable `<col>` track.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StableColumnTrack {
    pub(crate) id: String,
    /// The pixel track for a declared column; the floor contribution for an
    /// undeclared one.
    pub(crate) width: u32,
    pub(crate) flexible: bool,
    /// `true` when `width` came from a resize preference or a declared
    /// minimum/initial width and therefore paints as a `<col>` width.
    /// `false` leaves the `<col>` width unset so the fixed layout shares
    /// the remaining table width between such columns.
    pub(crate) declared: bool,
}

impl StableColumnTrack {
    /// A track with an explicit pixel width.
    pub(crate) fn new(id: impl Into<String>, width: u32) -> Self {
        Self {
            id: id.into(),
            width: width.max(1),
            flexible: false,
            declared: true,
        }
    }

    /// A track for a column that declared nothing: no `<col>` width, and
    /// only [`UNDECLARED_COLUMN_FLOOR_WIDTH`] towards the scroll minimum.
    pub(crate) fn undeclared(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            width: UNDECLARED_COLUMN_FLOOR_WIDTH,
            flexible: false,
            declared: false,
        }
    }

    /// Build the track for a column from its resize preference and declared
    /// width, in that order of precedence; neither yields an undeclared
    /// track.
    pub(crate) fn resolve(
        id: impl Into<String>,
        resized_width: Option<f64>,
        declared_width: Option<u32>,
    ) -> Self {
        match stable_column_width(resized_width, declared_width) {
            Some(width) => Self::new(id, width),
            None => Self::undeclared(id),
        }
    }

    /// The `<col>` inline style this track paints, if any.
    pub(crate) fn col_style(&self) -> Option<String> {
        (self.declared && !self.flexible).then(|| format!("width: {}px", self.width))
    }

    /// Makes this track absorb otherwise-unused table width. A single
    /// flexible track prevents the browser from proportionally stretching
    /// every explicit pixel track in a full-width fixed-layout table.
    pub(crate) fn flexible(mut self) -> Self {
        self.flexible = true;
        self
    }
}

/// Resolve an explicit track width without consulting body content: the
/// resize preference wins, then the declared minimum/initial width; a column
/// with neither has no explicit width (`None`) and becomes an undeclared
/// track.
pub(crate) fn stable_column_width(
    resized_width: Option<f64>,
    declared_width: Option<u32>,
) -> Option<u32> {
    resized_width
        .map(|width| width.round().max(1.0) as u32)
        .or(declared_width)
        .map(|width| width.max(1))
}

/// Minimum width for the content inside the horizontal scroll viewport: the
/// declared tracks at their pixel widths plus the floor for every undeclared
/// one. Below this the wrapper scrolls; above it the fixed layout shares the
/// slack between undeclared (or flexible) tracks and the table fits `w-full`.
pub(crate) fn stable_table_content_style(tracks: &[StableColumnTrack]) -> String {
    let width = tracks
        .iter()
        .fold(0_u32, |total, track| total.saturating_add(track.width))
        .max(1);
    format!("min-width: {width}px")
}

/// Shared fixed `<colgroup>` for client and server `DataTable` variants.
#[component]
pub(crate) fn StableTableColGroup(tracks: Signal<Vec<StableColumnTrack>>) -> impl IntoView {
    view! {
        <colgroup data-table-column-tracks="stable">
            <For
                each=move || tracks.get()
                // Width changes are legitimate geometry changes and may
                // replace the corresponding <col>; sorting does neither.
                key=|track| (track.id.clone(), track.width, track.flexible, track.declared)
                children=move |track| {
                    let style = track.col_style();
                    view! {
                        <col
                            data-table-column-track=track.id
                            data-table-column-track-flex=track.flexible.then_some("true")
                            data-table-column-track-auto=(!track.declared).then_some("true")
                            style=style
                        />
                    }
                }
            />
        </colgroup>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_geometry_prefers_resize_then_declaration_then_nothing() {
        assert_eq!(stable_column_width(Some(241.6), Some(120)), Some(242));
        assert_eq!(stable_column_width(None, Some(120)), Some(120));
        // ldui-qsqz: a column that declared nothing has NO explicit width --
        // it used to default to 160px, which made ten undeclared columns
        // 1600px wide at a 1440px viewport.
        assert_eq!(stable_column_width(None, None), None);
    }

    #[test]
    fn table_geometry_minimum_is_only_the_sum_of_declared_tracks() {
        let tracks = vec![
            StableColumnTrack::new("name", 160),
            StableColumnTrack::new("email", 240),
            StableColumnTrack::new("status", 96),
        ];
        assert_eq!(stable_table_content_style(&tracks), "min-width: 496px");
    }

    /// ldui-qsqz: ten undeclared columns must fit a 1280px container -- the
    /// scroll minimum is ten floors (960px), not ten stable defaults.
    #[test]
    fn ten_undeclared_columns_fit_a_1280px_container() {
        let tracks: Vec<_> = (0..10)
            .map(|i| StableColumnTrack::undeclared(format!("c{i}")))
            .collect();
        // Pinned at compile time: ten floors must fit the consumer's 1280px
        // container (clippy rejects a runtime assert on a constant).
        const _: () = assert!(10 * UNDECLARED_COLUMN_FLOOR_WIDTH <= 1280);
        assert_eq!(
            stable_table_content_style(&tracks),
            format!("min-width: {}px", 10 * UNDECLARED_COLUMN_FLOOR_WIDTH)
        );
        assert!(tracks.iter().all(|t| t.col_style().is_none()));
    }

    /// Declared tracks keep their pixel `<col>` width; undeclared and
    /// flexible ones paint none, so the fixed layout shares the slack.
    #[test]
    fn only_declared_fixed_tracks_paint_a_col_width() {
        assert_eq!(
            StableColumnTrack::new("name", 240).col_style().as_deref(),
            Some("width: 240px")
        );
        assert_eq!(StableColumnTrack::undeclared("notes").col_style(), None);
        assert_eq!(
            StableColumnTrack::new("actions", 160)
                .flexible()
                .col_style(),
            None
        );
        let resolved = StableColumnTrack::resolve("email", None, Some(300));
        assert!(resolved.declared);
        assert_eq!(resolved.width, 300);
        let auto = StableColumnTrack::resolve("email", None, None);
        assert!(!auto.declared);
        assert_eq!(auto.width, UNDECLARED_COLUMN_FLOOR_WIDTH);
    }

    #[test]
    fn table_geometry_saturates_instead_of_wrapping() {
        let tracks = vec![
            StableColumnTrack::new("a", u32::MAX),
            StableColumnTrack::new("b", 10),
        ];
        assert_eq!(
            stable_table_content_style(&tracks),
            format!("min-width: {}px", u32::MAX)
        );
    }

    #[test]
    fn table_geometry_marks_only_an_explicit_flex_sink() {
        let fixed = StableColumnTrack::new("client", 240);
        let flexible = StableColumnTrack::new("actions", 160).flexible();

        assert!(!fixed.flexible);
        assert!(flexible.flexible);
        assert_eq!(flexible.width, 160);
    }
}
