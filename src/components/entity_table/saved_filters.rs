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

    #[test]
    fn texts_defaults_and_substitution() {
        let texts = EntitySavedFilterTexts::default();
        assert_eq!(texts.save_button, "Save Filter");
        assert_eq!(texts.apply_named("Urgent only"), "Apply filter Urgent only");
        assert_eq!(
            texts.remove_named("Urgent only"),
            "Remove filter Urgent only"
        );
    }
}
