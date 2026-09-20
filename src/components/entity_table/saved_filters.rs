//! Controlled "saved filters" model for the opinionated `EntityTable`
//! filter row.
//!
//! The feature: a **Save Filter** button on the table toolbar names the
//! CURRENT filter values; each saved set becomes a badge on the same row;
//! clicking a badge proposes applying its values to the filter row; a small
//! `x` on the badge proposes deleting it.
//!
//! Every part of that is a PROPOSAL. This module owns no storage and never
//! mutates a filter signal. The consumer owns the saved list (via
//! [`EntitySavedFilters::filters`]) and the live values the bar compares
//! against (via [`EntitySavedFilters::current_values`], ordinarily derived
//! from the page's [`EntityAutoFilters::current_values`](super::EntityAutoFilters)),
//! decides whether a save/delete/apply is accepted, and -- when persistence
//! is wanted, e.g. the Office pages saving per-user to Postgres -- writes it
//! through its own server boundary in the callbacks. The crate renders only
//! what the consumer's signals say, so a rejected or still-pending write
//! leaves zero drift between the badge row and the store the consumer reads.
//!
//! Only the typed framework filters participate: a custom
//! `EntityColumnFilter::new` renderer has no value signal to capture, and an
//! undeclared column has no filter at all, so `values` of a saved filter can
//! only ever name columns from the `EntityAutoFilters` set it was captured
//! from. Applying sets only the signals the snapshot names plus clears the
//! others (`EntityAutoFilters::clear_all` semantics), so two different saved
//! sets never leave a stale value on a column the second one does not touch.

use leptos::prelude::*;

/// One named snapshot of the auto-filter values, owned by the consumer.
///
/// `values` are the `(column_id, value)` pairs captured from
/// [`EntityAutoFilters::current_values`](super::EntityAutoFilters); each
/// `column_id` names a column of that set and `value` is verbatim what the
/// column's filter control held. The list is plain data so the consumer can
/// persist it exactly as captured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntitySavedFilter {
    /// User-facing name, shown on the badge. Non-empty and trimmed by
    /// construction when created through the dialog flow.
    pub name: String,
    /// Captured `(column_id, value)` pairs.
    pub values: Vec<(String, String)>,
}

impl EntitySavedFilter {
    /// This filter's constraints in canonical form.
    ///
    /// Derived from [`Self::values`] rather than stored beside it, so a filter
    /// built by any route — including the name-keyed dialog flow that predates
    /// terms — reports the same canonical terms.
    #[must_use]
    pub fn terms(&self) -> Vec<EntitySavedFilterTerm> {
        entity_saved_filter_terms(self.values.iter().cloned())
    }

    /// The value this filter constrains `column_id` to, if it constrains it.
    ///
    /// `None` and `Some("")` are not two different answers here: a blank value
    /// never survives canonicalization, so an unset column and a cleared one
    /// both report `None`.
    #[must_use]
    pub fn value_of(&self, column_id: &str) -> Option<&str> {
        self.values
            .iter()
            .rev()
            .find(|(id, value)| id == column_id && !value.trim().is_empty())
            .map(|(_, value)| value.as_str())
    }

    /// A filter from `(column_id, value)` pairs, canonicalized by
    /// [`entity_saved_filter_terms`].
    ///
    /// `None` when no non-blank term survives — an empty filter row is not
    /// something to save.
    #[must_use]
    pub fn from_values(
        name: impl Into<String>,
        values: impl IntoIterator<Item = (String, String)>,
    ) -> Option<Self> {
        let name = name.into().trim().to_owned();
        if name.is_empty() {
            return None;
        }
        let terms = entity_saved_filter_terms(values);
        if terms.is_empty() {
            return None;
        }
        Some(Self {
            name,
            values: terms
                .into_iter()
                .map(|term| (term.column_id, term.value))
                .collect(),
        })
    }
}

/// Consumer-localized copy for the saved-filters bar and its dialog. Defaults
/// are English placeholders, like every `*Texts` struct in this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntitySavedFilterTexts {
    /// Toolbar button label. Default `"Save Filter"`.
    pub save_button: String,
    /// Dialog heading. Default `"Save current filters"`.
    pub dialog_title: String,
    /// Label of the name input inside the dialog; the label names the field
    /// itself. Default `"Name"`.
    pub name_label: String,
    /// Placeholder inside the name input. Default `"Name this filter set…"`.
    pub name_placeholder: String,
    /// Dialog save action. Default `"Save"`.
    pub save: String,
    /// Dialog cancel action. Default `"Cancel"`.
    pub cancel: String,
    /// Badge accessible-name template; `{name}` is the filter name. Default
    /// `"Apply filter {name}"`.
    pub apply_filter: String,
    /// Delete action template with `{name}`. Default `"Remove filter {name}"`.
    pub remove_filter: String,
    /// Empty-state hint rendered when a saved-filters bar exists but no
    /// filters are saved yet. Default `"No saved filters"`.
    pub empty: String,
    /// Visible caption rendered before the badge row, and — via a single
    /// `aria-labelledby` id — the badge group's accessible name (Office
    /// op-e6dsi). Default `"Saved filters"`.
    ///
    /// One string reaches the screen and the accessibility tree through the
    /// same element, so a translation or a copy edit cannot leave a hidden
    /// name disagreeing with the visible one. It is not rendered when no
    /// filters are saved; the [`Self::empty`] hint stands alone there.
    pub badges_label: String,
}

impl Default for EntitySavedFilterTexts {
    fn default() -> Self {
        Self {
            save_button: "Save Filter".to_owned(),
            dialog_title: "Save current filters".to_owned(),
            name_label: "Name".to_owned(),
            name_placeholder: "Name this filter set…".to_owned(),
            save: "Save".to_owned(),
            cancel: "Cancel".to_owned(),
            apply_filter: "Apply filter {name}".to_owned(),
            remove_filter: "Remove filter {name}".to_owned(),
            empty: "No saved filters".to_owned(),
            badges_label: "Saved filters".to_owned(),
        }
    }
}

impl EntitySavedFilterTexts {
    /// The apply template with the filter name substituted.
    #[must_use]
    pub fn apply_named(&self, name: &str) -> String {
        self.apply_filter.replace("{name}", name)
    }

    /// The remove template with the filter name substituted.
    #[must_use]
    pub fn remove_named(&self, name: &str) -> String {
        self.remove_filter.replace("{name}", name)
    }
}

/// Controlled saved-filters description handed to `EntityTable::saved_filters`.
///
/// All three callbacks PROPOSE; the consumer decides and, on acceptance,
/// rewrites [`Self::filters`]. `on_save` receives the trimmed, non-blank name
/// and the captured values together so the consumer can reject a duplicate or
/// persist the snapshot in one step. [`Self::current_values`] is what the bar
/// compares the saved list against to mark one badge active and what
/// `on_save` captures -- a consumer signals it from the same
/// `EntityAutoFilters` the page's `column_filters` came from.
#[derive(Clone)]
pub struct EntitySavedFilters {
    /// The consumer-owned saved list -- the rendered badges, in order.
    pub filters: Signal<Vec<EntitySavedFilter>, LocalStorage>,
    /// The live filter values the bar compares against, as `(column_id,
    /// value)` pairs in declaration order (the shape
    /// `EntityAutoFilters::current_values` returns).
    pub current_values: Signal<Vec<(String, String)>, LocalStorage>,
    /// Propose saving the current filter values under `name`. The pair
    /// `(name, values)` is one proposal so an acceptance can persist both.
    pub on_save: Callback<(String, Vec<(String, String)>)>,
    /// Propose deleting the saved filter with this name.
    pub on_delete: Callback<String>,
    /// Propose applying this saved filter to the filter row.
    pub on_apply: Callback<EntitySavedFilter>,
    /// Localized copy.
    pub texts: Signal<EntitySavedFilterTexts>,
}

impl EntitySavedFilters {
    /// Builds the description; the texts default to the English placeholders.
    pub fn new(
        filters: Signal<Vec<EntitySavedFilter>, LocalStorage>,
        current_values: Signal<Vec<(String, String)>, LocalStorage>,
        on_save: Callback<(String, Vec<(String, String)>)>,
        on_delete: Callback<String>,
        on_apply: Callback<EntitySavedFilter>,
    ) -> Self {
        Self {
            filters,
            current_values,
            on_save,
            on_delete,
            on_apply,
            texts: Signal::stored(EntitySavedFilterTexts::default()),
        }
    }

    /// Overrides the localized copy.
    #[must_use]
    pub fn texts(mut self, texts: impl Into<Signal<EntitySavedFilterTexts>>) -> Self {
        self.texts = texts.into();
        self
    }

    /// The name of the first saved filter whose captured values exactly match
    /// `current` -- the badge that is "active" for the current filter state,
    /// if any. Pure so the bar and tests agree.
    #[must_use]
    pub fn active_name(
        filters: &[EntitySavedFilter],
        current: &[(String, String)],
    ) -> Option<String> {
        filters
            .iter()
            .find(|saved| saved.values == current)
            .map(|saved| saved.name.clone())
    }
}

impl std::fmt::Debug for EntitySavedFilters {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EntitySavedFilters")
            .finish_non_exhaustive()
    }
}

/// A name is acceptable when it trims to non-empty.
pub(crate) fn saved_filter_name_is_valid(name: &str) -> bool {
    !name.trim().is_empty()
}

/// Most saved filters one table keeps before the oldest is evicted.
///
/// A settled product value, not a placeholder: five is what fits a toolbar
/// badge row without wrapping on a laptop, and Office confirmed it against
/// real use.
pub const ENTITY_SAVED_FILTER_LIMIT: usize = 5;

/// Longest badge label, in grapheme clusters, before
/// [`entity_saved_filter_badge_text`] truncates.
pub const ENTITY_SAVED_FILTER_BADGE_CHARS: usize = 40;

/// One column's constraint inside a saved filter.
///
/// Both halves are `String` and deliberately untyped: the filter control owns
/// what its value means, and this type only round-trips it. `value` is
/// verbatim what the control held.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntitySavedFilterTerm {
    /// The column this constrains.
    pub column_id: String,
    /// The stable filter value, exactly as the control holds it.
    pub value: String,
}

impl EntitySavedFilterTerm {
    /// A term from any string-ish pair.
    pub fn new(column_id: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            column_id: column_id.into(),
            value: value.into(),
        }
    }
}

/// Canonicalizes `(column_id, value)` pairs into saved-filter terms.
///
/// Three rules, and each one is load-bearing rather than tidiness:
///
/// - **Blank is dropped.** A pair whose column or value trims to empty is
///   discarded, so a *cleared* control and an *unset* control produce the same
///   saved filter. Two saves of the same visible filter row must compare equal;
///   everything below depends on that.
/// - **Last wins.** A repeated `column_id` keeps its final value.
/// - **Sorted by column id.** This is what makes [`PartialEq`] meaningful, and
///   `PartialEq` is what makes duplicate detection work. Without the sort, the
///   same filter row saved with its columns visited in a different order
///   compares unequal and silently saves twice.
///
/// Returns an empty `Vec` when nothing survives — an empty filter row is not
/// something to save.
#[must_use]
pub fn entity_saved_filter_terms(
    values: impl IntoIterator<Item = (String, String)>,
) -> Vec<EntitySavedFilterTerm> {
    let mut canonical: Vec<EntitySavedFilterTerm> = Vec::new();
    for (column_id, value) in values {
        if column_id.trim().is_empty() || value.trim().is_empty() {
            continue;
        }
        canonical.retain(|term| term.column_id != column_id);
        canonical.push(EntitySavedFilterTerm::new(column_id, value));
    }
    canonical.sort();
    canonical
}

/// What the bar proposes to the host; the host persists the reducer's result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntitySavedFilterChange {
    /// **Save filter** pressed with this filter row.
    Save(EntitySavedFilter),
    /// A badge's remove control pressed.
    Remove(EntitySavedFilter),
}

/// The single reducer every host applies, so the rules are identical on every
/// table (Office op-e6dsi).
///
/// Hosts persist what this returns rather than implementing the rules
/// themselves — which is what keeps two tables in one product from disagreeing
/// about what "already saved" means.
///
/// - A save whose filter already exists is a **no-op**, compared by
///   canonicalized value rather than by name.
/// - Saving past [`ENTITY_SAVED_FILTER_LIMIT`] **evicts the oldest**; it does
///   not refuse. A refusal needs somewhere to report itself, and a toolbar
///   badge row has nowhere.
/// - A remove drops every entry equal to the argument.
#[must_use]
pub fn entity_saved_filters_apply(
    saved: &[EntitySavedFilter],
    change: EntitySavedFilterChange,
) -> Vec<EntitySavedFilter> {
    let mut next = saved.to_vec();
    match change {
        EntitySavedFilterChange::Save(filter) => {
            if next.contains(&filter) {
                return next;
            }
            next.push(filter);
            let excess = next.len().saturating_sub(ENTITY_SAVED_FILTER_LIMIT);
            if excess > 0 {
                next.drain(..excess);
            }
        }
        EntitySavedFilterChange::Remove(filter) => next.retain(|entry| entry != &filter),
    }
    next
}

/// The control values that restore `filter` onto a table whose *current*
/// filterable columns are `restorable_columns`.
///
/// **Restoring is absolute, not additive**, and that is the whole design. The
/// plan iterates the columns the table has **now**, not the filter's terms, so:
///
/// - Every restorable column is driven — one the filter does not constrain is
///   explicitly set to `""`. Without this, restoring filter A on top of a
///   leftover filter B yields a row neither of them saved, which reads to a
///   user as a corrupt saved filter and is not.
/// - A term whose column no longer exists is simply absent from the output. It
///   is never looked up, so a changed schema can neither error nor resurrect a
///   dead column.
#[must_use]
pub fn entity_saved_filter_restore_plan(
    filter: &EntitySavedFilter,
    restorable_columns: &[&str],
) -> Vec<(String, String)> {
    restorable_columns
        .iter()
        .map(|column_id| {
            let value = filter.value_of(column_id).unwrap_or_default().to_owned();
            ((*column_id).to_owned(), value)
        })
        .collect()
}

/// Shortens a badge label to `max_chars` **grapheme clusters**, ending in `…`.
///
/// Clusters rather than `char`s because `char` is a Unicode scalar value: a
/// combining-accent sequence or an emoji ZWJ sequence splits mid-glyph and
/// renders as a broken character. A product whose data is Latin-1 can live
/// with `chars()`; a shared library cannot assume that.
///
/// The result never exceeds `max_chars` clusters, including the ellipsis — so
/// a budget of 0 yields an empty string rather than a lone `…`, which would
/// exceed it. Trailing whitespace is trimmed before the ellipsis so a cut at a
/// word boundary reads `foo…` and not `foo …`.
///
/// Truncation is **visual only**: callers keep the full label as the badge's
/// `title` and accessible name.
#[must_use]
pub fn entity_saved_filter_badge_text(label: &str, max_chars: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;

    if label.graphemes(true).count() <= max_chars {
        return label.to_owned();
    }
    if max_chars == 0 {
        return String::new();
    }
    let kept = max_chars - 1;
    let mut text = label.graphemes(true).take(kept).collect::<String>();
    text.truncate(text.trim_end().len());
    text.push('…');
    text
}

/// Builds a short label from a filter's terms, dropping whole terms that do
/// not fit rather than cutting one in half.
///
/// **When a string is composed of parts, cut between parts.** Joining the
/// terms and then truncating the join produced `client=Avery · statu` in
/// Office's product — which reads as data corruption rather than as a
/// shortened name. This adds a term only while the whole term fits.
///
/// Returns an empty string when not even the first term fits, which the caller
/// should treat as "no derived label" and fall back to its own naming.
#[must_use]
pub fn entity_saved_filter_compose_label(
    terms: &[EntitySavedFilterTerm],
    max_chars: usize,
) -> String {
    use unicode_segmentation::UnicodeSegmentation;

    const SEPARATOR: &str = " · ";
    let mut label = String::new();
    for term in terms {
        let part = format!("{}={}", term.column_id, term.value);
        let candidate = if label.is_empty() {
            part
        } else {
            format!("{label}{SEPARATOR}{part}")
        };
        if candidate.graphemes(true).count() > max_chars {
            break;
        }
        label = candidate;
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::reactive::owner::Owner;

    fn saved(name: &str, values: &[(&str, &str)]) -> EntitySavedFilter {
        EntitySavedFilter {
            name: name.to_owned(),
            values: values
                .iter()
                .map(|(id, v)| ((*id).to_owned(), (*v).to_owned()))
                .collect(),
        }
    }

    #[test]
    fn a_blank_name_is_refused_and_a_non_blank_one_is_trimmed_by_the_caller() {
        assert!(!saved_filter_name_is_valid(""));
        assert!(!saved_filter_name_is_valid("   "));
        assert!(saved_filter_name_is_valid("Urgent only"));
        assert!(saved_filter_name_is_valid("  Urgent only  "));
    }

    #[test]
    fn active_name_matches_only_an_exact_value_set() {
        let filters = vec![
            saved("Open", &[("status", "Open")]),
            saved("Acme Open", &[("client", "Acme"), ("status", "Open")]),
        ];
        let open: Vec<(String, String)> = vec![("status".to_owned(), "Open".to_owned())];
        assert_eq!(
            EntitySavedFilters::active_name(&filters, &open),
            Some("Open".to_owned())
        );
        let none: Vec<(String, String)> = vec![];
        assert_eq!(EntitySavedFilters::active_name(&filters, &none), None);
        // Order matters: values are compared as captured, in declaration order.
        let reversed: Vec<(String, String)> = vec![
            ("status".to_owned(), "Open".to_owned()),
            ("client".to_owned(), "Acme".to_owned()),
        ];
        assert_eq!(EntitySavedFilters::active_name(&filters, &reversed), None);
    }

    #[test]
    fn the_model_is_controlled_and_emits_proposals() {
        let owner = Owner::new();
        owner.with(|| {
            let saved_list: RwSignal<Vec<EntitySavedFilter>, LocalStorage> =
                RwSignal::new_local(Vec::new());
            let saved_writes: RwSignal<Vec<String>, LocalStorage> = RwSignal::new_local(vec![]);
            let deleted: RwSignal<Vec<String>, LocalStorage> = RwSignal::new_local(vec![]);
            let applied: RwSignal<Vec<String>, LocalStorage> = RwSignal::new_local(vec![]);

            let model = EntitySavedFilters::new(
                saved_list.into(),
                Signal::stored_local(vec![("status".to_owned(), "Open".to_owned())]),
                Callback::new(move |(name, _values): (String, Vec<(String, String)>)| {
                    saved_writes.update(|writes: &mut Vec<String>| writes.push(name));
                }),
                Callback::new(move |name: String| {
                    deleted.update(|d: &mut Vec<String>| d.push(name));
                }),
                Callback::new(move |filter: EntitySavedFilter| {
                    applied.update(|a: &mut Vec<String>| a.push(filter.name));
                }),
            );

            // Proposals record; they never mutate the consumer's list.
            model.on_save.run((
                "Urgent".to_owned(),
                vec![("status".to_owned(), "Open".to_owned())],
            ));
            model.on_delete.run("Urgent".to_owned());
            model.on_apply.run(saved("Urgent", &[("status", "Open")]));
            assert_eq!(saved_writes.get(), vec!["Urgent".to_owned()]);
            assert_eq!(deleted.get(), vec!["Urgent".to_owned()]);
            assert_eq!(applied.get(), vec!["Urgent".to_owned()]);
            assert!(
                saved_list.get().is_empty(),
                "the model never writes the consumer-owned list"
            );
        });
    }

    fn filter(name: &str, values: &[(&str, &str)]) -> EntitySavedFilter {
        EntitySavedFilter::from_values(
            name,
            values
                .iter()
                .map(|(c, v)| ((*c).to_owned(), (*v).to_owned())),
        )
        .expect("fixture filters are non-blank")
    }

    #[test]
    fn terms_drop_blanks_keep_the_last_value_and_sort_by_column() {
        let terms = entity_saved_filter_terms(
            [
                ("status", "Open"),
                ("client", "Acme"),
                ("status", "Closed"),
                ("note", "   "),
                ("  ", "orphan"),
            ]
            .into_iter()
            .map(|(c, v)| (c.to_owned(), v.to_owned())),
        );
        assert_eq!(
            terms,
            vec![
                EntitySavedFilterTerm::new("client", "Acme"),
                EntitySavedFilterTerm::new("status", "Closed"),
            ],
            "blank dropped, last wins, sorted by column id"
        );
    }

    /// The sort is what makes duplicate detection work. Two saves of the SAME
    /// visible filter row, captured with the columns visited in a different
    /// order, must compare equal -- otherwise the reducer's `contains` misses
    /// and the row saves twice.
    #[test]
    fn the_same_filter_row_captured_in_a_different_column_order_compares_equal() {
        let a = filter("a", &[("client", "Acme"), ("status", "Open")]);
        let b = filter("a", &[("status", "Open"), ("client", "Acme")]);
        assert_eq!(a, b);
        assert_eq!(
            entity_saved_filters_apply(&[a], EntitySavedFilterChange::Save(b)).len(),
            1,
            "an identical filter must not save twice"
        );
    }

    /// A cleared control and an unset control are the same saved filter, by
    /// construction -- so `value_of` cannot report an empty constraint.
    #[test]
    fn a_cleared_control_and_an_unset_control_are_indistinguishable() {
        let cleared = filter("f", &[("client", "Acme"), ("status", "  ")]);
        let unset = filter("f", &[("client", "Acme")]);
        assert_eq!(cleared, unset);
        assert_eq!(cleared.value_of("status"), None);
        assert_eq!(cleared.value_of("client"), Some("Acme"));
        assert!(
            EntitySavedFilter::from_values("empty", [("status".to_owned(), " ".to_owned())])
                .is_none(),
            "an empty filter row is not something to save"
        );
    }

    #[test]
    fn saving_past_the_limit_evicts_the_oldest_rather_than_refusing() {
        let mut saved: Vec<EntitySavedFilter> = Vec::new();
        for index in 0..ENTITY_SAVED_FILTER_LIMIT {
            saved = entity_saved_filters_apply(
                &saved,
                EntitySavedFilterChange::Save(filter(
                    &format!("f{index}"),
                    &[("client", &format!("c{index}"))],
                )),
            );
        }
        assert_eq!(saved.len(), ENTITY_SAVED_FILTER_LIMIT);

        let next = entity_saved_filters_apply(
            &saved,
            EntitySavedFilterChange::Save(filter("newest", &[("client", "zzz")])),
        );
        assert_eq!(next.len(), ENTITY_SAVED_FILTER_LIMIT, "the limit holds");
        assert_eq!(next[0].name, "f1", "the OLDEST was evicted, not the newest");
        assert_eq!(
            next[ENTITY_SAVED_FILTER_LIMIT - 1].name,
            "newest",
            "the save was accepted, not refused"
        );
    }

    #[test]
    fn removing_drops_every_equal_entry() {
        let target = filter("a", &[("client", "Acme")]);
        let other = filter("b", &[("client", "Beta")]);
        let next = entity_saved_filters_apply(
            &[target.clone(), other.clone()],
            EntitySavedFilterChange::Remove(target),
        );
        assert_eq!(next, vec![other]);
    }

    /// Restoring is ABSOLUTE. Every current column is driven, so a column the
    /// filter does not constrain is explicitly cleared -- otherwise restoring
    /// filter A over a leftover filter B yields a row neither of them saved.
    #[test]
    fn a_restore_plan_drives_every_current_column_and_ignores_departed_ones() {
        let saved = filter("a", &[("client", "Acme"), ("gone", "stale")]);
        let plan = entity_saved_filter_restore_plan(&saved, &["client", "status"]);
        assert_eq!(
            plan,
            vec![
                ("client".to_owned(), "Acme".to_owned()),
                ("status".to_owned(), String::new()),
            ],
            "an unconstrained current column is cleared, not skipped"
        );
        assert!(
            !plan.iter().any(|(column, _)| column == "gone"),
            "a term whose column no longer exists cannot resurrect it"
        );
    }

    #[test]
    fn badge_text_truncates_on_grapheme_clusters_and_respects_its_budget() {
        assert_eq!(entity_saved_filter_badge_text("short", 40), "short");
        assert_eq!(entity_saved_filter_badge_text("abcdef", 4), "abc\u{2026}");
        assert_eq!(
            entity_saved_filter_badge_text("ab cdef", 4),
            "ab\u{2026}",
            "trailing space is trimmed before the ellipsis"
        );
        assert_eq!(
            entity_saved_filter_badge_text("abc", 0),
            "",
            "a zero budget cannot be met by a lone ellipsis"
        );
        // A combining sequence is ONE cluster: `chars()` would split it and
        // render a bare combining mark on the preceding letter.
        let combining = "cafe\u{301}xyz";
        assert_eq!(combining.chars().count(), 8);
        let cut = entity_saved_filter_badge_text(combining, 5);
        assert!(
            cut.starts_with("cafe\u{301}"),
            "the accent must stay attached to its base letter, got {cut:?}"
        );
    }

    #[test]
    fn a_composed_label_drops_whole_terms_rather_than_cutting_one_in_half() {
        let terms = vec![
            EntitySavedFilterTerm::new("client", "Avery"),
            EntitySavedFilterTerm::new("status", "Open"),
        ];
        assert_eq!(
            entity_saved_filter_compose_label(&terms, 80),
            "client=Avery \u{b7} status=Open"
        );
        let cut = entity_saved_filter_compose_label(&terms, 20);
        assert_eq!(
            cut, "client=Avery",
            "a term that does not fit is dropped whole, never cut to `statu`"
        );
        assert_eq!(
            entity_saved_filter_compose_label(&terms, 3),
            "",
            "nothing fits, so the caller must fall back to its own naming"
        );
    }

    #[test]
    fn texts_defaults_and_substitution() {
        let texts = EntitySavedFilterTexts::default();
        assert_eq!(texts.save_button, "Save Filter");
        assert_eq!(texts.apply_named("Urgent only"), "Apply filter Urgent only");
        assert_eq!(
            texts.remove_named("Urgent only"),
            "Remove filter Urgent only"
        );
        assert_eq!(
            texts.badges_label, "Saved filters",
            "the badge row's visible caption doubles as the group's accessible              name, so it must be real user-facing copy and translatable like              the rest"
        );
    }
}
