use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn ToolbarDemo() -> impl IntoView {
    let (last_clicked, set_last_clicked) = signal(String::from("(none yet)"));
    let (bold_on, set_bold_on) = signal(true);
    let (italic_on, set_italic_on) = signal(false);

    let format_items = move || -> Vec<ToolbarItem> {
        vec![
            ToolbarItem::new("bold", "B")
                .tooltip("Bold")
                .toggle(bold_on.get()),
            ToolbarItem::new("italic", "I")
                .tooltip("Italic")
                .toggle(italic_on.get()),
            ToolbarItem::new("underline", "U").tooltip("Underline"),
            ToolbarItem::new("strike", "S")
                .tooltip("Strikethrough")
                .disabled(),
        ]
    };

    let on_format_click = Callback::new(move |id: String| {
        set_last_clicked.set(id.clone());
        match id.as_str() {
            "bold" => set_bold_on.update(|v| *v = !*v),
            "italic" => set_italic_on.update(|v| *v = !*v),
            _ => {}
        }
    });

    // Deliberately more items than a narrow strip can hold, to demonstrate
    // the automatic overflow ("⋯") collapse.
    let many_items = move || -> Vec<ToolbarItem> {
        (1..=10)
            .map(|i| {
                ToolbarItem::new(format!("cmd-{i}"), format!("Cmd {i}"))
                    .tooltip(format!("Command {i}"))
            })
            .collect()
    };

    view! {
        <ContentLayout
            title="Toolbar"
            description="A horizontal strip of icon/label command and toggle buttons with tooltips, a checked-underline accent, disabled state, and automatic overflow collapse into a dropdown when items don't fit."
        >
            <Section title="Basic Toolbar" col=true>
                <p class="text-sm opacity-60">"Command buttons, a disabled item, and toggle buttons with a checked-underline accent."</p>
                <Toolbar
                    items=Signal::derive(format_items)
                    on_item_click=on_format_click
                />
                <p class="text-sm opacity-60 mt-2">"Last clicked: " {move || last_clicked.get()}</p>
            </Section>

            <Section title="Sizes" col=true>
                <p class="text-sm opacity-60">"ToolbarSize controls the underlying button size."</p>
                <div class="flex flex-col gap-4">
                    <Toolbar
                        size=ToolbarSize::Xs
                        items=Signal::derive(|| {
                            vec![
                                ToolbarItem::new("xs-1", "A"),
                                ToolbarItem::new("xs-2", "B"),
                                ToolbarItem::new("xs-3", "C"),
                            ]
                        })
                    />
                    <Toolbar
                        size=ToolbarSize::Lg
                        items=Signal::derive(|| {
                            vec![
                                ToolbarItem::new("lg-1", "A"),
                                ToolbarItem::new("lg-2", "B"),
                                ToolbarItem::new("lg-3", "C"),
                            ]
                        })
                    />
                </div>
            </Section>

            <Section title="Automatic Overflow" col=true>
                <p class="text-sm opacity-60">
                    "Drag this strip's resize handle narrower — items that no longer fit collapse into the '⋯' dropdown menu."
                </p>
                <div class="max-w-xs border border-dashed border-base-300 rounded-box p-2 resize-x overflow-auto">
                    <Toolbar items=Signal::derive(many_items) />
                </div>
            </Section>
        </ContentLayout>
    }
}
