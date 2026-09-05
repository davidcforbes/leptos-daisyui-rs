//! Controlled UI evidence only: commands and acknowledgments are simulated.
mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use pixelproof_web::{Harness, Key, ViewportSize};
use serde_json::{Value, json};

async fn eval(h: &Harness, script: &str) -> Value {
    h.page()
        .evaluate(script)
        .await
        .expect("evaluate softphone")
        .into_value()
        .expect("softphone JSON")
}

async fn state(h: &Harness) -> Value {
    eval(h, r#"(() => {
        const root = document.querySelector('#softphone-demo');
        const host = document.querySelector('[data-testid=softphone-host]');
        return {
            count: Number(host.dataset.commandCount), phase: host.dataset.phase,
            selected: host.dataset.selected, pending: host.dataset.pending,
            last: host.dataset.lastCommand, recording: host.dataset.recording,
            transcribing: host.dataset.transcribing, muted: host.dataset.muted,
            timer: root.querySelector('[data-softphone-timer]').textContent.trim(),
            selectedValue: root.querySelector('select')?.value ?? null,
            locked: root.querySelector('select')?.disabled ?? null,
            actions: Object.fromEntries([...root.querySelectorAll('[data-softphone-action]')].map(b =>
                [b.dataset.softphoneAction, {disabled: b.disabled, pressed: b.getAttribute('aria-pressed'), name: b.textContent.trim()}]))
        };
    })()"#).await
}

async fn action(h: &Harness, name: &str) {
    click(
        h,
        &format!("#softphone-demo [data-softphone-action='{name}']"),
    )
    .await;
}

async fn pending(h: &Harness, name: &str) {
    wait_for_selector(
        h,
        &format!("[data-testid=softphone-host][data-pending='{name}']"),
    )
    .await;
}

async fn phase(h: &Harness, name: &str) {
    wait_for_selector(
        h,
        &format!("[data-testid=softphone-host][data-phase='{name}']"),
    )
    .await;
}

async fn screenshot(h: &Harness, path: &str) {
    common::prepare_region_capture(
        h,
        "#softphone-demo",
        ViewportSize::new(if path.contains("compact") { 375 } else { 1280 }, 1100),
    )
    .await;
    std::fs::create_dir_all("target").expect("screenshot directory");
    std::fs::write(
        path,
        h.screenshot_bytes().await.expect("softphone screenshot"),
    )
    .expect("save softphone review image");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn commands_require_confirmation_and_clock_survives_hold_then_freezes() {
    let h = harness_at("/components/softphone").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, "#softphone-demo-number").await;
    h.page()
        .find_element("#softphone-demo-number")
        .await
        .unwrap()
        .focus()
        .await
        .unwrap();
    // The blank prompt precedes mobile and office in this native select.
    h.press_key_sequence(&[
        Key::Space,
        Key::Home,
        Key::ArrowDown,
        Key::ArrowDown,
        Key::Enter,
    ])
    .await
    .unwrap();
    wait_for_selector(&h, "[data-testid=softphone-host][data-selected=office]").await;
    let selected = state(&h).await;
    assert_eq!(selected["selectedValue"], "office", "{selected}");
    assert!(
        selected["last"]
            .as_str()
            .unwrap()
            .contains("SelectNumber(\"office\")"),
        "{selected}"
    );

    action(&h, "call").await;
    pending(&h, "call").await;
    let calling = state(&h).await;
    assert_eq!(calling["phase"], "dialing", "{calling}");
    assert_eq!(calling["locked"], true, "{calling}");
    assert!(
        calling["last"].as_str().unwrap().contains("office"),
        "{calling}"
    );
    assert_eq!(calling["actions"]["end-call"]["disabled"], false);
    // Synthetic attempts exercise handler guards independently of disabled markup.
    eval(&h, r#"(() => { const b = document.querySelector('#softphone-demo [data-softphone-action=record]'); b.disabled = false; b.dispatchEvent(new MouseEvent('click', {bubbles:true})); return true; })()"#).await;
    assert_eq!(state(&h).await["count"], calling["count"]);
    click(&h, "#softphone-accept").await;
    phase(&h, "active").await;
    assert_eq!(state(&h).await["timer"], "01:05");
    click(&h, "#softphone-advance").await;
    assert_eq!(state(&h).await["timer"], "02:10");

    action(&h, "record").await;
    pending(&h, "record").await;
    let requested = state(&h).await;
    assert_eq!(requested["recording"], "false", "{requested}");
    assert_eq!(requested["actions"]["record"]["pressed"], "false");
    assert!(
        requested["last"]
            .as_str()
            .unwrap()
            .contains("SetRecording(true)")
    );
    click(&h, "#softphone-reject").await;
    wait_for_selector(&h, "#softphone-demo [data-softphone-error][role=alert]").await;
    assert_eq!(state(&h).await["recording"], "false");
    action(&h, "record").await;
    click(&h, "#softphone-accept").await;
    wait_for_selector(&h, "#softphone-demo [data-softphone-recording]").await;
    assert_eq!(state(&h).await["actions"]["record"]["pressed"], "true");

    action(&h, "keypad").await;
    wait_for_selector(&h, "#softphone-demo [data-softphone-keypad]").await;
    action(&h, "hold").await;
    click(&h, "#softphone-accept").await;
    phase(&h, "held").await;
    assert_eq!(
        eval(
            &h,
            "document.querySelector('#softphone-demo [data-softphone-keypad]') === null"
        )
        .await,
        true,
        "confirming Hold must remove an open keypad"
    );
    click(&h, "#softphone-advance").await;
    let held = state(&h).await;
    assert_eq!(held["timer"], "03:15");
    assert_eq!(held["locked"], true);
    assert_eq!(held["actions"]["hold"]["pressed"], "true");

    action(&h, "hold").await;
    pending(&h, "hold").await;
    assert!(
        state(&h).await["last"]
            .as_str()
            .unwrap()
            .contains("SetHeld(false)")
    );
    assert_eq!(
        state(&h).await["phase"],
        "held",
        "Resume is a proposal until accepted"
    );
    click(&h, "#softphone-accept").await;
    phase(&h, "active").await;
    assert_eq!(state(&h).await["actions"]["hold"]["pressed"], "false");
    assert_eq!(
        eval(
            &h,
            "document.querySelector('#softphone-demo [data-softphone-keypad]') === null"
        )
        .await,
        true,
        "resuming does not reopen the local keypad"
    );

    action(&h, "transcribe").await;
    pending(&h, "transcribe").await;
    assert_eq!(state(&h).await["transcribing"], "false");
    action(&h, "end-call").await;
    pending(&h, "end-call").await;
    let ending = state(&h).await;
    assert_eq!(ending["actions"]["end-call"]["disabled"], true);
    eval(&h, r#"(() => { const b = document.querySelector('#softphone-demo [data-softphone-action=end-call]'); b.disabled = false; b.dispatchEvent(new MouseEvent('click', {bubbles:true})); return true; })()"#).await;
    assert_eq!(
        state(&h).await["count"],
        ending["count"],
        "duplicate end must not escape the pending guard"
    );
    click(&h, "#softphone-accept").await;
    phase(&h, "ended").await;
    let ended = state(&h).await;
    assert_eq!(ended["timer"], "03:15");
    click(&h, "#softphone-advance").await;
    assert_eq!(state(&h).await["timer"], ended["timer"]);
    assert_no_browser_errors(&h, "softphone controlled lifecycle").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn number_guards_keypad_and_responsive_accessible_controls() {
    let h = harness_at("/components/softphone").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, "#softphone-demo-number").await;
    let initial = state(&h).await;
    eval(&h, r#"(() => { const s = document.querySelector('#softphone-demo-number'); s.add(new Option('Unknown', 'unknown')); s.value = 'unknown'; s.dispatchEvent(new Event('change', {bubbles:true})); return true; })()"#).await;
    let rejected = state(&h).await;
    assert_eq!(rejected["count"], initial["count"]);
    assert_eq!(rejected["selectedValue"], "mobile");
    click(&h, "#softphone-single").await;
    wait_for_selector(&h, "#softphone-demo [data-softphone-number]").await;
    assert_eq!(state(&h).await["selectedValue"], Value::Null);
    click(&h, "#softphone-empty").await;
    assert_eq!(state(&h).await["actions"]["call"]["disabled"], true);
    click(&h, "#softphone-reset").await;
    action(&h, "call").await;
    click(&h, "#softphone-accept").await;
    phase(&h, "active").await;
    action(&h, "keypad").await;
    wait_for_selector(&h, "#softphone-demo [data-softphone-keypad]").await;
    let before_digit = state(&h).await;
    click(&h, "#softphone-demo [data-softphone-digit='5']").await;
    let digit = state(&h).await;
    assert_eq!(
        digit["count"].as_u64().unwrap(),
        before_digit["count"].as_u64().unwrap() + 1
    );
    assert!(digit["last"].as_str().unwrap().contains("SendDigit('5')"));
    eval(&h, "(() => { const i = document.createElement('input'); i.id = 'softphone-outside-input'; i.setAttribute('aria-label', 'Unrelated text'); document.body.append(i); i.focus(); return true; })()").await;
    h.page()
        .find_element("#softphone-outside-input")
        .await
        .unwrap()
        .type_str("123")
        .await
        .unwrap();
    assert_eq!(
        state(&h).await["count"],
        digit["count"],
        "typing outside the keypad must not send DTMF"
    );
    eval(
        &h,
        "document.querySelector('#softphone-outside-input').remove(); true",
    )
    .await;
    action(&h, "keypad").await;

    for (name, field, command) in [
        ("mute", "muted", "SetMuted(true)"),
        ("transcribe", "transcribing", "SetTranscribing(true)"),
    ] {
        action(&h, name).await;
        pending(&h, name).await;
        let request = state(&h).await;
        assert_eq!(request[field], "false", "pending {name}: {request}");
        assert_eq!(request["actions"][name]["pressed"], "false");
        assert!(
            request["last"].as_str().unwrap().contains(command),
            "{request}"
        );
        click(&h, "#softphone-accept").await;
        wait_for_selector(
            &h,
            &format!("[data-testid=softphone-host][data-{field}=true]"),
        )
        .await;
        assert_eq!(state(&h).await["actions"][name]["pressed"], "true");
        // A second request proposes the inverse, leaving the confirmed state
        // pressed until the host accepts it.
        action(&h, name).await;
        pending(&h, name).await;
        assert_eq!(state(&h).await[field], "true");
        click(&h, "#softphone-accept").await;
        wait_for_selector(
            &h,
            &format!("[data-testid=softphone-host][data-{field}=false]"),
        )
        .await;
        assert_eq!(state(&h).await["actions"][name]["pressed"], "false");
    }

    for width in [1280, 375] {
        h.set_viewport(ViewportSize {
            width,
            height: 1100,
        })
        .await
        .unwrap();
        let geometry = eval(&h, r#"(() => {
            const r = document.querySelector('#softphone-demo'); const b = r.getBoundingClientRect();
            return {width:b.width, overflow:r.scrollWidth > r.clientWidth + 1,
                timerLive:r.querySelector('[role=timer]').getAttribute('aria-live'),
                timerSize:parseFloat(getComputedStyle(r.querySelector('[role=timer]')).fontSize),
                controls:[...r.querySelectorAll('button')].map(e => { const c=e.getBoundingClientRect(); return {
                    name:(e.getAttribute('aria-label') || e.textContent).trim(),
                    fits:c.left >= b.left && c.right <= b.right + 1,
                    visible:c.width > 0 && c.height >= 44 && getComputedStyle(e).visibility !== 'hidden'
                }; })};
        })()"#).await;
        assert!(
            geometry["width"].as_f64().unwrap() <= f64::from(width),
            "{geometry}"
        );
        assert_eq!(geometry["overflow"], false, "{geometry}");
        assert_eq!(geometry["timerLive"], "off");
        assert_eq!(
            geometry["timerSize"], 24,
            "duration must keep its prominent type size: {geometry}"
        );
        let controls = geometry["controls"].as_array().unwrap();
        assert_eq!(controls.len(), 7, "{geometry}");
        assert!(
            controls.iter().all(|c| c["fits"] == true
                && c["visible"] == true
                && !c["name"].as_str().unwrap().is_empty()),
            "{geometry}"
        );
        screenshot(
            &h,
            if width == 375 {
                "target/softphone-compact.png"
            } else {
                "target/softphone-active.png"
            },
        )
        .await;
    }

    let english = state(&h).await;
    click(&h, "#softphone-french").await;
    let french = state(&h).await;
    assert_eq!(french["actions"]["hold"]["name"], "Mettre en attente");
    assert_eq!(french["actions"]["record"]["name"], "Enregistrer");
    assert_ne!(
        french["actions"]["end-call"]["name"],
        english["actions"]["end-call"]["name"]
    );
    assert_eq!(
        french["count"], english["count"],
        "localization must not emit commands"
    );
    assert_eq!(
        eval(
            &h,
            "document.querySelector('#softphone-demo').getAttribute('aria-label')"
        )
        .await,
        "Appel client"
    );
    assert_eq!(
        eval(
            &h,
            "document.querySelector('#softphone-demo [role=timer]').getAttribute('aria-label')"
        )
        .await,
        "Durée de l’appel"
    );
    click(&h, "#softphone-french").await;
    assert_eq!(
        state(&h).await["actions"]["hold"]["name"],
        english["actions"]["hold"]["name"]
    );

    let normal_text = eval(&h, "document.querySelector('#softphone-demo').textContent").await;
    click(&h, "#softphone-long").await;
    let text_geometry = eval(&h, r#"(() => {
        const root = document.querySelector('#softphone-demo');
        const bounds = root.getBoundingClientRect();
        const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
        const overflow = [];
        while (walker.nextNode()) {
            const node = walker.currentNode;
            if (!node.textContent.trim()) continue;
            const range = document.createRange(); range.selectNodeContents(node);
            for (const r of range.getClientRects()) {
                if (r.width && (r.left < bounds.left - 1 || r.right > bounds.right + 1)) overflow.push(node.textContent);
            }
        }
        return {text:root.textContent, overflow};
    })()"#).await;
    assert_ne!(
        text_geometry["text"], normal_text,
        "long identity fixture must change the displayed client"
    );
    assert_eq!(
        text_geometry["overflow"],
        json!([]),
        "long identity must wrap inside the 375px console: {text_geometry}"
    );

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js").unwrap();
    axe.run(h.page()).await.unwrap();
    let violations = eval(&h, r#"(async () => { const r = await axe.run(document.querySelector('#softphone-demo'), {runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}}); return r.violations.filter(v => ['serious','critical'].includes(v.impact)); })()"#).await;
    assert_eq!(
        violations,
        json!([]),
        "softphone accessibility: {violations}"
    );

    action(&h, "voicemail").await;
    pending(&h, "voicemail").await;
    let routing = state(&h).await;
    assert_eq!(routing["phase"], "active");
    assert!(
        routing["last"]
            .as_str()
            .unwrap()
            .contains("RouteToVoicemail")
    );
    click(&h, "#softphone-accept").await;
    phase(&h, "ended").await;

    action(&h, "call").await;
    click(&h, "#softphone-accept").await;
    phase(&h, "active").await;
    action(&h, "keypad").await;
    wait_for_selector(&h, "#softphone-demo [data-softphone-keypad]").await;
    let before_capabilities = state(&h).await;
    click(&h, "#softphone-capabilities").await;
    let unsupported = state(&h).await;
    assert_eq!(
        unsupported["actions"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        vec!["end-call"]
    );
    assert_eq!(
        eval(
            &h,
            "document.querySelector('#softphone-demo [data-softphone-keypad]') === null"
        )
        .await,
        true,
        "removing keypad capability must remove an already-open keypad"
    );
    assert_eq!(
        unsupported["count"], before_capabilities["count"],
        "changing host capabilities must not emit commands"
    );
    assert_no_browser_errors(&h, "softphone choices, keypad and responsive controls").await;
}
