use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::markdown::MarkdownView;

/// A Venn diagram: one of the Mermaid types the old native renderer could
/// not draw at all, here to prove pele covers it.
const VENN_SOURCE: &str = r#"venn-beta
  set A["Design"]
  set B["Engineering"]
  union A,B["Product"]"#;

/// Deliberately broken: pele's own message must appear in the error box.
const BROKEN_SOURCE: &str = "flowchart LR\n  A[Start] --> B{Decision\n  B --> C";

const MARKDOWN_SOURCE: &str = "The same engine draws fenced `mermaid` blocks in `MarkdownView`:\n\n```mermaid\nflowchart TD\n  Draft --> Review --> Done\n```\n";

#[component]
pub fn MermaidDisplayDemo() -> impl IntoView {
    // No theme plumbing: pele takes every colour from --pele-* CSS variables,
    // which styles/tokens.css maps onto the daisyUI theme, so switching the
    // theme recolours these diagrams without re-rendering them.
    view! {
        <ContentLayout
            title="MermaidDisplay"
            description="Renders Mermaid diagram source as inline SVG with pele, coloured by the active daisyUI theme"
        >
            <Section title="Flowchart" col=true>
                <p class="text-sm text-base-content/75">
                    "A left-to-right flowchart with a decision node."
                </p>
                <MermaidDisplay
                    attr:data-testid="mermaid-flowchart"
                    source="flowchart LR\n  A[Start] --> B{Decision}\n  B -->|Yes| C[OK]\n  B -->|No| D[Cancel]"
                />
            </Section>

            <Section title="Sequence Diagram" col=true>
                <p class="text-sm text-base-content/75">
                    "A simple message exchange between two participants."
                </p>
                <MermaidDisplay
                    attr:data-testid="mermaid-sequence"
                    source="sequenceDiagram\n  Alice->>Bob: Hello\n  Bob-->>Alice: Hi back"
                />
            </Section>

            <Section title="Pie Chart" col=true>
                <p class="text-sm text-base-content/75">
                    "A pie chart showing pet ownership distribution; slice colours are the daisyUI series mapping."
                </p>
                <MermaidDisplay
                    attr:data-testid="mermaid-pie"
                    source=r#"pie
  "Dogs" : 40
  "Cats" : 30
  "Birds" : 20
  "Fish" : 10"#
                />
            </Section>

            <Section title="Venn Diagram" col=true>
                <p class="text-sm text-base-content/75">
                    "A diagram type the previous native renderer did not support."
                </p>
                <MermaidDisplay attr:data-testid="mermaid-venn" source=VENN_SOURCE />
            </Section>

            <Section title="Error" col=true>
                <p class="text-sm text-base-content/75">
                    "Invalid source shows pele's message in the error box instead of a diagram."
                </p>
                <MermaidDisplay attr:data-testid="mermaid-error" source=BROKEN_SOURCE />
            </Section>

            <Section title="In MarkdownView" col=true>
                <div data-testid="mermaid-markdown">
                    <MarkdownView source=MARKDOWN_SOURCE />
                </div>
            </Section>
        </ContentLayout>
    }
}
