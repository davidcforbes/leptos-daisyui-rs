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

/// pele review repro pele-qd0.4: a 663-byte Gantt chart whose `W` date
/// tokens kept pele 1.8.0 busy for 7 s or more. The vendored fixed build
/// refuses it in milliseconds with a `limit` error.
fn hostile_gantt() -> String {
    format!(
        "gantt\n  dateFormat {}\n  excludes weekends\n  A :a1, 2020-01-01, 2000000d",
        ["W"; 300].join("-")
    )
}

/// A 120-node chain: ordinary, but far more than 1 ms of layout, so with a
/// 1 ms limit it always times out in the worker.
fn long_chain() -> String {
    let mut source = String::from("flowchart LR");
    for i in 0..120 {
        source.push_str(&format!("\n  N{i} --> N{}", i + 1));
    }
    source
}

/// A sankey chart: pele draws its links with gradients (`defs`,
/// `linearGradient`, `stop` and `fill="url(#id) colour"`), which the
/// sanitiser must keep.
const SANKEY_SOURCE: &str =
    "sankey-beta\nSolar,Grid,60\nWind,Grid,40\nGrid,Homes,70\nGrid,Industry,30";

const MARKDOWN_SOURCE: &str = "The same engine draws fenced `mermaid` blocks in `MarkdownView`:\n\n```mermaid\nflowchart TD\n  Draft --> Review --> Done\n```\n";

#[component]
pub fn MermaidDisplayDemo() -> impl IntoView {
    let (hostile, set_hostile) = signal(String::new());
    let (limited, set_limited) = signal(String::new());
    let (after_hostile, set_after_hostile) = signal(String::new());
    let start_hostile = move |_| {
        set_hostile.set(hostile_gantt());
        set_limited.set(long_chain());
        // Queued after both, so drawing it proves a fresh worker replaced
        // the one the time limit terminated.
        set_timeout(
            move || set_after_hostile.set("flowchart LR\n  Timeout --> Recovered".to_string()),
            std::time::Duration::from_millis(50),
        );
    };
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

            <Section title="Nested subgraphs" col=true>
                <p class="text-sm text-base-content/75">
                    "An edge crosses nested cluster titles; pele haloes each title with a color-mix() stroke, which the SVG sanitiser keeps."
                </p>
                <MermaidDisplay
                    attr:data-testid="mermaid-nested"
                    source="flowchart TD\n  top --> leaf\n  subgraph L1[Level one title]\n    subgraph L2[Level two title]\n      subgraph L3[Level three title]\n        leaf\n      end\n    end\n  end"
                />
            </Section>

            <Section title="Sankey" col=true>
                <p class="text-sm text-base-content/75">
                    "Links are drawn with gradients, which the SVG sanitiser keeps."
                </p>
                <MermaidDisplay attr:data-testid="mermaid-sankey" source=SANKEY_SOURCE />
            </Section>

            <Section title="Error" col=true>
                <p class="text-sm text-base-content/75">
                    "Invalid source shows pele's message in the error box instead of a diagram."
                </p>
                <MermaidDisplay attr:data-testid="mermaid-error" source=BROKEN_SOURCE />
            </Section>

            <Section title="Hostile diagram and time limit" col=true>
                <p class="text-sm text-base-content/75">
                    "pele runs in a worker with a time limit. The first chart kept pele 1.8.0 busy for seconds; the vendored build refuses it at once. The second has a 1 ms limit, so it times out, and the third draws in a fresh worker."
                </p>
                <div>
                    <Button attr:data-testid="mermaid-hostile-start" on_click=Callback::new(start_hostile)>
                        "Draw the hostile diagram"
                    </Button>
                </div>
                <MermaidDisplay attr:data-testid="mermaid-hostile" source=hostile />
                <MermaidDisplay
                    attr:data-testid="mermaid-time-limited"
                    source=limited
                    render_timeout_ms=Some(1)
                />
                <MermaidDisplay attr:data-testid="mermaid-after-hostile" source=after_hostile />
            </Section>

            <Section title="In MarkdownView" col=true>
                <div data-testid="mermaid-markdown">
                    <MarkdownView source=MARKDOWN_SOURCE />
                </div>
            </Section>
        </ContentLayout>
    }
}
