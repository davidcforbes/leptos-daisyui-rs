//! Data and pure rules for `PersonPicker`. Everything here is testable
//! without a DOM; the components only wire it up.

/// One person on the roster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerPerson {
    /// Stable key (Office: `worker_ref`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Office, role or similar.
    pub secondary: String,
    /// Supplied, not derived: correct initials across every culture's names
    /// is hard, and consumers already have them.
    pub initials: String,
    /// Optional availability.
    pub presence: Option<PersonPresence>,
}

/// Availability. Rendered as a dot AND a word -- a dot alone is
/// colour-only information (WCAG 1.4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonPresence {
    /// Free to take work.
    Available,
    /// Occupied.
    Busy,
    /// Temporarily away.
    Away,
    /// Not signed in.
    Offline,
}

impl PersonPresence {
    /// The dot's fill class.
    pub fn dot_class(self) -> &'static str {
        match self {
            Self::Available => "bg-success",
            Self::Busy => "bg-error",
            Self::Away => "bg-warning",
            Self::Offline => "bg-base-300",
        }
    }

    /// Stable data-hook value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Busy => "busy",
            Self::Away => "away",
            Self::Offline => "offline",
        }
    }
}

/// The selection a click or Space/Enter on `clicked` proposes: deselect
/// when it is already selected, otherwise select it.
pub fn next_selection(current: Option<&str>, clicked: &str) -> Option<String> {
    if current == Some(clicked) {
        None
    } else {
        Some(clicked.to_owned())
    }
}

/// Case-insensitive substring match on name or secondary line. A blank
/// query matches everyone.
pub fn matches_search(person: &PickerPerson, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || person.name.to_lowercase().contains(&query)
        || person.secondary.to_lowercase().contains(&query)
}

/// The roster as the search currently shows it, in roster order.
pub(crate) fn visible_people(roster: &[PickerPerson], query: &str) -> Vec<PickerPerson> {
    roster
        .iter()
        .filter(|person| matches_search(person, query))
        .cloned()
        .collect()
}

/// The selected person, but only if they are on the roster. A stale id the
/// roster no longer contains yields `None` (Office op-4c18j).
pub fn nameable_selection<'a>(
    roster: &'a [PickerPerson],
    selected: Option<&str>,
) -> Option<&'a PickerPerson> {
    let selected = selected?;
    roster.iter().find(|person| person.id == selected)
}

/// The one visible option that carries `tabindex="0"`: the focused one if
/// visible, else the selected one if visible, else the first. Never `None`
/// while anyone is visible -- the invariant `Heatmap` and `BarChart` broke.
pub(crate) fn tab_stop_id(
    visible: &[PickerPerson],
    focused: Option<&str>,
    selected: Option<&str>,
) -> Option<String> {
    for wanted in [focused, selected].into_iter().flatten() {
        if visible.iter().any(|person| person.id == wanted) {
            return Some(wanted.to_owned());
        }
    }
    visible.first().map(|person| person.id.clone())
}

/// A listbox navigation step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Next,
    Previous,
    First,
    Last,
}

impl Step {
    /// A vertical listbox moves on Up/Down only.
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        match key {
            "ArrowDown" => Some(Self::Next),
            "ArrowUp" => Some(Self::Previous),
            "Home" => Some(Self::First),
            "End" => Some(Self::Last),
            _ => None,
        }
    }
}

/// Where a step moves focus. Does NOT wrap -- the APG listbox contract.
pub(crate) fn step_id(visible: &[PickerPerson], current: &str, step: Step) -> Option<String> {
    let last = visible.len().checked_sub(1)?;
    let index = visible.iter().position(|person| person.id == current);
    let target = match (step, index) {
        (Step::First, _) | (_, None) => 0,
        (Step::Last, _) => last,
        (Step::Next, Some(index)) => (index + 1).min(last),
        (Step::Previous, Some(index)) => index.saturating_sub(1),
    };
    visible.get(target).map(|person| person.id.clone())
}
