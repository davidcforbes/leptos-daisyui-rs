//! Real-browser proof for the `.ld-focus-ring` entrance (ldui-reod): the ring
//! colour must be present from the very first frame after a control receives
//! keyboard focus, and only `outline-offset` may animate (0 -> 2px).
//!
//! The defect: `ld-focus-ring-in` faded `outline-color` in from
//! `transparent`, so for the first `--ld-duration-fast` after a Tab every
//! ring-carrying control's computed outline was a transparent 2px line. A
//! keyboard probe that focuses a control and reads its computed outline in
//! the same tick saw no ring at all on SOLID buttons (4iiz-Office's Search /
//! EN / ES buttons and the active hub tab), because nothing but the outline
//! changes on their focus.
//!
//! Drives the general demo app (`html_target: None`) on the existing
//! `/components/button` route. Kept in its own file/xtask step
//! (`cargo xtask test-focus-ring`) rather than folded into
//! `reactivity_smoke.rs`, whose check count is pinned.
//!
//! Two things make this a check that WORKS rather than one that passes:
//!
//! - The harness loads every page under `?pp-freeze=1`, whose kill switch is
//!   `animation: none !important` on every element. Under it the keyframes
//!   never run, so a frame-0 probe would be green with the old CSS too. The
//!   test therefore disables that `<style data-pixelproof="freeze">` sheet
//!   before probing and asserts the real `ld-focus-ring-in` animation is the
//!   one running (`animationName`) with the ring flush at `0px` on frame 0 --
//!   an assertion that fails if the page is still frozen.
//! - Focus and the computed-style read happen in ONE synchronous `evaluate`
//!   with no await between them, so the read sees the animation at time 0
//!   (the `from` keyframe), exactly as the consumer's probe did.
mod common;

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use common::{
    assert_no_browser_errors, begin_browser_error_capture, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

const PAGE: &str = "/components/button";

/// A solid primary button: `Button` emits `btn ld-eased ld-pressable
/// ld-focus-ring` plus the colour class, and the button demo's first section
/// renders `<Button color=ButtonColor::Primary>` with no style modifier. Ghost,
/// outline and link styles are excluded because daisyUI also moves their
/// `box-shadow` on focus, which is the second property that hid the defect.
const SOLID_BUTTON: &str = "button.btn.btn-primary.ld-focus-ring:not(.btn-outline):not(.btn-ghost):not(.btn-link):not(.btn-disabled):not(:disabled)";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate focus-ring probe")
        .into_value()
        .expect("focus-ring expression returns JSON")
}

/// Press plain Tab with real CDP key events. Mirrors `common::press_enter`;
/// a real key press makes keyboard the last input modality, so Chromium's
/// `:focus-visible` heuristic treats the scripted `focus()` that follows as
/// keyboard focus.
async fn press_tab(h: &pixelproof_web::Harness) {
    let down = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::RawKeyDown)
        .key("Tab")
        .code("Tab")
        .windows_virtual_key_code(9)
        .native_virtual_key_code(9)
        .build()
        .expect("Tab key-down params");
    h.page().execute(down).await.expect("dispatch Tab key-down");
    let up = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::KeyUp)
        .key("Tab")
        .code("Tab")
        .windows_virtual_key_code(9)
        .native_virtual_key_code(9)
        .build()
        .expect("Tab key-up params");
    h.page().execute(up).await.expect("dispatch Tab key-up");
}

/// The rgba alpha of a computed colour string (`rgb(...)` is opaque).
fn alpha_of(color: &str) -> f64 {
    let color = color.trim();
    if color.starts_with("rgba(") {
        color
            .trim_start_matches("rgba(")
            .trim_end_matches(')')
            .split(',')
            .nth(3)
            .and_then(|a| a.trim().parse::<f64>().ok())
            .unwrap_or(0.0)
    } else if color.starts_with("rgb(")
        || color.starts_with("color(")
        || color.starts_with("oklch(")
    {
        // No alpha channel written means opaque; a `/ <alpha>` inside a
        // modern colour function is the only other way to carry one.
        color
            .split('/')
            .nth(1)
            .and_then(|a| a.trim().trim_end_matches(')').trim().parse::<f64>().ok())
            .unwrap_or(1.0)
    } else if color == "transparent" {
        0.0
    } else {
        1.0
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-focus-ring)"]
async fn focus_ring_colour_is_present_on_frame_zero_and_the_offset_settles() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, SOLID_BUTTON).await;

    // Lift the PixelProof kill switch (`animation: none !important`) so the
    // crate's real `ld-focus-ring-in` keyframes run. Without this the probe
    // cannot distinguish the fixed CSS from the defective one.
    let unfrozen = eval_json(
        &h,
        r#"(() => {
            const sheets = [...document.querySelectorAll('style[data-pixelproof="freeze"]')];
            for (const s of sheets) { s.disabled = true; if (s.sheet) s.sheet.disabled = true; }
            return { freezeSheets: sheets.length };
        })()"#,
    )
    .await;
    assert_eq!(
        unfrozen["freezeSheets"],
        json!(1),
        "expected exactly one PixelProof freeze sheet to lift: {unfrozen}"
    );

    // A real Tab press makes keyboard the last input modality.
    press_tab(&h).await;

    // Focus and the frame-0 read in one synchronous evaluate: no await between
    // `focus()` and `getComputedStyle`, so this is what a keyboard user's very
    // first frame -- and the consumer's probe -- sees.
    let at_focus = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const el = document.querySelector('{SOLID_BUTTON}');
                el.focus();
                const cs = getComputedStyle(el);
                return {{
                    isActive: document.activeElement === el,
                    focusVisible: el.matches(':focus-visible'),
                    animationName: cs.animationName,
                    outlineColor: cs.outlineColor,
                    outlineStyle: cs.outlineStyle,
                    outlineWidth: cs.outlineWidth,
                    outlineOffset: cs.outlineOffset,
                    classes: el.className,
                }};
            }})()"#
        ),
    )
    .await;

    assert_eq!(
        at_focus["isActive"],
        json!(true),
        "the solid button must take focus: {at_focus}"
    );
    assert_eq!(
        at_focus["focusVisible"],
        json!(true),
        "scripted focus after a real Tab must match :focus-visible (Chromium keyboard-modality heuristic): {at_focus}"
    );
    assert_eq!(
        at_focus["animationName"],
        json!("ld-focus-ring-in"),
        "the crate's entrance must be the animation running (is the freeze sheet still on?): {at_focus}"
    );
    assert_eq!(
        at_focus["outlineStyle"],
        json!("solid"),
        "frame 0 must already show a solid ring: {at_focus}"
    );
    assert_eq!(
        at_focus["outlineWidth"],
        json!("2px"),
        "frame 0 must already show the 2px ring: {at_focus}"
    );
    let colour = at_focus["outlineColor"].as_str().unwrap_or_default();
    assert_ne!(
        colour, "rgba(0, 0, 0, 0)",
        "frame 0 must not be a transparent ring (the ldui-reod defect): {at_focus}"
    );
    assert!(
        alpha_of(colour) > 0.0,
        "frame 0 ring colour must be opaque, got {colour}: {at_focus}"
    );
    // Only the offset animates: at time 0 it is the `from` keyframe's 0px.
    // This is the assertion that proves the animation is genuinely running.
    assert_eq!(
        at_focus["outlineOffset"],
        json!("0px"),
        "frame 0 must show the ring flush at 0px offset -- only the offset animates: {at_focus}"
    );

    // Well past `--ld-duration-fast` (83ms) the offset has settled at the
    // rule's resting 2px and the colour never went anywhere.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let settled = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const el = document.querySelector('{SOLID_BUTTON}');
                const cs = getComputedStyle(el);
                return {{
                    isActive: document.activeElement === el,
                    focusVisible: el.matches(':focus-visible'),
                    outlineColor: cs.outlineColor,
                    outlineStyle: cs.outlineStyle,
                    outlineWidth: cs.outlineWidth,
                    outlineOffset: cs.outlineOffset,
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(
        settled["focusVisible"],
        json!(true),
        "focus must still be visible after the entrance: {settled}"
    );
    assert_eq!(
        settled["outlineOffset"],
        json!("2px"),
        "the ring must settle at the 2px resting offset: {settled}"
    );
    assert_eq!(settled["outlineStyle"], json!("solid"), "{settled}");
    assert_eq!(settled["outlineWidth"], json!("2px"), "{settled}");
    assert_eq!(
        settled["outlineColor"], at_focus["outlineColor"],
        "the ring colour must be the same at rest as on frame 0: {settled} vs {at_focus}"
    );

    assert_no_browser_errors(&h, "focus ring frame-0 colour").await;
}
