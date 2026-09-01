# KPI strip

`KpiStrip` is the Layer-2 pattern named in `Future-Architecture.md` for a
responsive row of consistently sized stat cards. It exists because daisyUI's
low-level `Stats`/`Stat` pair renders a *joined* strip -- a shared background
and internal dividers -- so eight metrics composed as `Stat` children inside
a `Stats` container read as one table row instead of eight independent
cards. `KpiStrip` builds ordinary bordered boxes in a responsive CSS grid
instead, each at the framework's declared card elevation (see
[Card elevation](#card-elevation)). `Stats`/`Stat` are unchanged and remain independently
usable -- reach for them directly when daisyUI's own joined presentation is
actually what's wanted.

## When to reach for this vs `Stats`/`Stat`

| Need | Use |
|---|---|
| A row of visually independent stat cards (dashboard summary, workspace header) | `KpiStrip` + `KpiItem` |
| daisyUI's own joined stats strip (a single card with internal dividers) | `Stats` + `Stat` directly |
| One featured metric, or an unusual layout `KpiStrip`'s grid doesn't fit | `KpiCard` directly |

## The typed item model

`KpiStrip` takes `items: Signal<Vec<KpiItem>>`. `KpiItem` is plain owned
data, not a `Signal`-bearing struct -- the whole list is itself reactive, the
same posture as `ActiveFilterChip`/`DatasetOption` elsewhere in this crate.
Rebuilding the list (a data refresh, a locale change) is how a `KpiStrip`
updates; there is no per-field signal to thread through.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::components::StatDeltaTrend;
use leptos_daisyui_rs::patterns::{KpiItem, KpiStatus, KpiStrip, KpiTrend};

let items = Signal::derive(|| {
    vec![
        KpiItem::new("open", "Open matters", "128")
            .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive).label("this week")),
        KpiItem::new("overdue", "Overdue tasks", "6")
            .status(KpiStatus::Warning)
            .help("Tasks past their due date, across every assignee."),
        KpiItem::new("revenue", "Revenue booked", "$18,400")
            .status(KpiStatus::Success),
        KpiItem::new("sync", "Last sync", "").unavailable(),
    ]
});

view! { <KpiStrip items=items /> }
```

Builder methods on `KpiItem`:

- `KpiItem::new(id, label, value)` -- an available card.
- `.unavailable()` -- clears the value; the card renders a muted placeholder
  (never a fabricated `0` or empty string) and the framework-owned
  `unavailable` text from `KpiStripTexts`.
- `.description(text)` -- optional supporting copy; renders nothing when
  empty.
- `.status(KpiStatus)` -- optional semantic emphasis (`Info`/`Success`/
  `Warning`/`Error`). Drives the value text color, a LEFT accent edge, and
  the comparison bar's fill together. The edge is always laid out and
  `Neutral` (the default) paints it the house blue, so a status is what
  OVERRIDES the default rather than what adds an edge -- see `ldui-kmpa`.
  Status is also emitted as `data-kpi-card-status`, so it is readable
  without sampling a colour.
- `.trend(KpiTrend)` -- optional up/down/steady indicator, reusing
  `StatDeltaTrend` so `KpiStrip` and `StatDelta` agree on what "positive"
  means for a given metric. `KpiTrend::new(value, direction)` plus an
  optional `.label(text)`.
- `.help(text)` -- optional help text. Renders a small "?" affordance with a
  hover tooltip, and is also wired to the card's `aria-describedby` so it
  reaches assistive tech without requiring a hover.
- `.baseline(KpiBaseline)` -- optional current-versus-baseline comparison
  row. See below.
- `.action(KpiAction)` -- optional activation affordance. See below.

## Baseline comparison (ldui-ztgo)

A dashboard KPI is usually two facts, not one: the current number, and how
it stands against a trailing baseline. `KpiBaseline` carries both raw
numbers plus the caller's own name for the baseline:

```rust,no_run
use leptos_daisyui_rs::patterns::{KpiBaseline, KpiItem, KpiStatus};

let item = KpiItem::new("intakes", "Intakes", "280")
    .status(KpiStatus::Success)
    .baseline(KpiBaseline::against(280.0, 250.0).label("12-week avg / 250"));
```

That renders `280`, a bar, the readout `112%`, the baseline's name, and the
sentence `12% above baseline`.

### The bar is bounded; the number is not

This is the property to hold onto, because getting it wrong turns a
dashboard into a liar. The comparison track's right edge is pinned at
`baseline * KPI_BASELINE_TRACK_HEADROOM` (1.25), so:

- the **baseline marker sits at exactly 80% of the track on every card in
  the strip**, and never moves. Twelve cards therefore compare their fills
  against one shared tick.
- a value between 0 and 125% of baseline draws a fill that ends where it
  really is;
- a value past 125% **saturates**: the fill is clamped to the track, the
  card sets `data-kpi-baseline-saturated="true"`, and the readout beside it
  keeps printing the true figure -- `425%`, not `100%`.

Because the marker stays at 80% rather than at the track's end, a saturated
bar reads as "well past the baseline". A bar clamped at 100% of its track
never means "at the cap"; the cap is the tick, still visible behind the
fill.

`CapacityBar`'s own default `max` is deliberately overridden. That default
is `cap * 1.25` *clamped up to at least `value`*, which would rescale the
track to fit an over-baseline value and slide the marker left -- on exactly
the cards where the marker's position matters most.

### Four unavailable states, four sentences

A baseline that cannot be divided by is not an edge case to swallow; it is
a fact to report. `KpiBaselineAvailability` makes the caller declare which:

| Declared | State | Renders |
|---|---|---|
| `KpiBaseline::against(c, b)` with finite `b > 0` | `Above` / `Level` / `Below` | bar + readout + directional sentence |
| `KpiBaseline::absent(c)` | `NoBaseline` | no bar, no percentage, the `baseline_absent` sentence |
| `KpiBaseline::settling(c)` | `Settling` | no bar, no percentage, the `baseline_settling` sentence |
| `against(c, 0.0)`, a negative, `NaN`, an infinity, or a non-finite current | `NoBaseline` **plus** `data-kpi-baseline-degraded="true"` | the `baseline_absent` sentence |

Nothing in that table produces `NaN`, `inf`, or a fabricated `0%`. The last
row is the only one that is a defect, and it is flagged rather than
silently folded into the third -- "there is no baseline", "the window is
still filling", and "the caller handed over an unusable number" are three
different facts.

A value a hair over its baseline (250.4 against 250) rounds to `100%` and
therefore speaks the `Level` sentence, never `0% above`: the direction word
and the printed percentage come from ONE rounding, so they cannot disagree.

### Colour never carries the judgement

The bar takes its fill colour from the card's typed `KpiStatus`, and the
over-baseline band takes the **same** colour. The framework has no basis
for deciding whether higher is better: for "intakes" it is, for "days to
close" and "cost per matter" it is not. `CapacityBarColor::for_direction`
exists and would paint at-or-above green -- it is deliberately not used
here. "Over" is signalled by the fill crossing the marker and by the
sentence; favourable/unfavourable is the caller's `status` to declare.

### Alignment is unaffected

The comparison row renders BELOW the value, so a card with a baseline and a
card without keep identical label and value offsets. `ldui-tbaw`'s two
mechanisms are untouched: the label still reserves two line boxes
(`min-h-8`) and the accent edge is still always laid out. In a grid row the
shells are `h-full`, so a taller baseline-bearing card simply sets the row
height and its neighbours stretch to match.

## Activation (ldui-ztgo)

A card becomes activatable only when BOTH halves are present: the item's
own `KpiAction` copy and an `on_activate` callback on the card or strip.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{KpiAction, KpiBaseline, KpiItem, KpiStrip};

let open_detail = Callback::new(|id: String| {
    // The framework emits the stable `KpiItem::id` and nothing else.
    let _ = id;
});

let items = Signal::derive(|| {
    vec![
        KpiItem::new("intakes", "Intakes", "280")
            .baseline(KpiBaseline::against(280.0, 250.0).label("12-week avg / 250"))
            .action(KpiAction::new("View details")),
        // No action: stays a read-only, non-focusable group.
        KpiItem::new("last-sync", "Last sync", "").unavailable(),
    ]
});

view! { <KpiStrip items=items on_activate=open_detail /> }
```

**One framework-rendered control, not whole-card activation.** The card
stays a `role="group"` `<div>` and gains one `Pressable` inside it. Three
reasons, in order of how expensive each would be to discover later:

1. `<button>` takes phrasing content, and the card body is built from `<p>`
   elements plus an `sr-only` help `<span id>`. Wrapping the body in a
   button is invalid HTML.
2. The card's accessible-name grammar is `role="group"` + `aria-label`, and
   every existing caller depends on it. A card that became a button would
   announce as a control on pages where nothing changed but the framework
   version.
3. A whole-card click handler that is not a real control needs a duplicated
   keyboard path and a synthetic tab stop -- and then the help affordance
   sits inside a control, which is the nested-interactive defect.

Consequences worth stating plainly:

- An activatable card is exactly **one** tab stop. The help affordance is a
  non-interactive `aria-hidden` span whose real text reaches assistive tech
  through `aria-describedby`, so there is never a control inside a control.
- A non-activatable card gains no `tabindex`, no `<button>`, and no
  `data-kpi-card-activatable`. Every card written before this bead has
  neither half of the gate, so none of them changed.
- Pointer, Enter and Space all work, because the control is a real
  `<button>` -- nothing is re-implemented.
- The accessible name defaults to `"<visible label>, <the card's accessible
  name>"`, keeping the visible label as a prefix (WCAG 2.5.3 Label in Name)
  while distinguishing one of twelve identically-labelled "View details"
  buttons. `KpiAction::accessible_label` overrides it.
- `KpiAction::disabled(true)` renders the native `disabled` attribute, so
  the control leaves the tab order and stays in the accessibility tree.

### Elevation: the card keeps the STATIC depth

An activatable card does **not** adopt `ld-elevated`'s interactive hover
lift; it keeps `ld-card-depth` (ldui-k4fn). The card is not the control --
the `Pressable` inside it is. Lifting the whole card on hover would promise
that pressing anywhere on it does something, which is a bigger lie than the
read-only tile `ld-card-depth` was chosen to avoid. The interactive
affordance lives exactly where the interaction does: `Pressable` already
carries `ld-pressable` (press scale) and `ld-focus-ring` (focus-visible
ring), eased by `ld-eased`.

### Reconciliation and focus

`KpiStrip` reconciles its cards with a keyed `<For>` whose key covers the
WHOLE item, not just its id. `KpiCard` takes its item by value and holds no
reactive signal over it, so an id-only key would leave a card showing a
stale number after a refresh -- and a locale change (same ids, translated
labels) would never re-render at all. Keying on the whole item means an
unchanged card keeps its DOM, and its focused action button keeps focus,
while a card whose data moved is rebuilt.

## Grid, layout profile, and `compact`

`KpiStrip` owns the responsive grid, and it asks how wide **the strip** is,
never how wide the window is -- the steps are CSS container queries
(`@sm`/`@lg`/`@4xl`/`@5xl`), because a strip in a 648px content column was
rendering 67px cards the moment the *viewport* crossed 1280px (ldui-tnyq).
Fewer items than columns simply leave the remaining explicit-column tracks
empty -- CSS Grid does not stretch existing cards to fill them, so card size
stays equal regardless of count. The strip never scrolls horizontally.

### `KpiStripLayout`: a typed ladder, not a column count (ldui-k3ip)

| profile | ladder | for |
|---|---|---|
| `KpiStripLayout::AutoEight` (default) | 2 / 3 / 4 / 8 | an operational strip of short cards |
| `KpiStripLayout::BalancedSix` | 2 / 3 / 4 / 6 | a balanced fixed dashboard scorecard |

```rust
view! { <KpiStrip items=items layout=KpiStripLayout::BalancedSix /> }
```

The prop is a named intent rather than a `columns: usize`, because an
integer would accept twelve columns of 40px, would put breakpoint policy
back in the consumer, and would say nothing about what happens at narrower
widths -- the framework owns the whole ladder down from the widest rung.
The default is unchanged, so every existing caller keeps the exact
2 / 3 / 4 / 8 grid it had.

**The rungs are derived, not chosen.** Card width is
`(container - gap * (columns - 1)) / columns`, with `gap-4` = 16px, and a
card needs about 114px to hold a two-line label (`ldui-tbaw`'s fit sweep,
as shipped by `ldui-tnyq`) -- about 125px if it also carries a help
control, which costs the label 20px of its row (`ldui-yhvf`):

| profile | rung | container | columns | card |
|---|---|---|---|---|
| both | base | 320px | 2 | 152.0px |
| both | `@sm` | 384px | 3 | 117.3px |
| both | `@lg` | 512px | 4 | 116.0px |
| `AutoEight` | `@5xl` | 1024px | 8 | 114.0px |
| `BalancedSix` | `@4xl` | 896px | 6 | 136.0px |

Six columns start at `@4xl` rather than `@3xl` (768px) because at 768px six
columns are 114.7px: enough for a bare two-line label, not enough for a
help-bearing one -- and a scorecard is exactly where help-bearing cards
live. At the 1046px container `ldui-tnyq` measured on a 1680px window,
`BalancedSix` gives 6 columns of 161.0px against `AutoEight`'s 8 of
116.8px, so the balanced profile's cards are **wider**: a two-line label, a
help trigger and a baseline comparison bar all gain room by choosing it.

### Item counts that do not divide

A short final row is deliberate. The tracks are explicit (`grid-cols-6`,
never an `auto-fit` minmax), so seven cards in a balanced-six strip render
six then one, and that one keeps its own sixth-width track rather than
stretching across the row. Stretching would make a five-card strip's cards
a different size from a six-card strip's, and would make the lone last card
read as a more important thing than its peers -- equal card geometry is the
property this pattern exists to protect. Twelve and six both divide evenly,
which is why the balanced-six ladder's every rung (2, 3, 4, 6) is a divisor
of twelve: a twelve-card set stays a balanced peer group at *every* width,
not only the widest one.

`compact=true` tightens card padding and gap (`p-4`/`gap-4` down to
`p-3`/`gap-3`, both still on the canonical spacing scale) and steps the
value text down one rung on the `.ld-text-*` ramp (`ld-text-display` to
`ld-text-title`), for dense contexts such as a sidebar summary or an
embedded card.

## Label wrapping (ldui-tbaw)

A `KpiCard` label wraps up to two lines (`line-clamp-2`) rather than
ellipsizing on one -- ordinary Office-length labels ("No-Hire Conversions",
"Payments Collected", "Customer Success Pts", ...) are fully visible at the
consumer's eight-card 1680px width; only deliberately over-long copy clamps
after two lines. The label box always reserves the height of two
`ld-text-small` line boxes (`min-h-8`, 32px) regardless of whether a given
label actually needs one line or two, so a short-label card and a
two-line-label card in the same row stay equal height and their
values/descriptions/help controls start at the identical vertical offset --
clamping the text alone would still let a one-line label leave a shorter,
unreserved box than a wrapped one. Visual clamping never shortens the
accessible name: `aria-label` is computed from the label string directly, so
even a clamped label reaches assistive tech in full.

## Accessibility

Each `KpiCard` is `role="group"` with a computed `aria-label` combining the
label, the value (or the unavailable fallback), and the trend direction as a
word ("trending up"/"trending down"/"steady") rather than only a glyph --
so a screen reader announces one coherent phrase per card instead of reading
unrelated child text nodes. Help text is exposed via `aria-describedby`
independent of hover.

## Localization

Caller-supplied text (label/value/description/help) localizes by rebuilding
the `items` list for the active locale, like any other reactive prop in this
crate. The pattern's own generated copy -- the unavailable-value fallback and
the trend words folded into the accessible name -- is a separate,
overridable `texts: Signal<KpiStripTexts>` prop, forwarded from `KpiStrip` to
every card:

```rust,no_run
use leptos_daisyui_rs::patterns::KpiStripTexts;

let texts = KpiStripTexts {
    unavailable: "Indisponible".to_string(),
    trend_up: "en hausse".to_string(),
    trend_down: "en baisse".to_string(),
    trend_steady: "stable".to_string(),
    // Every comparison sentence is framework-owned copy too, so a locale
    // switch reaches it without rebuilding the items.
    baseline_ratio: "{ratio} %".to_string(),
    baseline_above: "{delta} % au-dessus de la {baseline}".to_string(),
    baseline_below: "{delta} % en dessous de la {baseline}".to_string(),
    baseline_level: "Conforme a la {baseline}".to_string(),
    baseline_absent: "Pas encore de reference".to_string(),
    baseline_settling: "Reference en cours de constitution".to_string(),
};
```

The five `baseline_*` fields are templates, not finished sentences. Three
placeholders are substituted:

- `{ratio}` -- current as a percentage OF the baseline (`112`).
- `{delta}` -- the UNSIGNED deviation in percentage points (`12`). Which
  side it falls on is carried by which template is chosen, so the number is
  never signed twice.
- `{baseline}` -- the caller's own `KpiBaseline::label`.

In the no-baseline and settling templates there is no ratio and no
deviation, so `{ratio}` and `{delta}` substitute to `unavailable` rather
than to a fabricated `0`.

## Section heading and period selection stay caller-owned

`KpiStrip` renders only the cards. A section heading above it (title,
period-selector, refresh action) is ordinary composition -- typically
`SectionHeading` from this same module -- not a `KpiStrip` prop:

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{KpiStrip, SectionHeading};
# use leptos_daisyui_rs::patterns::KpiItem;
# fn items() -> Vec<KpiItem> { vec![] }

view! {
    <section>
        <SectionHeading title="Work stats" />
        <KpiStrip items=Signal::derive(items) class="mt-4" />
    </section>
}
```

## Add to `input.css`

```css
@source inline("@container grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 gap-3 gap-4 w-full");
@source inline("@5xl:grid-cols-8 @4xl:grid-cols-6");
@source inline("rounded-box border border-base-300 bg-base-100 h-full min-w-0 overflow-hidden");
@source inline("forced-colors:border-[CanvasText]");
@source inline("h-(--border-width-accent) w-full");
@source inline("bg-info bg-success bg-warning bg-error");
@source inline("flex flex-col items-center gap-1 gap-2 p-3 p-4 min-w-0 shrink-0");
@source inline("line-clamp-2 min-h-8");
@source inline("font-semibold uppercase tracking-wide tabular-nums break-words italic");
@source inline("text-base-content/75 text-base-content/40 text-base-content/60 text-info text-success text-warning text-error");
@source inline("tooltip tooltip-top inline-flex h-4 w-4 items-center justify-center rounded-full border sr-only");
```

The `.ld-text-*` classes and `.ld-card-depth` are **not** listed above and
must not be added via `@source inline`: they are not Tailwind utilities
(`@source` scanning cannot generate them), they are plain rules generated
into `styles/tokens.css` by `cargo xtask gen-tokens` (ldui-h7tw,
ldui-k4fn). Import that file once, as shown under
[CSS Configuration](../../CLAUDE.md#css-configuration), and both resolve
with no further action -- unlike this crate's motion `--ld-*` custom
properties and classes (durations, easings, `.ld-eased`/`.ld-focus-ring`/
`.ld-elevated`/etc.), which still require mounting
`UiTokensPreamble`/`UiAnimationsPreamble` at runtime. See
`src/tokens/preamble.rs` for which family is which.

## Card elevation

The card's resting depth is the framework's own semantic class,
`ld-card-depth`, **never** a stock Tailwind `shadow-*` utility
(ldui-k4fn -- see [`doc/visual-quality/ad-hoc-shadow.md`](../visual-quality/ad-hoc-shadow.md)).
The rule is one line:

```css
.ld-card-depth { box-shadow: var(--ld-card-shadow, var(--ld-elevation-4)); }
```

`--ld-elevation-4` is `ui_tokens::elevation::LEVEL_4`, the shared token
crate's declared *card resting elevation* -- the same level the Direct2D
desktop face draws behind a card, and the same level the interactive
`ld-elevated` rests at, so a static card and a hoverable one sit at one
depth. It is deliberately **not** `ld-elevated` itself: that class lifts to
LEVEL_8 with a `translateY(-1px)` on hover, which would make a read-only
tile look clickable. `ld-card-depth` sets exactly one property and has no
hover, transition, or transform.

`--ld-card-shadow` is the product-theme hook and is **never declared by this
crate**. A product that must ship its own approved card shadow sets it once:

```css
/* the product's own theme stylesheet */
:root { --ld-card-shadow: 0 1px 4px rgba(0, 0, 0, 0.16); }
```

Because the framework declares nothing, that needs no `!important`, no
descendant selector reaching into `KpiCard`'s markup, and no page-local fork
of the class -- the three things the opinionated-component ownership
boundary rules out. Remove the declaration and the framework default paints
again.

Both `.ld-card-depth` and `--ld-elevation-*` ship in the **generated**
`styles/tokens.css`, not only in `UiTokensPreamble`'s runtime `<style>`. That
is load-bearing: the class replaced a stock `shadow-sm`, so a runtime-only
definition would leave a consumer who never mounts the preamble with *no*
shadow at all -- worse than the drift it fixes, and silent
(`tests/ld_class_stylesheet_coverage.rs` pins it).

## Reference

Demo: `/components/kpi_strip` -- including a twelve-card Office-like
dashboard covering above/level/below, saturated, no-baseline, settling, a
zero baseline, a disabled action, and an unavailable value. The
`/components/admin_workbench` fixture deliberately mixes four
comparison-bearing cards with four plain ones, so that page's existing
equal-height and aligned-value browser assertions double as the alignment
proof.

Source: `src/patterns/kpi_strip.rs`. Browser proof:
`tests/admin_workbench_smoke.rs` (`cargo xtask test-admin-workbench`).
