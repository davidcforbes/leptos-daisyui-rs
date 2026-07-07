use super::style::{sla_chip_label, sla_chip_tone, SLA_CHIP_DEFAULT_THRESHOLD_MS};
use crate::components::icon::{Icon, IconSize};
use crate::merge_classes;
use leptos::{html::Span, prelude::*};
use std::time::Duration;

/// # SLA Chip Component
///
/// A live SLA-countdown chip: a colored deadline indicator for a queue or
/// work-card header. Ported from d2d-ui's owner-drawn `SlaChip` control --
/// the tone/label logic ([`sla_chip_tone`](super::style::sla_chip_tone) /
/// [`sla_chip_label`](super::style::sla_chip_label)) is carried over
/// near-verbatim, and the Direct2D `rect`/brush/`draw()` painting is replaced
/// by a daisyUI `badge`.
///
/// Five visual states, driven purely by the deadline vs a caller-supplied
/// `now_ms` (the caller owns the clock -- this component has no internal
/// timer of its own, though [`use_sla_now`] is provided as an opt-in one):
/// - **green** -- inside target (more than `threshold_ms` remaining)
/// - **amber** -- approaching (within `threshold_ms` of the deadline)
/// - **red** -- breached; shows `+Xh Ym over`
/// - **none** -- no SLA defined (neutral "No SLA"); never fakes a timer
/// - **stale** -- any tone, but dimmed (the data feed is frozen)
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{SlaChip, use_sla_now};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     // Ticks every second; drives every SlaChip on the page.
///     let now_ms = use_sla_now(1_000);
///
///     view! {
///         // 90 minutes from now, inside the default 2h threshold -> amber.
///         <SlaChip deadline_ms=Some(js_sys::Date::now() as i64 + 90 * 60_000) now_ms=now_ms />
///
///         // No deadline -> neutral "No SLA".
///         <SlaChip now_ms=now_ms />
///
///         // Enriched: leading severity icon + matching border, larger size.
///         <SlaChip
///             deadline_ms=Some(js_sys::Date::now() as i64 - 60 * 60_000)
///             now_ms=now_ms
///             big=true
///             show_icon=true
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("badge badge-soft badge-md badge-lg");
/// @source inline("badge-success badge-warning badge-error badge-neutral");
/// @source inline("border border-success/45 border-warning/45 border-error/45 border-neutral/45");
/// @source inline("opacity-60 gap-1");
/// @source inline("inline-block w-4 h-4");
/// ```
///
/// ## Node References
/// - `node_ref` - References the top `<span>` element ([HTMLSpanElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLSpanElement))
#[component]
pub fn SlaChip(
    /// Deadline in wall-clock milliseconds. `None` (the default) means no SLA
    /// is defined for this item and the chip renders a neutral "No SLA" pill.
    #[prop(optional, into)]
    deadline_ms: Signal<Option<i64>>,

    /// Current wall-clock time in milliseconds. The caller owns the clock --
    /// drive this from any `Signal`, or from [`use_sla_now`] for a
    /// ticking chip.
    #[prop(into)]
    now_ms: Signal<i64>,

    /// "Approaching" window (ms) before the deadline that flips the tone
    /// from green to amber. Defaults to
    /// [`SLA_CHIP_DEFAULT_THRESHOLD_MS`](super::style::SLA_CHIP_DEFAULT_THRESHOLD_MS)
    /// (2 hours).
    #[prop(optional, into, default = Signal::derive(|| SLA_CHIP_DEFAULT_THRESHOLD_MS))]
    threshold_ms: Signal<i64>,

    /// Larger chip (`badge-lg`) for e.g. a work-card header vs a table cell.
    #[prop(optional, into)]
    big: Signal<bool>,

    /// Dim the chip -- the data feed is frozen/stale.
    #[prop(optional, into)]
    stale: Signal<bool>,

    /// Enriched variant: draw a leading severity icon plus a matching border
    /// so the pale/soft pill still reads at a glance on a light page.
    /// beads-p4v4
    #[prop(optional, into)]
    show_icon: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the chip's top `<span>` element
    #[prop(optional)]
    node_ref: NodeRef<Span>,
) -> impl IntoView {
    let tone = move || sla_chip_tone(deadline_ms.get(), now_ms.get(), threshold_ms.get());
    let label = move || sla_chip_label(deadline_ms.get(), now_ms.get());

    view! {
        <span
            node_ref=node_ref
            class=move || {
                let t = tone();
                merge_classes!(
                    "badge badge-soft",
                    t.as_str(),
                    if big.get() { "badge-lg" } else { "badge-md" },
                    if stale.get() { "opacity-60" } else { "" },
                    if show_icon.get() { t.border_class() } else { "" },
                    if show_icon.get() && t.icon_name().is_some() { "gap-1" } else { "" },
                    class
                )
            }
        >
            <Show when=move || show_icon.get() && tone().icon_name().is_some()>
                <Icon
                    name=Signal::derive(move || {
                        tone().icon_name().unwrap_or_default().to_string()
                    })
                    size=IconSize::XSmall
                />
            </Show>
            {label}
        </span>
    }
}

/// Convenience hook: a [`Signal<i64>`] that ticks the current wall-clock time
/// (`js_sys::Date::now()`, in milliseconds) every `poll_ms` milliseconds, for
/// driving [`SlaChip`]'s `now_ms` prop without the caller owning its own
/// timer. Purely optional -- any other `Signal<i64>` (a fixed value in tests,
/// a value derived from a websocket tick, etc.) works just as well. Mirrors
/// the `set_interval` polling pattern already used by
/// [`AiChat`](crate::components::ai_chat::AiChat) to drive its own re-render
/// loop.
pub fn use_sla_now(poll_ms: u64) -> Signal<i64> {
    let now = RwSignal::new(js_sys::Date::now() as i64);
    set_interval(
        move || now.set(js_sys::Date::now() as i64),
        Duration::from_millis(poll_ms),
    );
    now.into()
}
