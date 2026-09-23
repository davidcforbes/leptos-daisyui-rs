use super::model::{PickerPerson, Step, next_selection, step_id, tab_stop_id};
use super::texts::PersonPickerTexts;
use crate::components::{Icon, IconSize, IconTileSize};
use leptos::{ev, prelude::*};
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::JsCast;

/// Mints one id per option for its stale-presence description, so an
/// option's `aria-describedby` never depends on the caller's person id being
/// a valid, page-unique DOM id.
fn next_presence_as_of_id() -> String {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!("ld-person-as-of-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Single-select listbox that can be emptied.
///
/// A LISTBOX, not the radiogroup `SelectableSummaryGroup` uses: radio
/// semantics forbid deselect-by-click, and this roster requires a second
/// click to deselect. One tab stop; Up/Down/Home/End move focus WITHOUT
/// selecting (selection drives assignment, so reading must not reassign);
/// Space/Enter toggle.
#[component]
pub fn PersonListbox(
    /// The people currently visible, already filtered.
    #[prop(into)]
    people: Signal<Vec<PickerPerson>>,
    /// The page-owned selection.
    #[prop(into)]
    selected: Signal<Option<String>>,
    /// Proposes `Some(id)` to select or `None` to clear.
    on_select: Callback<Option<String>>,
    /// The listbox's accessible name.
    #[prop(into)]
    label: Signal<String>,
    /// Copy, for the presence words.
    #[prop(into)]
    texts: Signal<PersonPickerTexts>,
    /// Stale-presence caption (ldui-8hmy), e.g. `"Status as of 1:52 PM"`.
    /// `None` renders live presence exactly as before. `Some` greys every
    /// dot in place and makes the caption each dot's tooltip and its
    /// option's accessible description. See [`PersonPicker`]'s prop of the
    /// same name.
    ///
    /// [`PersonPicker`]: super::PersonPicker
    #[prop(optional, into)]
    presence_as_of: Signal<Option<String>>,
) -> impl IntoView {
    // Focus and selection are SEPARATE here, unlike a radiogroup.
    let focused = RwSignal::new(None::<String>);
    let tab_stop = Memo::new(move |_| {
        let focused = focused.get();
        let selected = selected.get();
        people.with(|people| tab_stop_id(people, focused.as_deref(), selected.as_deref()))
    });

    let handle_keydown = move |event: ev::KeyboardEvent| {
        let Some(current) = tab_stop.get_untracked() else {
            return;
        };
        let key = event.key();
        if key == " " || key == "Enter" {
            event.prevent_default();
            on_select.run(next_selection(
                selected.get_untracked().as_deref(),
                &current,
            ));
            return;
        }
        let Some(step) = Step::from_key(&key) else {
            return;
        };
        event.prevent_default();
        if let Some(id) = people.with_untracked(|people| step_id(people, &current, step)) {
            focused.set(Some(id.clone()));
            focus_option(event.target(), &id);
        }
    };

    view! {
        <ul
            role="listbox"
            aria-label=move || label.get()
            class="grid min-w-0 gap-2"
            data-person-listbox="true"
            on:keydown=handle_keydown
        >
            <For
                each=move || people.get()
                key=|person| person.id.clone()
                children=move |person| {
                    view! {
                        <PersonOption
                            person=person
                            selected=selected
                            tab_stop=tab_stop
                            focused=focused
                            on_select=on_select
                            texts=texts
                            presence_as_of=presence_as_of
                        />
                    }
                }
            />
        </ul>
    }
}

#[component]
fn PersonOption(
    person: PickerPerson,
    selected: Signal<Option<String>>,
    tab_stop: Memo<Option<String>>,
    focused: RwSignal<Option<String>>,
    on_select: Callback<Option<String>>,
    texts: Signal<PersonPickerTexts>,
    presence_as_of: Signal<Option<String>>,
) -> impl IntoView {
    let id = person.id.clone();
    let is_selected = {
        let id = id.clone();
        Signal::derive(move || selected.with(|s| s.as_deref() == Some(id.as_str())))
    };
    let is_tab_stop = {
        let id = id.clone();
        Signal::derive(move || tab_stop.with(|t| t.as_deref() == Some(id.as_str())))
    };
    let secondary = {
        let secondary = person.secondary.clone();
        let presence = person.presence;
        Signal::derive(move || match presence {
            Some(presence) => format!(
                "{secondary} · {}",
                texts.with(|t| t.presence(presence).to_owned())
            ),
            None => secondary.clone(),
        })
    };
    let activity = person.activity.clone();
    let click_id = id.clone();
    let on_click = move |_: ev::MouseEvent| {
        focused.set(Some(click_id.clone()));
        on_select.run(next_selection(
            selected.get_untracked().as_deref(),
            &click_id,
        ));
    };
    let avatar_class = format!(
        "relative grid shrink-0 place-items-center rounded-full bg-neutral font-semibold text-neutral-content {}",
        IconTileSize::Md.as_str()
    );
    // `Unknown` has no dot class, so it renders no dot; its word still shows.
    // Stale (ldui-8hmy): the SAME span, greyed in place -- only the fill
    // class changes, so nothing moves -- with the caption as its tooltip.
    // The dot stays `aria-hidden` decoration; assistive tech gets the
    // caption as the OPTION's description, the element that takes focus.
    let has_dot = person
        .presence
        .is_some_and(|presence| presence.dot_class().is_some());
    let as_of = Signal::derive(move || {
        if has_dot {
            presence_as_of
                .get()
                .filter(|caption| !caption.trim().is_empty())
        } else {
            None
        }
    });
    let as_of_id = next_presence_as_of_id();
    let presence_dot = person.presence.and_then(|presence| {
        presence.dot_class().map(|_| {
            view! {
                <span
                    class=move || {
                        let fill = presence
                            .dot_class_when(as_of.with(Option::is_some))
                            .unwrap_or_default();
                        format!("absolute bottom-0 right-0 size-3 rounded-full {fill}")
                    }
                    aria-hidden="true"
                    title=move || as_of.get()
                    data-person-presence=presence.as_str()
                    data-person-presence-stale=move || as_of.with(Option::is_some).then_some("true")
                ></span>
            }
        })
    });
    let describedby_id = as_of_id.clone();
    let as_of_hint = move || {
        let id = as_of_id.clone();
        as_of.get().map(|caption| {
            // aria-hidden so the caption joins the option's DESCRIPTION
            // through aria-describedby without also joining its NAME,
            // which a listbox option computes from its content.
            view! {
                <span class="sr-only" aria-hidden="true" id=id data-person-presence-as-of="true">
                    {caption}
                </span>
            }
        })
    };

    view! {
        <li
            role="option"
            aria-selected=move || if is_selected.get() { "true" } else { "false" }
            tabindex=move || if is_tab_stop.get() { "0" } else { "-1" }
            data-person-option=id.clone()
            data-person-selected=move || is_selected.get().then_some("true")
            aria-describedby=move || as_of.with(Option::is_some).then(|| describedby_id.clone())
            class=move || option_class(is_selected.get())
            on:click=on_click
        >
            {as_of_hint}
            <span class=avatar_class>{person.initials.clone()} {presence_dot}</span>
            <span class="min-w-0 flex-1">
                <span class="block whitespace-normal font-semibold [overflow-wrap:anywhere]">
                    {person.name.clone()}
                </span>
                <span class=move || secondary_class(is_selected.get())>{secondary}</span>
                {activity.map(|activity| {
                    view! {
                        <span
                            class=move || secondary_class(is_selected.get())
                            data-person-activity="true"
                        >
                            {activity}
                        </span>
                    }
                })}
            </span>
            <span class="shrink-0" aria-hidden="true" data-person-check="true">
                {move || {
                    is_selected.get().then(|| view! { <Icon name="check" size=IconSize::Small /> })
                }}
            </span>
        </li>
    }
}

/// Selected: solid primary plus a border, so the state survives Windows
/// high-contrast mode, which strips backgrounds. The check icon carries it
/// too, so it never depends on colour alone.
fn option_class(selected: bool) -> &'static str {
    if selected {
        "ld-focus-ring flex w-full min-w-0 cursor-pointer items-start gap-3 rounded-box border border-primary bg-primary p-3 text-primary-content forced-colors:border-[Highlight]"
    } else {
        "ld-focus-ring flex w-full min-w-0 cursor-pointer items-start gap-3 rounded-box border border-base-300 bg-base-100 p-3 text-base-content hover:bg-base-200"
    }
}

/// `/75`, never `/60`: `/60` fails AA at 14px (3.37:1). Selected uses the
/// full content colour so its contrast is the theme's own pair.
fn secondary_class(selected: bool) -> &'static str {
    if selected {
        "block whitespace-normal text-sm [overflow-wrap:anywhere]"
    } else {
        "block whitespace-normal text-sm text-base-content/75 [overflow-wrap:anywhere]"
    }
}

/// Focus the option carrying `id`, searched within the listbox the event
/// came from and matched by attribute -- never by position.
fn focus_option(target: Option<web_sys::EventTarget>, id: &str) {
    let Some(element) = target.and_then(|target| target.dyn_into::<web_sys::Element>().ok()) else {
        return;
    };
    let Ok(Some(listbox)) = element.closest("[data-person-listbox]") else {
        return;
    };
    let Ok(options) = listbox.query_selector_all("[data-person-option]") else {
        return;
    };
    for index in 0..options.length() {
        let Some(node) = options.item(index) else {
            continue;
        };
        let Ok(option) = node.dyn_into::<web_sys::HtmlElement>() else {
            continue;
        };
        if option.get_attribute("data-person-option").as_deref() == Some(id) {
            let _ = option.focus();
            return;
        }
    }
}
