use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::markdown::MarkdownView;

/// Everyday math as a chat message or note writes it.
const EVERYDAY: &str = r#"Inline math sits in the sentence: $E=mc^2$, $a^2 + b^2 = c^2$ and $\frac{n(n+1)}{2}$.

$$\int_0^1 x^2\,dx = \frac{1}{3}$$

$$\begin{pmatrix} a & b \\ c & d \end{pmatrix} \begin{pmatrix} x \\ y \end{pmatrix} = \begin{pmatrix} ax + by \\ cx + dy \end{pmatrix}$$

$$f(x) = \begin{cases} x^2 & x \ge 0 \\ -x & \text{otherwise} \end{cases}$$

$$\begin{aligned} \nabla \cdot \mathbf{E} &= \frac{\rho}{\varepsilon_0} \\ \nabla \times \mathbf{B} &= \mu_0 \mathbf{J} \end{aligned}$$

Struck terms use the Chromium polyfill: $\cancel{x} + \boxed{y}$.
"#;

/// Formulas that cannot render: the source stays, the reason is the title.
const ERRORS: &str = r#"An unknown command: $\notacommand{x}$.

A macro definition is refused before conversion: $\def\a{\a\a}\a$.
"#;

/// Repeat `\frac{1}{` `n` times around `x`.
fn nested_fracs(n: usize) -> String {
    format!("{}x{}", r"\frac{1}{".repeat(n), "}".repeat(n))
}

/// Hostile input, rendered only on demand so the page first loads clean.
/// Deep nesting would overflow wasm's stack inside math-core and trap the
/// whole app; the guard must refuse it, while nesting just under the
/// guard's limit still renders.
fn hostile_markdown() -> String {
    // A braced \frac level is two guard entries (the command and its brace).
    let under = leptos_daisyui_rs::markdown::math::guard::MAX_MATH_DEPTH / 2;
    let unbraced = format!("{}x", r"\frac1".repeat(500));
    let optional = format!("{}x{}", r"\sqrt[".repeat(200), "]{y}".repeat(200));
    format!(
        "Nested {under} fractions: ${under_src}$\n\n\
         Nested 500 levels: ${deep}$\n\n\
         A javascript link: $\\href{{javascript:alert(1)}}{{x}}$\n\n\
         Markup in text: $\\text{{<img src=x onerror=alert(1)>}}$\n\n\
         A pinned colour: $\\color{{red}}{{x}} + y$\n\n\
         Unbraced nesting: ${unbraced}$\n\n\
         Nested optional arguments: ${optional}$\n\n\
         A huge space: $a\\hspace{{100em}}b$\n",
        under_src = nested_fracs(under),
        deep = nested_fracs(500),
    )
}

#[component]
pub fn MarkdownMathDemo() -> impl IntoView {
    let (hostile, set_hostile) = signal(String::new());
    let (pings, set_pings) = signal(0u32);
    view! {
        <ContentLayout
            title="Markdown math"
            description="MarkdownView renders $...$ and $$...$$ as native MathML with the Rust math-core crate: no JavaScript, no CDN"
        >
            <Section title="Inline and display math" col=true>
                <div data-testid="math-everyday">
                    <MarkdownView source=EVERYDAY />
                </div>
            </Section>
            <Section title="Formulas that cannot render" col=true>
                <p class="text-sm text-base-content/75">
                    "The source stays visible; hover it for the reason."
                </p>
                <div data-testid="math-errors">
                    <MarkdownView source=ERRORS />
                </div>
            </Section>
            <Section title="Hostile input" col=true>
                <p class="text-sm text-base-content/75">
                    "Deep nesting, links, markup and pinned colours. The page must stay responsive afterwards."
                </p>
                <div class="flex flex-wrap items-center gap-4">
                    <Button
                        attr:data-testid="math-hostile-start"
                        on_click=Callback::new(move |_| set_hostile.set(hostile_markdown()))
                    >
                        "Render hostile formulas"
                    </Button>
                    <Button
                        attr:data-testid="math-ping"
                        on_click=Callback::new(move |_| set_pings.update(|n| *n += 1))
                    >
                        "Ping"
                    </Button>
                    <span data-testid="math-ping-count">{move || pings.get()}</span>
                </div>
                <div data-testid="math-hostile">
                    <MarkdownView source=hostile />
                </div>
            </Section>
        </ContentLayout>
    }
}
