//! Controlled checked-state, change proposals and DOM identity for
//! [`Checkbox`](super::Checkbox) (`ldui-fqan`).
//!
//! # Accepted truth is caller-owned
//!
//! This is the same contract [`ServerTableMultiSelection`] and
//! [`ModalCloseProposal`] already carry, narrowed to one boolean: the caller
//! holds the accepted value as a `Signal`, every user gesture emits exactly one
//! [`CheckboxChangeProposal`], and the component never writes an optimistic
//! value of its own. A proposal the caller ignores therefore leaves nothing to
//! reconcile.
//!
//! A checkbox needs one extra step the other two do not. The browser flips
//! `input.checked` *natively*, before any handler runs, so "never diverge
//! optimistically" is not achievable by simply declining to write — the write
//! already happened. [`Checkbox`](super::Checkbox)'s change handler therefore
//! re-asserts the accepted value onto the element **before** proposing, which
//! is what makes a declined proposal a visual no-op.
//!
//! # `indeterminate` is a DOM property, not an attribute
//!
//! There is no `indeterminate` content attribute at all: markup that says
//! `indeterminate="true"` sets nothing. It must be written as
//! `HTMLInputElement.indeterminate`, and the browser *clears* the flag as part
//! of handling a click, so it also has to be re-asserted in the change handler
//! or a tri-state checkbox silently degrades into a plain one after the first
//! interaction (`ldui-nz6d`). [`CheckboxState`] is the single source both the
//! render path and the handler read, so the two cannot drift.
//!
//! # Identity
//!
//! `id`/`name` follow the scheme `ldui-j6sh` established for the table
//! controls: a caller-supplied value wins, otherwise a process-unique minted
//! one, and `name` matters separately from `id` because `name` is what makes
//! the element a real form control. The rules for what a supplied value is
//! normalized into are deliberately identical to
//! `components::data_table::identity`'s (that module is private to
//! `data_table`, so the rules are restated here rather than shared; a test
//! pins them against that module's own documented example so the two cannot
//! quietly fork).
//!
//! [`ServerTableMultiSelection`]: crate::components::ServerTableMultiSelection
//! [`ModalCloseProposal`]: crate::components::ModalCloseProposal

use leptos::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Combining the controlled binding with the uncontrolled `default_checked`
/// seed is rejected, not resolved: honouring either one silently would give
/// the page two sources of truth for the same boolean, which is exactly the
/// failure this contract exists to remove.
pub(crate) const CONFLICTING_CHECKED_OWNERSHIP_CONFIGURATION: &str =
    "Checkbox accepts either a controlled binding or default_checked, not both";

/// A visible `label` already *is* the accessible name. Adding `aria_label` on
/// top replaces it with different words, which is a WCAG 2.5.3 (Label in Name)
/// failure and breaks speech control — so it is refused rather than resolved
/// to one of them.
pub(crate) const CONFLICTING_LABEL_CONFIGURATION: &str =
    "Checkbox accepts either label or aria_label, not both";

/// The reserved `id` namespace this crate mints into, shared with
/// `data_table`'s control ids. A caller-supplied id should not start with it.
pub(crate) const RESERVED_CHECKBOX_ID_NAMESPACE: &str = "ldui-";

/// Process-wide sequence behind [`next_checkbox_control_id`].
static CHECKBOX_CONTROL_ID: AtomicU64 = AtomicU64::new(0);

/// A process-unique `id` for one mounted checkbox, minted only when the
/// component needs an id the caller did not supply (see
/// [`resolve_checkbox_id`]).
///
/// A monotonic counter rather than randomness, mirroring
/// `next_data_table_control_id`: unique across every checkbox mounted in one
/// page's lifetime, which is all `label[for]` association needs. It depends on
/// mount order, so it is never promoted into a `name` — see
/// [`resolve_checkbox_name`].
pub(crate) fn next_checkbox_control_id() -> String {
    format!(
        "{RESERVED_CHECKBOX_ID_NAMESPACE}checkbox-{}",
        CHECKBOX_CONTROL_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// Normalizes a caller-supplied `id` into something usable as an HTML `id`.
///
/// `[A-Za-z0-9_-]` survives verbatim so a readable `past-due-filter` stays
/// readable in the DOM; anything else (a space, a dot, non-ASCII) becomes `_`
/// plus two lowercase hex digits, because an `id` containing whitespace is
/// invalid and one containing `.`/`#`/`:` is a CSS-selector trap. An empty or
/// all-whitespace value is treated as absent.
pub(crate) fn normalize_control_id(supplied: &str) -> Option<String> {
    let trimmed = supplied.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(trimmed.len());
    for byte in trimmed.as_bytes() {
        if byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_' {
            out.push(*byte as char);
        } else {
            out.push('_');
            out.push_str(&format!("{byte:02x}"));
        }
    }
    Some(out)
}

/// Resolves the `id` actually rendered, in precedence order: a usable
/// caller-supplied value, then the id a surrounding
/// [`Field`](crate::components::Field) minted for the control it wraps, then —
/// only when the component needs one of its own — the per-instance `minted`
/// fallback.
///
/// Returning `None` is the backward-compatible case and is load-bearing: a
/// `<Checkbox />` that opts into none of the identity props must keep
/// rendering with no `id` attribute at all.
pub(crate) fn resolve_checkbox_id(
    supplied: Option<String>,
    field_id: Option<String>,
    mint_when_absent: bool,
    minted: &str,
) -> Option<String> {
    if let Some(id) = supplied.as_deref().and_then(normalize_control_id) {
        return Some(id);
    }
    if let Some(id) = field_id {
        return Some(id);
    }
    mint_when_absent.then(|| minted.to_owned())
}

/// Resolves the `name` actually rendered.
///
/// A caller-supplied `name` wins and is passed through **verbatim** — a form
/// key is the server's vocabulary (`filters[past_due]`), not an HTML id, and
/// normalizing it would silently rename the submitted field.
///
/// Otherwise a caller-supplied `id` becomes the `name`, which is `ldui-j6sh`'s
/// `id == name` rule for framework-owned controls. A **minted** id never
/// becomes a `name`: the mint depends on mount order, so using it as a form key
/// would change what the form submits whenever the page's component order
/// changed — an unstable `name` is worse than no `name`, because the breakage
/// is silent and lands on the server.
pub(crate) fn resolve_checkbox_name(
    supplied_name: Option<String>,
    supplied_id: Option<String>,
) -> Option<String> {
    if let Some(name) = supplied_name.filter(|name| !name.trim().is_empty()) {
        return Some(name);
    }
    supplied_id.as_deref().and_then(normalize_control_id)
}

/// The accepted tri-state a controlled [`Checkbox`](super::Checkbox) renders.
///
/// One value drives `checked`, the `indeterminate` DOM property and
/// `aria-checked` together, so what is drawn and what is announced cannot
/// diverge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CheckboxState {
    /// Accepted `false`, not mixed.
    #[default]
    Unchecked,
    /// Accepted `true`, not mixed.
    Checked,
    /// Mixed: neither on nor off, the `indeterminate` DOM property.
    Mixed,
}

impl CheckboxState {
    /// The state a `(checked, indeterminate)` pair of accepted signals means.
    ///
    /// Mixed wins over `checked`: a caller whose two signals disagree is
    /// describing a partial selection, and rendering it as a plain tick would
    /// claim more than the caller said.
    pub fn from_accepted(checked: bool, indeterminate: bool) -> Self {
        if indeterminate {
            Self::Mixed
        } else if checked {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }

    /// Whether the input renders with `checked` set.
    pub fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    /// Whether the input renders with the `indeterminate` DOM property set.
    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Mixed)
    }

    /// The `aria-checked` token for this state.
    ///
    /// Mixed must be announced as `mixed`; a native checkbox reports
    /// `aria-checked="false"` while indeterminate, so the tri-state is
    /// otherwise inaudible.
    pub fn aria_checked(self) -> &'static str {
        match self {
            Self::Unchecked => "false",
            Self::Checked => "true",
            Self::Mixed => "mixed",
        }
    }

    /// The boolean a user gesture from this state proposes.
    ///
    /// Mixed proposes `true`, matching every native tri-state control: from a
    /// partial selection the useful next step is "select all of it".
    pub fn toggles_to(self) -> bool {
        !matches!(self, Self::Checked)
    }

    /// Stable DOM marker for tests and consumers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unchecked => "unchecked",
            Self::Checked => "checked",
            Self::Mixed => "mixed",
        }
    }
}

/// One user-proposed replacement for the caller's accepted checkbox value.
///
/// `checked` is the COMPLETE proposed value, not a delta: apply it or decline
/// it wholesale. Nothing is applied until the caller's own signal changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckboxChangeProposal {
    /// The proposed accepted value.
    pub checked: bool,
    /// The accepted state the gesture was made against, read at gesture time.
    ///
    /// A caller that needs to tell "the user cleared a partial selection" from
    /// "the user unticked a full one" reads this; a caller that does not can
    /// ignore it and just take [`checked`](Self::checked).
    pub from: CheckboxState,
}

impl CheckboxChangeProposal {
    /// The proposal a gesture made against `from` produces.
    pub fn from_state(from: CheckboxState) -> Self {
        Self {
            checked: from.toggles_to(),
            from,
        }
    }
}

/// Opt-in controlled ownership of a [`Checkbox`](super::Checkbox)'s checked
/// state.
///
/// Constructing one is the *only* way to put a checkbox into controlled mode,
/// so a half-configured control — an accepted signal with no owner to notify,
/// or a callback with no accepted truth to re-assert — cannot be expressed.
///
/// ```rust,no_run
/// # use leptos::prelude::*;
/// # use leptos_daisyui_rs::components::*;
/// # fn demo() {
/// let past_due_only = RwSignal::new(false);
/// let binding = CheckboxBinding::controlled(
///     past_due_only.into(),
///     Callback::new(move |proposal: CheckboxChangeProposal| {
///         // Accepted truth stays caller-owned: apply, or decline.
///         past_due_only.set(proposal.checked);
///     }),
/// );
/// # let _ = binding;
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct CheckboxBinding {
    pub(crate) checked: Signal<bool>,
    pub(crate) indeterminate: Option<Signal<bool>>,
    pub(crate) on_change: Callback<CheckboxChangeProposal>,
}

impl CheckboxBinding {
    /// Creates controlled ownership over one accepted boolean. `on_change`
    /// receives one complete proposal per user gesture.
    pub fn controlled(checked: Signal<bool>, on_change: Callback<CheckboxChangeProposal>) -> Self {
        Self {
            checked,
            indeterminate: None,
            on_change,
        }
    }

    /// Declares the accepted mixed/partial state, written to the DOM as the
    /// `indeterminate` property and announced as `aria-checked="mixed"`.
    pub fn with_indeterminate(mut self, indeterminate: Signal<bool>) -> Self {
        self.indeterminate = Some(indeterminate);
        self
    }

    /// The caller-owned accepted value.
    pub fn checked(self) -> Signal<bool> {
        self.checked
    }

    /// The accepted tri-state, tracked.
    pub fn state(self) -> CheckboxState {
        CheckboxState::from_accepted(
            self.checked.get(),
            self.indeterminate.map(|s| s.get()).unwrap_or(false),
        )
    }

    /// The accepted tri-state, read without subscribing — what the change
    /// handler re-asserts onto the element the browser just toggled.
    pub fn state_untracked(self) -> CheckboxState {
        CheckboxState::from_accepted(
            self.checked.get_untracked(),
            self.indeterminate
                .map(|s| s.get_untracked())
                .unwrap_or(false),
        )
    }

    /// Whether this binding declared a mixed state at all.
    pub fn has_indeterminate(self) -> bool {
        self.indeterminate.is_some()
    }
}

/// Which ownership model a [`Checkbox`](super::Checkbox) configuration
/// resolves to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CheckboxOwnership {
    /// No binding: the browser owns the checked state, exactly as before this
    /// contract existed.
    Uncontrolled,
    /// The caller owns accepted truth and receives proposals.
    Controlled,
}

/// Rejects ambiguous configurations instead of resolving them.
///
/// Both refusals are ownership questions: who owns the checked state, and
/// which words are the control's accessible name.
pub(crate) fn resolve_checkbox_ownership(
    has_binding: bool,
    has_default_checked: bool,
    has_label: bool,
    has_aria_label: bool,
) -> Result<CheckboxOwnership, &'static str> {
    if has_label && has_aria_label {
        return Err(CONFLICTING_LABEL_CONFIGURATION);
    }
    match (has_binding, has_default_checked) {
        (true, true) => Err(CONFLICTING_CHECKED_OWNERSHIP_CONFIGURATION),
        (true, false) => Ok(CheckboxOwnership::Controlled),
        (false, _) => Ok(CheckboxOwnership::Uncontrolled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn mixed_wins_over_checked_so_a_partial_selection_never_reads_as_complete() {
        assert_eq!(
            CheckboxState::from_accepted(false, false),
            CheckboxState::Unchecked
        );
        assert_eq!(
            CheckboxState::from_accepted(true, false),
            CheckboxState::Checked
        );
        assert_eq!(
            CheckboxState::from_accepted(false, true),
            CheckboxState::Mixed
        );
        assert_eq!(
            CheckboxState::from_accepted(true, true),
            CheckboxState::Mixed
        );
    }

    #[test]
    fn each_state_maps_to_exactly_one_dom_presentation() {
        for (state, checked, indeterminate, aria, marker) in [
            (CheckboxState::Unchecked, false, false, "false", "unchecked"),
            (CheckboxState::Checked, true, false, "true", "checked"),
            (CheckboxState::Mixed, false, true, "mixed", "mixed"),
        ] {
            assert_eq!(state.is_checked(), checked, "{state:?}");
            assert_eq!(state.is_indeterminate(), indeterminate, "{state:?}");
            assert_eq!(state.aria_checked(), aria, "{state:?}");
            assert_eq!(state.as_str(), marker, "{state:?}");
        }
    }

    #[test]
    fn a_gesture_from_mixed_proposes_true_and_only_checked_proposes_false() {
        assert!(CheckboxState::Unchecked.toggles_to());
        assert!(CheckboxState::Mixed.toggles_to());
        assert!(!CheckboxState::Checked.toggles_to());
    }

    #[test]
    fn a_proposal_carries_both_the_next_value_and_the_state_it_came_from() {
        for from in [
            CheckboxState::Unchecked,
            CheckboxState::Checked,
            CheckboxState::Mixed,
        ] {
            let proposal = CheckboxChangeProposal::from_state(from);
            assert_eq!(proposal.from, from);
            assert_eq!(proposal.checked, from.toggles_to());
        }
        // The distinction the `from` field exists for: both clear to `false`
        // in a caller's naive reading, but only one of them was a full tick.
        assert_ne!(
            CheckboxChangeProposal::from_state(CheckboxState::Mixed),
            CheckboxChangeProposal::from_state(CheckboxState::Checked)
        );
    }

    #[test]
    fn incompatible_ownership_is_rejected_rather_than_resolved() {
        assert_eq!(
            resolve_checkbox_ownership(true, true, false, false),
            Err(CONFLICTING_CHECKED_OWNERSHIP_CONFIGURATION)
        );
        assert_eq!(
            resolve_checkbox_ownership(false, false, true, true),
            Err(CONFLICTING_LABEL_CONFIGURATION)
        );
        // A label conflict is reported even when the ownership half is fine,
        // and takes precedence so the message names the first thing to fix.
        assert_eq!(
            resolve_checkbox_ownership(true, true, true, true),
            Err(CONFLICTING_LABEL_CONFIGURATION)
        );
    }

    #[test]
    fn the_uncontrolled_default_survives_every_non_conflicting_configuration() {
        for (label, aria) in [(false, false), (true, false), (false, true)] {
            assert_eq!(
                resolve_checkbox_ownership(false, false, label, aria),
                Ok(CheckboxOwnership::Uncontrolled),
                "a checkbox with no binding must stay uncontrolled"
            );
            assert_eq!(
                resolve_checkbox_ownership(false, true, label, aria),
                Ok(CheckboxOwnership::Uncontrolled),
                "default_checked alone is the uncontrolled seed, not a binding"
            );
            assert_eq!(
                resolve_checkbox_ownership(true, false, label, aria),
                Ok(CheckboxOwnership::Controlled)
            );
        }
    }

    /// The id shape `Field` mints, assembled at runtime rather than written as
    /// a literal: `tests/ld_class_stylesheet_coverage.rs` scans this crate's
    /// source for `ld-*` string literals and would read the fixture as an
    /// undefined CSS class.
    fn field_minted_id() -> String {
        format!("ld-field-{}", 3)
    }

    #[test]
    fn a_caller_supplied_id_wins_over_the_field_and_the_mint() {
        assert_eq!(
            resolve_checkbox_id(
                Some("past-due-filter".into()),
                Some(field_minted_id()),
                true,
                "ldui-checkbox-9",
            ),
            Some("past-due-filter".to_owned())
        );
        assert_eq!(
            resolve_checkbox_id(None, Some(field_minted_id()), true, "ldui-checkbox-9"),
            Some(field_minted_id())
        );
        assert_eq!(
            resolve_checkbox_id(None, None, true, "ldui-checkbox-9"),
            Some("ldui-checkbox-9".to_owned())
        );
    }

    #[test]
    fn a_checkbox_that_needs_no_id_renders_none_at_all() {
        // The backward-compatibility case: `<Checkbox />` opts into nothing,
        // so it must keep emitting no `id` attribute whatsoever.
        assert_eq!(
            resolve_checkbox_id(None, None, false, "ldui-checkbox-9"),
            None
        );
        // An unusable supplied value falls through rather than rendering an
        // invalid id.
        assert_eq!(
            resolve_checkbox_id(Some("   ".into()), None, false, "m"),
            None
        );
        assert_eq!(
            resolve_checkbox_id(Some(String::new()), None, false, "m"),
            None
        );
    }

    #[test]
    fn a_supplied_id_is_escaped_the_same_way_the_table_controls_escape_theirs() {
        // Pinned against `data_table::identity`'s own documented example, so
        // the two id schemes cannot quietly fork (that module is private to
        // `data_table`, so this is the only way to hold them together).
        assert_eq!(
            normalize_control_id("  office perf.table  "),
            Some("office_20perf_2etable".to_owned())
        );
        for supplied in ["a b", "a.b", "a#b", "a:b", "тбл", "ok-name_1"] {
            let resolved = normalize_control_id(supplied).expect("non-empty");
            assert!(!resolved.is_empty());
            assert!(
                resolved
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
                "{supplied:?} resolved to {resolved:?}, which is not id-safe"
            );
        }
    }

    #[test]
    fn a_supplied_name_wins_verbatim_and_a_supplied_id_becomes_the_name() {
        // Verbatim: a form key is the server's vocabulary, not an HTML id.
        assert_eq!(
            resolve_checkbox_name(Some("filters[past_due]".into()), Some("past-due".into())),
            Some("filters[past_due]".to_owned())
        );
        assert_eq!(
            resolve_checkbox_name(None, Some("past-due".into())),
            Some("past-due".to_owned())
        );
        // Same escaping as the id, so `name` and `id` agree when the id wins.
        assert_eq!(
            resolve_checkbox_name(None, Some("past due".into())),
            Some("past_20due".to_owned())
        );
    }

    #[test]
    fn a_minted_id_never_becomes_a_form_name() {
        // The whole point: a mount-order-dependent name would silently change
        // what the form submits when the page's component order changed.
        assert_eq!(resolve_checkbox_name(None, None), None);
        assert_eq!(resolve_checkbox_name(Some("  ".into()), None), None);
        let minted = next_checkbox_control_id();
        assert!(minted.starts_with(RESERVED_CHECKBOX_ID_NAMESPACE));
        assert_eq!(
            resolve_checkbox_id(None, None, true, &minted),
            Some(minted.clone())
        );
        assert_eq!(
            resolve_checkbox_name(None, None),
            None,
            "the minted id must not leak into `name`"
        );
    }

    #[test]
    fn two_checkboxes_on_one_page_never_mint_the_same_id() {
        let ids: HashSet<String> = (0..64).map(|_| next_checkbox_control_id()).collect();
        assert_eq!(ids.len(), 64);
        assert!(
            ids.iter()
                .all(|id| id.starts_with(RESERVED_CHECKBOX_ID_NAMESPACE))
        );
    }

    // ── The rendered markup is actually wired to this contract ──
    //
    // Every function above can be perfect while the view uses none of them.
    // These scan the view source, the same idiom `data_table::identity`'s and
    // `filter_sidebar`'s tests use, so dropping one of these lines is a native
    // failure here rather than something a consumer notices months later.

    const COMPONENT_SRC: &str = include_str!("component.rs");

    #[test]
    fn the_change_handler_re_asserts_accepted_truth_before_proposing() {
        // The browser has ALREADY toggled `checked` by the time this runs, so
        // "don't write anything optimistic" is not enough -- the accepted
        // value has to be written back.
        let reassert = COMPONENT_SRC
            .find("input.set_checked(accepted.is_checked());")
            .expect("the change handler must re-assert the accepted checked value");
        let propose = COMPONENT_SRC
            .find("model.on_change.run(CheckboxChangeProposal::from_state(accepted));")
            .expect("the change handler must emit exactly one typed proposal");
        assert!(
            reassert < propose,
            "the accepted value must be re-asserted BEFORE the proposal is emitted, so a \
             synchronous acceptance is not immediately overwritten by the re-assertion"
        );
        assert_eq!(
            COMPONENT_SRC
                .matches("model.on_change.run(CheckboxChangeProposal::from_state(accepted));")
                .count(),
            1,
            "one gesture must propose exactly once"
        );
    }

    #[test]
    fn indeterminate_is_written_as_a_property_and_re_asserted_after_a_click() {
        // `indeterminate` has no content attribute, and the browser clears the
        // flag while handling a click (ldui-nz6d), so BOTH of these lines are
        // load-bearing: the render write alone degrades to a plain checkbox
        // after the first interaction.
        assert!(
            COMPONENT_SRC
                .contains("prop:indeterminate=move || accepted_state.get().is_indeterminate()"),
            "the controlled input must write `indeterminate` as a DOM property"
        );
        assert!(
            COMPONENT_SRC.contains("input.set_indeterminate(accepted.is_indeterminate());"),
            "the change handler must re-assert `indeterminate`, which the browser clears on click"
        );
        // Every `indeterminate=` in the view must be a `prop:` one: an
        // attribute spelling sets nothing at all, silently.
        assert_eq!(
            COMPONENT_SRC.matches("indeterminate=").count(),
            COMPONENT_SRC.matches("prop:indeterminate=").count(),
            "`indeterminate` must never be emitted as an attribute -- markup sets nothing"
        );
    }

    #[test]
    fn the_uncontrolled_path_attaches_no_change_handler_and_no_checked_property() {
        // Backward compatibility, structurally: an `on:change` or a
        // `prop:checked` on the uncontrolled branch would fight a caller's own
        // spread handler and its `attr:checked` seed.
        let uncontrolled = COMPONENT_SRC
            .split("// ── uncontrolled ──")
            .nth(1)
            .and_then(|rest| rest.split("// ── end uncontrolled ──").next())
            .expect("the uncontrolled branch must stay marked in the source");
        assert!(
            !uncontrolled.contains("on:change"),
            "the uncontrolled branch must not attach a change handler"
        );
        assert!(
            !uncontrolled.contains("prop:checked"),
            "the uncontrolled branch must not write the checked property"
        );
        assert!(
            !uncontrolled.contains("aria-checked"),
            "an uncontrolled checkbox reports its own native checked state"
        );
    }

    #[test]
    fn a_refused_configuration_renders_a_fail_closed_alert_and_no_input() {
        // ServerDataTable's precedent, not EntityTable's panic: a checkbox is
        // a leaf control that may be rendered hundreds of times in a list, and
        // a panic in a CSR wasm app takes the whole page down.
        assert!(COMPONENT_SRC.contains("role=\"alert\""));
        assert!(COMPONENT_SRC.contains("data-checkbox-config-error=message"));
        assert!(
            COMPONENT_SRC.contains("resolve_checkbox_ownership("),
            "the component must go through the shared refusal, not re-derive it"
        );
    }
}
