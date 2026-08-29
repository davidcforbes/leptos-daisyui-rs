//! Focused real-browser contract for AppShell application chrome.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, harness_at};
use pixelproof_web::ViewportSize;
use serde_json::{Value, json};

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .unwrap_or_else(|error| panic!("evaluate `{expression}`: {error}"))
        .into_value()
        .unwrap_or_else(|error| panic!("JSON value for `{expression}`: {error}"))
}

async fn shell_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const fixture = document.querySelector('#app-shell-top-bar-fixture');
            const root = fixture.querySelector('[data-app-shell-root]');
            const top = root.querySelector('[data-app-shell-top-bar-region]');
            const start = top.querySelector('[data-app-shell-top-bar-start]');
            const center = top.querySelector('[data-app-shell-top-bar-center]');
            const end = top.querySelector('[data-app-shell-top-bar-end]');
            const body = root.querySelector('[data-app-shell-body]');
            const main = body.querySelector('[role="main"]');
            const status = root.querySelector(':scope > :last-child');
            const box = element => {
                const rect = element.getBoundingClientRect();
                return { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom, width: rect.width };
            };
            const topBefore = box(top);
            const statusBefore = box(status);
            main.scrollTop = Math.min(180, main.scrollHeight - main.clientHeight);
            const focusables = Array.from(root.querySelectorAll('a[href], input, select, button, [tabindex="0"]'))
                .map(element => element.matches('input') ? 'search'
                    : element.matches('select') ? 'language'
                    : element.matches('button') ? 'account'
                    : element.getAttribute('role') === 'main' ? 'main'
                    : 'brand');
            return {
                root: box(root),
                fixture: box(fixture),
                top: topBefore,
                topAfter: box(top),
                start: box(start),
                center: box(center),
                end: box(end),
                status: statusBefore,
                statusAfter: box(status),
                bannerRole: top.getAttribute('role'),
                bannerLabel: top.getAttribute('aria-label'),
                rootOverflow: getComputedStyle(root).overflow,
                topWrap: getComputedStyle(top).flexWrap,
                mainOverflowY: getComputedStyle(main).overflowY,
                mainScrollTop: main.scrollTop,
                mainScrollable: main.scrollHeight > main.clientHeight,
                bodyMarker: body.hasAttribute('data-app-shell-body'),
                pageOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
                focusables,
            };
        })()"#,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires current demo server"]
async fn app_shell_top_bar_stays_pinned_and_wraps_without_page_overflow() {
    let harness = harness_at("/app-shell").await;
    begin_browser_error_capture(&harness).await;

    let legacy = eval_json(
        &harness,
        r#"(() => {
            const root = document.querySelector('#app-shell-no-top-fixture [data-app-shell-root]');
            return {
                top: root.dataset.appShellTopBar,
                status: root.dataset.appShellStatusBar,
                bodyWrappers: root.querySelectorAll(':scope > [data-app-shell-body]').length,
                display: getComputedStyle(root).display,
                direction: getComputedStyle(root).flexDirection,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        legacy,
        json!({
            "top": "absent",
            "status": "absent",
            "bodyWrappers": 0,
            "display": "flex",
            "direction": "row",
        }),
        "no-region callers retain the original single-row DOM"
    );

    let wide = shell_snapshot(&harness).await;
    assert_eq!(wide["bannerRole"], json!("banner"));
    assert_eq!(wide["bannerLabel"], json!("Satellite application controls"));
    assert_eq!(wide["bodyMarker"], json!(true));
    assert_eq!(wide["mainScrollable"], json!(true));
    assert!(
        wide["mainScrollTop"]
            .as_f64()
            .is_some_and(|value| value > 0.0)
    );
    assert_eq!(wide["mainOverflowY"], json!("auto"));
    assert_eq!(wide["rootOverflow"], json!("hidden"));
    assert_eq!(wide["pageOverflow"], json!(false));
    assert_eq!(
        wide["top"], wide["topAfter"],
        "main scroll moved top chrome"
    );
    assert_eq!(
        wide["status"], wide["statusAfter"],
        "main scroll moved status chrome"
    );
    assert_eq!(
        wide["focusables"],
        json!(["brand", "search", "language", "account", "main"]),
        "DOM and keyboard order must remain start, center, end, then content"
    );

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let compact = shell_snapshot(&harness).await;
    assert_eq!(compact["topWrap"], json!("wrap"));
    assert_eq!(compact["pageOverflow"], json!(false));
    assert!(
        compact["root"]["right"].as_f64().unwrap()
            <= compact["fixture"]["right"].as_f64().unwrap() + 1.0,
        "compact shell escaped its fixture: {compact}"
    );
    assert!(
        compact["center"]["top"].as_f64().unwrap()
            >= compact["start"]["bottom"].as_f64().unwrap() - 1.0,
        "compact center slot did not take an intentional wrapped row: {compact}"
    );
    assert_no_browser_errors(&harness, "AppShell top-bar wide/compact contract").await;
}
