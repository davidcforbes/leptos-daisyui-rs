# PersonPicker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `PersonPicker`, an opinionated right-hand person roster (search, collapse rail, single-select listbox with deselect, selected badge, footer) reusable across 20+ pages.

**Architecture:** A new `src/patterns/person_picker/` module split into a pure model, a `PersonListbox` unit that owns only the listbox keyboard/selection contract, and a `PersonPicker` panel that wraps `FilterSidebar`. Selection is controlled: the page owns `selected`, the component proposes via `on_select`. `SnapshotTablePage::side_panel` stops imposing a width so the panel's fixed 360px is honoured.

**Tech Stack:** Rust 2024, Leptos 0.8 CSR, daisyUI 5, Tailwind 4, `pixelproof-web` + chromiumoxide browser suites, `cargo xtask`.

**Spec:** `doc/plans/2026-09-21-person-picker-design.md` — read it first; this plan argues from it.

## Global Constraints

- Run every `cargo xtask` command from the **repo root** (`C:\dev\leptos-daisyui-rs`); from `demo/` it fails with `os error 267`.
- Unset sccache first in any raw cargo shell: `Remove-Item Env:RUSTC_WRAPPER` (PowerShell) — web-sys otherwise fails `os error 206`.
- Format **per package**, never `--all`: `cargo fmt -p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask`.
- Lint with the gate's own flags: `cargo clippy -p leptos-daisyui-rs --all-targets --features test-mode -- -D warnings`. A bare clippy passing is not evidence.
- The gate is `cargo xtask verify` (20 steps). Read its `N/20 passed` summary line, never an exit echo.
- Muted text is `text-base-content/75` or darker. `/60` fails WCAG AA at 14px (3.37:1).
- Spacing only on the canonical scale (Tailwind `1 2 3 4 6 8 12 16 24`). Avatar size uses `IconTileSize`, never literal `w-*`/`h-*`.
- Tests find elements by stable data hooks (`data-person-*`, `data-testid`), **never** by DOM position.
- A new `#[tokio::test]` browser suite must be registered in its own xtask lane; prove it ran by the lane's test count, not by the lane passing.
- Browser lanes build the demo to wasm (~8 min) and must run on a **quiescent tree** — no edits, including `cargo fmt`, while one runs.
- Doc comments: keep every inline code span on ONE `///` line (a span wrapped across lines ICEs clippy 1.95).
- Every commit ends with `Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>`.
- After any scripted or tool edit, `git diff --numstat` and `git diff --ignore-cr-at-eol --numstat` must agree; if they differ, line endings were rewritten — restore them.

## File Map

| Path | Action | Responsibility |
|---|---|---|
| `src/patterns/person_picker/mod.rs` | create | module wiring and re-exports |
| `src/patterns/person_picker/model.rs` | create | `PickerPerson`, `PersonPresence`, pure rules |
| `src/patterns/person_picker/texts.rs` | create | `PersonPickerTexts`, English default and `::es()` |
| `src/patterns/person_picker/listbox.rs` | create | `PersonListbox` + `PersonOption` |
| `src/patterns/person_picker/panel.rs` | create | `PersonPicker` |
| `src/patterns/person_picker/tests.rs` | create | native tests |
| `src/patterns/mod.rs` | modify | register the module and re-export |
| `src/patterns/snapshot_table_page.rs` | modify | `side_panel` column drops `lg:w-80` |
| `demo/src/demos/snapshot_table_page.rs` | modify | side-panel fixture content gets an explicit width |
| `tests/snapshot_table_page_filter_actions_smoke.rs` | modify | side-panel width assertion follows content |
| `demo/src/demos/person_picker.rs` | create | showcase page, doubling as the browser fixture |
| `demo/src/demos/mod.rs` | modify | register the demo |
| `demo/src/main.rs` | modify | route |
| `demo/src/core/layout.rs` | modify | nav entry |
| `tests/person_picker_smoke.rs` | create | browser suite |
| `xtask/src/main.rs` | modify | `test-person-picker` lane |
| `doc/patterns/person-picker.md` | create | consumer guide + Office migration notes |

---

### Task 1: Model, texts and module skeleton

**Files:**
- Create: `src/patterns/person_picker/mod.rs`, `model.rs`, `texts.rs`, `tests.rs`
- Modify: `src/patterns/mod.rs`

**Interfaces:**
- Produces (used by Tasks 2–3):
  - `pub struct PickerPerson { pub id: String, pub name: String, pub secondary: String, pub initials: String, pub presence: Option<PersonPresence> }`
  - `pub enum PersonPresence { Available, Busy, Away, Offline }` with `pub fn dot_class(self) -> &'static str` and `pub fn as_str(self) -> &'static str`
  - `pub fn next_selection(current: Option<&str>, clicked: &str) -> Option<String>`
  - `pub fn matches_search(person: &PickerPerson, query: &str) -> bool`
  - `pub(crate) fn visible_people(roster: &[PickerPerson], query: &str) -> Vec<PickerPerson>`
  - `pub fn nameable_selection<'a>(roster: &'a [PickerPerson], selected: Option<&str>) -> Option<&'a PickerPerson>`
  - `pub(crate) fn tab_stop_id(visible: &[PickerPerson], focused: Option<&str>, selected: Option<&str>) -> Option<String>`
  - `pub(crate) enum Step { Next, Previous, First, Last }` with `pub(crate) fn from_key(key: &str) -> Option<Step>`
  - `pub(crate) fn step_id(visible: &[PickerPerson], current: &str, step: Step) -> Option<String>`
  - `pub struct PersonPickerTexts { .. }` with `Default`, `pub fn es() -> Self`, `pub fn presence(&self, presence: PersonPresence) -> &str`

- [ ] **Step 1: Create the module files with the tests first**

`src/patterns/person_picker/mod.rs`:

```rust
//! Opinionated person roster: search, collapse rail, a single-select
//! listbox that can be deselected, a selected badge and a footer.
//! See `doc/plans/2026-09-21-person-picker-design.md`.

mod model;
mod texts;

pub use model::{
    PersonPresence, PickerPerson, matches_search, nameable_selection, next_selection,
};
pub use texts::PersonPickerTexts;

#[cfg(test)]
mod tests;
```

`src/patterns/person_picker/tests.rs`:

```rust
use super::model::*;
use super::texts::PersonPickerTexts;

fn person(id: &str, name: &str, secondary: &str) -> PickerPerson {
    PickerPerson {
        id: id.to_owned(),
        name: name.to_owned(),
        secondary: secondary.to_owned(),
        initials: "XX".to_owned(),
        presence: None,
    }
}

fn roster() -> Vec<PickerPerson> {
    vec![
        person("a", "Ana Lopez", "Denver"),
        person("b", "Ben Ortiz", "Austin"),
        person("c", "Cara Diaz", "Denver"),
    ]
}

#[test]
fn next_selection_selects_toggles_off_and_switches() {
    assert_eq!(next_selection(None, "a"), Some("a".to_owned()), "select from none");
    assert_eq!(next_selection(Some("a"), "a"), None, "second click deselects");
    assert_eq!(next_selection(Some("a"), "b"), Some("b".to_owned()), "switch");
    assert_eq!(next_selection(Some("gone"), "a"), Some("a".to_owned()), "stale current");
}

#[test]
fn search_matches_name_or_secondary_case_insensitively() {
    let ana = person("a", "Ana Lopez", "Denver");
    assert!(matches_search(&ana, ""), "empty query matches all");
    assert!(matches_search(&ana, "   "), "blank query matches all");
    assert!(matches_search(&ana, "lopez"));
    assert!(matches_search(&ana, "DENV"), "secondary line, any case");
    assert!(!matches_search(&ana, "austin"));
    assert_eq!(
        visible_people(&roster(), "denver")
            .iter()
            .map(|p| p.id.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "c"]
    );
}

/// Office op-4c18j: a stale id must not make the panel name someone
/// nothing on screen identifies.
#[test]
fn only_a_selection_in_the_roster_is_nameable() {
    let roster = roster();
    assert_eq!(nameable_selection(&roster, Some("b")).map(|p| p.id.as_str()), Some("b"));
    assert_eq!(nameable_selection(&roster, Some("stale")), None);
    assert_eq!(nameable_selection(&roster, None), None);
}

/// Exactly one reachable card: focused if visible, else selected if
/// visible, else the first -- never none while anyone is visible.
#[test]
fn the_tab_stop_falls_back_so_a_visible_list_is_never_unreachable() {
    let all = roster();
    assert_eq!(tab_stop_id(&all, Some("b"), Some("c")).as_deref(), Some("b"), "focused wins");
    assert_eq!(tab_stop_id(&all, None, Some("c")).as_deref(), Some("c"), "then selected");
    assert_eq!(tab_stop_id(&all, None, None).as_deref(), Some("a"), "then first");
    let filtered = visible_people(&all, "austin");
    assert_eq!(
        tab_stop_id(&filtered, Some("a"), Some("c")).as_deref(),
        Some("b"),
        "focused and selected both filtered out: first visible, not none"
    );
    assert_eq!(tab_stop_id(&[], Some("a"), None), None, "nobody visible");
}

#[test]
fn arrow_keys_step_without_wrapping_and_home_end_jump() {
    let all = roster();
    assert_eq!(Step::from_key("ArrowDown"), Some(Step::Next));
    assert_eq!(Step::from_key("ArrowUp"), Some(Step::Previous));
    assert_eq!(Step::from_key("Home"), Some(Step::First));
    assert_eq!(Step::from_key("End"), Some(Step::Last));
    assert_eq!(Step::from_key("ArrowRight"), None, "a vertical list ignores left/right");
    assert_eq!(step_id(&all, "a", Step::Next).as_deref(), Some("b"));
    assert_eq!(step_id(&all, "c", Step::Next).as_deref(), Some("c"), "no wrap at the end");
    assert_eq!(step_id(&all, "a", Step::Previous).as_deref(), Some("a"), "no wrap at the start");
    assert_eq!(step_id(&all, "b", Step::Last).as_deref(), Some("c"));
    assert_eq!(step_id(&all, "c", Step::First).as_deref(), Some("a"));
}

#[test]
fn english_and_spanish_texts_differ_and_name_every_presence() {
    let en = PersonPickerTexts::default();
    let es = PersonPickerTexts::es();
    assert_ne!(en.search_placeholder, es.search_placeholder);
    for presence in [
        PersonPresence::Available,
        PersonPresence::Busy,
        PersonPresence::Away,
        PersonPresence::Offline,
    ] {
        assert!(!en.presence(presence).is_empty());
        assert_ne!(en.presence(presence), es.presence(presence));
    }
}
```

- [ ] **Step 2: Register the module**

In `src/patterns/mod.rs`, add `mod person_picker;` in alphabetical order after `mod page_state_panel;`, and add near the other `pub use` lines:

```rust
pub use person_picker::{
    PersonPickerTexts, PersonPresence, PickerPerson, matches_search, nameable_selection,
    next_selection,
};
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p leptos-daisyui-rs --features test-mode --lib person_picker`
Expected: FAIL to compile — `model` and `texts` modules do not exist.

- [ ] **Step 4: Write `model.rs`**

```rust
//! Data and pure rules for `PersonPicker`. Everything here is testable
//! without a DOM; the components only wire it up.

/// One person on the roster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerPerson {
    /// Stable key (Office: `worker_ref`).
    pub id: String,
    pub name: String,
    /// Office, role or similar.
    pub secondary: String,
    /// Supplied, not derived: correct initials across every culture's names
    /// is hard, and consumers already have them.
    pub initials: String,
    pub presence: Option<PersonPresence>,
}

/// Availability. Rendered as a dot AND a word -- a dot alone is
/// colour-only information (WCAG 1.4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonPresence {
    Available,
    Busy,
    Away,
    Offline,
}

impl PersonPresence {
    /// The dot's fill class.
    pub fn dot_class(self) -> &'static str {
        match self {
            Self::Available => "bg-success",
            Self::Busy => "bg-error",
            Self::Away => "bg-warning",
            Self::Offline => "bg-base-300",
        }
    }

    /// Stable data-hook value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Busy => "busy",
            Self::Away => "away",
            Self::Offline => "offline",
        }
    }
}

/// The selection a click or Space/Enter on `clicked` proposes: deselect
/// when it is already selected, otherwise select it.
pub fn next_selection(current: Option<&str>, clicked: &str) -> Option<String> {
    if current == Some(clicked) {
        None
    } else {
        Some(clicked.to_owned())
    }
}

/// Case-insensitive substring match on name or secondary line. A blank
/// query matches everyone.
pub fn matches_search(person: &PickerPerson, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || person.name.to_lowercase().contains(&query)
        || person.secondary.to_lowercase().contains(&query)
}

/// The roster as the search currently shows it, in roster order.
pub(crate) fn visible_people(roster: &[PickerPerson], query: &str) -> Vec<PickerPerson> {
    roster
        .iter()
        .filter(|person| matches_search(person, query))
        .cloned()
        .collect()
}

/// The selected person, but only if they are on the roster. A stale id the
/// roster no longer contains yields `None` (Office op-4c18j).
pub fn nameable_selection<'a>(
    roster: &'a [PickerPerson],
    selected: Option<&str>,
) -> Option<&'a PickerPerson> {
    let selected = selected?;
    roster.iter().find(|person| person.id == selected)
}

/// The one visible option that carries `tabindex="0"`: the focused one if
/// visible, else the selected one if visible, else the first. Never `None`
/// while anyone is visible -- the invariant `Heatmap` and `BarChart` broke.
pub(crate) fn tab_stop_id(
    visible: &[PickerPerson],
    focused: Option<&str>,
    selected: Option<&str>,
) -> Option<String> {
    for wanted in [focused, selected].into_iter().flatten() {
        if visible.iter().any(|person| person.id == wanted) {
            return Some(wanted.to_owned());
        }
    }
    visible.first().map(|person| person.id.clone())
}

/// A listbox navigation step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Next,
    Previous,
    First,
    Last,
}

impl Step {
    /// A vertical listbox moves on Up/Down only.
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        match key {
            "ArrowDown" => Some(Self::Next),
            "ArrowUp" => Some(Self::Previous),
            "Home" => Some(Self::First),
            "End" => Some(Self::Last),
            _ => None,
        }
    }
}

/// Where a step moves focus. Does NOT wrap -- the APG listbox contract.
pub(crate) fn step_id(visible: &[PickerPerson], current: &str, step: Step) -> Option<String> {
    let last = visible.len().checked_sub(1)?;
    let index = visible.iter().position(|person| person.id == current);
    let target = match (step, index) {
        (Step::First, _) | (_, None) => 0,
        (Step::Last, _) => last,
        (Step::Next, Some(index)) => (index + 1).min(last),
        (Step::Previous, Some(index)) => index.saturating_sub(1),
    };
    visible.get(target).map(|person| person.id.clone())
}
```

- [ ] **Step 5: Write `texts.rs`**

```rust
use super::model::PersonPresence;

/// Framework-owned copy. Pass it as a `Signal` so a language that arrives
/// after mount propagates instead of freezing (Office op-e6dsi).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonPickerTexts {
    pub search_placeholder: String,
    pub search_label: String,
    pub clear_search: String,
    /// The listbox's accessible name.
    pub roster_label: String,
    pub empty_roster: String,
    pub no_matches: String,
    pub selected_prefix: String,
    pub clear_selection: String,
    pub show: String,
    pub hide: String,
    pub available: String,
    pub busy: String,
    pub away: String,
    pub offline: String,
}

impl Default for PersonPickerTexts {
    fn default() -> Self {
        Self {
            search_placeholder: "Find a person".to_owned(),
            search_label: "Search people".to_owned(),
            clear_search: "Clear search".to_owned(),
            roster_label: "People".to_owned(),
            empty_roster: "No one is on this roster.".to_owned(),
            no_matches: "No one matches your search.".to_owned(),
            selected_prefix: "Selected:".to_owned(),
            clear_selection: "Clear selection".to_owned(),
            show: "Show".to_owned(),
            hide: "Hide".to_owned(),
            available: "Available".to_owned(),
            busy: "Busy".to_owned(),
            away: "Away".to_owned(),
            offline: "Offline".to_owned(),
        }
    }
}

impl PersonPickerTexts {
    /// Spanish.
    pub fn es() -> Self {
        Self {
            search_placeholder: "Buscar una persona".to_owned(),
            search_label: "Buscar personas".to_owned(),
            clear_search: "Borrar búsqueda".to_owned(),
            roster_label: "Personas".to_owned(),
            empty_roster: "No hay nadie en esta lista.".to_owned(),
            no_matches: "Nadie coincide con su búsqueda.".to_owned(),
            selected_prefix: "Seleccionado:".to_owned(),
            clear_selection: "Borrar selección".to_owned(),
            show: "Mostrar".to_owned(),
            hide: "Ocultar".to_owned(),
            available: "Disponible".to_owned(),
            busy: "Ocupado".to_owned(),
            away: "Ausente".to_owned(),
            offline: "Desconectado".to_owned(),
        }
    }

    /// The word shown beside the presence dot.
    pub fn presence(&self, presence: PersonPresence) -> &str {
        match presence {
            PersonPresence::Available => &self.available,
            PersonPresence::Busy => &self.busy,
            PersonPresence::Away => &self.away,
            PersonPresence::Offline => &self.offline,
        }
    }
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p leptos-daisyui-rs --features test-mode --lib person_picker`
Expected: PASS, 6 tests. (Clippy will flag the `pub(crate)` items as unused until Task 2 consumes them — do not add `allow`s; Task 2 lands before the next lint.)

- [ ] **Step 7: Commit**

```bash
git add src/patterns/person_picker src/patterns/mod.rs
git commit -m "feat(patterns): PersonPicker model and texts (pure rules, native-tested)"
```

---

### Task 2: `PersonListbox`

**Files:**
- Create: `src/patterns/person_picker/listbox.rs`
- Modify: `src/patterns/person_picker/mod.rs`

**Interfaces:**
- Consumes: everything Task 1 produced.
- Produces: `pub fn PersonListbox(people: Signal<Vec<PickerPerson>>, selected: Signal<Option<String>>, on_select: Callback<Option<String>>, label: Signal<String>, texts: Signal<PersonPickerTexts>) -> impl IntoView`, rendering `ul[role=listbox][data-person-listbox]` with `li[role=option][data-person-option=<id>]`, `aria-selected`, `tabindex`, and `data-person-selected="true"` on the selected option.

This unit has no native test: its behaviour is DOM events and focus, proven by the browser suite in Task 6. Its rules are already native-tested in Task 1.

- [ ] **Step 1: Write `listbox.rs`**

```rust
use super::model::{PickerPerson, Step, next_selection, step_id, tab_stop_id};
use super::texts::PersonPickerTexts;
use crate::components::{Icon, IconSize, IconTileSize};
use leptos::{ev, prelude::*};
use wasm_bindgen::JsCast;

/// Single-select listbox that can be emptied.
///
/// A LISTBOX, not the radiogroup `SelectableSummaryGroup` uses: radio
/// semantics forbid deselect-by-click, and this roster requires a second
/// click to deselect. One tab stop; Up/Down/Home/End move focus WITHOUT
/// selecting (selection drives assignment, so reading must not reassign);
/// Space/Enter toggle.
#[component]
pub fn PersonListbox(
    #[prop(into)] people: Signal<Vec<PickerPerson>>,
    #[prop(into)] selected: Signal<Option<String>>,
    on_select: Callback<Option<String>>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] texts: Signal<PersonPickerTexts>,
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
            on_select.run(next_selection(selected.get_untracked().as_deref(), &current));
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
                children=move |person| view! {
                    <PersonOption
                        person=person
                        selected=selected
                        tab_stop=tab_stop
                        focused=focused
                        on_select=on_select
                        texts=texts
                    />
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
            Some(presence) => {
                format!("{secondary} · {}", texts.with(|t| t.presence(presence).to_owned()))
            }
            None => secondary.clone(),
        })
    };
    let click_id = id.clone();
    let on_click = move |_: ev::MouseEvent| {
        focused.set(Some(click_id.clone()));
        on_select.run(next_selection(selected.get_untracked().as_deref(), &click_id));
    };

    view! {
        <li
            role="option"
            aria-selected=move || if is_selected.get() { "true" } else { "false" }
            tabindex=move || if is_tab_stop.get() { "0" } else { "-1" }
            data-person-option=id.clone()
            data-person-selected=move || is_selected.get().then_some("true")
            class=move || option_class(is_selected.get())
            on:click=on_click
        >
            <span class=format!(
                "relative grid shrink-0 place-items-center rounded-full bg-neutral font-semibold text-neutral-content {}",
                IconTileSize::Md.as_str()
            )>
                {person.initials.clone()}
                {person.presence.map(|presence| view! {
                    <span
                        class=format!("absolute bottom-0 right-0 size-3 rounded-full {}", presence.dot_class())
                        aria-hidden="true"
                        data-person-presence=presence.as_str()
                    ></span>
                })}
            </span>
            <span class="min-w-0 flex-1">
                <span class="block whitespace-normal font-semibold [overflow-wrap:anywhere]">
                    {person.name.clone()}
                </span>
                <span class=move || secondary_class(is_selected.get())>{secondary}</span>
            </span>
            <span class="shrink-0" aria-hidden="true" data-person-check="true">
                {move || is_selected.get().then(|| view! { <Icon name="check" size=IconSize::Small /> })}
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
```

- [ ] **Step 2: Wire it into `mod.rs`**

Add `mod listbox;` and `pub use listbox::PersonListbox;` to `src/patterns/person_picker/mod.rs`, and add `PersonListbox` to the `pub use person_picker::{..}` list in `src/patterns/mod.rs`.

- [ ] **Step 3: Lint with the gate's flags**

Run: `cargo clippy -p leptos-daisyui-rs --all-targets --features test-mode -- -D warnings`
Expected: exit 0. The Task 1 `pub(crate)` items are now used.

- [ ] **Step 4: Run the native tests again**

Run: `cargo test -p leptos-daisyui-rs --features test-mode --lib person_picker`
Expected: PASS, 6 tests.

- [ ] **Step 5: Commit**

```bash
git add src/patterns/person_picker src/patterns/mod.rs
git commit -m "feat(patterns): PersonListbox -- single-select listbox that can be emptied"
```

---

### Task 3: `PersonPicker` panel

**Files:**
- Create: `src/patterns/person_picker/panel.rs`
- Modify: `src/patterns/person_picker/mod.rs`, `src/patterns/mod.rs`

**Interfaces:**
- Consumes: `PersonListbox`, `visible_people`, `nameable_selection`, `PersonPickerTexts`; `crate::components::{FilterSidebar, SidebarSide}`.
- Produces: `pub fn PersonPicker(roster: Signal<Vec<PickerPerson>>, selected: Signal<Option<String>>, on_select: Callback<Option<String>>, collapsed: Signal<bool>, on_toggle_collapsed: Callback<()>, title: Signal<String>, texts: Signal<PersonPickerTexts> /* default English */, footer: Option<Children>) -> impl IntoView`. Data hooks: `[data-person-picker]`, `[data-person-search]`, `[data-person-search-clear]`, `[data-person-selected-summary=<id>]`, `[data-person-selection-clear]`, `[data-person-roster]`, `[data-person-roster-empty]`, `[data-person-no-matches]`, `[data-person-picker-footer]`.

- [ ] **Step 1: Write `panel.rs`**

```rust
use super::listbox::PersonListbox;
use super::model::{PickerPerson, nameable_selection, visible_people};
use super::texts::PersonPickerTexts;
use crate::components::{FilterSidebar, SidebarSide};
use leptos::{ev, prelude::*};

/// The opinionated roster panel. Fixed 360px, text wraps, the card is the
/// selection control. The page owns `selected` and `collapsed`; search text
/// is internal view state and never clears the selection.
#[component]
pub fn PersonPicker(
    #[prop(into)] roster: Signal<Vec<PickerPerson>>,
    #[prop(into)] selected: Signal<Option<String>>,
    on_select: Callback<Option<String>>,
    #[prop(into)] collapsed: Signal<bool>,
    on_toggle_collapsed: Callback<()>,
    #[prop(into)] title: Signal<String>,
    #[prop(into, default = Signal::stored(PersonPickerTexts::default()))]
    texts: Signal<PersonPickerTexts>,
    #[prop(optional)] footer: Option<Children>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let visible = Memo::new(move |_| {
        let query = search.get();
        roster.with(|roster| visible_people(roster, &query))
    });
    // Memos on emptiness so the branch below re-renders only when it flips,
    // not on every keystroke (which would remount the listbox).
    let roster_empty = Memo::new(move |_| roster.with(Vec::is_empty));
    let visible_empty = Memo::new(move |_| visible.with(Vec::is_empty));
    let named = Memo::new(move |_| {
        let selected = selected.get();
        roster.with(|roster| nameable_selection(roster, selected.as_deref()).cloned())
    });

    let on_search = move |event: ev::Event| search.set(event_target_value(&event));

    view! {
        <FilterSidebar
            collapsed=collapsed
            on_toggle=on_toggle_collapsed
            // Counts only a NAMEABLE pick (Office op-4c18j).
            active_count=Signal::derive(move || usize::from(named.get().is_some()))
            title=title
            toggle_label=Signal::derive(move || {
                texts.with(|t| if collapsed.get() { t.show.clone() } else { t.hide.clone() })
            })
            side=SidebarSide::Right
            expanded_width="w-[360px] min-w-[360px] max-w-[360px]"
            attr:data-person-picker="true"
        >
            <div class="grid min-w-0 gap-3 p-3">
                <div class="relative min-w-0">
                    <input
                        type="search"
                        // The native cancel button would be a second x.
                        class="input input-sm w-full pr-8 [&::-webkit-search-cancel-button]:hidden"
                        prop:value=move || search.get()
                        placeholder=move || texts.with(|t| t.search_placeholder.clone())
                        aria-label=move || texts.with(|t| t.search_label.clone())
                        data-person-search="true"
                        on:input=on_search
                    />
                    {move || (!search.with(String::is_empty)).then(|| view! {
                        <button
                            type="button"
                            class="btn btn-ghost btn-xs btn-circle absolute right-1 top-1/2 -translate-y-1/2"
                            aria-label=move || texts.with(|t| t.clear_search.clone())
                            data-person-search-clear="true"
                            on:click=move |_| search.set(String::new())
                        >
                            "×"
                        </button>
                    })}
                </div>

                {move || named.get().map(|person| view! {
                    <div
                        class="flex min-w-0 items-center gap-2 rounded-box bg-base-200 px-3 py-2 text-sm"
                        data-person-selected-summary=person.id.clone()
                    >
                        <span class="min-w-0 flex-1 whitespace-normal [overflow-wrap:anywhere]">
                            {move || texts.with(|t| t.selected_prefix.clone())}
                            " "
                            <strong>{person.name.clone()}</strong>
                        </span>
                        <button
                            type="button"
                            class="btn btn-ghost btn-xs btn-circle shrink-0"
                            aria-label=move || texts.with(|t| t.clear_selection.clone())
                            data-person-selection-clear="true"
                            on:click=move |_| on_select.run(None)
                        >
                            "×"
                        </button>
                    </div>
                })}

                // overflow-y-auto ALONE forces overflow-x to auto as well --
                // the cause of the horizontal scroll this replaces.
                <div
                    class="max-h-[70dvh] min-w-0 overflow-y-auto overflow-x-hidden"
                    data-person-roster="true"
                >
                    {move || {
                        if roster_empty.get() {
                            view! {
                                <p class="text-sm text-base-content/75" data-person-roster-empty="true">
                                    {move || texts.with(|t| t.empty_roster.clone())}
                                </p>
                            }
                            .into_any()
                        } else if visible_empty.get() {
                            view! {
                                <p class="text-sm text-base-content/75" data-person-no-matches="true">
                                    {move || texts.with(|t| t.no_matches.clone())}
                                </p>
                            }
                            .into_any()
                        } else {
                            view! {
                                <PersonListbox
                                    people=visible
                                    selected=selected
                                    on_select=on_select
                                    label=Signal::derive(move || texts.with(|t| t.roster_label.clone()))
                                    texts=texts
                                />
                            }
                            .into_any()
                        }
                    }}
                </div>

                {footer.map(|footer| view! {
                    <div class="min-w-0" data-person-picker-footer="true">{footer()}</div>
                })}
            </div>
        </FilterSidebar>
    }
}
```

- [ ] **Step 2: Wire it in**

Add `mod panel;` and `pub use panel::PersonPicker;` to `src/patterns/person_picker/mod.rs`, and `PersonPicker` to the `pub use person_picker::{..}` list in `src/patterns/mod.rs`.

- [ ] **Step 3: Lint with the gate's flags**

Run: `cargo clippy -p leptos-daisyui-rs --all-targets --features test-mode -- -D warnings`
Expected: exit 0. If the `view!` macro reports a misleading type error on an unrelated attribute, extract the offending inline closure into a named `let` first (known Leptos inference trap).

- [ ] **Step 4: Lint the wasm target too**

Run: `cargo clippy -p leptos-daisyui-rs --lib --target wasm32-unknown-unknown --features test-mode -- -D warnings`
Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add src/patterns/person_picker src/patterns/mod.rs
git commit -m "feat(patterns): PersonPicker panel -- search, rail, badge, footer, 360px"
```

---

### Task 4: `side_panel` stops imposing a width

**Files:**
- Modify: `src/patterns/snapshot_table_page.rs` (the `<aside ... data-snapshot-page-slot="side-panel">` class, and the `side_panel` prop doc)
- Modify: `demo/src/demos/snapshot_table_page.rs` (the `side-panel-content` div)
- Modify: `tests/snapshot_table_page_filter_actions_smoke.rs` (the `panelWidth` assertion)

**Interfaces:**
- Produces: the side-panel column sizes to its content from `lg` up; below `lg` it stays `w-full`.

- [ ] **Step 1: Update the browser assertion first**

In `tests/snapshot_table_page_filter_actions_smoke.rs`, replace:

```rust
    assert!(
        side["panelWidth"].as_f64().is_some_and(|w| (300.0..=340.0).contains(&w)),
        "the panel is the w-80 column: {side}"
    );
```

with:

```rust
    assert!(
        side["panelWidth"].as_f64().is_some_and(|w| (355.0..=365.0).contains(&w)),
        "the column takes its CONTENT's width (the fixture panel is 360px); a slot \
         must not impose a width, or a 360px PersonPicker scrolls sideways in it: {side}"
    );
```

- [ ] **Step 2: Give the fixture content an explicit width**

In `demo/src/demos/snapshot_table_page.rs`, change the side-panel content div's class from `"rounded-box border border-base-300 p-4"` to `"w-[360px] rounded-box border border-base-300 p-4"`.

- [ ] **Step 3: Drop the imposed width**

In `src/patterns/snapshot_table_page.rs`, change the aside's class from `"w-full min-w-0 shrink-0 lg:w-80 lg:min-h-0 lg:overflow-auto"` to `"w-full min-w-0 shrink-0 lg:w-auto lg:min-h-0 lg:overflow-auto"`, and in the `side_panel` prop doc replace the sentence beginning `The panel is \`w-80\` on the row` with: `From \`lg\` the column takes its content's width -- a slot must not impose one, or a fixed-width panel such as \`PersonPicker\` overflows it -- and scrolls internally rather than stretching the row.`

- [ ] **Step 4: Lint**

Run: `cargo clippy -p leptos-daisyui-rs --all-targets --features test-mode -- -D warnings` and `cargo clippy -p leptos-daisyui-showcase --all-targets -- -D warnings`
Expected: exit 0 for both. (The browser run for this assertion happens in Task 7 with the rest.)

- [ ] **Step 5: Commit**

```bash
git add src/patterns/snapshot_table_page.rs demo/src/demos/snapshot_table_page.rs tests/snapshot_table_page_filter_actions_smoke.rs
git commit -m "fix(patterns): side_panel column takes its content's width, not 320px"
```

---

### Task 5: Demo page (showcase and fixture)

**Files:**
- Create: `demo/src/demos/person_picker.rs`
- Modify: `demo/src/demos/mod.rs`, `demo/src/main.rs`, `demo/src/core/layout.rs`

**Interfaces:**
- Consumes: `leptos_daisyui_rs::patterns::{PersonPicker, PersonPickerTexts, PersonPresence, PickerPerson}`.
- Produces: route `/components/person_picker`; hooks `[data-testid="person-picker-selected"]` (text: the selected id or `(none)`), `[data-testid="person-picker-spanish"]` (toggle button). Roster ids `ana`, `ben`, `cara`, `dmitri`, `long`, `email`; `long` has a very long name; `email` has an unbroken email as its secondary line.

- [ ] **Step 1: Write the page**

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{PersonPicker, PersonPickerTexts, PersonPresence, PickerPerson};

fn person(
    id: &str,
    name: &str,
    secondary: &str,
    initials: &str,
    presence: Option<PersonPresence>,
) -> PickerPerson {
    PickerPerson {
        id: id.to_owned(),
        name: name.to_owned(),
        secondary: secondary.to_owned(),
        initials: initials.to_owned(),
        presence,
    }
}

fn roster() -> Vec<PickerPerson> {
    vec![
        person("ana", "Ana Lopez", "Denver", "AL", Some(PersonPresence::Available)),
        person("ben", "Ben Ortiz", "Austin", "BO", Some(PersonPresence::Busy)),
        person("cara", "Cara Diaz", "Denver", "CD", Some(PersonPresence::Away)),
        person("dmitri", "Dmitri Volkov", "Phoenix", "DV", None),
        // Wrapping fixtures: must fold, never scroll sideways.
        person(
            "long",
            "Maria de los Angeles Fernandez-Castellanos y Rodriguez",
            "Dallas - Client Success and Standing Orders",
            "MF",
            Some(PersonPresence::Offline),
        ),
        person(
            "email",
            "Priya Raghunathan",
            "priya.raghunathan.client-coordination@example-lawfirm.com",
            "PR",
            Some(PersonPresence::Available),
        ),
    ]
}

/// Showcase for `PersonPicker`; also the browser suite's fixture.
#[component]
pub fn PersonPickerDemo() -> impl IntoView {
    let selected = RwSignal::new(None::<String>);
    let collapsed = RwSignal::new(false);
    let spanish = RwSignal::new(false);
    let texts = Signal::derive(move || {
        if spanish.get() { PersonPickerTexts::es() } else { PersonPickerTexts::default() }
    });

    view! {
        <section class="space-y-4 p-4" data-testid="person-picker-demo">
            <h1 class="ld-text-display font-semibold">"Person Picker"</h1>
            <p class="text-base-content/75">
                "An opinionated roster: click a card to select, click it again to deselect."
            </p>
            <div class="flex flex-wrap items-center gap-4">
                <span>
                    "Selected: "
                    <code data-testid="person-picker-selected">
                        {move || selected.get().unwrap_or_else(|| "(none)".to_owned())}
                    </code>
                </span>
                <button
                    type="button"
                    class="btn btn-sm"
                    data-testid="person-picker-spanish"
                    on:click=move |_| spanish.update(|value| *value = !*value)
                >
                    "Toggle Spanish"
                </button>
            </div>
            <div class="flex justify-end">
                <PersonPicker
                    roster=Signal::stored(roster())
                    selected=selected
                    on_select=Callback::new(move |next| selected.set(next))
                    collapsed=collapsed
                    on_toggle_collapsed=Callback::new(move |()| collapsed.update(|c| *c = !*c))
                    title=Signal::stored("Client Coordinators".to_owned())
                    texts=texts
                />
            </div>
        </section>
    }
}
```

- [ ] **Step 2: Register it**

- `demo/src/demos/mod.rs`: add `pub mod person_picker;` after `pub mod page_quick_actions;` (alphabetical), and `pub use person_picker::*;` in the matching `pub use` block.
- `demo/src/main.rs`: add `<Route path=path!("/person_picker") view=PersonPickerDemo />` in alphabetical order among the component routes.
- `demo/src/core/layout.rs`: in the same category as "Selectable Summary", add

```rust
                ComponentItem {
                    name: "Person Picker",
                    href: "/components/person_picker",
                    value: "person_picker",
                },
```

in alphabetical order.

- [ ] **Step 3: Check the demo compiles**

Run: `cargo xtask check-demo`
Expected: `PASS check-demo`.

- [ ] **Step 4: Commit**

```bash
git add demo/src
git commit -m "demo: PersonPicker showcase page (doubles as the browser fixture)"
```

---

### Task 6: Browser suite and its lane

**Files:**
- Create: `tests/person_picker_smoke.rs`
- Modify: `xtask/src/main.rs`

**Interfaces:**
- Consumes: the Task 5 page and every data hook listed in Tasks 2, 3 and 5.

- [ ] **Step 1: Write the suite**

```rust
//! Browser proof for `PersonPicker` (doc/plans/2026-09-21-person-picker-design.md):
//! the listbox keyboard contract, toggle-off, the one-reachable-card
//! invariant under search, both x controls, wrapping without horizontal
//! overflow, the fixed width, and WCAG AA contrast in both states.

mod common;

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use pixelproof_web::ViewportSize;
use serde_json::{Value, json};

const PAGE: &str = "/components/person_picker";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate person picker fixture")
        .into_value()
        .expect("fixture JSON")
}

async fn open(path: &str) -> pixelproof_web::Harness {
    let h = harness_at(path).await;
    begin_browser_error_capture(&h).await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set a wide viewport");
    common::wait_for_selector(&h, "[data-person-listbox]").await;
    h
}

async fn press_key(h: &pixelproof_web::Harness, key: &str, code: &str, key_code: i64) {
    for kind in [DispatchKeyEventType::KeyDown, DispatchKeyEventType::KeyUp] {
        let mut params = DispatchKeyEventParams::builder()
            .r#type(kind.clone())
            .key(key)
            .code(code)
            .windows_virtual_key_code(key_code)
            .native_virtual_key_code(key_code);
        if key == " " && kind == DispatchKeyEventType::KeyDown {
            params = params.text(" ");
        }
        h.page()
            .execute(params.build().expect("key params"))
            .await
            .expect("dispatch key");
    }
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

async fn state(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const opts = [...document.querySelectorAll('[data-person-picker] [data-person-option]')];
            const id = el => el ? el.getAttribute('data-person-option') : null;
            return {
                visible: opts.map(id),
                tabStops: opts.filter(o => o.getAttribute('tabindex') === '0').map(id),
                ariaSelected: opts.filter(o => o.getAttribute('aria-selected') === 'true').map(id),
                active: id(document.activeElement?.closest?.('[data-person-option]')),
                output: document.querySelector('[data-testid="person-picker-selected"]').textContent.trim(),
                summary: document.querySelector('[data-person-selected-summary]')
                    ?.getAttribute('data-person-selected-summary') ?? null,
                searchClear: !!document.querySelector('[data-person-search-clear]'),
            };
        })()"#,
    )
    .await
}

async fn focus_tab_stop(h: &pixelproof_web::Harness) {
    eval_json(
        h,
        r#"(() => { document.querySelector('[data-person-listbox] [tabindex="0"]').focus(); return true; })()"#,
    )
    .await;
}

async fn type_search(h: &pixelproof_web::Harness, text: &str) {
    let input = h
        .page()
        .find_element("[data-person-search]")
        .await
        .expect("search input");
    input.focus().await.expect("focus search");
    input.type_str(text).await.expect("type search");
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-person-picker)"]
async fn keyboard_moves_focus_without_selecting_and_space_or_click_toggles() {
    let h = open(PAGE).await;

    let s = state(&h).await;
    assert_eq!(s["tabStops"], json!(["ana"]), "one tab stop, the first person: {s}");
    assert_eq!(s["ariaSelected"], json!([]), "nothing selected yet: {s}");

    focus_tab_stop(&h).await;
    press_key(&h, "ArrowDown", "ArrowDown", 40).await;
    let s = state(&h).await;
    assert_eq!(s["active"], json!("ben"), "ArrowDown moves focus: {s}");
    assert_eq!(s["ariaSelected"], json!([]), "...WITHOUT selecting: {s}");
    assert_eq!(s["tabStops"], json!(["ben"]), "the tab stop follows focus: {s}");

    press_key(&h, " ", "Space", 32).await;
    let s = state(&h).await;
    assert_eq!(s["ariaSelected"], json!(["ben"]), "Space selects: {s}");
    assert_eq!(s["output"], json!("ben"), "the page received the proposal: {s}");

    press_key(&h, " ", "Space", 32).await;
    assert_eq!(state(&h).await["output"], json!("(none)"), "Space again deselects");

    press_key(&h, "End", "End", 35).await;
    press_key(&h, "Enter", "Enter", 13).await;
    let s = state(&h).await;
    assert_eq!(s["active"], json!("email"), "End jumps to the last person: {s}");
    assert_eq!(s["output"], json!("email"), "Enter selects: {s}");

    click(&h, "[data-person-option=\"ana\"]").await;
    assert_eq!(state(&h).await["output"], json!("ana"), "click switches the selection");
    click(&h, "[data-person-option=\"ana\"]").await;
    assert_eq!(state(&h).await["output"], json!("(none)"), "second click deselects");

    assert_no_browser_errors(&h, "person picker keyboard and click").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-person-picker)"]
async fn search_keeps_one_tab_stop_and_each_x_clears_what_it_sits_on() {
    let h = open(PAGE).await;

    click(&h, "[data-person-option=\"cara\"]").await;
    type_search(&h, "austin").await;
    let s = state(&h).await;
    assert_eq!(s["visible"], json!(["ben"]), "search filters: {s}");
    assert_eq!(s["tabStops"], json!(["ben"]), "exactly one reachable card after filtering: {s}");
    assert_eq!(s["output"], json!("cara"), "search never clears the selection: {s}");
    assert_eq!(s["summary"], json!("cara"), "the badge still names the hidden pick: {s}");
    assert_eq!(s["searchClear"], json!(true), "a non-empty search shows its x: {s}");

    click(&h, "[data-person-search-clear]").await;
    let s = state(&h).await;
    assert_eq!(s["visible"].as_array().map(Vec::len), Some(6), "search x clears the search: {s}");
    assert_eq!(s["output"], json!("cara"), "...and leaves the selection alone: {s}");
    assert_eq!(s["searchClear"], json!(false), "no x on an empty search: {s}");

    click(&h, "[data-person-selection-clear]").await;
    let s = state(&h).await;
    assert_eq!(s["output"], json!("(none)"), "badge x clears the selection: {s}");
    assert_eq!(s["summary"], json!(null), "and the badge goes: {s}");

    type_search(&h, "zzzz").await;
    let empty = eval_json(
        &h,
        r#"(() => ({
            noMatches: !!document.querySelector('[data-person-no-matches]'),
            listbox: !!document.querySelector('[data-person-listbox]'),
        }))()"#,
    )
    .await;
    assert_eq!(empty, json!({"noMatches": true, "listbox": false}), "no empty listbox: {empty}");

    assert_no_browser_errors(&h, "person picker search").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-person-picker)"]
async fn cards_wrap_at_a_fixed_360px_and_pass_contrast_at_rest_and_selected() {
    let h = open(PAGE).await;

    let layout = eval_json(
        &h,
        r#"(() => {
            const picker = document.querySelector('[data-person-picker]');
            const roster = document.querySelector('[data-person-roster]');
            const overflowing = [...document.querySelectorAll('[data-person-option]')]
                .filter(o => o.scrollWidth > o.clientWidth + 1)
                .map(o => o.getAttribute('data-person-option'));
            const long = document.querySelector('[data-person-option="long"]');
            return {
                width: picker.getBoundingClientRect().width,
                rosterOverflowX: roster.scrollWidth > roster.clientWidth + 1,
                overflowing,
                longWrapped: long.getBoundingClientRect().height > 60,
            };
        })()"#,
    )
    .await;
    assert!(
        layout["width"].as_f64().is_some_and(|w| (355.0..=365.0).contains(&w)),
        "fixed 360px: {layout}"
    );
    assert_eq!(layout["rosterOverflowX"], json!(false), "no horizontal scroll: {layout}");
    assert_eq!(layout["overflowing"], json!([]), "no card overflows sideways: {layout}");
    assert_eq!(layout["longWrapped"], json!(true), "the long name wraps: {layout}");

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let _ = axe.run(h.page()).await.expect("inject axe-core");
    async fn contrast(h: &pixelproof_web::Harness) -> Value {
        eval_json(
            h,
            r#"(async () => {
                const report = await axe.run(document.querySelector('[data-person-picker]'), {
                    runOnly: { type: 'rule', values: ['color-contrast'] },
                    resultTypes: ['violations'],
                });
                return report.violations.map(v => ({
                    id: v.id,
                    nodes: v.nodes.slice(0, 5).map(n => ({ target: n.target, summary: n.failureSummary })),
                }));
            })()"#,
        )
        .await
    }
    let rest = contrast(&h).await;
    assert_eq!(rest, json!([]), "AA contrast at rest: {rest}");
    click(&h, "[data-person-option=\"ana\"]").await;
    let picked = contrast(&h).await;
    assert_eq!(picked, json!([]), "AA contrast with a selection: {picked}");

    assert_no_browser_errors(&h, "person picker layout and contrast").await;
}
```

- [ ] **Step 2: Register the lane**

In `xtask/src/main.rs`, add after `fn selectable_summary_step()`:

```rust
/// Focused browser proof for `PersonPicker`
/// (`doc/plans/2026-09-21-person-picker-design.md`): the listbox keyboard
/// contract with toggle-off, one reachable card under search, both x
/// controls, wrapping at a fixed 360px, and AA contrast in both states.
fn person_picker_step() -> Step {
    Step {
        name: "test-person-picker",
        run: Run::BrowserSuite {
            test: "person_picker_smoke",
            html_target: None,
        },
    }
}
```

In the dispatch `match`, add `"test-person-picker" => run_steps(&[person_picker_step()]),` beside `"test-selectable-summary"`, and add `|test-person-picker` to the usage string after `test-selectable-summary`.

- [ ] **Step 3: Compile the suite without running it**

Run: `cargo test -p leptos-daisyui-rs --features test-mode --test person_picker_smoke --no-run`
Expected: compiles. **This proves nothing about behaviour** — every browser suite in this repo that was handed off compile-only failed on first execution. Step 4 is the proof.

- [ ] **Step 4: Run the lane on a quiescent tree**

Run: `cargo xtask test-person-picker` (budget ~8 minutes; do not edit anything meanwhile)
Expected: `PASS test-person-picker` and `test result: ok. 3 passed` — **the count must be 3**. If a test fails on geometry, measure before changing the assertion; if it fails with `STALE STYLESHEET (ldui-hun)` and the markers differ, the tree moved — re-run rather than investigate.

- [ ] **Step 5: Commit**

```bash
git add tests/person_picker_smoke.rs xtask/src/main.rs
git commit -m "test(patterns): PersonPicker browser proof in its own xtask lane"
```

---

### Task 7: Docs, full verification, push, hand-off

**Files:**
- Create: `doc/patterns/person-picker.md`

- [ ] **Step 1: Write the consumer guide**

`doc/patterns/person-picker.md`:

````markdown
# PersonPicker

An opinionated person roster for the right edge of a page: search, a
collapse rail, a single-select list, a selected badge and a footer. Design
and rationale: `doc/plans/2026-09-21-person-picker-design.md`.

```rust
let selected = RwSignal::new(None::<String>);
let collapsed = RwSignal::new(false);

view! {
    <PersonPicker
        roster=roster                       // Signal<Vec<PickerPerson>>
        selected=selected
        on_select=Callback::new(move |next| selected.set(next))
        collapsed=collapsed
        on_toggle_collapsed=Callback::new(move |()| collapsed.update(|c| *c = !*c))
        title=Signal::stored("Client Coordinators".to_owned())
        texts=texts                         // Signal<PersonPickerTexts>; reactive
    />
}
```

## Contract

- **The card is the control.** Click selects; clicking the selected person
  again deselects; clicking another moves the selection. There is no
  per-page card override, deliberately — that is where drift came from.
- **Keyboard:** one tab stop; Up/Down/Home/End move focus **without**
  selecting; Space/Enter toggle.
- **Search** filters by name or secondary line and never clears the
  selection. Its x clears the search.
- **The selected badge** names the pick only if it is on the roster, and its
  x clears the selection.
- **Fixed 360px**; names and secondary lines wrap, never scroll sideways.
- `selected` is controlled: `on_select` proposes `Some(id)` or `None`.

## Why a listbox and not a radiogroup

`SelectableSummaryGroup` is a radiogroup because its cards never have "no
selection". Radio semantics forbid deselect-by-click, which this roster
requires. Do not harmonise the two.

## Migrating 4iiz-Office's `CoordinatorPanel`

1. Map each `CoordinatorRosterMember` to `PickerPerson`
   (`id = worker_ref`, `secondary = office`, `initials` as today).
2. Replace the `picked: RwSignal<Option<String>>` write with
   `selected=picked` and `on_select=Callback::new(move |next| picked.set(next))`.
3. Delete `member_view` and every page's custom card — this is where the
   extra Select button came from.
4. Keep role gating (`in_class`) in Office: it is business logic, not
   framework.
5. Remove the `search` prop the panel took; search is internal now.
6. **Check presence first:** confirm Conversations' states fit
   `Available / Busy / Away / Offline`. If they do not, ask for the enum to
   be extended here rather than reintroducing a custom card.
7. If the panel sits in `SnapshotTablePage::side_panel`, nothing else is
   needed: the column now takes the panel's 360px.
````

- [ ] **Step 2: Format**

Run: `cargo fmt -p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask`
Then confirm `git diff --numstat` and `git diff --ignore-cr-at-eol --numstat` agree.

- [ ] **Step 3: Full gate**

Run: `cargo xtask verify`
Expected: `20/20 passed`.

- [ ] **Step 4: Browser lanes, sequentially, on a quiescent tree**

Run: `cargo xtask test-person-picker`, then `cargo xtask test-snapshot-table-page-filter-actions`, then `cargo xtask verify-pattern client-snapshot-list --browser`.
Expected: all PASS; person-picker `3 passed`; filter-actions `2 passed` (the Task 4 width assertion now at 355–365); client-snapshot `34 passed`.

- [ ] **Step 5: Commit and push**

```bash
git add doc/patterns/person-picker.md
git commit -m "docs(patterns): PersonPicker guide and CoordinatorPanel migration"
git push fork main
```

Then `git fetch fork` and confirm `git rev-list --left-right --count fork/main...main` is `0 0`. The pre-push hook is advisory and pushes even when red, so its message is not evidence — Step 3's own summary is.

- [ ] **Step 6: Hand off to Office**

Send the 4iiz-office session the commit range, the migration steps from the guide, and the presence question (step 6 of the migration) as a question to answer before they migrate Conversations. Do not edit Office's repo.

---

## Self-Review

**Spec coverage.** Units and API → Tasks 1–3. Listbox-not-radiogroup → Task 2 doc comment, Task 7 guide. Controlled selection → Task 3 signature. Toggle rule, search, nameable selection, tab stop → Task 1 (native) and Task 6 (browser). One reachable card under search → Task 1 test 4, Task 6 test 2. Both x controls → Task 3, Task 6 test 2. Collapse rail counts only a nameable pick → Task 3 `active_count`. Empty states without an empty listbox → Task 3, Task 6 test 2. 360px, wrapping, no horizontal scroll → Task 3 classes, Task 6 test 3. Selected background plus check and forced-colours border → Task 2 `option_class`. Presence dot and word → Task 2. `/75` muted text and measured contrast → Tasks 2–3, Task 6 test 3. Reactive texts → Task 3 prop. `side_panel` width → Task 4. Demo → Task 5. Registered lane → Task 6. Office migration notes and the presence assumption → Task 7.

**Placeholders.** None.

**Type consistency.** `next_selection`, `matches_search`, `visible_people`, `nameable_selection`, `tab_stop_id`, `Step`, `step_id` are defined in Task 1 and used with the same signatures in Tasks 2–3. `PersonPickerTexts` fields used in Tasks 2–3 (`presence`, `search_placeholder`, `search_label`, `clear_search`, `roster_label`, `empty_roster`, `no_matches`, `selected_prefix`, `clear_selection`, `show`, `hide`) all exist in Task 1. Data hooks used in Task 6 are all produced in Tasks 2, 3 and 5.
