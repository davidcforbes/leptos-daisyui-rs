use crate::components::data_table::pagination::{PageSlot, page_window, row_range};
use crate::components::data_table::types::DataTableTexts;
use leptos::prelude::*;

/// Number of page slots (numbers, not counting ellipses) shown before the
/// strip collapses the middle range behind an ellipsis. Mirrors d2d-ui's
/// pagination window (`MAX_VISIBLE` ~7, radius 2 -- see `pagination.rs`).
const MAX_VISIBLE_PAGES: usize = 7;

/// DataTable pagination controls: a "Showing X-Y of Z" row-range caption on
/// the left, and Prev/numbered-page/Next buttons on the right.
#[component]
pub fn DataTableControls(
    /// Current page index (0-based)
    #[prop(into)]
    current_page: Signal<usize>,

    /// Callback to set current page
    set_current_page: WriteSignal<usize>,

    /// Total number of pages
    #[prop(into)]
    total_pages: Signal<usize>,

    /// Total number of (filtered) items across all pages -- drives the
    /// row-range caption.
    #[prop(into)]
    total_items: Signal<usize>,

    /// Rows per page -- drives the row-range caption.
    #[prop(into)]
    page_size: Signal<usize>,

    /// Custom text strings
    #[prop(into)]
    texts: Signal<DataTableTexts>,

    /// Pagination container class
    #[prop(optional, into)]
    pagination_class: &'static str,

    /// Prev/Next button class
    #[prop(optional, into)]
    button_class: &'static str,

    /// Numbered page button class (non-active pages)
    #[prop(optional, into)]
    page_button_class: &'static str,

    /// Numbered page button class for the current/active page
    #[prop(optional, into)]
    active_page_button_class: &'static str,

    /// Row-range caption class
    #[prop(optional, into)]
    range_class: &'static str,
) -> impl IntoView {
    let on_previous = move |_ev: leptos::ev::MouseEvent| {
        if current_page.get() > 0 {
            set_current_page.set(current_page.get() - 1);
        }
    };

    #[allow(unused_variables)]
    let on_next = move |_ev: leptos::ev::MouseEvent| {
        if current_page.get() + 1 < total_pages.get() {
            set_current_page.set(current_page.get() + 1);
        }
    };

    // Extract >= comparison outside view! macro to avoid RSX parser
    // interpreting `>=` as an HTML tag close.
    let prev_disabled = move || current_page.get() == 0;
    let next_disabled = move || current_page.get() + 1 >= total_pages.get();

    view! {
        <div class=pagination_class>
            <span class=range_class>
                {move || {
                    let total = total_items.get();
                    if total == 0 {
                        String::new()
                    } else {
                        let (start, end) = row_range(current_page.get(), page_size.get(), total);
                        texts
                            .with(|t| t.row_range.clone())
                            .replace("{start}", &start.to_string())
                            .replace("{end}", &end.to_string())
                            .replace("{total}", &total.to_string())
                    }
                }}
            </span>

            <div class="flex items-center gap-2">
                <button
                    class=button_class
                    disabled=prev_disabled
                    on:click=on_previous
                >
                    {move || texts.with(|t| t.previous.clone())}
                </button>

                <div class="join">
                    {move || {
                        page_window(current_page.get(), total_pages.get(), MAX_VISIBLE_PAGES)
                            .into_iter()
                            .map(|slot| match slot {
                                PageSlot::Page(idx) => {
                                    let is_active = idx == current_page.get();
                                    let class = if is_active {
                                        active_page_button_class
                                    } else {
                                        page_button_class
                                    };
                                    view! {
                                        <button
                                            class=class
                                            disabled=is_active
                                            on:click=move |_ev: leptos::ev::MouseEvent| {
                                                set_current_page.set(idx)
                                            }
                                        >
                                            {(idx + 1).to_string()}
                                        </button>
                                    }
                                        .into_any()
                                }
                                PageSlot::Ellipsis => {
                                    view! {
                                        <button
                                            class="btn btn-sm join-item btn-disabled"
                                            disabled=true
                                        >
                                            "…"
                                        </button>
                                    }
                                        .into_any()
                                }
                            })
                            .collect_view()
                    }}
                </div>

                <button
                    class=button_class
                    disabled=next_disabled
                    on:click=on_next
                >
                    {move || texts.with(|t| t.next.clone())}
                </button>
            </div>
        </div>
    }
}
