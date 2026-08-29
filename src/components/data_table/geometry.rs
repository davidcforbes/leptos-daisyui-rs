//! Stable column-track geometry for opinionated tables.
//!
//! The track model is deliberately derived only from column declarations and
//! controlled resize widths. Body cells and the currently rendered page never
//! participate, so sorting/filtering/paging cannot move the table shell.

use leptos::prelude::*;

/// Default declared width for a column that has neither a resize preference
/// nor an explicit minimum/initial width.
pub(crate) const DEFAULT_STABLE_COLUMN_WIDTH: u32 = 160;

/// One stable `<col>` track.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StableColumnTrack {
    pub(crate) id: String,
    pub(crate) width: u32,
    pub(crate) flexible: bool,
}

impl StableColumnTrack {
    pub(crate) fn new(id: impl Into<String>, width: u32) -> Self {
        Self {
            id: id.into(),
            width: width.max(1),
            flexible: false,
        }
    }

    /// Makes this track absorb otherwise-unused table width. A single
    /// flexible track prevents the browser from proportionally stretching
    /// every explicit pixel track in a full-width fixed-layout table.
    pub(crate) fn flexible(mut self) -> Self {
        self.flexible = true;
        self
    }
}

/// Resolve a declared track without consulting body content.
pub(crate) fn stable_column_width(resized_width: Option<f64>, declared_width: Option<u32>) -> u32 {
    resized_width
        .map(|width| width.round().max(1.0) as u32)
        .or(declared_width)
        .unwrap_or(DEFAULT_STABLE_COLUMN_WIDTH)
        .max(1)
}

/// Minimum width for the content inside the horizontal scroll viewport.
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
                key=|track| (track.id.clone(), track.width, track.flexible)
                children=move |track| {
                    let style = (!track.flexible)
                        .then(|| format!("width: {}px", track.width));
                    view! {
                        <col
                            data-table-column-track=track.id
                            data-table-column-track-flex=track.flexible.then_some("true")
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
    fn table_geometry_prefers_resize_then_declaration_then_stable_default() {
        assert_eq!(stable_column_width(Some(241.6), Some(120)), 242);
        assert_eq!(stable_column_width(None, Some(120)), 120);
        assert_eq!(stable_column_width(None, None), DEFAULT_STABLE_COLUMN_WIDTH);
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
