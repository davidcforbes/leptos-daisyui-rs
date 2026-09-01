//! Component-level decisions that are pure functions of the data, and so are
//! provable natively rather than only in a browser: which roles and rules are
//! drawn, what a bar is named, and what an activation may carry.

use super::*;
use crate::charts::bar_chart::normalize::signed_domain;

fn bar(key: &str, label: &str, value: Option<f64>, status: BarStatus) -> NormalizedBar {
    NormalizedBar {
        key: key.to_string(),
        label: label.to_string(),
        value,
        display_value: None,
        status,
        color: None,
        dom_id: format!("{key}-0"),
    }
}

fn domain(values: &[f64]) -> Domain {
    signed_domain(values.iter().copied()).expect("finite values have a domain")
}

// ── the zero rule ───────────────────────────────────────────────────────────

#[test]
fn a_vertical_chart_still_draws_the_baseline_it_always_drew() {
    // For all-positive data the zero line IS the old baseline, at the same
    // coordinates, so nothing about an existing vertical chart moves.
    assert!(draws_zero_rule(
        BarChartLayout::Vertical,
        domain(&[1.0, 2.0])
    ));
    assert!(draws_zero_rule(BarChartLayout::Auto, domain(&[1.0, 2.0])));
    assert!(draws_zero_rule(
        BarChartLayout::Vertical,
        domain(&[-1.0, 2.0])
    ));
}

#[test]
fn a_legacy_horizontal_chart_gains_no_rule_it_did_not_have() {
    // The original horizontal renderer drew no baseline at all. It gets one
    // only where the data actually needs a reference.
    assert!(!draws_zero_rule(
        BarChartLayout::Horizontal,
        domain(&[1.0, 2.0])
    ));
    assert!(!draws_zero_rule(BarChartLayout::Horizontal, domain(&[0.0])));
    assert!(draws_zero_rule(
        BarChartLayout::Horizontal,
        domain(&[-1.0, 2.0])
    ));
    assert!(draws_zero_rule(
        BarChartLayout::Horizontal,
        domain(&[-3.0, -1.0])
    ));
}

#[test]
fn the_diverging_layout_always_shows_its_reference() {
    // A caller reaches for this layout precisely because direction is the
    // message; a filtering that leaves only positive values must not silently
    // drop the line every bar is read against.
    for values in [vec![1.0, 2.0], vec![-1.0, -2.0], vec![-1.0, 3.0], vec![0.0]] {
        assert!(
            draws_zero_rule(BarChartLayout::DivergingHorizontal, domain(&values)),
            "{values:?}"
        );
    }
}

// ── the accessibility contract ──────────────────────────────────────────────

#[test]
fn an_interactive_chart_is_a_group_and_never_an_image() {
    // `role="img"` makes descendants presentational, so focusable targets
    // inside one are an axe blocker (nested-interactive + svg-img-alt). The
    // reactivity lane's vendored axe gate is what caught this on LineChart.
    assert_eq!(svg_role(true), "group");
    assert_eq!(
        svg_role(false),
        "img",
        "a chart with no targets is a pure image and should say so"
    );
}

#[test]
fn only_a_wired_callback_earns_button_semantics() {
    assert_eq!(target_role(true), "button");
    assert_eq!(
        target_role(false),
        "group",
        "a descriptive chart must not announce bars as buttons that do nothing"
    );
}

// ── status, encoded in more than colour ─────────────────────────────────────

#[test]
fn only_a_judged_bar_gets_a_cap_and_the_two_judgements_differ_in_pattern() {
    // Hue alone is unavailable in forced-colors mode and to a reader with a
    // colour vision deficiency; a solid cap and a dashed cap are not.
    assert_eq!(status_dash(BarStatus::Neutral), None);
    let favorable = status_dash(BarStatus::Favorable).expect("a judged bar is capped");
    let unfavorable = status_dash(BarStatus::Unfavorable).expect("a judged bar is capped");
    assert_ne!(
        favorable, unfavorable,
        "the two judgements must differ in a channel that is not colour"
    );
}

// ── accessible naming ───────────────────────────────────────────────────────

#[test]
fn an_unjudged_bar_is_named_by_its_label_and_value_alone() {
    let texts = BarChartTexts::default();
    let format = BarValueFormat::default();

    assert_eq!(
        accessible_name(
            &bar("north", "North", Some(-12.5), BarStatus::Neutral),
            &format,
            &texts
        ),
        "North: -12.5",
        "a neutral activity measure must not be given a judgement it does not have"
    );
}

#[test]
fn a_judged_bar_states_its_judgement_in_words() {
    let texts = BarChartTexts::default();
    let format = BarValueFormat::default().with_unit(" pts").with_decimals(0);

    assert_eq!(
        accessible_name(
            &bar("north", "North", Some(-12.0), BarStatus::Unfavorable),
            &format,
            &texts
        ),
        "North: -12 pts, Unfavorable"
    );
    assert_eq!(
        accessible_name(
            &bar("west", "West", Some(8.0), BarStatus::Favorable),
            &format,
            &texts
        ),
        "West: 8 pts, Favorable"
    );
}

#[test]
fn a_missing_bar_is_named_missing_rather_than_zero() {
    let texts = BarChartTexts::default();

    assert_eq!(
        accessible_name(
            &bar("south", "South", None, BarStatus::Neutral),
            &BarValueFormat::default(),
            &texts
        ),
        "South: No value"
    );
}

#[test]
fn every_word_in_an_accessible_name_comes_from_the_supplied_copy() {
    // The localization criterion at the naming layer: switching the texts
    // switches the words, and nothing else.
    let spanish = BarChartTexts {
        no_value: "Sin dato".to_string(),
        status_unfavorable: "Desfavorable".to_string(),
        ..BarChartTexts::default()
    };
    let format = BarValueFormat::default();

    assert_eq!(
        accessible_name(
            &bar("north", "North", Some(-12.0), BarStatus::Unfavorable),
            &format,
            &spanish
        ),
        "North: -12.0, Desfavorable"
    );
    assert_eq!(
        accessible_name(
            &bar("south", "South", None, BarStatus::Neutral),
            &format,
            &spanish
        ),
        "South: Sin dato"
    );
}

// ── the activation payload ──────────────────────────────────────────────────

#[test]
fn an_activation_carries_the_stable_key_and_never_an_index() {
    let texts = BarChartTexts::default();
    let format = BarValueFormat::default();
    let payload = activation_for(
        &bar("north", "North", Some(-12.5), BarStatus::Unfavorable),
        &format,
        &texts,
        BarChartActivationSource::Keyboard,
        modifiers_of(true, false, false, false),
    )
    .expect("a finite bar is activatable");

    assert_eq!(payload.category_key, "north");
    assert_eq!(payload.category_label, "North");
    assert_eq!(payload.value, -12.5);
    assert_eq!(payload.display_value, "-12.5");
    assert_eq!(payload.status, BarStatus::Unfavorable);
    assert_eq!(payload.source, BarChartActivationSource::Keyboard);
    assert!(payload.modifiers.shift);
    assert!(!payload.modifiers.ctrl);
}

#[test]
fn an_activation_states_the_value_exactly_as_the_chart_drew_it() {
    // One formatter, four surfaces. A host that echoes `display_value` back
    // into its own UI must not print a different number from the bar.
    let texts = BarChartTexts::default();
    let format = BarValueFormat::default().with_unit("%").with_decimals(1);
    let mut item = bar("north", "North", Some(-12.456), BarStatus::Neutral);
    let payload = activation_for(
        &item,
        &format,
        &texts,
        BarChartActivationSource::Pointer,
        BarChartModifiers::default(),
    )
    .expect("a finite bar is activatable");
    assert_eq!(payload.display_value, "-12.5%");
    assert_eq!(payload.display_value, item.value_text(&format, &texts));

    item.display_value = Some("12.5% behind".to_string());
    let payload = activation_for(
        &item,
        &format,
        &texts,
        BarChartActivationSource::Pointer,
        BarChartModifiers::default(),
    )
    .expect("a finite bar is activatable");
    assert_eq!(payload.display_value, "12.5% behind");
}

#[test]
fn a_missing_or_non_finite_bar_can_never_be_activated() {
    // The acceptance criterion that a gap never leaks NaN, infinity or a
    // fabricated zero: there is simply no payload to emit.
    let texts = BarChartTexts::default();
    let format = BarValueFormat::default();
    for value in [None, Some(f64::NAN), Some(f64::INFINITY)] {
        let mut candidate = bar("south", "South", value, BarStatus::Neutral);
        // Normalization is what rejects non-finite values, so apply it here
        // exactly as the render path does.
        candidate.value = candidate.value.filter(|value| value.is_finite());
        assert!(
            activation_for(
                &candidate,
                &format,
                &texts,
                BarChartActivationSource::Pointer,
                BarChartModifiers::default()
            )
            .is_none(),
            "{value:?}"
        );
    }
}

#[test]
fn a_finite_zero_is_activatable_because_it_is_a_real_measurement() {
    let payload = activation_for(
        &bar("east", "East", Some(0.0), BarStatus::Neutral),
        &BarValueFormat::default(),
        &BarChartTexts::default(),
        BarChartActivationSource::Pointer,
        BarChartModifiers::default(),
    )
    .expect("zero is a measurement, not a gap");

    assert_eq!(payload.value, 0.0);
    assert_eq!(payload.display_value, "0.0");
}

// ── DOM identity ────────────────────────────────────────────────────────────

#[test]
fn target_ids_are_unique_per_chart_instance_and_bar() {
    assert_eq!(target_id(3, 2), "bar-chart-3-bar-2");
    assert_ne!(target_id(3, 2), target_id(4, 2));
    assert_ne!(target_id(3, 2), target_id(3, 1));
}
