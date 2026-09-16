//! Real-browser proof for the KNOWLEDGE sources `patterns::AiChatWorkspace`
//! mixes into a turn (ldui-iilm.7): corpus ingest and reindex, the four typed
//! query modes, grounded vs assistant posture, the AI-memory recall receipt
//! and its per-lane attribution, the remember guardrail, and the curation gap
//! between an entry that is WRITTEN and an entry that is FINDABLE.
//!
//! Its own document, and its own lane, for two reasons:
//!
//! * The page mounts TWO workspaces — a healthy memory store and one behind
//!   `ChatWorkspaceFault::MemoryStoreOffline` — because "a receipt with four
//!   attribution lanes" is only evidence that a search ran if an unreachable
//!   store on the same document renders a typed honesty row and NO receipt.
//! * Neither instance is `scripted`, so both keep
//!   `KnowledgeSelection::default()`'s GROUNDED posture. The failure-ladder
//!   document cannot: a grounded prompt matching no seeded document has its
//!   script REPLACED by the localized not-found sentence, which silently
//!   turns every scripted assertion into a measurement of the substitution.
//!   Test 4 measures that substitution deliberately, which is exactly why it
//!   needs a document where it is not an accident.
//!
//! Two needles worth restating, both of which have already cost a lane:
//! `BackendCall`'s `Debug` prints `posture: Assistant` and `query_mode:
//! FullText` — the RUST VARIANT NAMES, not `as_id()`'s `"assistant"` /
//! `"full_text"` — and nothing here may be selected by document position.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

const PAGE: &str = "/ai-chat-fixture-knowledge";

/// The healthy workspace, which owns the document's debug oracle.
const KNOWLEDGE: &str = "knowledge";
/// Its negative control: the same seed behind `MemoryStoreOffline`.
const OFFLINE: &str = "offline";

/// `knowledge_rail::scope_value` for the two seeded folders. Both collapse to
/// `CorpusScope::as_id() == "folder"`, which is why the row hook carries the
/// path as well.
const INTAKE: &str = "folder:kb/intake";
const COURT: &str = "folder:kb/court";

/// A query that matches at least one seeded item in every one of the four
/// recall corpora (`memory::SEED_RECALL_QUERY`).
const SEED_RECALL_QUERY: &str = "court date reminder tone for client email drafts";

/// The exact sentence a grounded turn with no matching document answers with
/// (`AiChatWorkspaceTexts::grounded_not_found`, EN).
const NOT_FOUND: &str = "I could not find that in this folder.";

/// A prompt sharing fewer than two significant words with every seeded
/// document, so a grounded turn must report not-found.
const UNMATCHED: &str = "Zebra kites orbit purple mountains";

/// A prompt overlapping the continuance memo on several significant words,
/// so a grounded turn must report grounded with at least one citation.
const MATCHED: &str = "continuance request clerk scheduled";

/// The root selector for one workspace on this two-workspace document.
fn case_root(case: &str) -> String {
    format!("[data-ai-chat-case=\"{case}\"] [data-ai-chat-workspace]")
}

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate ai-chat knowledge fixture")
        .into_value()
        .expect("ai-chat knowledge expression returns JSON")
}

/// The backend's ordered call log, as `Debug` strings, published by the
/// fixture PAGE (`calls()` is an inherent test-mode method, deliberately not
/// on `ChatWorkspaceBackend`, so the composite could not publish it).
async fn backend_calls(h: &pixelproof_web::Harness) -> Vec<String> {
    let raw = eval_json(
        h,
        r#"(() => {
            const raw = window.__APP_DEBUG__ ? window.__APP_DEBUG__.state() : '{}';
            return JSON.parse(raw).ai_chat_calls ?? [];
        })()"#,
    )
    .await;
    raw.as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn count_calls(calls: &[String], prefix: &str) -> usize {
    calls.iter().filter(|c| c.starts_with(prefix)).count()
}

/// Everything the knowledge and evidence rails render, off ONE workspace
/// root. Every value comes from a stable `data-*` hook.
async fn rail_shape(h: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                if (!root) return null;
                const q = sel => root.querySelector(sel);
                const all = sel => Array.from(root.querySelectorAll(sel));
                return {{
                    corpora: all('[data-ai-chat-corpus]').map(el => ({{
                        scope: el.getAttribute('data-ai-chat-corpus'),
                        phase: el.getAttribute('data-ai-chat-ingest-phase'),
                        error: el.querySelector('[data-ai-chat-ingest-error]')
                            ?.textContent?.trim() ?? null,
                    }})),
                    scopeOptions: Array.from(
                        q('[data-ai-chat-corpus-scope]')?.options ?? []
                    ).map(o => o.value),
                    queryMode: q('[data-ai-chat-query-mode]')?.value ?? null,
                    posture: q('[data-ai-chat-posture]')?.value ?? null,
                    queryAttribution: q('[data-ai-chat-query-attribution]')
                        ?.getAttribute('data-ai-chat-query-attribution') ?? null,
                    queryAttributionText: q('[data-ai-chat-query-attribution]')
                        ?.textContent?.trim() ?? null,
                    grounding: q('[data-ai-chat-grounding]')
                        ?.getAttribute('data-ai-chat-grounding') ?? null,
                    citations: all('[data-ai-chat-citation]')
                        .map(e => e.getAttribute('data-ai-chat-citation')),
                    facts: all('[data-ai-chat-fact]').map(e => ({{
                        qualification: e.getAttribute('data-ai-chat-fact-qualification'),
                        text: e.textContent.trim(),
                    }})),
                    memoryEnabled: q('[data-ai-chat-memory-enabled]')?.checked ?? null,
                    captureEnabled: q('[data-ai-chat-capture-enabled]')?.checked ?? null,
                    receipt: q('[data-ai-chat-memory-receipt]')
                        ?.getAttribute('data-ai-chat-memory-receipt') ?? null,
                    search: q('[data-ai-chat-memory-search]')?.textContent?.trim() ?? null,
                    hits: all('[data-ai-chat-memory-hit]').map(e => ({{
                        attribution: e.getAttribute('data-ai-chat-memory-attribution'),
                        score: Number(e.getAttribute('data-ai-chat-memory-score')),
                        rank: Number(e.getAttribute('data-ai-chat-memory-rank')),
                        text: e.textContent.trim(),
                    }})),
                    noHits: q('[data-ai-chat-recall-no-hits]') !== null,
                    offline: q('[data-ai-chat-memory-offline]') !== null,
                    guardrail: q('[data-ai-chat-guardrail]')
                        ?.getAttribute('data-ai-chat-guardrail') ?? null,
                    rememberValue: q('[data-ai-chat-remember-input]')?.value ?? null,
                    rememberKind: q('[data-ai-chat-remember-kind]')?.value ?? null,
                    entries: all('[data-ai-chat-kb-entry]').map(e => ({{
                        id: e.getAttribute('data-ai-chat-kb-entry'),
                        kind: e.getAttribute('data-ai-chat-kb-kind'),
                        state: e.getAttribute('data-ai-chat-kb-state'),
                        awaiting: e.querySelector('[data-ai-chat-kb-awaiting-curation]') !== null,
                        text: e.textContent.trim(),
                    }})),
                    officeScopes: all('[data-ai-chat-kb-scope]')
                        .map(e => e.getAttribute('data-ai-chat-kb-scope')),
                    bubbles: all('[data-chat-role="assistant"] .chat-bubble')
                        .map(e => e.textContent.trim()),
                    chatState: q('[data-ai-chat-state]')
                        ?.getAttribute('data-ai-chat-state') ?? null,
                }};
            }})()"#
        ),
    )
    .await
}

/// The ingest phase one corpus row currently carries, or `""` when the row is
/// not rendered.
async fn phase_of(h: &pixelproof_web::Harness, root: &str, scope: &str) -> String {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const el = document.querySelector(
                    '{root} [data-ai-chat-corpus="{scope}"]'
                );
                return el ? el.getAttribute('data-ai-chat-ingest-phase') : '';
            }})()"#
        ),
    )
    .await
    .as_str()
    .unwrap_or_default()
    .to_owned()
}

/// Sample both seeded folders' phases every 50 ms until `scope` settles, and
/// return the observed sequences with consecutive duplicates collapsed.
///
/// Two scopes, one loop, on purpose: the negative control (the OTHER folder
/// never moves) is only meaningful if it was sampled during the same window
/// the first one was climbing, not read once afterwards.
async fn record_ladder(
    h: &pixelproof_web::Harness,
    root: &str,
    scope: &str,
    other: &str,
) -> (Vec<String>, Vec<String>) {
    let mut seen: Vec<String> = Vec::new();
    let mut other_seen: Vec<String> = Vec::new();
    let mut waited = 0;
    loop {
        let phase = phase_of(h, root, scope).await;
        if seen.last().map(String::as_str) != Some(phase.as_str()) {
            seen.push(phase.clone());
        }
        let neighbour = phase_of(h, root, other).await;
        if other_seen.last().map(String::as_str) != Some(neighbour.as_str()) {
            other_seen.push(neighbour);
        }
        // Terminal ONLY once a working phase has actually been observed. The
        // sampler starts right after the click, so its first read may still
        // be the pre-click `ready`; exiting on that would return a one-entry
        // ladder and report the reindex as instantaneous.
        let worked = seen
            .iter()
            .any(|p| p == "walking" || p == "indexing" || p == "clustering");
        if worked && (phase == "ready" || phase == "failed") {
            // Drop that leading `ready` for the same reason: it is the
            // harness's timing, not a rung. Every rung of the ladder itself
            // is still asserted, in order.
            if seen.first().map(String::as_str) == Some("ready") {
                seen.remove(0);
            }
            return (seen, other_seen);
        }
        assert!(
            waited < 15_000,
            "the ingest ladder never settled (saw {seen:?})"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        waited += 50;
    }
}

/// Set one `<select>` through a real change event.
async fn set_select(h: &pixelproof_web::Harness, selector: &str, value: &str) {
    let ok: bool = eval_json(
        h,
        &format!(
            r#"(() => {{
                const sel = document.querySelector('{selector}');
                if (!sel) return false;
                sel.value = '{value}';
                sel.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return sel.value === '{value}';
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(ok, "the select {selector} must accept {value}");
}

/// Set one text input or textarea through a real input event.
async fn set_text(h: &pixelproof_web::Harness, selector: &str, value: &str) {
    let ok: bool = eval_json(
        h,
        &format!(
            r#"(() => {{
                const el = document.querySelector('{selector}');
                if (!el) return false;
                el.focus();
                el.value = {value:?};
                el.dispatchEvent(new InputEvent('input', {{
                    bubbles: true, inputType: 'insertText'
                }}));
                return el.value === {value:?};
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(ok, "the field {selector} must accept its text");
}

/// Toggle one checkbox through a real change event, and report the value it
/// was set TO.
async fn set_checkbox(h: &pixelproof_web::Harness, selector: &str, checked: bool) {
    let ok: bool = eval_json(
        h,
        &format!(
            r#"(() => {{
                const el = document.querySelector('{selector}');
                if (!el) return false;
                el.checked = {checked};
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return el.checked === {checked};
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(ok, "the checkbox {selector} must accept {checked}");
}

async fn ready_at(h: &pixelproof_web::Harness, root: &str) {
    wait_for_selector(h, &format!("{root} [data-ai-chat-panel]")).await;
    wait_for_selector(h, &format!("{root} [data-ai-chat-knowledge-rail]")).await;
    wait_for_selector(h, &format!("{root} [data-ai-chat-corpus]")).await;
}

async fn wait_for_idle_at(h: &pixelproof_web::Harness, root: &str, budget_ms: u64) {
    let step = 200;
    let mut waited = 0;
    loop {
        let state = rail_shape(h, root).await["chatState"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if state == "idle" || state == "error" {
            return;
        }
        assert!(
            waited < budget_ms,
            "the panel never stopped working (last state {state:?})"
        );
        tokio::time::sleep(std::time::Duration::from_millis(step)).await;
        waited += step;
    }
}

/// Send one prompt and wait for the panel to settle AND the evidence rail to
/// publish a grounding verdict. The composite reads the turn record on its
/// own 200 ms tick, so the panel reaches idle a beat before the rail does.
async fn ask(h: &pixelproof_web::Harness, root: &str, prompt: &str) {
    set_text(h, &format!("{root} [data-ai-chat-composer]"), prompt).await;
    click(h, &format!("{root} [data-ai-chat-send]")).await;
    wait_for_idle_at(h, root, 30_000).await;
    let mut waited = 0;
    while waited < 5_000 {
        if !rail_shape(h, root).await["grounding"].is_null() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        waited += 200;
    }
}

/// Run one recall on `root` and wait for a NEW receipt (or the offline row).
///
/// Keyed on the receipt id changing, not on a receipt merely existing: the
/// second recall of a session would otherwise return the FIRST one's hits
/// the instant it was asked for, and every assertion after it would be
/// reading the previous search.
async fn recall(h: &pixelproof_web::Harness, root: &str, query: &str) -> Value {
    let before = rail_shape(h, root).await["receipt"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    set_text(h, &format!("{root} [data-ai-chat-recall-input]"), query).await;
    click(h, &format!("{root} [data-ai-chat-recall]")).await;
    let mut waited = 0;
    loop {
        let shape = rail_shape(h, root).await;
        let id = shape["receipt"].as_str().unwrap_or_default().to_owned();
        let settled = (!id.is_empty() && id != before) || shape["offline"] == json!(true);
        if settled || waited >= 6_000 {
            return shape;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        waited += 100;
    }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|x| x.as_str().unwrap_or_default().to_owned())
        .collect()
}

// ── 1 ───────────────────────────────────────────────────────────────────────

/// Every published corpus is listed with its own phase, and a reindex walks
/// the ladder one observable rung at a time.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn corpus_scopes_and_ingest_phases_advance_deterministically() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let root = case_root(KNOWLEDGE);
    ready_at(&h, &root).await;

    let shape = rail_shape(&h, &root).await;
    let scopes: Vec<String> = shape["corpora"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|c| c["scope"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(
        scopes,
        vec![INTAKE.to_owned(), COURT.to_owned(), "all".to_owned()],
        "every published corpus is listed, keyed by scope and PATH — \
         `CorpusScope::as_id` collapses both folders to \"folder\": {shape}"
    );
    let phases: Vec<String> = shape["corpora"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|c| c["phase"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(
        phases,
        vec!["ready".to_owned(); 3],
        "the seed starts indexed: {shape}"
    );
    // The error hook exists ONLY for a failed phase. If it were rendered
    // empty on every row, "is this corpus broken?" would be unanswerable.
    for corpus in shape["corpora"].as_array().unwrap_or(&vec![]) {
        assert_eq!(corpus["error"], json!(null), "{corpus}");
    }

    click(&h, &format!("{root} [data-ai-chat-reindex=\"{INTAKE}\"]")).await;
    let (ladder, neighbour) = record_ladder(&h, &root, INTAKE, COURT).await;
    assert_eq!(
        ladder,
        vec![
            "walking".to_owned(),
            "indexing".to_owned(),
            "clustering".to_owned(),
            "ready".to_owned(),
        ],
        "the ladder is walked one rung at a time, in order, and every rung is \
         observable — a composite that refreshed `knowledge()` immediately \
         after the reindex would skip `walking` entirely"
    );
    assert_eq!(
        neighbour,
        vec!["ready".to_owned()],
        "and the OTHER folder never moved while it climbed"
    );

    assert_no_browser_errors(&h, "corpus ingest ladder").await;
}

// ── 2 ───────────────────────────────────────────────────────────────────────

/// The reindex reaches the transport carrying the exact scope, and a second
/// one restarts the ladder rather than continuing the first.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn reindex_reaches_the_transport_and_restarts_the_phase_ladder() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let root = case_root(KNOWLEDGE);
    ready_at(&h, &root).await;

    click(&h, &format!("{root} [data-ai-chat-reindex=\"{INTAKE}\"]")).await;
    let (first, _) = record_ladder(&h, &root, INTAKE, COURT).await;
    assert!(first.contains(&"walking".to_owned()), "{first:?}");

    let calls = backend_calls(&h).await;
    // `BackendCall`'s Debug prints the Rust shape, not `as_id()`.
    assert!(
        calls.iter().any(|c| c == r#"Reindex(Folder("kb/intake"))"#),
        "the reindex carries the exact scope: {calls:?}"
    );
    assert!(
        !calls.iter().any(|c| c == r#"Reindex(Folder("kb/court"))"#),
        "and only that scope: {calls:?}"
    );

    // A second reindex of the SAME scope restarts at walking. An
    // implementation that only ever advanced would sit at `ready`.
    click(&h, &format!("{root} [data-ai-chat-reindex=\"{INTAKE}\"]")).await;
    let (second, neighbour) = record_ladder(&h, &root, INTAKE, COURT).await;
    assert_eq!(
        second,
        vec![
            "walking".to_owned(),
            "indexing".to_owned(),
            "clustering".to_owned(),
            "ready".to_owned(),
        ],
        "the ladder restarts from the bottom"
    );
    assert_eq!(
        neighbour,
        vec!["ready".to_owned()],
        "the untouched folder is still the negative control"
    );
    assert_eq!(
        count_calls(&backend_calls(&h).await, "Reindex(Folder(\"kb/intake\""),
        2,
        "two reindexes reached the transport, not one and a re-render"
    );

    assert_no_browser_errors(&h, "reindex transport").await;
}

// ── 3 ───────────────────────────────────────────────────────────────────────

/// Each of the four query modes is sent to the backend as its own typed value
/// and attributed in the evidence rail in its own words.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn query_modes_are_sent_as_typed_and_render_their_own_attribution() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let root = case_root(KNOWLEDGE);
    ready_at(&h, &root).await;

    let mut attributions: Vec<String> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for (id, debug_name) in [
        ("full_text", "FullText"),
        ("similarity", "Similarity"),
        ("llm", "Llm"),
        ("fused", "Fused"),
    ] {
        set_select(&h, &format!("{root} [data-ai-chat-query-mode]"), id).await;
        // A knowledge change reopens the session, so wait for the panel back.
        wait_for_selector(&h, &format!("{root} [data-ai-chat-panel]")).await;
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;

        let shape = rail_shape(&h, &root).await;
        assert_eq!(
            shape["queryAttribution"].as_str(),
            Some(id),
            "the evidence rail attributes the mode it was opened with: {shape}"
        );
        attributions.push(id.to_owned());
        texts.push(
            shape["queryAttributionText"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
        );

        let calls = backend_calls(&h).await;
        assert!(
            calls.iter().any(|c| c.starts_with("OpenSession")
                && c.contains(&format!("query_mode: {debug_name}"))),
            "the typed mode reached the transport ({debug_name}): {calls:?}"
        );
    }

    let mut unique = texts.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        4,
        "each mode reads differently — an attribution that said the same \
         thing for all four would attribute nothing: {texts:?}"
    );
    assert_eq!(attributions.len(), 4);

    assert_no_browser_errors(&h, "query modes").await;
}

// ── 4 ───────────────────────────────────────────────────────────────────────

/// A grounded turn that finds nothing says exactly that, and an assistant
/// turn on the SAME prompt never borrows the sentence.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn grounded_posture_answers_the_exact_absence_sentence() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let root = case_root(KNOWLEDGE);
    ready_at(&h, &root).await;

    let start = rail_shape(&h, &root).await;
    assert_eq!(
        start["posture"].as_str(),
        Some("grounded"),
        "this document keeps the default GROUNDED posture: {start}"
    );

    ask(&h, &root, UNMATCHED).await;
    let missed = rail_shape(&h, &root).await;
    assert_eq!(
        missed["grounding"].as_str(),
        Some("not_found"),
        "a grounded search that found nothing is its own verdict, not an \
         empty citation list: {missed}"
    );
    let bubbles = strings(&missed["bubbles"]);
    let answer = bubbles.last().cloned().unwrap_or_default();
    assert_eq!(
        answer, NOT_FOUND,
        "the WHOLE answer is the absence sentence — a `contains` check would \
         pass for an answer that guessed and then hedged: {bubbles:?}"
    );
    assert!(
        strings(&missed["citations"]).is_empty(),
        "and it cites nothing: {missed}"
    );

    ask(&h, &root, MATCHED).await;
    let hit = rail_shape(&h, &root).await;
    assert_eq!(
        hit["grounding"].as_str(),
        Some("grounded"),
        "a prompt overlapping a seeded document grounds: {hit}"
    );
    assert!(
        !strings(&hit["citations"]).is_empty(),
        "and names what it drew on: {hit}"
    );
    let grounded_answer = strings(&hit["bubbles"]).last().cloned().unwrap_or_default();
    assert_ne!(grounded_answer, NOT_FOUND, "{grounded_answer:?}");

    // The negative control, on the SAME prompt that just produced the
    // sentence: assistant posture answers generally instead.
    set_select(&h, &format!("{root} [data-ai-chat-posture]"), "assistant").await;
    wait_for_selector(&h, &format!("{root} [data-ai-chat-panel]")).await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    ask(&h, &root, UNMATCHED).await;
    let assisted = rail_shape(&h, &root).await;
    assert_eq!(
        assisted["grounding"].as_str(),
        Some("assistant_only"),
        "{assisted}"
    );
    let assisted_answer = strings(&assisted["bubbles"])
        .last()
        .cloned()
        .unwrap_or_default();
    assert_ne!(
        assisted_answer, NOT_FOUND,
        "assistant posture must never emit the grounded absence sentence, \
         even for a prompt no document covers: {assisted_answer:?}"
    );
    assert!(!assisted_answer.trim().is_empty(), "{assisted:?}");

    assert_no_browser_errors(&h, "grounded posture").await;
}

// ── 5 ───────────────────────────────────────────────────────────────────────

/// The remember guardrail refuses four different things for four different
/// reasons, keeps the actor's words each time, and the entry it finally
/// accepts is written WITHOUT being findable.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn remember_guardrail_refuses_identifiers_and_accepts_prose() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let root = case_root(KNOWLEDGE);
    ready_at(&h, &root).await;

    let kind_sel = format!("{root} [data-ai-chat-remember-kind]");
    let body_sel = format!("{root} [data-ai-chat-remember-input]");
    let remember = format!("{root} [data-ai-chat-remember]");

    for (kind, body, expected) in [
        (
            "tone",
            "Follow the cadence we agreed on matter 24-8842.",
            "contains_matter_number",
        ),
        (
            "tone",
            "Copy paralegal@example.com on every scheduling note.",
            "contains_email",
        ),
        (
            "tone",
            "Call 555 867 5309 before sending a scheduling note.",
            "contains_phone",
        ),
        // Not a text problem at all: the real memory service refuses a
        // curated CLASS on write, before the body is examined.
        (
            "lesson",
            "Scheduling notes read better when they open with the date.",
            "curated_kind_only",
        ),
    ] {
        set_select(&h, &kind_sel, kind).await;
        set_text(&h, &body_sel, body).await;
        click(&h, &remember).await;
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        let shape = rail_shape(&h, &root).await;
        assert_eq!(
            shape["guardrail"].as_str(),
            Some(expected),
            "{body:?} must be refused as {expected}: {shape}"
        );
        assert_eq!(
            shape["rememberValue"].as_str(),
            Some(body),
            "a refused draft is KEPT — an actor who tripped the guardrail \
             edits their words rather than retyping them: {shape}"
        );
    }

    let accepted = "Remind the team about courtroom cadence before Friday.";
    set_select(&h, &kind_sel, "tone").await;
    set_text(&h, &body_sel, accepted).await;
    click(&h, &remember).await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;

    let written = rail_shape(&h, &root).await;
    assert_eq!(
        written["guardrail"],
        json!(null),
        "plain prose is accepted: {written}"
    );
    assert_eq!(
        written["rememberValue"].as_str(),
        Some(""),
        "and an ACCEPTED write clears the box: {written}"
    );
    let calls = backend_calls(&h).await;
    assert!(
        calls.iter().any(|c| c == "Remember(Tone)"),
        "the accepted class reached the transport: {calls:?}"
    );

    let entries = written["entries"].as_array().cloned().unwrap_or_default();
    let mine = entries
        .iter()
        .find(|e| e["text"].as_str().unwrap_or_default().contains("cadence"))
        .unwrap_or_else(|| panic!("the written entry is listed: {written}"));
    assert_eq!(mine["state"].as_str(), Some("candidate"), "{mine}");
    assert_eq!(
        mine["awaiting"],
        json!(true),
        "a candidate ALWAYS carries the curation honesty row, or the list \
         reads as \"saved and searchable\": {mine}"
    );
    let entry_id = mine["id"].as_str().unwrap_or_default().to_owned();

    // The curation gap, proven on one document: written, and not findable.
    let before = recall(&h, &root, accepted).await;
    let titles: Vec<String> = before["hits"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|x| x["text"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert!(
        !titles.iter().any(|t| t.contains("cadence")),
        "recall does not return a candidate — the real service's `proposed` \
         items are invisible to search until a curator approves them: \
         {titles:?}"
    );

    click(
        &h,
        &format!("{root} [data-ai-chat-kb-confirm=\"{entry_id}\"]"),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let after = recall(&h, &root, accepted).await;
    let after_titles: Vec<String> = after["hits"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|x| x["text"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert!(
        after_titles.iter().any(|t| t.contains("cadence")),
        "confirmation is what makes it findable: {after_titles:?}"
    );

    assert_no_browser_errors(&h, "remember guardrail").await;
}

// ── 6 ───────────────────────────────────────────────────────────────────────

/// The seeded memory entries carry their own kind and state, a recall receipt
/// is ordered and attributed per lane, and turning memory use off empties the
/// results WITHOUT hiding that a search ran.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn knowledgebase_kinds_states_and_toggles() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let root = case_root(KNOWLEDGE);
    ready_at(&h, &root).await;

    let shape = rail_shape(&h, &root).await;
    let seeded: Vec<(String, String, String)> = shape["entries"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|e| {
            (
                e["id"].as_str().unwrap_or_default().to_owned(),
                e["kind"].as_str().unwrap_or_default().to_owned(),
                e["state"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        seeded,
        vec![
            ("mem-0001".into(), "language".into(), "confirmed".into()),
            ("mem-0002".into(), "tone".into(), "candidate".into()),
            (
                "mem-0003".into(),
                "working_method".into(),
                "withdrawn".into()
            ),
        ],
        "three seeded entries, each with its own typed kind and state — never \
         `{{:?}}` of a Rust variant: {shape}"
    );
    assert_eq!(
        strings(&shape["officeScopes"]),
        vec!["group_important".to_owned(), "foundation".to_owned()],
        "both curated collections are badged by scope id: {shape}"
    );

    // The receipt: a real search, ordered, attributed, and quoted verbatim.
    let recalled = recall(&h, &root, SEED_RECALL_QUERY).await;
    let receipt = recalled["receipt"].as_str().unwrap_or_default().to_owned();
    assert!(
        receipt.len() == 9
            && receipt.starts_with("rcpt-")
            && receipt[5..].chars().all(|c| c.is_ascii_digit()),
        "the receipt id is the host's own `rcpt-NNNN`: {receipt:?}"
    );
    assert!(
        recalled["search"]
            .as_str()
            .unwrap_or_default()
            .contains("reciprocal rank"),
        "the host's description of the search is quoted verbatim, never \
         paraphrased: {recalled}"
    );
    let hits = recalled["hits"].as_array().cloned().unwrap_or_default();
    assert!(!hits.is_empty(), "{recalled}");
    let scores: Vec<f64> = hits
        .iter()
        .map(|x| x["score"].as_f64().unwrap_or_default())
        .collect();
    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert_eq!(scores, sorted, "hits are rendered in rrf order: {scores:?}");
    let lanes: Vec<String> = hits
        .iter()
        .map(|x| x["attribution"].as_str().unwrap_or_default().to_owned())
        .collect();
    for lane in ["words", "meaning", "graph", "thread"] {
        assert!(
            lanes.contains(&lane.to_owned()),
            "the seeded query covers every lane, and each hit says WHICH \
             lane it came from: {lanes:?}"
        );
    }

    // Confirming a candidate reaches the transport and changes the badge.
    click(
        &h,
        &format!("{root} [data-ai-chat-kb-confirm=\"mem-0002\"]"),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let confirmed = rail_shape(&h, &root).await;
    let row = confirmed["entries"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find(|e| e["id"].as_str() == Some("mem-0002"))
        .cloned()
        .unwrap_or(Value::Null);
    assert_eq!(row["state"].as_str(), Some("confirmed"), "{confirmed}");
    assert_eq!(
        row["awaiting"],
        json!(false),
        "and the curation row goes with it: {row}"
    );
    let calls = backend_calls(&h).await;
    assert!(
        calls
            .iter()
            .any(|c| c == r#"SetMemoryState("mem-0002", Confirmed)"#),
        "{calls:?}"
    );

    // Turning memory use off empties the RESULTS without hiding the SEARCH.
    let recalls_before = count_calls(&backend_calls(&h).await, "Recall(");
    set_checkbox(&h, &format!("{root} [data-ai-chat-memory-enabled]"), false).await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let calls = backend_calls(&h).await;
    assert!(
        calls.iter().any(|c| c.starts_with("SetMemoryFlags(false")),
        "{calls:?}"
    );

    let off = recall(&h, &root, SEED_RECALL_QUERY).await;
    assert_eq!(
        off["hits"].as_array().map(Vec::len),
        Some(0),
        "a disabled store returns nothing: {off}"
    );
    assert_eq!(
        off["noHits"],
        json!(true),
        "and says so, rather than leaving the previous hits on screen: {off}"
    );
    assert_eq!(
        count_calls(&backend_calls(&h).await, "Recall("),
        recalls_before + 1,
        "the search still REACHED the store — an empty result is an answer, \
         not a skipped call"
    );

    assert_no_browser_errors(&h, "knowledgebase kinds and toggles").await;
}

// ── 7 ───────────────────────────────────────────────────────────────────────

/// Every fact the evidence rail renders carries its qualification. A value
/// beside no caveat is a stronger claim than the host made.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn grounded_facts_always_carry_a_qualification() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let root = case_root(KNOWLEDGE);
    ready_at(&h, &root).await;

    ask(&h, &root, MATCHED).await;
    let shape = rail_shape(&h, &root).await;
    let facts = shape["facts"].as_array().cloned().unwrap_or_default();
    assert!(
        !facts.is_empty(),
        "a grounded turn produces facts, or the check below is vacuous: {shape}"
    );
    for fact in &facts {
        let q = fact["qualification"].as_str().unwrap_or_default();
        assert!(
            !q.trim().is_empty(),
            "a fact without its qualification must never render: {fact}"
        );
        assert!(
            fact["text"].as_str().unwrap_or_default().contains(q),
            "and the caveat is VISIBLE, not only in the attribute: {fact}"
        );
    }

    // Document-wide, not just within this root: the offline workspace beside
    // it must not have produced an unqualified fact either.
    let stray: i64 = eval_json(
        &h,
        r#"(() => Array.from(document.querySelectorAll('[data-ai-chat-fact]'))
            .filter(e => !(e.getAttribute('data-ai-chat-fact-qualification') ?? '').trim())
            .length)()"#,
    )
    .await
    .as_i64()
    .unwrap_or(-1);
    assert_eq!(stray, 0, "no unqualified fact anywhere on the document");

    assert_no_browser_errors(&h, "qualified facts").await;
}

// ── 8 ───────────────────────────────────────────────────────────────────────

/// The knowledge panel, a rendered receipt and the offline honesty row are
/// all reachable: zero blocking axe findings with everything open.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat-knowledge)"]
async fn axe_clean_with_knowledge_panel_open() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    let knowledge = case_root(KNOWLEDGE);
    let offline = case_root(OFFLINE);
    ready_at(&h, &knowledge).await;
    ready_at(&h, &offline).await;

    // A receipt on one side, the typed offline row on the other. Auditing an
    // empty rail would prove nothing about the parts this phase added.
    let healthy = recall(&h, &knowledge, SEED_RECALL_QUERY).await;
    assert!(
        !healthy["receipt"].is_null(),
        "the healthy store rendered a receipt: {healthy}"
    );
    let down = recall(&h, &offline, SEED_RECALL_QUERY).await;
    assert_eq!(
        down["offline"],
        json!(true),
        "the offline store renders its typed honesty row: {down}"
    );
    assert_eq!(
        down["receipt"],
        json!(null),
        "and NO receipt — a receipt is evidence a search ran, so minting one \
         for a store that never answered would be the workspace vouching for \
         something that did not happen: {down}"
    );
    assert_eq!(down["hits"].as_array().map(Vec::len), Some(0), "{down}");

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(h.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("ai-chat knowledge rail")
        .unwrap_or_else(|error| panic!("{error}; {}", report.summary()));

    assert_no_browser_errors(&h, "knowledge axe").await;
}
