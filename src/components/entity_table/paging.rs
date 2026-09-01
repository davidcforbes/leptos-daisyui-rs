//! Group-aware page planning for [`EntityTable`](super::EntityTable)
//! (`ldui-5in5`).
//!
//! # Why a plan instead of arithmetic
//!
//! Ungrouped paging is arithmetic: page `p` holds rows `p * capacity ..`. Every
//! part of the render can therefore recompute it independently and still agree.
//! A grouped table cannot: keeping a group whole means some pages deliberately
//! stop short of capacity, so "which rows are on page `p`" stops being a
//! function of `p` and `capacity` alone.
//!
//! [`EntityPagePlan`] is that one answer, computed once per displayed order and
//! read by the body, the row-range summary, the pager, the displayed-page
//! selection population and the display projection — the same discipline
//! `ldui-5p06` imposed on the page *size*, extended to the page *boundaries*.
//! There is deliberately no second place that divides a row count by a
//! capacity.
//!
//! # The rule
//!
//! A group that **fits within one page capacity** is never split merely to fill
//! the remainder of the previous page: when the next group would start on the
//! current page and cannot finish there, the page ends early and the group
//! starts whole on the next one.
//!
//! A group **larger than the whole capacity** cannot be kept whole by any
//! packing, so it degrades honestly to exactly the previous behavior — it fills
//! the remainder of the current page and continues onto the next under the
//! existing clearly marked continuation heading. Deferring it to a fresh page
//! would still split it, would still need the continuation heading, and would
//! waste a page of rows for nothing.
//!
//! Two invariants keep that rule total, and both are load-bearing:
//!
//! - **Every page holds at least one row.** The early break only fires when the
//!   page already holds a row, so a plan can never emit an empty page and the
//!   loop can never fail to advance.
//! - **Row counts stay truthful.** The plan partitions the displayed rows
//!   exactly — the pages are contiguous, disjoint and cover every row — so the
//!   footer range is read off the plan rather than multiplied out of the page
//!   index, and it keeps describing real data rows. Group headings are
//!   presentation and are not rows here, exactly as before.

use crate::components::data_table::{clamp_page, page_bounds, page_count, row_range};
use std::ops::Range;

/// The row window each page renders, in displayed-order offsets.
///
/// Constructed from the displayed order *after* filtering, sorting, grouping
/// and collapse, which is what makes "recompute group boundaries before
/// paginating" structural rather than a rule someone has to remember.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityPagePlan {
    shape: EntityPageShape,
}

/// Deliberately two shapes rather than one computed table of page starts.
///
/// An ungrouped table must page EXACTLY as it always did, so its variant
/// delegates to the very `page_count`, `page_bounds`, `row_range` and
/// `clamp_page` functions it used before this module existed -- not to a
/// reimplementation that happens to agree today. Only the grouped variant
/// carries computed boundaries, because only it has any.
#[derive(Clone, Debug, PartialEq, Eq)]
enum EntityPageShape {
    Uniform {
        total_rows: usize,
        capacity: usize,
    },
    Grouped {
        starts: Vec<usize>,
        total_rows: usize,
    },
}

impl Default for EntityPagePlan {
    fn default() -> Self {
        Self::uniform(0, 1)
    }
}

impl EntityPagePlan {
    /// The plan an ungrouped table has always had: fixed-size pages, answered
    /// by the shipped pagination arithmetic itself.
    #[must_use]
    pub fn uniform(total_rows: usize, capacity: usize) -> Self {
        Self {
            shape: EntityPageShape::Uniform {
                total_rows,
                capacity: capacity.max(1),
            },
        }
    }

    /// The group-aware plan: consecutive run lengths of the displayed rows, in
    /// displayed order, packed under the rule in this module's documentation.
    ///
    /// `run_lengths` describes only rows the table actually displays, so a
    /// collapsed group contributes nothing here -- it holds no rows, and its
    /// heading is placed separately by
    /// [`entity_grouped_page_sections`](super::grouping::entity_grouped_page_sections).
    #[must_use]
    pub fn grouped(run_lengths: &[usize], capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let total_rows: usize = run_lengths.iter().sum();
        if total_rows == 0 {
            return Self {
                shape: EntityPageShape::Grouped {
                    starts: Vec::new(),
                    total_rows: 0,
                },
            };
        }

        let mut starts = vec![0_usize];
        // Rows already placed on the page currently being filled.
        let mut used = 0_usize;
        // Absolute offset of that page's first row.
        let mut page_start = 0_usize;

        for length in run_lengths.iter().copied().filter(|length| *length > 0) {
            let mut remaining = length;
            // Only the FIRST chunk of a group can be deferred to keep the group
            // whole; once part of it is placed, deferring the rest would strand
            // a heading without its rows.
            let mut starting_the_group = true;
            while remaining > 0 {
                let free = capacity - used;
                // The whole group fits on a page of this capacity, but not in
                // what is left of this one: end the page early rather than
                // spend the remainder on a fragment.
                let defer_to_keep_whole =
                    starting_the_group && used > 0 && length <= capacity && length > free;
                if free == 0 || defer_to_keep_whole {
                    page_start += used;
                    starts.push(page_start);
                    used = 0;
                    continue;
                }
                // Reached only when the group is larger than a whole page (or
                // has already started here): fill greedily and let the existing
                // continuation heading mark the split.
                let taken = remaining.min(free);
                used += taken;
                remaining -= taken;
                starting_the_group = false;
            }
        }

        Self {
            shape: EntityPageShape::Grouped { starts, total_rows },
        }
    }

    /// How many pages the plan holds. Zero when there are no displayed rows.
    #[must_use]
    pub fn page_count(&self) -> usize {
        match &self.shape {
            EntityPageShape::Uniform {
                total_rows,
                capacity,
            } => page_count(*total_rows, *capacity),
            EntityPageShape::Grouped { starts, .. } => starts.len(),
        }
    }

    /// Total displayed data rows across every page.
    #[must_use]
    pub const fn total_rows(&self) -> usize {
        match &self.shape {
            EntityPageShape::Uniform { total_rows, .. }
            | EntityPageShape::Grouped { total_rows, .. } => *total_rows,
        }
    }

    /// Clamps a zero-based page index onto a page that exists.
    #[must_use]
    pub fn clamp(&self, page: usize) -> usize {
        match &self.shape {
            EntityPageShape::Uniform {
                total_rows,
                capacity,
            } => clamp_page(page, *capacity, *total_rows),
            EntityPageShape::Grouped { starts, .. } => starts.len().saturating_sub(1).min(page),
        }
    }

    /// The displayed-order row window for a page, clamped onto a real page.
    #[must_use]
    pub fn bounds(&self, page: usize) -> Range<usize> {
        match &self.shape {
            EntityPageShape::Uniform {
                total_rows,
                capacity,
            } => page_bounds(page, *capacity, *total_rows),
            EntityPageShape::Grouped { starts, total_rows } => {
                if starts.is_empty() {
                    return 0..0;
                }
                let page = self.clamp(page);
                let start = starts[page];
                let end = starts.get(page + 1).copied().unwrap_or(*total_rows);
                start..end
            }
        }
    }

    /// One-based inclusive `(start, end)` for the footer summary, or `(0, 0)`
    /// when the page holds nothing.
    ///
    /// Read off the plan rather than multiplied out of the page index: with
    /// variable page contents `page * capacity + 1` is simply the wrong number,
    /// and a footer that keeps reciting it is exactly the "truthful counts"
    /// half of the bead.
    #[must_use]
    pub fn row_range(&self, page: usize) -> (usize, usize) {
        match &self.shape {
            EntityPageShape::Uniform {
                total_rows,
                capacity,
            } => row_range(page, *capacity, *total_rows),
            EntityPageShape::Grouped { starts, .. } => {
                if page >= starts.len() {
                    return (0, 0);
                }
                let bounds = self.bounds(page);
                (bounds.start + 1, bounds.end)
            }
        }
    }
}

/// Consecutive run lengths of a parallel group-key array.
///
/// The keys are already partitioned by group rank, so equal adjacent keys are
/// one run; this never needs to sort or deduplicate.
#[must_use]
pub(crate) fn entity_displayed_run_lengths(group_keys: &[String]) -> Vec<usize> {
    let mut runs: Vec<usize> = Vec::new();
    let mut previous: Option<&String> = None;
    for key in group_keys {
        match previous {
            Some(seen) if seen == key => {
                if let Some(last) = runs.last_mut() {
                    *last += 1;
                }
            }
            _ => runs.push(1),
        }
        previous = Some(key);
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{clamp_page, page_bounds, page_count, row_range};

    fn pages(plan: &EntityPagePlan) -> Vec<usize> {
        (0..plan.page_count())
            .map(|page| plan.bounds(page).len())
            .collect()
    }

    #[test]
    fn a_single_run_pages_exactly_like_the_arithmetic_it_replaces() {
        // The ungrouped variant DELEGATES to these functions, so what is worth
        // proving is that the grouped packing agrees with them whenever there
        // is nothing to keep whole -- one group is the ungrouped table.
        for total in 0..40_usize {
            for capacity in 1..12_usize {
                let plan = EntityPagePlan::grouped(&[total], capacity);
                assert_eq!(
                    plan.page_count(),
                    page_count(total, capacity),
                    "page count diverged at total={total} capacity={capacity}"
                );
                for page in 0..(plan.page_count() + 2) {
                    assert_eq!(
                        plan.bounds(page),
                        page_bounds(page, capacity, total),
                        "bounds diverged at page={page} total={total} capacity={capacity}"
                    );
                    assert_eq!(
                        plan.row_range(page),
                        row_range(page, capacity, total),
                        "row range diverged at page={page} total={total} capacity={capacity}"
                    );
                    assert_eq!(
                        plan.clamp(page),
                        clamp_page(page, capacity, total),
                        "clamp diverged at page={page} total={total} capacity={capacity}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_group_that_fits_is_not_split_to_fill_the_previous_page() {
        // The exact Office Coordinator Activity shape: three 17-row groups at a
        // capacity of 18. Fill-first produced 18/18/15 with two split groups;
        // the rule produces one page per group.
        let plan = EntityPagePlan::grouped(&[17, 17, 17], 18);
        assert_eq!(pages(&plan), vec![17, 17, 17]);
        assert_eq!(plan.row_range(0), (1, 17));
        assert_eq!(plan.row_range(1), (18, 34));
        assert_eq!(plan.row_range(2), (35, 51));
    }

    #[test]
    fn the_second_office_report_shape_keeps_every_fitting_group_whole() {
        // NPS Lessons Learned: 14-row viewport fit over ragged office groups.
        // Fill-first split Greensboro 2/2, Raleigh 3/1 and the unlinked run
        // 2/6; nothing here may be split, because every group fits in 14.
        let plan = EntityPagePlan::grouped(&[4, 3, 4, 1, 4, 3, 8], 14);
        assert_eq!(pages(&plan), vec![12, 7, 8]);
        assert_eq!(plan.total_rows(), 27);
    }

    #[test]
    fn a_group_larger_than_a_page_still_splits_and_never_wastes_a_page() {
        // 30 rows cannot fit in 25 by any packing, so the rule degrades to the
        // previous fill-first behavior and the existing continuation heading
        // marks the split -- rather than leaving 8 rows of page 1 unused for a
        // group that would be split anyway.
        let plan = EntityPagePlan::grouped(&[17, 30], 25);
        assert_eq!(pages(&plan), vec![25, 22]);

        // And a group that is a whole multiple of the capacity starts fresh
        // only because the previous page was already exactly full.
        let plan = EntityPagePlan::grouped(&[25, 50], 25);
        assert_eq!(pages(&plan), vec![25, 25, 25]);
    }

    #[test]
    fn every_page_holds_at_least_one_row_and_the_pages_partition_the_rows() {
        // The properties that make the rule total: it can neither emit an empty
        // page (which would render as a heading-only page and never advance)
        // nor lose or duplicate a row.
        let shapes: &[&[usize]] = &[
            &[],
            &[1],
            &[1, 1, 1, 1, 1],
            &[17, 17, 17],
            &[30, 1, 30, 1],
            &[5, 5, 5, 5, 5, 5],
            &[100],
            &[3, 40, 3],
            &[9, 9, 9, 9, 1],
        ];
        for shape in shapes {
            for capacity in 1..14_usize {
                let plan = EntityPagePlan::grouped(shape, capacity);
                let total: usize = shape.iter().sum();
                assert_eq!(plan.total_rows(), total);
                let mut covered = 0;
                for page in 0..plan.page_count() {
                    let bounds = plan.bounds(page);
                    assert_eq!(bounds.start, covered, "pages must be contiguous");
                    assert!(
                        !bounds.is_empty(),
                        "empty page {page} for {shape:?} at capacity {capacity}"
                    );
                    assert!(
                        bounds.len() <= capacity,
                        "page {page} overflowed capacity {capacity} for {shape:?}"
                    );
                    covered = bounds.end;
                }
                assert_eq!(covered, total, "pages must cover every displayed row");
                assert_eq!(plan.page_count() == 0, total == 0);
            }
        }
    }

    #[test]
    fn a_split_group_is_always_one_that_could_not_fit() {
        // The rule restated as a property over the resulting boundaries: a page
        // boundary may fall inside a group only when that group is larger than
        // the capacity.
        let shapes: &[&[usize]] = &[&[17, 17, 17], &[4, 3, 4, 1, 4, 3, 8], &[2, 9, 2, 9, 2]];
        for shape in shapes {
            for capacity in 2..20_usize {
                let plan = EntityPagePlan::grouped(shape, capacity);
                // Absolute offsets at which each group starts.
                let mut group_bounds = Vec::new();
                let mut offset = 0;
                for length in shape.iter().copied() {
                    group_bounds.push(offset..offset + length);
                    offset += length;
                }
                for page in 1..plan.page_count() {
                    let boundary = plan.bounds(page).start;
                    for (group, range) in group_bounds.iter().enumerate() {
                        if range.start < boundary && boundary < range.end {
                            assert!(
                                shape[group] > capacity,
                                "group {group} ({} rows) was split at capacity {capacity} in \
                                 {shape:?} although it fits",
                                shape[group]
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn run_lengths_come_from_the_displayed_keys_in_displayed_order() {
        let keys = ["a", "a", "a", "b", "c", "c"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(entity_displayed_run_lengths(&keys), vec![3, 1, 2]);
        assert!(entity_displayed_run_lengths(&[]).is_empty());

        // A key that reappears after another group is a SECOND run: the grouped
        // order is already partitioned, so equal-but-separated keys can only
        // mean two runs, and merging them would mis-size both pages.
        let split = ["a", "b", "a"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(entity_displayed_run_lengths(&split), vec![1, 1, 1]);
    }

    #[test]
    fn an_ungrouped_plan_and_a_single_run_agree() {
        // A table with one group is the ungrouped table, and must page like it.
        for total in 1..30_usize {
            for capacity in 1..9_usize {
                assert_eq!(
                    pages(&EntityPagePlan::grouped(&[total], capacity)),
                    pages(&EntityPagePlan::uniform(total, capacity)),
                    "one run of {total} diverged from uniform at capacity {capacity}"
                );
            }
        }
    }
}
