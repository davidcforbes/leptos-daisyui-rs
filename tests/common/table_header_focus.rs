//! Rendered state oracle for both shared sortable-header renderers (ldui-cxy8).
use chromiumoxide::cdp::browser_protocol::{
    emulation::{MediaFeature, SetEmulatedMediaParams},
    input::{DispatchMouseEventParams, DispatchMouseEventType, MouseButton},
};
use pixelproof_web::{Harness, Key};
use serde_json::Value;

async fn evaluate(h: &Harness, expression: &str) -> Value {
    h.page()
        .evaluate(expression)
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

pub async fn measure(h: &Harness, selector: &str) -> Value {
    evaluate(h, &format!(r#"(() => {{
        const b = document.querySelector({selector});
        const s = getComputedStyle(b);
        const ctx = document.createElement('canvas').getContext('2d');
        const rgba = value => {{
            ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = value;
            ctx.fillRect(0, 0, 1, 1); return [...ctx.getImageData(0, 0, 1, 1).data];
        }};
        const composite = (front, back) => front.slice(0, 3).map((c, i) =>
            c * front[3] / 255 + back[i] * (1 - front[3] / 255));
        const layers = []; for (let n = b; n; n = n.parentElement)
            layers.push(rgba(getComputedStyle(n).backgroundColor));
        let background = [255, 255, 255];
        for (const layer of layers.reverse()) background = composite(layer, background);
        const luminance = rgb => rgb.map(c => c / 255).map(c =>
            c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4)
            .reduce((n, c, i) => n + c * [.2126, .7152, .0722][i], 0);
        const opacity = node => {{ let value = 1; for (let n = node; n; n = n.parentElement)
            value *= Number(getComputedStyle(n).opacity); return value; }};
        const ratio = (color, alpha = 1) => {{
            const ink = rgba(color); ink[3] *= alpha;
            const a = luminance(composite(ink, background));
            const z = luminance(background);
            return (Math.max(a, z) + .05) / (Math.min(a, z) + .05);
        }};
        const icon = b.querySelector('[data-entity-sort-indicator], [data-table-sort-indicator]');
        const rect = b.getBoundingClientRect();
        return {{ color: s.color, background: s.backgroundColor, compositeBackground: background,
            contrast: ratio(s.color, opacity(b)), indicatorContrast: icon ? ratio(getComputedStyle(icon).color, opacity(icon)) : null,
            indicatorPresent: !!icon, indicatorOpacity: icon ? opacity(icon) : null,
            transitionsDisabled: [b, icon].filter(Boolean).every(node => getComputedStyle(node).transitionDuration.split(',').every(duration => Number.parseFloat(duration) === 0)),
            focusVisible: b.matches(':focus-visible'), hover: b.matches(':hover'), active: b.matches(':active'),
            outline: s.outlineStyle, outlineWidth: s.outlineWidth, outlineColor: s.outlineColor, outlineContrast: ratio(s.outlineColor),
            label: b.getAttribute('aria-label'), sort: b.closest('th').getAttribute('aria-sort'),
            width: rect.width, height: rect.height }};
    }})()"#, selector = serde_json::to_string(selector).unwrap())).await
}

fn readable(value: &Value) -> bool {
    value["indicatorPresent"] == true
        && value["contrast"].as_f64().unwrap() >= 4.5
        && value["indicatorContrast"]
            .as_f64()
            .is_some_and(|ratio| ratio >= 4.5)
}

pub async fn wait_sort(h: &Harness, selector: &str, expected: Value) {
    for _ in 0..100 {
        if measure(h, selector).await["sort"] == expected {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!(
        "sort did not settle to {expected}: {}",
        measure(h, selector).await
    );
}

async fn wait_pair(h: &Harness, selector: &str, background: &str, color: &str) -> Value {
    for _ in 0..100 {
        let value = measure(h, selector).await;
        if value["background"] == background && value["color"] == color {
            return value;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!(
        "header color pair did not settle: {}",
        measure(h, selector).await
    );
}

/// Seed this deep fixture's position, then leave and re-enter through actual
/// sequential navigation. The seed is not the evidence: Shift+Tab must leave
/// the header and Tab must return with the browser's :focus-visible state.
pub async fn tab_to(h: &Harness, selector: &str) {
    let expression = format!(
        "document.activeElement === document.querySelector({})",
        serde_json::to_string(selector).unwrap()
    );
    h.page()
        .find_element(selector)
        .await
        .unwrap()
        .focus()
        .await
        .unwrap();
    h.press_key_sequence(&[Key::ShiftTab]).await.unwrap();
    assert_eq!(
        evaluate(h, &expression).await,
        false,
        "Shift+Tab must leave {selector}"
    );
    h.press_key_sequence(&[Key::Tab]).await.unwrap();
    assert_eq!(
        evaluate(h, &expression).await,
        true,
        "Tab must reach {selector}"
    );
}

pub async fn check(h: &Harness, selector: &str, receipt: &str) {
    evaluate(
        h,
        "(() => { document.documentElement.setAttribute('data-theme', 'light'); return true; })()",
    )
    .await;
    super::force_desktop_hover_media(h).await;
    h.press_key_sequence(&[Key::Tab]).await.unwrap();
    if evaluate(
        h,
        &format!(
            "document.activeElement === document.querySelector({})",
            serde_json::to_string(selector).unwrap()
        ),
    )
    .await
        == true
    {
        h.press_key_sequence(&[Key::Tab]).await.unwrap();
    }
    h.page()
        .find_element(selector)
        .await
        .unwrap()
        .scroll_into_view()
        .await
        .unwrap();
    super::move_pointer_to_svg_fraction(h, selector, 0.5, 0.5).await;
    let pointer_hover = measure(h, selector).await;
    assert_eq!(pointer_hover["focusVisible"], false);
    assert_eq!(pointer_hover["hover"], true);
    assert_eq!(
        pointer_hover["transitionsDisabled"], true,
        "hover proof requires settled pp-freeze styles: {pointer_hover}"
    );
    assert!(
        readable(&pointer_hover),
        "pointer-only hovered header: {pointer_hover}"
    );
    eprintln!("HEADER_HOVER {receipt} {pointer_hover}");
    h.page()
        .execute(
            DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseMoved)
                .x(1.0)
                .y(1.0)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    tab_to(h, selector).await;
    let focused = measure(h, selector).await;
    eprintln!("HEADER_FOCUS {receipt} {focused}");
    assert_eq!(focused["focusVisible"], true);
    assert!(
        readable(&focused),
        "focused header must meet 4.5:1: {focused}"
    );
    assert_ne!(
        focused["outline"], "none",
        "focus outline must remain visible"
    );
    assert_ne!(focused["outlineWidth"], "0px");
    assert!(
        focused["outlineContrast"].as_f64().unwrap() >= 3.0,
        "focus outline contrast: {focused}"
    );
    std::fs::create_dir_all(super::repo_root().join(".review/header-focus")).unwrap();
    std::fs::write(
        super::repo_root().join(format!(".review/header-focus/{receipt}.png")),
        h.screenshot_bytes().await.unwrap(),
    )
    .unwrap();

    super::move_pointer_to_svg_fraction(h, selector, 0.5, 0.5).await;
    let hovered = measure(h, selector).await;
    assert_eq!(hovered["hover"], true);
    assert!(readable(&hovered), "hovered focused header: {hovered}");
    for key in ["width", "height", "sort", "label"] {
        assert_eq!(focused[key], hovered[key], "hover changed {key}");
    }

    // Sensitivity proof: restore the observed broken pair in the rendered
    // browser, then remove it. Never alter source bundles or approve baselines.
    evaluate(
        h,
        &format!(
            r#"(() => {{
        const button = document.querySelector({selector});
        button.dataset.focusProbeOriginalStyle = button.getAttribute('style') ?? '';
        button.style.setProperty('transition', 'none', 'important');
        button.style.setProperty('color', 'white', 'important');
        button.style.setProperty('background-color', '#F8FAFB', 'important'); return true;
    }})()"#,
            selector = serde_json::to_string(selector).unwrap()
        ),
    )
    .await;
    let broken = wait_pair(h, selector, "rgb(248, 250, 251)", "rgb(255, 255, 255)").await;
    assert!(
        !readable(&broken),
        "negative control escaped contrast oracle: {broken}"
    );
    eprintln!("HEADER_FOCUS_NEGATIVE_CONTROL {receipt} {broken}");
    evaluate(h, &format!(r#"(() => {{
        const button = document.querySelector({selector});
        if (button.dataset.focusProbeOriginalStyle) button.setAttribute('style', button.dataset.focusProbeOriginalStyle);
        else button.removeAttribute('style'); delete button.dataset.focusProbeOriginalStyle; return true;
    }})()"#, selector = serde_json::to_string(selector).unwrap())).await;
    let restored = wait_pair(
        h,
        selector,
        hovered["background"].as_str().unwrap(),
        hovered["color"].as_str().unwrap(),
    )
    .await;
    assert!(readable(&restored), "restored header: {restored}");
    evaluate(h, &format!(r#"(() => {{
        const marker = document.querySelector({selector}).querySelector('[data-entity-sort-indicator], [data-table-sort-indicator]');
        marker.dataset.focusProbeOriginalStyle = marker.getAttribute('style') ?? '';
        marker.style.setProperty('opacity', '0.01', 'important'); return true;
    }})()"#, selector = serde_json::to_string(selector).unwrap())).await;
    let faint_marker = measure(h, selector).await;
    assert!(faint_marker["contrast"].as_f64().unwrap() >= 4.5);
    assert!(
        !readable(&faint_marker),
        "faint marker escaped opacity-sensitive oracle: {faint_marker}"
    );
    eprintln!("HEADER_MARKER_NEGATIVE_CONTROL {receipt} {faint_marker}");
    evaluate(h, &format!(r#"(() => {{
        const marker = document.querySelector({selector}).querySelector('[data-entity-sort-indicator], [data-table-sort-indicator]');
        if (marker.dataset.focusProbeOriginalStyle) marker.setAttribute('style', marker.dataset.focusProbeOriginalStyle);
        else marker.removeAttribute('style'); delete marker.dataset.focusProbeOriginalStyle; return true;
    }})()"#, selector = serde_json::to_string(selector).unwrap())).await;
    assert!(readable(&measure(h, selector).await));

    // A held pointer press must be readable too. Release outside the button so
    // this presentation probe cannot activate a sort or issue a server query.
    let (x, y) = super::fraction_point(h, selector, 0.5, 0.5).await;
    h.page()
        .execute(
            DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MousePressed)
                .x(x)
                .y(y)
                .button(MouseButton::Left)
                .click_count(1)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    let pressed = measure(h, selector).await;
    assert_eq!(pressed["active"], true);
    assert!(readable(&pressed), "pressed header: {pressed}");
    eprintln!("HEADER_ACTIVE {receipt} {pressed}");
    for kind in [
        DispatchMouseEventType::MouseMoved,
        DispatchMouseEventType::MouseReleased,
    ] {
        h.page()
            .execute(
                DispatchMouseEventParams::builder()
                    .r#type(kind)
                    .x(1.0)
                    .y(1.0)
                    .button(MouseButton::Left)
                    .click_count(1)
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap();
    }
    tab_to(h, selector).await;
    h.page()
        .execute(
            SetEmulatedMediaParams::builder()
                .feature(MediaFeature::new("forced-colors", "active"))
                .build(),
        )
        .await
        .unwrap();
    assert_eq!(
        evaluate(h, "matchMedia('(forced-colors: active)').matches").await,
        true
    );
    let forced = measure(h, selector).await;
    assert!(readable(&forced), "forced-colors header: {forced}");
    assert_eq!(forced["focusVisible"], true);
    assert_ne!(forced["outline"], "none");
    assert!(
        forced["outlineContrast"].as_f64().unwrap() >= 3.0,
        "forced-colors focus outline contrast: {forced}"
    );
    eprintln!("HEADER_FORCED_COLORS {receipt} {forced}");
    h.page()
        .execute(
            SetEmulatedMediaParams::builder()
                .feature(MediaFeature::new("forced-colors", "none"))
                .build(),
        )
        .await
        .unwrap();
    evaluate(
        h,
        "(() => { document.documentElement.setAttribute('data-theme', 'dark'); return true; })()",
    )
    .await;
    let dark = measure(h, selector).await;
    assert!(readable(&dark), "dark theme focused header: {dark}");
    eprintln!("HEADER_DARK_FOCUS {receipt} {dark}");
    evaluate(
        h,
        "(() => { document.documentElement.setAttribute('data-theme', 'light'); return true; })()",
    )
    .await;
}
