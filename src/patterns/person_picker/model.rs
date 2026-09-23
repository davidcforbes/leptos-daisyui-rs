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
    /// Optional one-line caption of what the person is DOING right now
    /// ("Replying to #4127"). Deliberately separate from `presence`: doing
    /// is not availability, and must never become a presence state
    /// (4iiz-Office's `Viewing / Replying` focus dimension). Text, not
    /// markup, so a page cannot change the card's shape.
    pub activity: Option<String>,
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
    /// Was signed in; their session has ended (a closed tab).
    Offline,
    /// No presence record AT ALL -- has not signed in today. Deliberately
    /// distinct from `Offline`: rendering it as offline tells a leader that
    /// someone who simply has not opened the app yet has gone offline,
    /// which is confidently wrong data (4iiz-Office op-wkppl, where the
    /// server emits it for every roster member without a heartbeat).
    Unknown,
}

/// The single muted fill every stale presence dot uses (ldui-8hmy). Distinct
/// from every live fill, including `Offline`'s `bg-base-300`, so a test can
/// tell "greyed because stale" from "offline".
pub const STALE_PRESENCE_DOT_CLASS: &str = "bg-base-content/40";

impl PersonPresence {
    /// The dot's fill class, or `None` for no dot. `Unknown` gets NO dot: a
    /// dot claims a known state, and there is none to claim.
    pub fn dot_class(self) -> Option<&'static str> {
        match self {
            Self::Available => Some("bg-success"),
            Self::Busy => Some("bg-error"),
            Self::Away => Some("bg-warning"),
            Self::Offline => Some("bg-base-300"),
            Self::Unknown => None,
        }
    }

    /// The dot's fill class when the caller says presence is STALE
    /// (ldui-8hmy), or `None` for no dot. Every known state greys to one
    /// neutral fill -- the kept status is last-known, not current, so no
    /// colour may claim it -- while `Unknown` still gets no dot. Same size
    /// and position as the live dot; only the fill changes.
    pub fn stale_dot_class(self) -> Option<&'static str> {
        self.dot_class().map(|_| STALE_PRESENCE_DOT_CLASS)
    }

    /// [`dot_class`](Self::dot_class) or
    /// [`stale_dot_class`](Self::stale_dot_class), by staleness.
    pub fn dot_class_when(self, stale: bool) -> Option<&'static str> {
        if stale {
            self.stale_dot_class()
        } else {
            self.dot_class()
        }
    }

    /// Stable data-hook value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Busy => "busy",
            Self::Away => "away",
            Self::Offline => "offline",
            Self::Unknown => "unknown",
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
