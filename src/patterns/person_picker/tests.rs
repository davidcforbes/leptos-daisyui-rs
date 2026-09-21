use super::model::*;
use super::texts::PersonPickerTexts;

fn person(id: &str, name: &str, secondary: &str) -> PickerPerson {
    PickerPerson {
        id: id.to_owned(),
        name: name.to_owned(),
        secondary: secondary.to_owned(),
        initials: "XX".to_owned(),
        presence: None,
    }
}

fn roster() -> Vec<PickerPerson> {
    vec![
        person("a", "Ana Lopez", "Denver"),
        person("b", "Ben Ortiz", "Austin"),
        person("c", "Cara Diaz", "Denver"),
    ]
}

#[test]
fn next_selection_selects_toggles_off_and_switches() {
    assert_eq!(
        next_selection(None, "a"),
        Some("a".to_owned()),
        "select from none"
    );
    assert_eq!(
        next_selection(Some("a"), "a"),
        None,
        "second click deselects"
    );
    assert_eq!(
        next_selection(Some("a"), "b"),
        Some("b".to_owned()),
        "switch"
    );
    assert_eq!(
        next_selection(Some("gone"), "a"),
        Some("a".to_owned()),
        "stale current"
    );
}

#[test]
fn search_matches_name_or_secondary_case_insensitively() {
    let ana = person("a", "Ana Lopez", "Denver");
    assert!(matches_search(&ana, ""), "empty query matches all");
    assert!(matches_search(&ana, "   "), "blank query matches all");
    assert!(matches_search(&ana, "lopez"));
    assert!(matches_search(&ana, "DENV"), "secondary line, any case");
    assert!(!matches_search(&ana, "austin"));
    assert_eq!(
        visible_people(&roster(), "denver")
            .iter()
            .map(|p| p.id.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "c"]
    );
}

/// Office op-4c18j: a stale id must not make the panel name someone
/// nothing on screen identifies.
#[test]
fn only_a_selection_in_the_roster_is_nameable() {
    let roster = roster();
    assert_eq!(
        nameable_selection(&roster, Some("b")).map(|p| p.id.as_str()),
        Some("b")
    );
    assert_eq!(nameable_selection(&roster, Some("stale")), None);
    assert_eq!(nameable_selection(&roster, None), None);
}

/// Exactly one reachable card: focused if visible, else selected if
/// visible, else the first -- never none while anyone is visible.
#[test]
fn the_tab_stop_falls_back_so_a_visible_list_is_never_unreachable() {
    let all = roster();
    assert_eq!(
        tab_stop_id(&all, Some("b"), Some("c")).as_deref(),
        Some("b"),
        "focused wins"
    );
    assert_eq!(
        tab_stop_id(&all, None, Some("c")).as_deref(),
        Some("c"),
        "then selected"
    );
    assert_eq!(
        tab_stop_id(&all, None, None).as_deref(),
        Some("a"),
        "then first"
    );
    let filtered = visible_people(&all, "austin");
    assert_eq!(
        tab_stop_id(&filtered, Some("a"), Some("c")).as_deref(),
        Some("b"),
        "focused and selected both filtered out: first visible, not none"
    );
    assert_eq!(tab_stop_id(&[], Some("a"), None), None, "nobody visible");
}

#[test]
fn arrow_keys_step_without_wrapping_and_home_end_jump() {
    let all = roster();
    assert_eq!(Step::from_key("ArrowDown"), Some(Step::Next));
    assert_eq!(Step::from_key("ArrowUp"), Some(Step::Previous));
    assert_eq!(Step::from_key("Home"), Some(Step::First));
    assert_eq!(Step::from_key("End"), Some(Step::Last));
    assert_eq!(
        Step::from_key("ArrowRight"),
        None,
        "a vertical list ignores left/right"
    );
    assert_eq!(step_id(&all, "a", Step::Next).as_deref(), Some("b"));
    assert_eq!(
        step_id(&all, "c", Step::Next).as_deref(),
        Some("c"),
        "no wrap at the end"
    );
    assert_eq!(
        step_id(&all, "a", Step::Previous).as_deref(),
        Some("a"),
        "no wrap at the start"
    );
    assert_eq!(step_id(&all, "b", Step::Last).as_deref(), Some("c"));
    assert_eq!(step_id(&all, "c", Step::First).as_deref(), Some("a"));
}

#[test]
fn english_and_spanish_texts_differ_and_name_every_presence() {
    let en = PersonPickerTexts::default();
    let es = PersonPickerTexts::es();
    assert_ne!(en.search_placeholder, es.search_placeholder);
    for presence in [
        PersonPresence::Available,
        PersonPresence::Busy,
        PersonPresence::Away,
        PersonPresence::Offline,
        PersonPresence::Unknown,
    ] {
        assert!(!en.presence(presence).is_empty());
        assert_ne!(en.presence(presence), es.presence(presence));
    }
}

/// op-wkppl: "has not signed in today" must never read as "went offline".
/// Offline claims a known state and gets a dot; Unknown has no state to
/// claim, so it gets no dot and a different word.
#[test]
fn unknown_is_not_offline() {
    assert!(PersonPresence::Offline.dot_class().is_some());
    assert_eq!(PersonPresence::Unknown.dot_class(), None);
    let en = PersonPickerTexts::default();
    assert_ne!(
        en.presence(PersonPresence::Unknown),
        en.presence(PersonPresence::Offline)
    );
    assert_eq!(en.presence(PersonPresence::Unknown), "Not signed in");
}
