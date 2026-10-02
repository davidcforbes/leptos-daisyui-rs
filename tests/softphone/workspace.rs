//! Workspace evidence runs in the existing release softphone lane.
use super::{common, eval};
use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use pixelproof_web::{Harness, Key, ViewportSize};
use serde_json::{Value, json};

const ROOT: &str = "#client-call-workspace-demo";

async fn snapshot(h: &Harness) -> Value {
    eval(h, r#"(() => {
        const root = document.querySelector('#client-call-workspace-demo');
        const host = document.querySelector('[data-testid=call-workspace-host]');
        return {host:{...host.dataset}, fields:Object.fromEntries([...root.querySelectorAll('[data-call-field]')].map(e => [e.dataset.callField,{value:e.value,disabled:e.disabled,label:document.querySelector(`label[for="${e.id}"]`)?.textContent.trim()}])),
          actions:Object.fromEntries([...root.querySelectorAll('[data-call-action]')].map(e => [e.dataset.callAction,{disabled:e.disabled,name:e.textContent.trim()}])),
          status:root.querySelector('[data-call-status]')?.textContent.trim() ?? null,
          savedNumber:root.querySelector('[data-call-confirmed-number]')?.textContent.trim() ?? null,
          guidance:root.querySelector('[data-call-guidance-state]')?.textContent.trim() ?? null,
          guidanceState:root.querySelector('[data-call-guidance-state]')?.getAttribute('data-call-guidance-state') ?? null,
          regenerationState:root.querySelector('[data-call-regeneration-state]')?.getAttribute('data-call-regeneration-state') ?? null,
          beats:[...root.querySelectorAll('[data-call-beat]')].map(e=>({id:e.getAttribute('data-call-beat'),text:e.textContent.trim()})),
          noFileDetails:root.querySelectorAll('[data-call-no-file-detail]').length,
          talk:root.querySelector('[data-call-talk-time]')?.textContent.trim() ?? null,
          pad:!!root.querySelector('[data-call-pad]'), timer:root.querySelector('[role=timer]')?.textContent.trim() ?? null};
    })()"#).await
}

async fn action(h: &Harness, action: &str) {
    click(h, &format!("{ROOT} [data-call-action='{action}']")).await;
}

async fn type_into(h: &Harness, field: &str, text: &str) {
    let input = h
        .page()
        .find_element(format!("{ROOT} [data-call-field='{field}']"))
        .await
        .unwrap();
    input.focus().await.unwrap();
    input.type_str(text).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

async fn choose_outcome(h: &Harness, index: usize) {
    choose_field(h, "outcome", index).await;
}

async fn choose_field(h: &Harness, field: &str, index: usize) {
    h.page()
        .find_element(format!("{ROOT} [data-call-field='{field}']"))
        .await
        .unwrap()
        .focus()
        .await
        .unwrap();
    let mut keys = vec![Key::Space, Key::Home];
    keys.extend(std::iter::repeat_n(Key::ArrowDown, index));
    keys.push(Key::Enter);
    h.press_key_sequence(&keys).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn workspace_office_targets_refusals_and_provider_duration() {
    let h = harness_at("/components/client-call-workspace").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, ROOT).await;
    click(&h, "#call-workspace-office").await;
    let initial = snapshot(&h).await;
    assert_eq!(initial["fields"]["number-target"]["value"], "contact-phone");
    assert_eq!(initial["host"]["talkSeconds"], "");
    assert_eq!(
        initial["talk"],
        Value::Null,
        "op-fg6s2: no talk-time caveat before a call"
    );
    assert_eq!(
        eval(
            &h,
            "document.querySelector('[data-call-saved-number=mobile]').disabled"
        )
        .await,
        true
    );
    eval(&h, "(() => { const b=document.querySelector('[data-call-saved-number=mobile]'); b.disabled=false; b.click(); b.disabled=true; return true; })()").await;
    assert_eq!(
        snapshot(&h).await["host"]["count"],
        initial["host"]["count"]
    );
    assert_eq!(
        snapshot(&h).await["host"]["destination"],
        initial["host"]["destination"]
    );

    click(&h, "#call-workspace-reject-edits").await;
    choose_field(&h, "number-target", 1).await;
    let rejected = snapshot(&h).await;
    assert_eq!(
        rejected["fields"]["number-target"]["value"],
        "contact-phone"
    );
    assert_eq!(rejected["host"]["numberTarget"], "contact-phone");
    assert_eq!(rejected["host"]["writes"], "0");
    click(&h, "#call-workspace-reject-edits").await;
    choose_field(&h, "number-target", 1).await;
    let alternate = snapshot(&h).await;
    assert_eq!(
        alternate["fields"]["number-target"]["value"],
        "contact-mobile"
    );
    assert_eq!(alternate["host"]["numberTarget"], "contact-mobile");
    assert_eq!(alternate["savedNumber"], "+1 (415) 555-0199");
    action(&h, "save-number").await;
    assert!(
        snapshot(&h).await["host"]["last"]
            .as_str()
            .unwrap()
            .contains("field_id: \"contact-mobile\"")
    );
    click(&h, "#call-workspace-reject").await;
    assert_eq!(snapshot(&h).await["savedNumber"], alternate["savedNumber"]);
    action(&h, "save-number").await;
    click(&h, "#call-workspace-accept").await;
    assert_eq!(snapshot(&h).await["savedNumber"], "+1 (415) 555-0142");
    let written = snapshot(&h).await;
    assert_eq!(
        eval(
            &h,
            "document.querySelector('[data-call-saved-number=mobile]').disabled"
        )
        .await,
        true
    );
    eval(&h, "(() => { const b=document.querySelector('[data-call-saved-number=mobile]'); b.disabled=false; b.click(); b.disabled=true; return true; })()").await;
    assert_eq!(
        snapshot(&h).await["host"]["count"],
        written["host"]["count"],
        "Writing the mobile field must not grant permission to call it"
    );
    choose_field(&h, "number-target", 0).await;
    assert_eq!(snapshot(&h).await["savedNumber"], initial["savedNumber"]);
    assert_eq!(snapshot(&h).await["host"]["writes"], "1");

    // The saved mobile value retains its calling restriction, including when
    // it now matches the primary number. Choose a separate callable number.
    assert_eq!(snapshot(&h).await["actions"]["dial"]["disabled"], true);
    click(&h, "[data-call-saved-number=alternate]").await;
    for refusal in ["agent-not-ready", "agent-not-available", "not-submitted"] {
        action(&h, "dial").await;
        click(&h, &format!("#call-workspace-{refusal}")).await;
        let refused = snapshot(&h).await;
        assert_eq!(refused["host"]["attempt"], refusal);
        assert_eq!(refused["actions"]["dial"]["disabled"], false);
        assert_ne!(refused["status"], initial["status"]);
    }
    action(&h, "dial").await;
    click(&h, "#call-workspace-uncertain").await;
    assert_eq!(snapshot(&h).await["actions"]["dial"]["disabled"], true);
    click(&h, "#call-workspace-finished").await;
    // Unknown talk time after a finished call is "Not confirmed", never zero.
    assert!(
        snapshot(&h).await["talk"]
            .as_str()
            .unwrap()
            .contains("Not confirmed")
    );
    click(&h, "#call-workspace-talk-zero").await;
    let zero = snapshot(&h).await;
    assert_eq!(zero["host"]["talkSeconds"], "0");
    assert!(zero["talk"].as_str().unwrap().contains("00:00"));
    // Break and revert the readout: the visible-duration oracle must catch
    // an unknown/zero regression while the authoritative host still says zero.
    eval(&h, "(() => { const e=document.querySelector('[data-call-talk-time]'); const text=document.createTreeWalker(e,NodeFilter.SHOW_TEXT).nextNode(); if(!text) throw new Error('talk readout Text node missing'); e.__probeText=text; e.__probeOriginal=text.data; text.data='Provider talk time: Unknown'; return text.isConnected; })()").await;
    assert!(
        !snapshot(&h).await["talk"]
            .as_str()
            .unwrap()
            .contains("00:00")
    );
    assert_eq!(snapshot(&h).await["host"]["talkSeconds"], "0");
    assert_eq!(eval(&h, "(() => { const e=document.querySelector('[data-call-talk-time]'); const text=e.__probeText; text.data=e.__probeOriginal; delete e.__probeText; delete e.__probeOriginal; return text.isConnected && text.parentElement===e; })()").await, true);
    assert!(
        snapshot(&h).await["talk"]
            .as_str()
            .unwrap()
            .contains("00:00")
    );
    assert_eq!(zero["host"]["durationMinutes"], "");
    type_into(&h, "duration", "7").await;
    click(&h, "#call-workspace-talk-125").await;
    let measured = snapshot(&h).await;
    assert_eq!(measured["host"]["talkSeconds"], "125");
    assert!(measured["talk"].as_str().unwrap().contains("02:05"));
    assert_eq!(measured["host"]["durationMinutes"], "7");
    choose_outcome(&h, 1).await;
    action(&h, "save-wrap-up").await;
    assert!(
        snapshot(&h).await["host"]["last"]
            .as_str()
            .unwrap()
            .contains("duration_minutes: Some(7)")
    );
    assert_no_browser_errors(&h, "office targets, typed refusals and provider duration").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn workspace_guidance_refresh_requires_completion_and_current_context() {
    let h = harness_at("/components/client-call-workspace").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, ROOT).await;
    click(&h, "#call-workspace-office").await;
    let original = snapshot(&h).await;
    assert_eq!(original["guidanceState"], "ready");
    assert_eq!(original["beats"].as_array().unwrap().len(), 2);
    assert_eq!(original["beats"][0]["id"], "opening");
    assert_eq!(original["beats"][1]["id"], "next-step");
    assert_eq!(original["noFileDetails"], 1);
    click(&h, "#call-workspace-preparing").await;
    let preparing = snapshot(&h).await;
    assert_eq!(preparing["guidanceState"], "preparing");
    assert!(preparing["guidance"].as_str().unwrap().contains("12"));
    assert!(preparing["guidance"].as_str().unwrap().contains("75"));
    assert_eq!(preparing["beats"], json!([]));
    assert_eq!(preparing["host"]["beats"], original["host"]["beats"]);
    click(&h, "#call-workspace-script-failed").await;
    assert_eq!(snapshot(&h).await["guidanceState"], "failed");
    action(&h, "regenerate").await;
    let busy = snapshot(&h).await;
    assert_eq!(busy["host"]["regenerationState"], "busy");
    assert_eq!(busy["actions"]["regenerate"]["disabled"], true);
    eval(&h, "document.querySelector('[data-call-action=regenerate]').dispatchEvent(new MouseEvent('click',{bubbles:true})); true").await;
    assert_eq!(snapshot(&h).await["host"]["count"], busy["host"]["count"]);
    // A completion before acceptance must leave the request outstanding.
    click(&h, "#call-workspace-script-complete").await;
    assert_eq!(snapshot(&h).await["host"]["regenerationState"], "busy");
    // Dial and refresh have separate host operations; accepting dial cannot
    // silently acknowledge or discard the refresh.
    action(&h, "dial").await;
    click(&h, "#call-workspace-accept").await;
    let dialed = snapshot(&h).await;
    assert_eq!(dialed["host"]["attempt"], "agent-ringing");
    assert_eq!(dialed["host"]["regenerationState"], "busy");
    assert_eq!(dialed["host"]["regenerationPending"], "true");
    click(&h, "#call-workspace-accept").await;
    let accepted = snapshot(&h).await;
    assert_eq!(accepted["host"]["regenerationState"], "accepted");
    assert_eq!(accepted["guidanceState"], "preparing");
    assert_eq!(accepted["host"]["beats"], original["host"]["beats"]);
    assert_eq!(accepted["actions"]["regenerate"]["disabled"], true);
    eval(&h, "document.querySelector('[data-call-action=regenerate]').dispatchEvent(new MouseEvent('click',{bubbles:true})); true").await;
    assert_eq!(
        snapshot(&h).await["host"]["count"],
        accepted["host"]["count"]
    );
    click(&h, "#call-workspace-reject").await;
    let failed = snapshot(&h).await;
    assert_eq!(failed["host"]["regenerationState"], "failed");
    assert_eq!(failed["beats"], original["beats"]);
    assert_eq!(failed["actions"]["regenerate"]["disabled"], false);
    click(&h, "#call-workspace-regeneration-conflict").await;
    let conflict = snapshot(&h).await;
    assert_eq!(conflict["actions"]["regenerate"]["disabled"], true);
    eval(&h, "document.querySelector('[data-call-action=regenerate]').dispatchEvent(new MouseEvent('click',{bubbles:true})); true").await;
    assert_eq!(
        snapshot(&h).await["host"]["count"],
        conflict["host"]["count"]
    );
    click(&h, "#call-workspace-regeneration-retry").await;
    action(&h, "regenerate").await;
    click(&h, "#call-workspace-accept").await;
    click(&h, "#call-workspace-script-complete").await;
    let complete = snapshot(&h).await;
    assert_eq!(complete["host"]["regenerationState"], "succeeded");
    assert_eq!(complete["guidanceState"], "ready");
    assert_ne!(complete["beats"][0]["text"], original["beats"][0]["text"]);
    assert_eq!(complete["host"]["regenerationPending"], "false");
    action(&h, "regenerate").await;
    click(&h, "#call-workspace-accept").await;
    click(&h, "#call-workspace-switch").await;
    let switched = snapshot(&h).await;
    click(&h, "#call-workspace-script-complete").await;
    let stale = snapshot(&h).await;
    assert_eq!(stale["host"]["context"], switched["host"]["context"]);
    assert_eq!(stale["host"]["beats"], switched["host"]["beats"]);
    assert_eq!(stale["host"]["regenerationState"], "");
    assert_eq!(stale["host"]["regenerationPending"], "false");
    assert_no_browser_errors(&h, "script refresh lifecycle and stale completion").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn workspace_rich_script_has_responsive_and_accessible_release_evidence() {
    let h = harness_at("/components/client-call-workspace").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, ROOT).await;
    click(&h, "#call-workspace-office").await;
    for width in [1280, 375] {
        h.set_viewport(ViewportSize::new(width, 2400))
            .await
            .unwrap();
        common::prepare_region_capture(&h, ROOT, ViewportSize::new(width, 2400)).await;
        let state = snapshot(&h).await;
        assert_eq!(state["guidanceState"], "ready");
        assert_eq!(state["beats"].as_array().unwrap().len(), 2);
        assert_eq!(state["noFileDetails"], 1);
        let geometry = eval(&h, r#"(() => {
            const r=document.querySelector('#client-call-workspace-demo'), b=r.getBoundingClientRect();
            return {overflow:r.scrollWidth>r.clientWidth+1,
              textFits:[...r.querySelectorAll('[data-call-beat], [data-call-no-file-detail], [data-call-number-blocked]')].every(e=>e.scrollWidth<=e.clientWidth+1),
              controlsFit:[...r.querySelectorAll('input,select,textarea,button')].filter(e=>e.getClientRects().length).every(e=>{
                const c=e.getBoundingClientRect(); return c.left>=b.left&&c.right<=b.right+1&&c.width>0;})};
        })()"#).await;
        assert_eq!(
            geometry,
            json!({"overflow":false,"textFits":true,"controlsFit":true})
        );
        let base = ldui_audit::from_ui_tokens(common::body_font_family(&h).await);
        let mut shadows = base.shadows.clone();
        shadows.extend([
            ldui_audit::ShadowSpec::new(0.0, 6.0, 12.0, 0.15).with_spread(-2.0),
            ldui_audit::ShadowSpec::new(0.0, 3.0, 6.0, 0.10).with_spread(-2.0),
        ]);
        let mut ramp = base.type_ramp.clone();
        ramp.push(18.0);
        let profile = base.shadows(shadows).type_ramp(ramp);
        let report = ldui_audit::audit_page(
            &h,
            &profile,
            &ldui_audit::SweepOptions {
                mount_selector: ROOT.into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        common::assert_not_truncated(&report, "office call script");
        println!("Office call script {width}px audit: {report:#?}");
        ldui_audit::verify("office call script", &report, &[]).unwrap();
        std::fs::write(
            format!("target/client-call-workspace-office-{width}.png"),
            h.screenshot_bytes().await.unwrap(),
        )
        .unwrap();
    }
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js").unwrap();
    axe.run(h.page()).await.unwrap();
    let violations = eval(&h, r#"(async () => {const r=await axe.run(document.querySelector('#client-call-workspace-demo'),{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}});return r.violations.filter(v=>['serious','critical'].includes(v.impact));})()"#).await;
    assert_eq!(
        violations,
        json!([]),
        "office script accessibility: {violations}"
    );
    assert_no_browser_errors(&h, "rich office script responsive release evidence").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn workspace_dispatch_and_contact_updates_require_host_evidence() {
    let h = harness_at("/components/client-call-workspace").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, ROOT).await;
    let initial = snapshot(&h).await;
    assert_eq!(initial["host"]["count"], "0");
    assert_eq!(initial["fields"]["outcome"]["value"], "");
    assert_eq!(initial["actions"]["save-number"]["disabled"], true);
    click(&h, "[data-call-saved-number=alternate]").await;
    let selected = snapshot(&h).await;
    assert_eq!(
        selected["fields"]["destination"]["value"],
        "+1 (415) 555-0186"
    );
    assert_eq!(selected["savedNumber"], initial["savedNumber"]);
    assert_eq!(selected["host"]["writes"], "0");
    action(&h, "save-number").await;
    let saving = snapshot(&h).await;
    assert_eq!(saving["host"]["pending"], "true");
    assert_eq!(saving["actions"]["dial"]["disabled"], true);
    assert_eq!(saving["savedNumber"], initial["savedNumber"]);
    assert_eq!(
        eval(
            &h,
            "document.querySelector('#call-workspace-finished').disabled"
        )
        .await,
        true
    );
    // Adversarial DOM bypass proves the callback also checks current pending state.
    eval(&h, "document.querySelector('[data-call-action=save-number]').dispatchEvent(new MouseEvent('click',{bubbles:true})); true").await;
    assert_eq!(snapshot(&h).await["host"]["count"], saving["host"]["count"]);
    click(&h, "#call-workspace-reject").await;
    assert_eq!(snapshot(&h).await["savedNumber"], initial["savedNumber"]);
    action(&h, "save-number").await;
    click(&h, "#call-workspace-accept").await;
    assert_eq!(snapshot(&h).await["savedNumber"], "+1 (415) 555-0186");
    assert_eq!(snapshot(&h).await["host"]["writes"], "1");

    action(&h, "keypad").await;
    click(&h, "[data-call-digit='2']").await;
    assert!(
        snapshot(&h).await["fields"]["destination"]["value"]
            .as_str()
            .unwrap()
            .ends_with("1862")
    );
    action(&h, "backspace").await;
    action(&h, "dial").await;
    let submitting = snapshot(&h).await;
    assert_eq!(submitting["host"]["attempt"], "submitting");
    assert_eq!(submitting["pad"], false);
    assert_eq!(submitting["timer"], Value::Null);
    eval(&h, "document.querySelector('[data-call-action=dial]').dispatchEvent(new MouseEvent('click',{bubbles:true})); true").await;
    assert_eq!(
        snapshot(&h).await["host"]["count"],
        submitting["host"]["count"]
    );
    click(&h, "#call-workspace-accept").await;
    let ringing = snapshot(&h).await;
    assert_eq!(ringing["host"]["attempt"], "agent-ringing");
    assert!(
        ringing["status"]
            .as_str()
            .unwrap()
            .contains("Client connection is not yet confirmed")
    );
    assert_eq!(ringing["timer"], Value::Null);
    assert_eq!(ringing["actions"]["dial"]["disabled"], true);
    action(&h, "dismiss").await;
    wait_for_selector(&h, "[data-testid=call-workspace-host][data-closed=true]").await;
    click(&h, "#call-workspace-reopen").await;
    assert_eq!(snapshot(&h).await["host"]["attempt"], "agent-ringing");
    click(&h, "#call-workspace-uncertain").await;
    let uncertain = snapshot(&h).await;
    assert_eq!(uncertain["actions"]["dial"]["disabled"], true);
    assert_eq!(uncertain["actions"]["save-wrap-up"]["disabled"], true);
    assert!(
        uncertain["status"]
            .as_str()
            .unwrap()
            .contains("A call may have been placed")
    );
    // Negative control: the disabled-state oracle catches an injected unsafe retry.
    eval(
        &h,
        "document.querySelector('[data-call-action=dial]').disabled=false; true",
    )
    .await;
    assert_ne!(
        snapshot(&h).await["actions"]["dial"]["disabled"],
        true,
        "negative control must be observed"
    );
    eval(
        &h,
        "document.querySelector('[data-call-action=dial]').disabled=true; true",
    )
    .await;
    assert_eq!(snapshot(&h).await["actions"]["dial"]["disabled"], true);

    click(&h, "#call-workspace-reset").await;
    action(&h, "dial").await;
    click(&h, "#call-workspace-switch").await;
    let switched = snapshot(&h).await;
    click(&h, "#call-workspace-accept").await;
    let stale = snapshot(&h).await;
    assert_eq!(stale["host"]["attempt"], "ready");
    assert_eq!(stale["host"]["context"], switched["host"]["context"]);
    click(&h, "#call-workspace-block").await;
    assert_eq!(snapshot(&h).await["actions"]["dial"]["disabled"], true);
    assert_no_browser_errors(&h, "workspace dispatch and contact writes").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn workspace_wrap_up_keyboard_rejection_and_responsive_evidence() {
    let h = harness_at("/components/client-call-workspace").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, ROOT).await;
    type_into(&h, "notes", "Client received the summary.").await;
    assert_eq!(
        snapshot(&h).await["host"]["notes"],
        "Client received the summary."
    );
    click(&h, "#call-workspace-reject-edits").await;
    type_into(&h, "notes", "X").await;
    assert_eq!(
        snapshot(&h).await["fields"]["notes"]["value"],
        "Client received the summary."
    );
    choose_outcome(&h, 1).await;
    assert_eq!(snapshot(&h).await["fields"]["outcome"]["value"], "");
    click(&h, "#call-workspace-reject-edits").await;
    choose_outcome(&h, 5).await;
    let callback = snapshot(&h).await;
    assert_eq!(
        callback["fields"]["outcome"]["value"],
        "requested-call-back"
    );
    assert_eq!(callback["host"]["outcome"], "requested-call-back");
    assert_eq!(eval(&h, "document.querySelector('[data-call-field=outcome]').required && document.querySelector('[data-call-field=follow-up]').required").await, true);
    assert_eq!(callback["actions"]["save-wrap-up"]["disabled"], true);
    click(&h, "#call-workspace-finished").await;
    assert_eq!(
        snapshot(&h).await["actions"]["save-wrap-up"]["disabled"],
        true
    );
    type_into(
        &h,
        "follow-up",
        "Consultant to call tomorrow after 2 pm Pacific.",
    )
    .await;
    assert_eq!(
        snapshot(&h).await["actions"]["save-wrap-up"]["disabled"],
        false
    );
    action(&h, "save-wrap-up").await;
    let pending = snapshot(&h).await;
    assert!(
        pending["host"]["last"]
            .as_str()
            .unwrap()
            .contains("duration_minutes: None")
    );
    assert_eq!(pending["fields"]["notes"]["disabled"], true);
    assert_eq!(pending["host"]["saved"], "false");
    assert_eq!(pending["host"]["writes"], "0");
    // Programmatic events are only an adversarial test, not keyboard evidence.
    eval(&h, "const n=document.querySelector('[data-call-field=notes]'); n.value='corruption'; n.dispatchEvent(new Event('input',{bubbles:true})); true").await;
    assert_eq!(
        snapshot(&h).await["fields"]["notes"]["value"],
        "Client received the summary."
    );
    click(&h, "#call-workspace-reject").await;
    assert_eq!(
        snapshot(&h).await["host"]["notes"],
        "Client received the summary."
    );
    action(&h, "save-wrap-up").await;
    click(&h, "#call-workspace-accept").await;
    assert_eq!(snapshot(&h).await["host"]["saved"], "true");
    assert_eq!(snapshot(&h).await["host"]["writes"], "1");

    click(&h, "#call-workspace-reset").await;
    click(&h, "#call-workspace-connected").await;
    wait_for_selector(&h, "[data-softphone-action=hold]").await;
    click(&h, "[data-softphone-action=hold]").await;
    assert_eq!(
        eval(
            &h,
            "document.querySelector('[data-softphone-action=hold]').getAttribute('aria-pressed')"
        )
        .await,
        "false"
    );
    assert_eq!(
        eval(
            &h,
            "document.querySelector('[data-softphone-action=end-call]').disabled"
        )
        .await,
        false
    );
    click(&h, "#call-workspace-accept").await;
    assert_eq!(
        eval(
            &h,
            "document.querySelector('[data-softphone-action=hold]').getAttribute('aria-pressed')"
        )
        .await,
        "true"
    );
    assert_eq!(snapshot(&h).await["timer"], "02:05");
    click(&h, "[data-softphone-action=end-call]").await;
    click(&h, "#call-workspace-accept").await;
    assert_eq!(
        eval(
            &h,
            "document.querySelector('[data-softphone-action=call]') === null"
        )
        .await,
        true
    );

    click(&h, "#call-workspace-reset").await;
    for width in [1280, 375] {
        h.set_viewport(ViewportSize::new(width, 1800))
            .await
            .unwrap();
        common::prepare_region_capture(&h, ROOT, ViewportSize::new(width, 1800)).await;
        let geometry = eval(&h, r#"(() => {
            const r=document.querySelector('#client-call-workspace-demo'), b=r.getBoundingClientRect();
            return {overflow:r.scrollWidth>r.clientWidth+1,width:b.width,
              controls:[...r.querySelectorAll('input,select,textarea,button')].filter(e=>e.getClientRects().length).map(e=>{
                const c=e.getBoundingClientRect(); return {fits:c.left>=b.left&&c.right<=b.right+1, width:c.width,
                  name:(e.getAttribute('aria-label')||document.querySelector(`label[for="${e.id}"]`)?.textContent||e.textContent).trim()};})};
        })()"#).await;
        assert_eq!(geometry["overflow"], false, "{geometry}");
        assert!(
            geometry["width"].as_f64().unwrap() <= f64::from(width),
            "{geometry}"
        );
        assert!(
            geometry["controls"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["fits"] == true
                    && c["width"].as_f64().unwrap() > 0.0
                    && !c["name"].as_str().unwrap().is_empty()),
            "{geometry}"
        );
        let base = ldui_audit::from_ui_tokens(common::body_font_family(&h).await);
        let mut shadows = base.shadows.clone();
        // Exact demo/input.css Enhanced Button Push Effect, shared with the
        // catalog audit profile. No other shadow values are accepted here.
        shadows.extend([
            ldui_audit::ShadowSpec::new(0.0, 6.0, 12.0, 0.15).with_spread(-2.0),
            ldui_audit::ShadowSpec::new(0.0, 3.0, 6.0, 0.10).with_spread(-2.0),
        ]);
        let mut ramp = base.type_ramp.clone();
        ramp.push(18.0); // Existing daisyUI text-lg web step.
        let profile = base.shadows(shadows).type_ramp(ramp);
        let options = ldui_audit::SweepOptions {
            mount_selector: ROOT.into(),
            ..Default::default()
        };
        let report = ldui_audit::audit_page(&h, &profile, &options)
            .await
            .unwrap();
        common::assert_not_truncated(&report, "client call workspace");
        println!("Workspace {width}px audit: {report:#?}");
        ldui_audit::verify("client call workspace", &report, &[]).unwrap();
        std::fs::write(
            format!("target/client-call-workspace-{width}.png"),
            h.screenshot_bytes().await.unwrap(),
        )
        .unwrap();
    }
    // A narrow panel on a desktop viewport must use its own available width.
    h.set_viewport(ViewportSize::new(1280, 1800)).await.unwrap();
    eval(
        &h,
        "document.querySelector('#client-call-workspace-demo').style.width='375px'; true",
    )
    .await;
    let panel = eval(&h, "(() => {const r=document.querySelector('#client-call-workspace-demo');return {columns:getComputedStyle(r.querySelector(':scope>.grid')).gridTemplateColumns.split(/\\s+/).length,overflow:r.scrollWidth>r.clientWidth+1};})()").await;
    assert_eq!(
        panel,
        json!({"columns":1,"overflow":false}),
        "desktop drawer: {panel}"
    );
    // Inject the original viewport-driven two-column error and catch it.
    eval(&h, "document.querySelector('#client-call-workspace-demo>.grid').style.gridTemplateColumns='repeat(2,minmax(0,1fr))'; true").await;
    assert_eq!(eval(&h, "getComputedStyle(document.querySelector('#client-call-workspace-demo>.grid')).gridTemplateColumns.split(/\\s+/).length").await, 2);
    eval(&h, "document.querySelector('#client-call-workspace-demo>.grid').style.gridTemplateColumns=''; true").await;
    assert_eq!(eval(&h, "getComputedStyle(document.querySelector('#client-call-workspace-demo>.grid')).gridTemplateColumns.split(/\\s+/).length").await, 1);
    common::prepare_region_capture(&h, ROOT, ViewportSize::new(1280, 1800)).await;
    std::fs::write(
        "target/client-call-workspace-desktop-panel.png",
        h.screenshot_bytes().await.unwrap(),
    )
    .unwrap();
    // Long unbroken identity must fit the compact surface, rather than be clipped.
    click(&h, "#call-workspace-switch").await;
    let fits = eval(&h, "(() => { const e=document.querySelector('#client-call-workspace-demo header h2');return e.scrollWidth<=e.clientWidth+1; })()").await;
    assert_eq!(fits, true);
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js").unwrap();
    axe.run(h.page()).await.unwrap();
    let violations = eval(&h, r#"(async () => {const r=await axe.run(document.querySelector('#client-call-workspace-demo'),{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}});return r.violations.filter(v=>['serious','critical'].includes(v.impact));})()"#).await;
    assert_eq!(
        violations,
        json!([]),
        "workspace accessibility: {violations}"
    );
    assert_no_browser_errors(&h, "workspace wrap-up, managed controls and compact layout").await;
}

/// ldui-eq1e (Office op-flpq1): the COMPACT layout in a side-rail width says
/// each thing once -- one header row, one status line, ONE visible lock line
/// while the numbers are locked (production rendered the lock sentence six
/// times), "Call again" after a finished call dispatching StartAnotherCall,
/// three recent calls behind a toggle and the script collapsed.
async fn compact_snapshot(h: &Harness) -> Value {
    eval(h, r#"(() => {
        const root = document.querySelector('#client-call-workspace-demo');
        const host = document.querySelector('[data-testid=call-workspace-host]');
        const shown = e => e.checkVisibility({visibilityProperty:true}) && !e.closest('.sr-only,[aria-hidden=true]');
        const leaf = [...root.querySelectorAll('p,span,li,h2,h3,summary,label,button')].filter(shown);
        const said = text => leaf.filter(e => e.textContent.trim() === text).length;
        const radios = [...root.querySelectorAll('input[type=radio][data-call-saved-number]')];
        const call = root.querySelector('[data-call-action=dial],[data-call-action=start-another-call]');
        const h2 = root.querySelector('h2');
        const toggle = root.querySelector('[data-call-history-toggle]');
        return {host:{...host.dataset}, layout:root.dataset.callLayout,
          name:h2.textContent.trim(), nameTitle:h2.title, nameOneLine:h2.getBoundingClientRect().height <= 32,
          regionLabelShown:said('Client call workspace'),
          status:shown(root.querySelector('[data-call-status]')) ? root.querySelector('[data-call-status-line]').textContent.trim() : null,
          uncertainHint:shown(root.querySelector('[data-call-uncertain-hint]')),
          radios:radios.map(r => ({id:r.dataset.callSavedNumber, checked:r.checked, disabled:r.disabled,
            described:(r.getAttribute('aria-describedby')||'').split(/\s+/).map(id => document.getElementById(id)?.textContent.trim()).filter(Boolean)})),
          blockedLines:[...root.querySelectorAll('[data-call-number-blocked]')].filter(shown).map(e => e.textContent.trim()),
          lockLines:said('Numbers lock during a call.'),
          afterCallLines:said('Press Call again to choose a number.'),
          fullLockSentence:leaf.filter(e => e.textContent.includes('cannot change while a call is in progress')).length,
          call:{action:call.dataset.callAction, text:[...call.childNodes].filter(n => !(n.nodeType===1 && n.matches('[data-button-disabled-reason]'))).map(n => n.textContent).join('').trim(), disabled:call.disabled,
            description:(call.getAttribute('aria-describedby')||'').split(/\s+/).map(id => document.getElementById(id)?.textContent.trim()).filter(Boolean).join(' ')},
          callLine:shown(root.querySelector('[data-call-disabled-reason=dial]')) ? root.querySelector('[data-call-disabled-reason=dial]').textContent.trim() : null,
          launchHint:root.querySelector('[data-call-launch-hint]')?.textContent.trim() ?? null,
          historyRows:root.querySelectorAll('[data-call-history-entry]').length,
          toggle:toggle ? {text:toggle.textContent.trim(), expanded:toggle.getAttribute('aria-expanded')} : null,
          scriptOpen:root.querySelector('[data-call-script-disclosure]')?.open ?? null,
          scriptSummary:root.querySelector('[data-call-script-disclosure] summary')?.textContent.trim() ?? null,
          header:(() => {
            const a=root.querySelector('[data-call-header-actions]'), x=root.querySelector('[data-call-action=dismiss]');
            if (!a) return null;
            const ar=a.getBoundingClientRect(), xr=x.getBoundingClientRect(), nr=h2.getBoundingClientRect();
            return {text:a.textContent.trim(), beforeClose:ar.right<=xr.left+0.5, afterName:ar.left>=nr.right-0.5,
              sameRow:Math.abs((ar.top+ar.bottom)/2-(xr.top+xr.bottom)/2)<=2, nameTruncatesFirst:ar.width>0 && a.scrollWidth<=a.clientWidth+1};
          })(),
          sections:[...root.querySelector('[data-call-body=compact]').children].filter(shown).map(e =>
            e.matches('[data-call-status]') ? 'status' : e.matches('[data-call-destination]') ? 'numbers'
            : e.matches('[data-call-history]') ? 'history' : e.matches('[data-call-script-disclosure]') ? 'script'
            : e.matches('[data-call-wrap-up]') ? 'wrap-up' : e.tagName.toLowerCase())};
    })()"#).await
}

/// The compact panel's declared depth vocabulary, beyond `ui_tokens`'
/// elevation levels: the demo's button press shadow (the same two layers the
/// office script test declares) and daisyUI 5's resting `.radio` inset, which
/// is the `.checkbox` rule `tests/style_audit_smoke.rs` already declares
/// (`0 1px oklch(0% 0 0 / calc(var(--depth)*.1)) inset`).
fn compact_profile(font_family: String) -> ldui_audit::StyleProfile {
    let base = ldui_audit::from_ui_tokens(font_family);
    let mut shadows = base.shadows.clone();
    shadows.extend([
        ldui_audit::ShadowSpec::new(0.0, 6.0, 12.0, 0.15).with_spread(-2.0),
        ldui_audit::ShadowSpec::new(0.0, 3.0, 6.0, 0.10).with_spread(-2.0),
        ldui_audit::ShadowSpec::new(0.0, 1.0, 0.0, 0.10).with_inset(),
    ]);
    let mut ramp = base.type_ramp.clone();
    ramp.push(18.0);
    base.shadows(shadows).type_ramp(ramp)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn workspace_compact_launcher_says_each_thing_once() {
    let h = harness_at("/components/client-call-workspace").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, ROOT).await;
    click(&h, "#call-workspace-compact").await;
    wait_for_selector(&h, "#client-call-workspace-demo[data-call-layout=compact]").await;
    let rest = compact_snapshot(&h).await;
    assert_eq!(rest["layout"], "compact");
    assert_eq!(rest["name"], "Elena Martinez");
    assert_eq!(rest["nameTitle"], "Elena Martinez");
    assert_eq!(rest["nameOneLine"], true, "{rest}");
    assert_eq!(rest["regionLabelShown"], 0, "no visible region-label line");
    assert_eq!(rest["status"], Value::Null, "no status before any attempt");
    let radios = rest["radios"].as_array().unwrap();
    assert_eq!(
        radios
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["phone", "alternate", "mobile"]
    );
    assert_eq!(radios[0]["checked"], true);
    assert_eq!(radios[2]["disabled"], true);
    assert_eq!(
        radios[2]["described"],
        json!(["Mobile calling is restricted for this contact."]),
        "a blocked number is described by its own reason"
    );
    assert_eq!(
        rest["blockedLines"],
        json!(["Mobile calling is restricted for this contact."])
    );
    assert_eq!(rest["lockLines"], 0);
    assert_eq!(rest["call"]["action"], "dial");
    assert_eq!(rest["call"]["text"], "Place call");
    assert_eq!(rest["call"]["disabled"], false);
    assert_eq!(rest["callLine"], Value::Null);
    assert_eq!(rest["launchHint"], "Opens in the demo phone window.");
    assert_eq!(rest["historyRows"], 3);
    assert_eq!(
        rest["toggle"],
        json!({"text":"Show all (5)","expanded":"false"})
    );
    assert_eq!(rest["scriptOpen"], false);
    assert_eq!(rest["scriptSummary"], "Account review script");
    assert_eq!(
        rest["header"],
        json!({"text":"Open in phone system","beforeClose":true,"afterName":true,"sameRow":true,"nameTruncatesFirst":true}),
        "the host's header actions sit between the name and Close: {rest}"
    );
    assert_eq!(
        rest["sections"],
        json!(["numbers", "history", "script", "wrap-up"])
    );

    // A radio is a choice the host must adopt: a rejected one snaps back.
    click(&h, "#call-workspace-reject-edits").await;
    click(
        &h,
        "#client-call-workspace-demo [data-call-saved-number=alternate]",
    )
    .await;
    let rejected = compact_snapshot(&h).await;
    assert_eq!(rejected["host"]["destination"], "+1 (415) 555-0142");
    assert_eq!(rejected["radios"][0]["checked"], true);
    assert_eq!(rejected["radios"][1]["checked"], false);
    click(&h, "#call-workspace-reject-edits").await;
    click(
        &h,
        "#client-call-workspace-demo [data-call-saved-number=alternate]",
    )
    .await;
    let chosen = compact_snapshot(&h).await;
    assert_eq!(chosen["host"]["destination"], "+1 (415) 555-0186");
    assert_eq!(chosen["radios"][1]["checked"], true);
    assert_eq!(chosen["radios"][0]["checked"], false);

    // During a call: ONE lock line, named by every radio and the button.
    click(&h, "#client-call-workspace-demo [data-call-action=dial]").await;
    for (step, button) in [
        ("submitting", None),
        ("agent-ringing", Some("#call-workspace-accept")),
    ] {
        if let Some(button) = button {
            click(&h, button).await;
        }
        let locked = compact_snapshot(&h).await;
        assert_eq!(locked["host"]["attempt"], step);
        assert_eq!(locked["lockLines"], 1, "{step}: {locked}");
        assert_eq!(locked["fullLockSentence"], 0, "{step}: {locked}");
        assert!(locked["status"].as_str().is_some(), "{step}: {locked}");
        for radio in locked["radios"].as_array().unwrap() {
            assert_eq!(radio["disabled"], true, "{step}");
            assert!(
                radio["described"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("Numbers lock during a call.")),
                "{step}: {radio}"
            );
        }
        assert_eq!(locked["call"]["disabled"], true);
        assert_eq!(locked["call"]["description"], "Numbers lock during a call.");
        assert_eq!(
            locked["callLine"],
            Value::Null,
            "the lock is not repeated under the button"
        );
    }

    // Finished: one status line, then Call again proposes a new attempt.
    click(&h, "#call-workspace-finished").await;
    click(&h, "#call-workspace-talk-125").await;
    let finished = compact_snapshot(&h).await;
    assert_eq!(
        finished["status"], "Call finished · Talk time: 02:05",
        "{finished}"
    );
    assert_eq!(finished["call"]["action"], "start-another-call");
    assert_eq!(finished["call"]["text"], "Call again");
    assert_eq!(finished["call"]["disabled"], false);
    assert_eq!(finished["lockLines"], 0, "the call is over: {finished}");
    assert_eq!(finished["afterCallLines"], 1, "{finished}");
    for radio in finished["radios"].as_array().unwrap() {
        assert_eq!(radio["disabled"], true);
        assert!(
            radio["described"]
                .as_array()
                .unwrap()
                .contains(&json!("Press Call again to choose a number.")),
            "{radio}"
        );
    }

    click(&h, "#client-call-workspace-demo [data-call-history-toggle]").await;
    click(
        &h,
        "#client-call-workspace-demo [data-call-script-disclosure] summary",
    )
    .await;
    let open = compact_snapshot(&h).await;
    assert_eq!(open["historyRows"], 5);
    assert_eq!(
        open["toggle"],
        json!({"text":"Show fewer","expanded":"true"})
    );
    assert_eq!(open["scriptOpen"], true);

    for width in [1280, 375] {
        h.set_viewport(ViewportSize::new(width, 2400))
            .await
            .unwrap();
        common::prepare_region_capture(&h, ROOT, ViewportSize::new(width, 2400)).await;
        let geometry = eval(&h, r#"(() => {
            const r=document.querySelector('#client-call-workspace-demo'), b=r.getBoundingClientRect();
            return {overflow:r.scrollWidth>r.clientWidth+1, rail:b.width<=384+1,
              controlsFit:[...r.querySelectorAll('input,select,textarea,button')].filter(e=>e.getClientRects().length).every(e=>{
                const c=e.getBoundingClientRect(); return c.left>=b.left&&c.right<=b.right+1&&c.width>0;})};
        })()"#).await;
        assert_eq!(
            geometry,
            json!({"overflow":false,"rail":true,"controlsFit":true})
        );
        let header = compact_snapshot(&h).await["header"].clone();
        assert_eq!(
            header,
            json!({"text":"Open in phone system","beforeClose":true,"afterName":true,"sameRow":true,"nameTruncatesFirst":true}),
            "{width}px header"
        );
        let profile = compact_profile(common::body_font_family(&h).await);
        let report = ldui_audit::audit_page(
            &h,
            &profile,
            &ldui_audit::SweepOptions {
                mount_selector: ROOT.into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        common::assert_not_truncated(&report, "compact call panel");
        println!("Compact call panel {width}px audit: {report:#?}");
        ldui_audit::verify("compact call panel", &report, &[]).unwrap();
        std::fs::write(
            format!("target/client-call-workspace-compact-{width}.png"),
            h.screenshot_bytes().await.unwrap(),
        )
        .unwrap();
    }
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js").unwrap();
    axe.run(h.page()).await.unwrap();
    let violations = eval(&h, r#"(async () => {const r=await axe.run(document.querySelector('#client-call-workspace-demo'),{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}});return r.violations.filter(v=>['serious','critical'].includes(v.impact));})()"#).await;
    assert_eq!(
        violations,
        json!([]),
        "compact call panel accessibility: {violations}"
    );

    click(
        &h,
        "#client-call-workspace-demo [data-call-action=start-another-call]",
    )
    .await;
    let again = compact_snapshot(&h).await;
    assert_eq!(again["host"]["last"], "StartAnotherCall");
    assert!(
        again["host"]["context"]
            .as_str()
            .unwrap()
            .starts_with("elena/again-"),
        "{again}"
    );
    assert_eq!(again["host"]["attempt"], "submitting");
    assert_eq!(again["host"]["destination"], "+1 (415) 555-0186");
    assert_eq!(again["lockLines"], 1);
    assert_no_browser_errors(&h, "compact launcher layout").await;
}

/// ldui-eq1e: Office's side panel passes no script and no call record (the
/// page shows both elsewhere), so the compact panel is the numbers, Call and
/// the recent calls -- and "Call again" still works with no record.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo host (cargo xtask test-softphone)"]
async fn workspace_compact_office_panel_is_numbers_call_and_recent_calls() {
    let h = harness_at("/components/client-call-workspace").await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, ROOT).await;
    click(&h, "#call-workspace-compact-office").await;
    wait_for_selector(&h, "#client-call-workspace-demo[data-call-layout=compact]").await;
    let rest = compact_snapshot(&h).await;
    assert_eq!(rest["sections"], json!(["numbers", "history"]), "{rest}");
    assert_eq!(rest["scriptOpen"], Value::Null);
    assert_eq!(rest["call"]["text"], "Place call");
    assert_eq!(rest["header"]["text"], "Open in phone system");

    click(&h, "#client-call-workspace-demo [data-call-action=dial]").await;
    click(&h, "#call-workspace-accept").await;
    click(&h, "#call-workspace-finished").await;
    let finished = compact_snapshot(&h).await;
    assert_eq!(
        finished["sections"],
        json!(["status", "numbers", "history"]),
        "{finished}"
    );
    assert_eq!(finished["lockLines"], 0);
    assert_eq!(finished["afterCallLines"], 1);
    assert_eq!(finished["call"]["text"], "Call again");
    assert_eq!(finished["call"]["disabled"], false);

    h.set_viewport(ViewportSize::new(375, 1600)).await.unwrap();
    common::prepare_region_capture(&h, ROOT, ViewportSize::new(375, 1600)).await;
    let profile = compact_profile(common::body_font_family(&h).await);
    let report = ldui_audit::audit_page(
        &h,
        &profile,
        &ldui_audit::SweepOptions {
            mount_selector: ROOT.into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    common::assert_not_truncated(&report, "compact office panel");
    println!("Compact office panel 375px audit: {report:#?}");
    ldui_audit::verify("compact office panel", &report, &[]).unwrap();
    std::fs::write(
        "target/client-call-workspace-compact-office-375.png",
        h.screenshot_bytes().await.unwrap(),
    )
    .unwrap();

    click(
        &h,
        "#client-call-workspace-demo [data-call-action=start-another-call]",
    )
    .await;
    let again = compact_snapshot(&h).await;
    assert_eq!(again["host"]["last"], "StartAnotherCall");
    assert_eq!(again["host"]["attempt"], "submitting");
    assert_eq!(again["sections"], json!(["status", "numbers", "history"]));
    assert_no_browser_errors(&h, "compact office panel").await;
}
