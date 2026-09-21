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
        person(
            "ana",
            "Ana Lopez",
            "Denver",
            "AL",
            Some(PersonPresence::Available),
        ),
        person(
            "ben",
            "Ben Ortiz",
            "Austin",
            "BO",
            Some(PersonPresence::Busy),
        ),
        person(
            "cara",
            "Cara Diaz",
            "Denver",
            "CD",
            Some(PersonPresence::Away),
        ),
        person("dmitri", "Dmitri Volkov", "Phoenix", "DV", None),
        // Wrapping fixtures: these must fold onto new lines, never scroll
        // sideways.
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
        if spanish.get() {
            PersonPickerTexts::es()
        } else {
            PersonPickerTexts::default()
        }
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
