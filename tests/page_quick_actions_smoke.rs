//! Real-browser proof for `PageQuickActions`/`PageQuickActionContent`
//! (ldui-ynmd.2): a wrapping icon-action row that fits or wraps
//! predictably beside a title/subtitle at wide width, stays reachable
//! without horizontal page overflow at compact width, exposes a clear
//! accessible group name, and preserves accessible labels (and, when
//! composed with `Tooltip`, hover/focus tooltips) for icon-only collapse.
//! Also proves `PageHeader`'s new typed `divider` option and its now-
//! wrapping actions host, and (ldui-rhnr) the `OnDivider` navigation
//! layout's back / breadcrumb trail / rule row under the title row.
//! Drives the general demo app (`html_target: None`)
//! against the existing `/components/page_quick_actions` showcase route,
//! matching `section_heading_smoke.rs`'s pattern.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use pixelproof_web::ViewportSize;
use serde_json::{Value, json};

const PAGE: &str = "/components/page_quick_actions";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate page-quick-actions fixture")
        .into_value()
        .expect("page-quick-actions expression returns JSON")
}

/// Shape of one `[data-testid]` fixture's `PageHeader` + actions host: the
/// divider marker, how many *distinct* accessible names its action controls
/// expose, and whether the actions host itself wraps (`flex-wrap`).
async fn snapshot(h: &pixelproof_web::Harness, fixture_testid: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const fixture = document.querySelector('[data-testid="{fixture_testid}"]');
                const header = fixture.querySelector('[data-page-header]');
                const group = fixture.querySelector('[data-page-quick-actions]');
                const controls = group
                    ? Array.from(group.querySelectorAll('button, a'))
                        .filter(el => el.closest('[data-page-quick-actions]') === group)
                    : [];
                return {{
                    divider: header ? header.getAttribute('data-page-header-divider') : null,
                    groupRole: group ? group.getAttribute('role') : null,
                    groupLabel: group ? group.getAttribute('aria-label') : null,
                    controlCount: controls.length,
                    controlNames: controls.map(el => (el.textContent || '').trim()),
                }};
            }})()"#
        ),
    )
    .await
}

/// The base-page fixture (no back button) renders all seven actions inside
/// one accessibly-named group, each with a distinct, non-empty accessible
/// name, and the header's default divider is shown.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-page-quick-actions)"]
async fn seven_actions_render_with_a_named_group_and_the_default_divider() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, "page-quick-actions-base").await;
    assert_eq!(s["divider"], json!("shown"), "default divider: {s}");
    assert_eq!(s["groupRole"], json!("group"), "accessible group role: {s}");
    assert_eq!(
        s["groupLabel"],
        json!("Case actions"),
        "accessible group name: {s}"
    );
    assert_eq!(s["controlCount"], json!(7), "seven quick actions: {s}");

    let names: Vec<String> = s["controlNames"]
        .as_array()
        .expect("controlNames array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        names.iter().all(|n| !n.is_empty()),
        "every action keeps a non-empty accessible name: {names:?}"
    );
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        names.len(),
        "every action's accessible name is distinct: {names:?}"
    );

    assert_no_browser_errors(&h, "page-quick-actions base variant").await;
}

/// `PageHeaderDivider::Hidden` actually omits the marker (and, by
/// construction, the border/padding classes) -- proven against the running
/// DOM, not just the source-level unit test.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-page-quick-actions)"]
async fn divider_hidden_variant_carries_the_hidden_marker() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, "page-quick-actions-no-divider").await;
    assert_eq!(
        s["divider"],
        json!("hidden"),
        "explicit hidden divider: {s}"
    );
    assert_eq!(s["controlCount"], json!(7), "seven quick actions: {s}");

    assert_no_browser_errors(&h, "page-quick-actions no-divider variant").await;
}

/// At a compact viewport, a fixture whose container is deliberately
/// narrower than seven actions wide must not push anything past its own
/// container's right edge -- the actions wrap onto further rows instead of
/// overflowing the page horizontally.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-page-quick-actions)"]
async fn compact_container_wraps_actions_without_horizontal_overflow() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let overflow = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="page-quick-actions-compact"]');
            const group = fixture.querySelector('[data-page-quick-actions]');
            const fixtureRight = fixture.getBoundingClientRect().right;
            const buttons = Array.from(group.querySelectorAll('button, a'));
            return buttons.every(el => el.getBoundingClientRect().right <= fixtureRight + 1);
        })()"#,
    )
    .await;
    assert_eq!(
        overflow,
        json!(true),
        "every action must stay within its fixture's right edge, wrapping rather than overflowing"
    );

    let rows = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="page-quick-actions-compact"]');
            const group = fixture.querySelector('[data-page-quick-actions]');
            const tops = Array.from(group.querySelectorAll('button, a')).map(
                el => Math.round(el.getBoundingClientRect().top)
            );
            return new Set(tops).size;
        })()"#,
    )
    .await;
    assert!(
        rows.as_u64().unwrap_or(0) > 1,
        "seven actions in a narrow container must wrap onto more than one row: {rows}"
    );

    assert_no_browser_errors(&h, "page-quick-actions compact variant").await;
}

/// The icon-only-collapse fixture keeps each label in the accessible tree
/// (`sr-only`, never `hidden`) even while visually collapsed at a compact
/// width, and each action is wrapped in a `Tooltip` (`data-tip`) carrying
/// the same text -- so a sighted mouse/keyboard user still gets the label
/// once it collapses.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-page-quick-actions)"]
async fn icon_only_collapse_preserves_tooltips_and_accessible_labels() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let s = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="page-quick-actions-collapse"]');
            const group = fixture.querySelector('[data-page-quick-actions]');
            const controls = Array.from(group.querySelectorAll('button'));
            return controls.map(el => {
                const tooltip = el.closest('.tooltip');
                const label = el.querySelector('[data-page-quick-action-label-visibility]')
                    ?.querySelector('span:last-child');
                const style = label ? getComputedStyle(label) : null;
                return {
                    accessibleText: (el.textContent || '').trim(),
                    tooltipTip: tooltip ? tooltip.getAttribute('data-tip') : null,
                    labelVisuallyHidden: style
                        ? (style.position === 'absolute' && parseInt(style.width, 10) <= 1)
                        : null,
                };
            });
        })()"#,
    )
    .await;

    let entries = s.as_array().expect("collapse entries array");
    assert_eq!(entries.len(), 2, "two icon-only-collapse actions: {s}");
    for entry in entries {
        assert!(
            !entry["accessibleText"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "sr-only label keeps the accessible name non-empty: {entry}"
        );
        assert!(
            entry["tooltipTip"].as_str().is_some_and(|t| !t.is_empty()),
            "each collapsed action is wrapped in a Tooltip carrying the same text: {entry}"
        );
        assert_eq!(
            entry["tooltipTip"], entry["accessibleText"],
            "tooltip text matches the accessible label: {entry}"
        );
    }

    assert_no_browser_errors(&h, "page-quick-actions icon-only collapse variant").await;
}

/// The localized fixture's title/subtitle are reactive: toggling the
/// fixture's language button swaps the rendered `PageHeader` copy in place
/// while the seven quick actions keep their (untranslated in this demo)
/// accessible names.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-page-quick-actions)"]
async fn localized_title_updates_reactively_beside_the_actions_row() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    const TITLE_EXPR: &str = r#"(() => {
        const fixture = document.querySelector('[data-testid="page-quick-actions-localized"]');
        return fixture.querySelector('h1')?.textContent ?? null;
    })()"#;

    let before = eval_json(&h, TITLE_EXPR).await;
    assert_eq!(
        before,
        json!("Active matter federation across every partner office"),
        "English by default: {before}"
    );

    click(&h, "[data-testid=\"page-quick-actions-localized-toggle\"]").await;
    let after = eval_json(&h, TITLE_EXPR).await;
    assert_eq!(
        after,
        json!("Federation des dossiers actifs a travers tous les cabinets partenaires"),
        "toggling swaps the reactive title signal in place: {after}"
    );

    let s = snapshot(&h, "page-quick-actions-localized").await;
    assert_eq!(
        s["controlCount"],
        json!(7),
        "actions row is unaffected by the title swap: {s}"
    );

    click(&h, "[data-testid=\"page-quick-actions-localized-toggle\"]").await;
    let restored = eval_json(&h, TITLE_EXPR).await;
    assert_eq!(
        restored, before,
        "toggling back restores the original title"
    );

    assert_no_browser_errors(&h, "page-quick-actions localized variant").await;
}

/// ldui-rhnr: `PageHeaderNavigationLayout::OnDivider` keeps the title and
/// actions on row 1, then renders ONE navigation landmark whose parts are
/// back, trail and rule in that order (each found by its own hook). Its
/// single hairline rule replaces the header's bottom border, sits on the
/// row's vertical center and fills the row to its right edge. Only the last
/// crumb is the current page.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-page-quick-actions)"]
async fn on_divider_row_carries_back_trail_and_one_rule_under_the_title_row() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1280, 900))
        .await
        .expect("set wide viewport");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let s = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="page-quick-actions-on-divider"]');
            const header = fixture.querySelector('[data-page-header]');
            const row = header.querySelector('[data-page-header-divider-row]');
            const title = header.querySelector('h1');
            const actions = header.querySelector('[data-page-actions-slot]');
            const rule = row.querySelector('[data-page-header-rule]');
            const box = el => el.getBoundingClientRect();
            const center = el => (box(el).top + box(el).bottom) / 2;
            const hooks = ['back', 'trail', 'rule'];
            const crumbs = Array.from(row.querySelectorAll('[data-page-header-crumb]'));
            const lastCrumb = crumbs[crumbs.length - 1];
            const crumbText = li => {
                const copy = li.cloneNode(true);
                copy.querySelectorAll('[data-page-header-crumb-separator]').forEach(el => el.remove());
                return copy.textContent.trim();
            };
            const headerStyle = getComputedStyle(header);
            return {
                layout: header.getAttribute('data-page-header-navigation-layout'),
                rowTag: row.tagName.toLowerCase(),
                rowLabel: row.getAttribute('aria-label'),
                order: Array.from(row.children).map(el =>
                    hooks.find(hook => el.hasAttribute('data-page-header-' + hook)) || el.tagName.toLowerCase()),
                rules: header.querySelectorAll('[data-page-header-rule]').length,
                headerBorderBottom: headerStyle.borderBottomWidth,
                headerPaddingBottom: headerStyle.paddingBottom,
                ruleBorderTop: getComputedStyle(rule).borderTopWidth,
                crumbs: crumbs.map(li => ({
                    kind: li.getAttribute('data-page-header-crumb'),
                    text: crumbText(li),
                    href: li.querySelector('a') ? li.querySelector('a').getAttribute('href') : null,
                })),
                current: Array.from(row.querySelectorAll('[aria-current="page"]')).map(el => ({
                    tag: el.tagName.toLowerCase(),
                    text: el.textContent.trim(),
                    inLastCrumb: el.closest('[data-page-header-crumb]') === lastCrumb,
                })),
                titleTop: box(title).top,
                titleRight: box(title).right,
                titleBottom: box(title).bottom,
                actionsTop: box(actions).top,
                actionsLeft: box(actions).left,
                actionsBottom: box(actions).bottom,
                rowTop: box(row).top,
                rowRight: box(row).right,
                rowCenter: center(row),
                ruleCenter: center(rule),
                ruleHeight: box(rule).height,
                ruleRight: box(rule).right,
                maxPartCenterOffset: Math.max(
                    ...Array.from(row.children).map(el => Math.abs(center(el) - center(row)))
                ),
            };
        })()"#,
    )
    .await;

    assert_eq!(s["layout"], json!("on-divider"), "layout marker: {s}");
    assert_eq!(
        s["rowTag"],
        json!("nav"),
        "the divider row is the landmark: {s}"
    );
    assert_eq!(
        s["rowLabel"],
        json!("Page navigation"),
        "landmark named by navigation_label: {s}"
    );
    assert_eq!(
        s["order"],
        json!(["back", "trail", "rule"]),
        "row order: {s}"
    );
    assert_eq!(s["rules"], json!(1), "exactly one rule in the header: {s}");
    assert_eq!(
        s["headerBorderBottom"],
        json!("0px"),
        "the row's rule replaces the header's bottom border: {s}"
    );
    assert_eq!(
        s["headerPaddingBottom"],
        json!("0px"),
        "no pb-4 gap under the header: {s}"
    );
    assert_eq!(s["ruleBorderTop"], json!("1px"), "a hairline rule: {s}");
    assert_eq!(
        s["crumbs"],
        json!([
            { "kind": "link", "text": "Coordinator", "href": "#coordinator" },
            { "kind": "current", "text": "Account Conversations", "href": null }
        ]),
        "crumbs: {s}"
    );
    assert_eq!(
        s["current"],
        json!([{ "tag": "span", "text": "Account Conversations", "inLastCrumb": true }]),
        "only the last crumb is the current page, as plain text: {s}"
    );

    let n = |key: &str| {
        s[key]
            .as_f64()
            .unwrap_or_else(|| panic!("{key} is not a number: {s}"))
    };
    assert!(
        (n("titleTop") - n("actionsTop")).abs() <= 1.0,
        "title and actions share row 1: {s}"
    );
    assert!(
        n("actionsLeft") >= n("titleRight"),
        "actions sit right of the title: {s}"
    );
    assert!(
        n("rowTop") >= n("titleBottom").max(n("actionsBottom")) - 0.5,
        "the divider row sits under row 1: {s}"
    );
    assert!(
        (n("ruleCenter") - n("rowCenter")).abs() <= 1.0,
        "the rule is vertically centered on the row: {s}"
    );
    assert!(n("ruleHeight") <= 1.5, "the rule is a hairline: {s}");
    assert!(
        (n("ruleRight") - n("rowRight")).abs() <= 1.0,
        "the rule fills the row to its right edge: {s}"
    );
    assert!(
        n("maxPartCenterOffset") <= 1.0,
        "back, trail and rule share one centered line: {s}"
    );

    assert_no_browser_errors(&h, "page-quick-actions on-divider variant").await;
}

/// ldui-rhnr at a narrow width: the trail is the part that gives way. The
/// fixture is narrowed to 16rem in place (the row is pure CSS, nothing
/// measures on mount), and the same measurement taken first at full width
/// is the negative control: the trail is whole there and truncated here,
/// while the back control keeps its width, the rule keeps its `min-w-8`
/// floor and the row stays one centered line inside the fixture.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-page-quick-actions)"]
async fn on_divider_trail_truncates_first_and_the_row_never_wraps() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1280, 900))
        .await
        .expect("set wide viewport");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let s = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="page-quick-actions-on-divider"]');
            const row = fixture.querySelector('[data-page-header-divider-row]');
            const back = row.querySelector('[data-page-header-back]');
            const trail = row.querySelector('[data-page-header-trail]');
            const rule = row.querySelector('[data-page-header-rule]');
            const box = el => el.getBoundingClientRect();
            const center = el => (box(el).top + box(el).bottom) / 2;
            const measure = () => ({
                fixtureWidth: box(fixture).width,
                fixtureRight: box(fixture).right,
                rowRight: box(row).right,
                backWidth: box(back).width,
                ruleWidth: box(rule).width,
                trailOverflow: trail.scrollWidth - trail.clientWidth,
                maxPartCenterOffset: Math.max(
                    ...[back, trail, rule].map(el => Math.abs(center(el) - center(row)))
                ),
            });
            const wide = measure();
            fixture.style.width = '16rem';
            const narrow = measure();
            return { wide, narrow, textOverflow: getComputedStyle(trail).textOverflow };
        })()"#,
    )
    .await;

    let n = |phase: &str, key: &str| {
        s[phase][key]
            .as_f64()
            .unwrap_or_else(|| panic!("{phase}.{key} is not a number: {s}"))
    };
    assert!(
        n("wide", "fixtureWidth") > 400.0 && n("narrow", "fixtureWidth") <= 256.5,
        "the 16rem constraint actually applied: {s}"
    );
    assert!(
        n("wide", "trailOverflow") <= 1.0,
        "negative control: the trail is whole when there is room: {s}"
    );
    assert!(
        n("narrow", "trailOverflow") > 8.0,
        "the trail truncates at a narrow width: {s}"
    );
    assert_eq!(
        s["textOverflow"],
        json!("ellipsis"),
        "with an ellipsis: {s}"
    );
    assert!(
        (n("narrow", "backWidth") - n("wide", "backWidth")).abs() <= 0.5,
        "the back control never shrinks: {s}"
    );
    assert!(
        n("narrow", "ruleWidth") >= 31.5,
        "the rule keeps its min-w-8 floor: {s}"
    );
    for phase in ["wide", "narrow"] {
        assert!(
            n(phase, "maxPartCenterOffset") <= 1.0,
            "{phase}: the row stays one centered line: {s}"
        );
        assert!(
            n(phase, "rowRight") <= n(phase, "fixtureRight") + 1.0,
            "{phase}: the row stays inside the fixture: {s}"
        );
    }

    assert_no_browser_errors(&h, "page-quick-actions on-divider narrow").await;
}
