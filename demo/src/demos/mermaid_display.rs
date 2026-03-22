use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn MermaidDisplayDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="MermaidDisplay"
            description="Renders mermaid diagram source text as inline SVG using native Rust rendering"
        >
            <Section title="Flowchart" col=true>
                <p class="text-sm text-base-content/60">
                    "A left-to-right flowchart with a decision node."
                </p>
                <MermaidDisplay source="flowchart LR\n  A[Start] --> B{Decision}\n  B -->|Yes| C[OK]\n  B -->|No| D[Cancel]" />
            </Section>

            <Section title="Sequence Diagram" col=true>
                <p class="text-sm text-base-content/60">
                    "A simple message exchange between two participants."
                </p>
                <MermaidDisplay source="sequenceDiagram\n  Alice->>Bob: Hello\n  Bob-->>Alice: Hi back" />
            </Section>

            <Section title="Pie Chart" col=true>
                <p class="text-sm text-base-content/60">
                    "A pie chart showing pet ownership distribution."
                </p>
                <MermaidDisplay source=r#"pie
  "Dogs" : 40
  "Cats" : 30
  "Birds" : 20
  "Fish" : 10"# />
            </Section>
        </ContentLayout>
    }
}
