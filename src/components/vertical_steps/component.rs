use super::style::{content_class, has_rail_segment, segment_lit, vstep_rail_class};
use super::types::{step_key, VerticalStep};
use crate::merge_classes;
use leptos::{html::Ol, prelude::*};

/// # VerticalSteps Component
///
/// A top-to-bottom "connection path" control: a status dot per step on a
/// vertical rail, each with a rich content slot (title + plain-language body
/// + optional technical sub-line + optional action button). Ported from
/// d2d-ui's owner-drawn `controls::vertical_steps::VerticalSteps` — a
/// self-painting Direct2D control used by aws-ssm-monitor's preflight view to
/// show a chain of dependent health checks (PC -> Gateway -> Database, etc).
///
/// A rail segment below a [`StepStatus::Ready`](super::StepStatus::Ready) step
/// is "lit" (`accent`-colored) with a small dash animating down it toward the
/// next step, giving a sense of forward progress through the chain.
///
/// ## When to use `VerticalSteps` vs. [`Steps`](crate::components::Steps)
/// - Use the plain daisyUI [`Steps`](crate::components::Steps)/[`Step`](crate::components::Step) pair (with
///   `direction=StepsDirection::Vertical`) for a simple, presentational
///   numbered/labeled sequence — a wizard progress indicator, an order-status
///   tracker — where each step is a short label and daisyUI's built-in
///   `step-primary`/`step-success`/etc. coloring is enough.
/// - Use `VerticalSteps` when each step needs its own rich content (a title
///   *and* a body line *and* an optional technical detail *and* an optional
///   action button) driven by a live [`StepStatus`](super::StepStatus) — e.g.
///   a system-health / preflight-check monitor where the user may need to act
///   on a specific failed step.
///
/// # Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{StepStatus, VerticalStep, VerticalSteps};
///
/// #[component]
/// fn App() -> impl IntoView {
///     let steps = vec![
///         VerticalStep::new(StepStatus::Ready, "PC", "Your computer is online"),
///         VerticalStep::new(StepStatus::NeedsYou, "Gateway", "Needs your sign-in")
///             .with_tech("vpn-gw-03.internal:443")
///             .with_action("Fix"),
///         VerticalStep::new(StepStatus::Pending, "Database", "Checked once gateway clears"),
///     ];
///     view! {
///         <VerticalSteps
///             items=Signal::derive(move || steps.clone())
///             on_action=Callback::new(|i: usize| leptos::logging::log!("fix step {i}"))
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col items-center gap-3 w-4 shrink-0 rounded-full w-3.5 h-3.5 w-0.5 flex-1 mt-1");
/// @source inline("bg-success bg-accent bg-base-100 bg-base-300 bg-warning bg-error border-2 border-success border-accent border-base-300 border-warning border-error animate-pulse");
/// @source inline("flex-1 pb-6 text-sm font-semibold text-base-content sr-only");
/// @source inline("text-xs text-base-content/70 text-base-content/50 font-mono mt-0.5 mt-1 mt-2");
/// @source inline("btn btn-xs btn-outline");
/// ```
/// The `ld-vstep-rail`/`ld-vstep-flow-dash` classes that draw the animated
/// flow dash are plain CSS (not Tailwind utilities) defined in
/// [`UiAnimationsPreamble`](crate::tokens::UiAnimationsPreamble) — mount that
/// component once near your app root for the animation to render (it also
/// disables the animation under `prefers-reduced-motion: reduce`).
///
/// ## Node References
/// - `node_ref` - References the steps `<ol>` element ([HTMLOListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLOListElement))
#[component]
pub fn VerticalSteps(
    /// Steps to render, top to bottom.
    #[prop(into)]
    items: Signal<Vec<VerticalStep>>,

    /// Fired with the step's index when its action button is clicked.
    #[prop(optional)]
    on_action: Option<Callback<usize>>,

    /// Additional CSS classes for the `<ol>` container.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the `<ol>` container.
    #[prop(optional)]
    node_ref: NodeRef<Ol>,
) -> impl IntoView {
    let indexed =
        move || -> Vec<(usize, VerticalStep)> { items.get().into_iter().enumerate().collect() };
    let total = move || items.with(Vec::len);

    view! {
        <ol node_ref=node_ref class=move || merge_classes!("flex flex-col", class)>
            <For
                each=indexed
                key=|(i, step)| (*i, step_key(step))
                children=move |(i, step)| {
                    // Reactive: appending a step must grow a rail segment out
                    // of the previously-last dot without re-keying that row.
                    let has_segment = move || has_rail_segment(i, total());
                    let lit = segment_lit(step.status);
                    let dot_class = step.status.dot_class();
                    let status_label = step.status.label();
                    let title = step.title.clone();
                    let body = step.body.clone();
                    let has_tech = step.tech.is_some();
                    let tech = step.tech.clone().unwrap_or_default();
                    let has_action = step.action_label.is_some();
                    let action_label = step.action_label.clone().unwrap_or_default();
                    view! {
                        <li class="flex gap-3">
                            <div class="flex flex-col items-center w-4">
                                <span
                                    class=format!("w-3.5 h-3.5 rounded-full shrink-0 {dot_class}")
                                    aria-hidden="true"
                                ></span>
                                <Show when=has_segment>
                                    <span class=format!(
                                        "ld-vstep-rail w-0.5 flex-1 mt-1 rounded-full {}",
                                        vstep_rail_class(lit),
                                    )>
                                        <Show when=move || lit>
                                            <span class="ld-vstep-flow-dash"></span>
                                        </Show>
                                    </span>
                                </Show>
                            </div>
                            <div class=move || content_class(has_segment())>
                                <p class="text-sm font-semibold text-base-content">
                                    <span class="sr-only">{format!("{status_label}: ")}</span>
                                    {title}
                                </p>
                                <p class="text-xs text-base-content/70 mt-0.5">{body}</p>
                                <Show when=move || has_tech>
                                    <p class="text-xs font-mono text-base-content/50 mt-1">
                                        {tech.clone()}
                                    </p>
                                </Show>
                                <Show when=move || has_action>
                                    <button
                                        type="button"
                                        class="btn btn-xs btn-outline mt-2"
                                        on:click=move |_| {
                                            if let Some(cb) = on_action {
                                                cb.run(i);
                                            }
                                        }
                                    >
                                        {action_label.clone()}
                                    </button>
                                </Show>
                            </div>
                        </li>
                    }
                }
            />
        </ol>
    }
}
