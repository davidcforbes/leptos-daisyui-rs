use super::style::SchedulerEventColor;

/// Hour-label clock format for the gutter labels. Ported from d2d-ui's
/// `DayScheduler::HourFormat`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HourFormat {
    /// 24-hour `HH:00` (default).
    #[default]
    TwentyFour,
    /// 12-hour `h AM/PM`.
    Twelve,
}

impl HourFormat {
    /// Format `hour` (0..=24) as a gutter label. Ported verbatim from
    /// d2d-ui's `HourFormat::label` (shared there with `WeekView`).
    pub fn label(self, hour: u32) -> String {
        match self {
            HourFormat::TwentyFour => format!("{hour:02}:00"),
            HourFormat::Twelve => {
                let h12 = match hour % 12 {
                    0 => 12,
                    h => h,
                };
                let ampm = if hour % 24 < 12 { "AM" } else { "PM" };
                format!("{h12} {ampm}")
            }
        }
    }
}

/// One scheduled block. `start_min` / `end_min` are minutes from midnight.
/// Mirrors d2d-ui's `SchedulerEvent`. (`Default` is a degenerate zero-length
/// midnight event — it exists for placeholder reads in reactive closures,
/// not as a sensible event; construct real events with
/// [`SchedulerEvent::new`].)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SchedulerEvent {
    /// Display title, shown inside the event block.
    pub title: String,
    /// Start time, in minutes from midnight.
    pub start_min: u32,
    /// End time, in minutes from midnight (always `> start_min`).
    pub end_min: u32,
    /// Semantic color for the event block's background tint and accent bar.
    pub color: SchedulerEventColor,
}

impl SchedulerEvent {
    /// Create a new event. `end_min` is clamped to at least `start_min + 1`
    /// so every event has a strictly positive, visible duration -- mirrors
    /// d2d-ui's `SchedulerEvent::new`.
    pub fn new(
        title: impl Into<String>,
        start_min: u32,
        end_min: u32,
        color: SchedulerEventColor,
    ) -> Self {
        Self {
            title: title.into(),
            start_min,
            end_min: end_min.max(start_min + 1),
            color,
        }
    }
}

/// `min` (minutes from midnight) as a `HH:MM` clock label — the vocabulary
/// of an event block's accessible name.
pub fn minute_label(min: u32) -> String {
    format!("{:02}:{:02}", (min / 60) % 24, min % 60)
}

/// The accessible name of an event block: title plus its time range, so a
/// screen-reader user hears "Standup, 09:00 to 09:15" rather than a bare
/// title floating in an unlabeled grid.
pub fn event_aria_label(ev: &SchedulerEvent) -> String {
    format!(
        "{}, {} to {}",
        ev.title,
        minute_label(ev.start_min),
        minute_label(ev.end_min)
    )
}

/// What a key press on a focused event block asks for. The mapping is pure
/// so the keyboard contract is unit-testable without a DOM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKeyIntent {
    /// Enter / Space — activate the event (open, edit — the consumer's verb).
    Activate,
    /// Arrow Up/Down — request the event move earlier/later by the given
    /// signed minute delta. The consumer owns the events and applies (or
    /// refuses) the move.
    Move(i32),
    /// Shift+Arrow Up/Down — request the event's end shrink/grow by the given
    /// signed minute delta.
    Resize(i32),
}

/// Map a key press on a focused event block to its [`EventKeyIntent`]:
/// Enter/Space activate, ArrowUp/ArrowDown move by `step_min`, and
/// Shift+Arrow resizes instead. Everything else (Tab above all) is `None` so
/// focus navigation keeps working.
pub fn event_key_intent(key: &str, shift: bool, step_min: u32) -> Option<EventKeyIntent> {
    let step = step_min.max(1) as i32;
    match key {
        "Enter" | " " => Some(EventKeyIntent::Activate),
        "ArrowUp" if shift => Some(EventKeyIntent::Resize(-step)),
        "ArrowDown" if shift => Some(EventKeyIntent::Resize(step)),
        "ArrowUp" => Some(EventKeyIntent::Move(-step)),
        "ArrowDown" => Some(EventKeyIntent::Move(step)),
        _ => None,
    }
}

/// Computed layout for one event's block, expressed as percentages so the
/// component can position it with plain CSS `top`/`left`/`width`/`height`.
/// `left_pct`/`width_pct` are relative to the scheduler's content column
/// (the area to the right of the hour gutter); `top_pct`/`height_pct` are
/// relative to the visible `start_hour..=end_hour` band.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EventLayout {
    /// Left offset, as a percentage of the content column's width.
    pub left_pct: f64,
    /// Width, as a percentage of the content column's width.
    pub width_pct: f64,
    /// Top offset, as a percentage of the visible hour band's height.
    pub top_pct: f64,
    /// Height, as a percentage of the visible hour band's height.
    pub height_pct: f64,
}

/// Percentage (0..=100) of `min` (minutes from midnight) within the
/// `start_hour..=end_hour` band, clamped to the band's edges. Mirrors
/// d2d-ui's pixel-space `DayScheduler::min_to_y`, expressed as a percentage
/// for CSS positioning instead of DIPs.
pub fn minute_to_percent(min: f64, start_hour: u32, end_hour: u32) -> f64 {
    let end_hour = end_hour.max(start_hour + 1);
    let start = start_hour as f64 * 60.0;
    let total = ((end_hour - start_hour) as f64 * 60.0).max(1.0);
    ((min - start) / total * 100.0).clamp(0.0, 100.0)
}

/// Lay `events` out into side-by-side lanes wherever they overlap in time,
/// and return each event's layout (index-aligned with `events`). Ported
/// near-verbatim from d2d-ui's `DayScheduler::event_rects`: events are
/// processed in start-time order and grown into a cluster while the next
/// event starts before the running cluster end; within a cluster, a greedy
/// first-fit assigns each event to the first lane whose previous occupant
/// has already ended (or opens a new lane). All events in a cluster share
/// the same lane count and width, even when not every pair in the cluster
/// actually overlaps in time (e.g. a chain A-B, B-C where A and C don't
/// overlap) -- this is d2d-ui's original behavior and is preserved here
/// rather than "improved" into optimal interval-graph packing.
pub fn compute_event_layout(
    events: &[SchedulerEvent],
    start_hour: u32,
    end_hour: u32,
) -> Vec<EventLayout> {
    let n = events.len();
    let mut layouts = vec![EventLayout::default(); n];

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| events[i].start_min);

    let mut i = 0;
    while i < order.len() {
        // Grow a cluster while the next event starts before the cluster ends.
        let mut cluster = vec![order[i]];
        let mut cluster_end = events[order[i]].end_min;
        let mut j = i + 1;
        while j < order.len() && events[order[j]].start_min < cluster_end {
            cluster.push(order[j]);
            cluster_end = cluster_end.max(events[order[j]].end_min);
            j += 1;
        }

        // Greedy lane assignment within the cluster.
        let mut lane_end: Vec<u32> = Vec::new();
        let mut lane_of = vec![0usize; cluster.len()];
        for (k, &ev) in cluster.iter().enumerate() {
            let s = events[ev].start_min;
            let li = match lane_end.iter().position(|&le| le <= s) {
                Some(li) => {
                    lane_end[li] = events[ev].end_min;
                    li
                }
                None => {
                    lane_end.push(events[ev].end_min);
                    lane_end.len() - 1
                }
            };
            lane_of[k] = li;
        }

        let lanes = lane_end.len().max(1) as f64;
        let lane_w = 100.0 / lanes;
        for (k, &ev) in cluster.iter().enumerate() {
            let top = minute_to_percent(events[ev].start_min as f64, start_hour, end_hour);
            let bottom = minute_to_percent(events[ev].end_min as f64, start_hour, end_hour);
            layouts[ev] = EventLayout {
                left_pct: lane_of[k] as f64 * lane_w,
                width_pct: lane_w,
                top_pct: top,
                height_pct: (bottom - top).max(0.0),
            };
        }
        i = j;
    }

    layouts
}

/// Effective pixel height of the scheduler grid: `height_px` when positive,
/// otherwise an auto-computed default of 60px per displayed hour across the
/// visible `start_hour..end_hour` band (at least one hour tall).
pub fn effective_height_px(height_px: f64, start_hour: u32, end_hour: u32) -> f64 {
    if height_px > 0.0 {
        height_px
    } else {
        let end_hour = end_hour.max(start_hour + 1);
        (end_hour - start_hour) as f64 * 60.0
    }
}
