//! Real-browser regression for ldui-1jxa: `Field` used to `provide_context`
//! onto whatever reactive `Owner` was current when it ran -- the *shared*
//! owner of every sibling built alongside it -- so a later, unrelated
//! control that never wraps in its own `Field` (a standalone `Input`/
//! `Select`, or one nested inside `DatasetSelector`/an `EntityTable` column
//! filter) picked up the same `FieldContext` and rendered the *same* `id` as
//! the original `Field`'s own control, which Chrome reports as "Duplicate
//! form field id in the same form".
//!
//! The native unit tests in `src/components/field/tests.rs`
//! (`context_scope` module) prove the owner-scoping mechanism directly --
//! this crate has no native DOM/SSR renderer, so they cannot read a rendered
//! `id` attribute back. This suite is the real-DOM half of the same
//! contract, against the dedicated `/field-context-scope` fixture
//! (`demo/src/demos/field_context_scope.rs`).
//!
//! Kept in its own file/xtask step rather than folded into
//! `reactivity_smoke.rs`, whose check count is pinned (see that file's own
//! header) -- same rationale as `section_heading_smoke.rs`.
//!
//! ```text
//! cargo xtask test-field-context-scoping     # spawns its own server, then tears it down
//! # or against a server you already have running:
//! trunk serve                                # in demo/ (npm install once first)
//! cargo test --test field_context_scoping -- --ignored --test-threads=1
//! ```
//!
//! NOTE: on this machine trunk/Windows cannot publish `dist`/`snippets`
//! (see `doc/ci-cd.md` and the repo's wasm-linker-crash memory), so these
//! `#[ignore]`d cases have only been verified to *compile*
//! (`cargo test --test field_context_scoping --no-run`) as part of
//! ldui-1jxa; running them is deferred to a machine/gate where the demo
//! server actually serves current wasm.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, harness_at};
use serde_json::{Value, json};

const PAGE: &str = "/field-context-scope";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate field-context-scope fixture")
        .into_value()
        .expect("field-context-scope expression returns JSON")
}

/// One snapshot of every control's rendered `id` on the fixture, plus the
/// `Field`'s own label `for` target -- everything the regression needs.
async fn snapshot(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const byTestId = (id) => document.querySelector(`[data-testid="${id}"]`);
            const fieldInput = byTestId('field-context-scope-field-input');
            const label = document.querySelector('#field-context-scope-fixture label[for]');
            const standaloneInput = byTestId('field-context-scope-standalone-input-control');
            const standaloneSelect = byTestId('field-context-scope-standalone-select-control');
            const bareInput = byTestId('field-context-scope-bare-input-control');
            return {
                fieldInputId: fieldInput ? fieldInput.id : null,
                labelFor: label ? label.getAttribute('for') : null,
                standaloneInputId: standaloneInput ? standaloneInput.id : null,
                standaloneSelectId: standaloneSelect ? standaloneSelect.id : null,
                bareInputId: bareInput ? bareInput.id : null,
            };
        })()"#,
    )
    .await
}

/// The `Field`-wrapped `Input` mints its own `ld-field-*` id and the visible
/// label points at exactly that id -- the association contract this fixture
/// also protects, unrelated to the leak itself.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-field-context-scoping)"]
async fn field_wrapped_input_mints_its_own_id_and_the_label_points_at_it() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h).await;
    let field_id = s["fieldInputId"]
        .as_str()
        .expect("Field-wrapped input has an id")
        .to_string();
    assert!(
        field_id.starts_with("ld-field-"),
        "Field mints an ld-field-* id: {s}"
    );
    assert_eq!(
        s["labelFor"],
        json!(field_id),
        "the visible label must point at exactly this Field's own control: {s}"
    );

    assert_no_browser_errors(&h, "field-context-scope field-wrapped input").await;
}

/// The regression itself: a standalone `Input`/`Select` built after the
/// `Field` in the same static tree -- neither wraps in its own `Field` --
/// must not inherit the `Field`'s `FieldContext` and therefore must not
/// carry its `ld-field-*` id. Each keeps the id the caller explicitly gave
/// it, and those ids are distinct from the `Field`'s and from each other.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-field-context-scoping)"]
async fn later_standalone_controls_do_not_inherit_the_earlier_fields_id() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h).await;
    let field_id = s["fieldInputId"]
        .as_str()
        .expect("Field-wrapped input has an id")
        .to_string();
    let standalone_input_id = s["standaloneInputId"]
        .as_str()
        .expect("standalone input has an id")
        .to_string();
    let standalone_select_id = s["standaloneSelectId"]
        .as_str()
        .expect("standalone select has an id")
        .to_string();

    assert_ne!(
        standalone_input_id, field_id,
        "ldui-1jxa: a later standalone Input must not inherit the earlier Field's id: {s}"
    );
    assert_ne!(
        standalone_select_id, field_id,
        "ldui-1jxa: a later standalone Select must not inherit the earlier Field's id: {s}"
    );
    assert_ne!(
        standalone_input_id, standalone_select_id,
        "the two standalone controls must not collide with each other either: {s}"
    );
    // Caller-supplied ids continue to win: both standalone controls were
    // given explicit ids at the call site, and those are exactly what
    // rendered.
    assert_eq!(
        standalone_input_id, "field-context-scope-standalone-input-control",
        "caller-supplied id must win: {s}"
    );
    assert_eq!(
        standalone_select_id, "field-context-scope-standalone-select-control",
        "caller-supplied id must win: {s}"
    );

    assert_no_browser_errors(&h, "field-context-scope standalone controls").await;
}

/// The strictest case: a plain `Input` with *no* caller-supplied id at all,
/// built after the `Field`. With the leak fixed there is nothing left that
/// could hand it an id, so it renders with none -- proving the absence of
/// the leaked `FieldContext` rather than a caller-supplied id merely masking
/// it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-field-context-scoping)"]
async fn a_later_bare_input_with_no_caller_id_gets_no_leaked_field_id_either() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h).await;
    let field_id = s["fieldInputId"]
        .as_str()
        .expect("Field-wrapped input has an id")
        .to_string();
    let bare_id = s["bareInputId"].as_str().unwrap_or("");
    assert_ne!(
        bare_id, field_id,
        "ldui-1jxa: an id-less later Input must not pick up the leaked FieldContext id: {s}"
    );
    assert!(
        bare_id.is_empty(),
        "no id was ever provided for this control -- any non-empty id here would have to have leaked in: {s}"
    );

    assert_no_browser_errors(&h, "field-context-scope bare input").await;
}
