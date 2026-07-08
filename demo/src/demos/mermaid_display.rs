use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::markdown::{ThemeMode, daisyui_theme_to_scheme};
use leptos_daisyui_rs::theme::use_theme_context;

#[component]
pub fn MermaidDisplayDemo() -> impl IntoView {
    // Follow the app's active theme so diagram lines/labels stay legible in
    // dark mode (the default light theme renders dark strokes on a dark page).
    let theme_ctx = use_theme_context();
    let mermaid_theme = Signal::derive(move || {
        let (_, mode) = daisyui_theme_to_scheme(&theme_ctx.base_theme());
        match mode {
            ThemeMode::Dark => MermaidTheme::Dark,
            ThemeMode::Light => MermaidTheme::Default,
        }
    });

    view! {
        <ContentLayout
            title="MermaidDisplay"
            description="Renders mermaid diagram source text as inline SVG using native Rust rendering"
        >
            <Section title="Flowchart" col=true>
                <p class="text-sm text-base-content/60">
                    "A left-to-right flowchart with a decision node."
                </p>
                <MermaidDisplay
                    theme=mermaid_theme
                    source="flowchart LR\n  A[Start] --> B{Decision}\n  B -->|Yes| C[OK]\n  B -->|No| D[Cancel]"
                />
            </Section>

            <Section title="Sequence Diagram" col=true>
                <p class="text-sm text-base-content/60">
                    "A simple message exchange between two participants."
                </p>
                <MermaidDisplay
                    theme=mermaid_theme
                    source="sequenceDiagram\n  Alice->>Bob: Hello\n  Bob-->>Alice: Hi back"
                />
            </Section>

            <Section title="Pie Chart" col=true>
                <p class="text-sm text-base-content/60">
                    "A pie chart showing pet ownership distribution."
                </p>
                <MermaidDisplay
                    theme=mermaid_theme
                    source=r#"pie
  "Dogs" : 40
  "Cats" : 30
  "Birds" : 20
  "Fish" : 10"#
                />
            </Section>
        </ContentLayout>
    }
}
