//! Dataset selection that is structurally separate from local filters.

use crate::components::{Button, Select, SelectSize};
use leptos::prelude::*;

/// Reactive framework-owned copy for dataset replacement presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetSelectorTexts {
    /// Accessible progress label while a replacement is pending.
    pub loading: String,
    /// Template for a displayed-dataset status; `{dataset}` is replaced by consumers.
    pub displayed: String,
    /// Template for a requested destination; `{dataset}` is replaced by consumers.
    pub requested: String,
    /// Prefix used when an older dataset is retained after replacement failure.
    pub retained_error: String,
    /// Retry action label.
    pub retry: String,
}

impl Default for DatasetSelectorTexts {
    fn default() -> Self {
        Self {
            loading: "Loading dataset".to_owned(),
            displayed: "Showing {dataset}".to_owned(),
            requested: "Loading {dataset}".to_owned(),
            retained_error: "Could not replace dataset".to_owned(),
            retry: "Retry".to_owned(),
        }
    }
}

/// One selectable complete dataset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetOption {
    /// Value sent to the page controller when selected.
    pub value: String,
    /// Human-readable option label.
    pub label: String,
    /// Whether this dataset is currently unavailable.
    pub disabled: bool,
}

impl DatasetOption {
    /// Creates an enabled dataset option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Marks an option unavailable.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Returns the label for the selected dataset value.
pub fn selected_dataset_label<'a>(options: &'a [DatasetOption], selected: &str) -> Option<&'a str> {
    options
        .iter()
        .find(|option| option.value == selected)
        .map(|option| option.label.as_str())
}

/// A busy dataset replacement remains supersedable. Only an explicit caller
/// gate disables the selector; `loading` controls busy semantics and progress
/// presentation without trapping the user in the in-flight choice.
pub(super) const fn selector_disabled(disabled: bool, _loading: bool) -> bool {
    disabled
}

/// Whether the selector renders its framework-owned "Showing {dataset}" /
/// "Loading {dataset}" caption (Office op-v7c5g).
///
/// The caption exists so a page whose select sits far from its table can
/// still say which dataset the rows belong to. The `compact` variant is the
/// portfolio rule for an office scope selector centred in the page header:
/// the bare select IS the statement of which office is showing, so the
/// caption would say it twice. A caller-supplied `status` child also
/// replaces the caption, exactly as before.
pub const fn dataset_selector_shows_caption(compact: bool, has_custom_status: bool) -> bool {
    !compact && !has_custom_status
}

/// Selector whose value determines which complete dataset is downloaded.
///
/// This component deliberately exposes `data-resettable-filter="false"` and
/// lives in the page-header dataset slot, not in [`FilterBar`](super::FilterBar).
///
/// ## Two presentations
///
/// * **Default** -- a `bg-base-200` card with an uppercase eyebrow label, the
///   select, a spinner while loading, and a "Showing {dataset}" caption
///   (or the caller's `status` child in its place).
/// * **`compact`** (Office op-v7c5g, the portfolio rule for the office
///   dropdown centred in the page-header row) -- the bare `<select>` and
///   nothing else: its accessible name is `label` (rendered as
///   `aria-label`), there is no eyebrow, no card, no spinner and no
///   caption. `aria-busy` still reports a pending replacement, and a
///   retained load error still renders its alert row beneath, because an
///   error the user cannot see is not a presentation choice. `status` is
///   ignored in this mode. The `data-dataset-selector` and
///   `data-resettable-filter` hooks move onto the select itself, and it
///   additionally carries `data-dataset-selector-compact="true"`.
#[component]
pub fn DatasetSelector(
    /// Visible and accessible control label.
    #[prop(into)]
    label: Signal<String>,
    /// Stable, caller-owned DOM identity for the underlying select.
    #[prop(optional, into)]
    control_id: MaybeProp<String>,
    /// Current dataset key.
    #[prop(into)]
    selected: Signal<String>,
    /// Available datasets. Values must be unique, stable keys. Labels and
    /// disabled state remain reactive when a refresh retains the same key.
    #[prop(into)]
    options: Signal<Vec<DatasetOption>>,
    /// Called when the user requests a different dataset.
    on_change: Callback<String>,
    /// Whether the requested dataset is loading.
    #[prop(optional, into)]
    loading: Signal<bool>,
    /// Whether dataset selection is disabled.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Optional dataset-load error shown without clearing the prior selection.
    #[prop(optional, into)]
    error: Signal<Option<String>>,
    /// Reactive framework-owned loading/error/retry copy.
    #[prop(into, default = Signal::stored(DatasetSelectorTexts::default()))]
    texts: Signal<DatasetSelectorTexts>,
    /// Optional explicit retry intent for a retained load failure.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
    /// Optional live/freshness status beside the selector.
    #[prop(optional)]
    status: Option<Children>,
    /// Additional outer classes. In `compact` mode these land on the
    /// select itself, which is the only element rendered.
    #[prop(optional, into)]
    class: &'static str,
    /// Render the bare select only -- no card, eyebrow, spinner or caption
    /// (Office op-v7c5g). See the component docs.
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let has_custom_status = status.is_some();
    let shows_caption = dataset_selector_shows_caption(compact, has_custom_status);
    let error_row = move || {
        error.get().map(|message| {
            view! {
                <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-error" role="alert">
                    <span>{move || texts.with(|texts| {
                        format!("{}: {message}", texts.retained_error)
                    })}</span>
                    {on_retry.map(|callback| view! {
                        <Button
                            class="btn-ghost btn-xs"
                            on_click=Callback::new(move |_| callback.run(()))
                        >
                            {move || texts.with(|texts| texts.retry.clone())}
                        </Button>
                    })}
                </div>
            }
        })
    };
    if compact {
        return view! {
            <Select
                size=SelectSize::Sm
                class=class
                id=control_id
                label=Signal::derive(move || Some(label.get()))
                value=selected
                options_revision=Signal::derive(move || {
                    options.with(|options| dataset_options_revision(options))
                })
                disabled=Signal::derive(move || {
                    selector_disabled(disabled.get(), loading.get())
                })
                on_change=on_change
                attr:data-dataset-selector="true"
                attr:data-dataset-selector-compact="true"
                attr:data-resettable-filter="false"
                attr:aria-busy=move || loading.get().then_some("true")
            >
                <For
                    each=move || options.get()
                    key=|option| option.value.clone()
                    children=move |option| view! {
                        <option value=option.value disabled=option.disabled>{option.label}</option>
                    }
                />
            </Select>
            {error_row}
        }
        .into_any();
    }
    view! {
        <div
            class=format!("rounded-box bg-base-200 px-3 py-2 {class}")
            data-dataset-selector="true"
            data-resettable-filter="false"
            aria-busy=move || loading.get().then_some("true")
        >
            <label class="flex flex-wrap items-center gap-2">
                <span class="text-xs font-semibold uppercase tracking-wide text-base-content/75">
                    {move || label.get()}
                </span>
                <Select
                    class="select-sm min-w-44 bg-base-100"
                    id=control_id
                    label=Signal::derive(move || Some(label.get()))
                    value=selected
                    // `ldui-uxdw`: this pattern owns BOTH signals, so it can
                    // tell `Select` when the option set changed and no caller
                    // has to know the hazard exists. Without it, options that
                    // arrive asynchronously leave the browser's index-0 choice
                    // in place while `selected` still says otherwise -- two
                    // Office pages showed "Charlotte" beside Raleigh's rows.
                    options_revision=Signal::derive(move || {
                        options.with(|options| dataset_options_revision(options))
                    })
                    disabled=Signal::derive(move || {
                        selector_disabled(disabled.get(), loading.get())
                    })
                    on_change=on_change
                >
                    <For
                        each=move || options.get()
                        key=|option| option.value.clone()
                        children=move |option| {
                            let value = option.value;
                            let key = value.clone();
                            // Keep DOM identity by value, but derive mutable
                            // metadata again when a same-key option changes.
                            let current = Memo::new(move |_| options.with(|items| {
                                items.iter().find(|item| item.value == key).cloned()
                            }));
                            view! {
                                <option value=value disabled=move || current.with(|option| {
                                    option.as_ref().is_none_or(|option| option.disabled)
                                })>
                                    {move || current.with(|option| {
                                        option.as_ref().map(|option| option.label.clone()).unwrap_or_default()
                                    })}
                                </option>
                            }
                        }
                    />
                </Select>
                {move || loading.get().then(|| view! {
                    <span
                        class="loading loading-spinner loading-sm"
                        aria-label=move || texts.with(|texts| texts.loading.clone())
                    ></span>
                })}
                {status.map(|status| status())}
                {shows_caption.then(|| view! {
                    <span class="text-xs text-base-content/70" data-dataset-selector-status="true">
                        {move || {
                            let selected = selected.get();
                            let label = options.with(|options| {
                                selected_dataset_label(options, &selected)
                                    .unwrap_or(selected.as_str())
                                    .to_owned()
                            });
                            texts.with(|texts| {
                                if loading.get() {
                                    texts.requested.replace("{dataset}", &label)
                                } else {
                                    texts.displayed.replace("{dataset}", &label)
                                }
                            })
                        }}
                    </span>
                })}
            </label>
            {error_row}
        </div>
    }
    .into_any()
}

/// An opaque revision of a dataset selector's option set (`ldui-uxdw`).
///
/// Fed to [`Select`]'s `options_revision` so the bound value is re-asserted
/// after the browser rebuilds the option list. Any value that changes with the
/// option set would do; this one is derived from the option VALUES because
/// those are what `<option value=…>` matches on, so it changes exactly when
/// the set the browser can select from changes.
///
/// Uses a unit separator rather than a comma so two option sets cannot collide
/// by containing a separator inside a value — `["a,b"]` and `["a", "b"]` are
/// different sets and must produce different revisions.
fn dataset_options_revision(options: &[DatasetOption]) -> String {
    let mut revision = String::with_capacity(options.len() * 8);
    for option in options {
        revision.push_str(&option.value);
        revision.push('\u{1f}');
    }
    revision
}

#[cfg(test)]
mod compact_tests {
    use super::dataset_selector_shows_caption;

    /// Office op-v7c5g: the compact variant renders no "Showing {dataset}"
    /// caption -- the bare select is the statement. BREAK: make
    /// `dataset_selector_shows_caption` ignore `compact`; the first assertion
    /// fails.
    #[test]
    fn compact_renders_no_caption() {
        assert!(!dataset_selector_shows_caption(true, false));
        assert!(!dataset_selector_shows_caption(true, true));
        assert!(
            dataset_selector_shows_caption(false, false),
            "the default keeps its caption"
        );
        assert!(
            !dataset_selector_shows_caption(false, true),
            "a caller status child replaces the caption, as before"
        );
    }

    /// The compact branch must not grow an eyebrow, a card or a caption back:
    /// the markup between `if compact {` and the default branch carries none
    /// of the default presentation's hooks.
    #[test]
    fn the_compact_branch_is_the_bare_select() {
        let source = include_str!("dataset_selector.rs");
        let (_, after) = source
            .split_once("    if compact {")
            .expect("compact branch");
        let (branch, _) = after
            .split_once("rounded-box bg-base-200")
            .expect("the default card follows the compact branch");
        assert!(
            !branch.contains("data-dataset-selector-status"),
            "no caption: {branch}"
        );
        assert!(!branch.contains("texts.displayed"), "no caption text");
        assert!(!branch.contains("loading-spinner"), "no spinner");
        assert!(!branch.contains("<label"), "no eyebrow label element");
        assert!(!branch.contains("uppercase"), "no eyebrow styling");
        assert!(
            branch.contains("label=Signal::derive(move || Some(label.get()))"),
            "the select's accessible name is the label"
        );
        assert!(branch.contains(r#"attr:data-dataset-selector-compact="true""#));
        assert!(branch.contains(r#"attr:data-resettable-filter="false""#));
    }
}

#[cfg(test)]
mod revision_tests {
    use super::{DatasetOption, dataset_options_revision};

    fn option(value: &str) -> DatasetOption {
        DatasetOption {
            value: value.to_owned(),
            label: value.to_owned(),
            disabled: false,
        }
    }

    /// The property the whole `ldui-uxdw` fix rests on: the revision must
    /// change whenever the option set does, or the re-assert effect never
    /// re-runs and the browser's index-0 choice silently wins.
    #[test]
    fn the_revision_changes_whenever_the_option_set_does() {
        let empty = dataset_options_revision(&[]);
        let loaded = dataset_options_revision(&[option("raleigh"), option("charlotte")]);

        // The exact transition that failed in production: options arrive after
        // first render, so empty -> loaded MUST be observable.
        assert_ne!(
            empty, loaded,
            "options arriving asynchronously must change the revision"
        );

        // Order matters: the browser's index-0 differs, so the same members in
        // a different order is a different set for this purpose.
        let reordered = dataset_options_revision(&[option("charlotte"), option("raleigh")]);
        assert_ne!(loaded, reordered);

        // Stable for an unchanged set, so this does not thrash the effect on
        // every unrelated render.
        assert_eq!(
            loaded,
            dataset_options_revision(&[option("raleigh"), option("charlotte")])
        );
    }

    /// A separator inside a value must not let two different sets collide.
    #[test]
    fn values_containing_a_separator_cannot_collide() {
        assert_ne!(
            dataset_options_revision(&[option("a,b")]),
            dataset_options_revision(&[option("a"), option("b")]),
        );
    }
}
