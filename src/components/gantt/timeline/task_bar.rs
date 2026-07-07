use chrono::{DateTime, Utc};
use leptos::prelude::*;

use crate::components::gantt::{GanttTask, TaskType};

/// Task bar component that renders a horizontal bar on the timeline.
///
/// Tasks with `task_type: TaskType::Milestone` render as a rotated-square
/// (diamond) marker centered on their start date instead of a duration bar:
/// a zero-duration point-in-time indicator. The marker is filled once
/// `progress >= 1.0` ("done"), otherwise it renders as an outline. Regular
/// `Task`/`Project` types are unaffected and keep the duration-bar rendering.
///
/// # CSS Classes
/// Add to your `input.css` (covered by this crate's blanket
/// `@source "../src/**/*.rs"` scan, listed here for completeness):
/// ```css
/// @source inline("task-bar absolute rounded transition-all");
/// @source inline("cursor-pointer cursor-not-allowed opacity-60 hover:shadow-lg");
/// @source inline("ring-2 ring-primary ring-offset-2 shadow-xl");
/// @source inline("relative h-full");
/// @source inline("progress-overlay absolute left-0 top-0 h-full bg-black/20");
/// @source inline("task-label flex items-center px-2 text-xs font-medium text-white");
/// @source inline("absolute right-1 top-1/2 -translate-y-1/2 text-white/80");
/// @source inline("milestone-marker inset-0 rotate-45 rounded-sm border-2");
/// @source inline("milestone-label left-full ml-2 whitespace-nowrap text-base-content");
/// ```
#[component]
pub fn TaskBar(
    /// The task to render
    #[prop(into)]
    task: Signal<GanttTask>,

    /// Start date of the timeline (for calculating position)
    #[prop(into)]
    timeline_start: Signal<DateTime<Utc>>,

    /// Width per day in pixels
    #[prop(into, default=Signal::derive(|| 60))]
    column_width: Signal<u32>,

    /// Height of the task bar in pixels
    #[prop(into, default=Signal::derive(|| 30))]
    bar_height: Signal<u32>,

    /// Y-position of the task bar
    #[prop(into, default=Signal::derive(|| 0))]
    y_position: Signal<u32>,

    /// Callback when the task bar is clicked
    #[prop(into, default=None)]
    on_click: Option<Callback<String>>,

    /// Whether this task is currently selected
    #[prop(into, default=Signal::derive(|| false))]
    is_selected: Signal<bool>,

    /// Whether this task is read-only (non-editable)
    #[prop(into, default=Signal::derive(|| false))]
    is_read_only: Signal<bool>,
) -> impl IntoView {
    let position_width = Signal::derive(move || {
        let t = task.get();
        let timeline_start_val = timeline_start.get();
        let col_width = column_width.get();

        calculate_bar_position(&t, timeline_start_val, col_width)
    });

    let is_milestone_task = Signal::derive(move || is_milestone(&task.get()));

    let milestone_layout = Signal::derive(move || {
        let t = task.get();
        calculate_milestone_marker(
            &t,
            timeline_start.get(),
            column_width.get(),
            bar_height.get(),
        )
    });

    let marker_done = Signal::derive(move || milestone_is_done(task.get().progress));

    let bar_color = Signal::derive(move || {
        let t = task.get();
        t.color.clone().unwrap_or_else(|| "#3b82f6".to_string()) // Default blue
    });

    let left_px = Signal::derive(move || {
        if is_milestone_task.get() {
            milestone_layout.get().0
        } else {
            position_width.get().0
        }
    });
    let top_px = Signal::derive(move || {
        if is_milestone_task.get() {
            y_position.get() + milestone_layout.get().1
        } else {
            y_position.get()
        }
    });
    let width_px = Signal::derive(move || {
        if is_milestone_task.get() {
            milestone_layout.get().2
        } else {
            position_width.get().1
        }
    });
    let height_px = Signal::derive(move || {
        if is_milestone_task.get() {
            milestone_layout.get().2
        } else {
            bar_height.get()
        }
    });

    view! {
        <div
            class="task-bar absolute transition-all"
            class:rounded=move || !is_milestone_task.get()
            class:cursor-pointer=move || !is_read_only.get()
            class:cursor-not-allowed=move || is_read_only.get()
            class:opacity-60=move || is_read_only.get()
            class:hover:shadow-lg=move || !is_read_only.get()
            class:ring-2=move || is_selected.get()
            class:ring-primary=move || is_selected.get()
            class:ring-offset-2=move || is_selected.get()
            class:shadow-xl=move || is_selected.get()
            style:left=move || format!("{}px", left_px.get())
            style:width=move || format!("{}px", width_px.get())
            style:top=move || format!("{}px", top_px.get())
            style:height=move || format!("{}px", height_px.get())
            style:background-color=move || {
                if is_milestone_task.get() { "transparent".to_string() } else { bar_color.get() }
            }
            attr:aria-readonly=move || is_read_only.get().to_string()
            on:click=move |_| {
                if !is_read_only.get()
                    && let Some(ref cb) = on_click {
                        cb.run(task.get().id.clone());
                    }
            }
        >
            <Show
                when=move || is_milestone_task.get()
                fallback=move || view! {
                    <div class="relative h-full">
                        // Progress bar overlay
                        <div
                            class="progress-overlay absolute left-0 top-0 h-full rounded bg-black/20"
                            style:width=move || format!("{}%", task.get().progress * 100.0)
                        />

                        // Task name label
                        <div class="task-label absolute left-0 top-0 flex h-full items-center px-2 text-xs font-medium text-white">
                            {move || task.get().name.clone()}
                        </div>

                        // Read-only lock icon indicator
                        <Show when=move || is_read_only.get()>
                            <div class="absolute right-1 top-1/2 -translate-y-1/2 text-white/80">
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="12"
                                    height="12"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
                                    <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                                </svg>
                            </div>
                        </Show>
                    </div>
                }
            >
                // Diamond marker: a rotated square filled when the milestone
                // is done (progress >= 100%), outlined otherwise.
                <div
                    class="milestone-marker absolute inset-0 rotate-45 rounded-sm border-2"
                    style:background-color=move || {
                        if marker_done.get() { bar_color.get() } else { "transparent".to_string() }
                    }
                    style:border-color=move || bar_color.get()
                />

                // Task name label, rendered beside the marker (not rotated)
                // since the marker itself is too small to hold text.
                <div class="milestone-label absolute left-full top-1/2 ml-2 -translate-y-1/2 whitespace-nowrap text-xs font-medium text-base-content">
                    {move || task.get().name.clone()}
                </div>
            </Show>
        </div>
    }
}

/// Calculate the X position (in px) of a date relative to the timeline start.
fn timeline_x_position(
    date: DateTime<Utc>,
    timeline_start: DateTime<Utc>,
    column_width: u32,
) -> u32 {
    let days_from_start = (date - timeline_start).num_days();
    (days_from_start as u32) * column_width
}

/// Calculate the X position and width of a task bar based on its dates
fn calculate_bar_position(
    task: &GanttTask,
    timeline_start: DateTime<Utc>,
    column_width: u32,
) -> (u32, u32) {
    let x_position = timeline_x_position(task.start, timeline_start, column_width);

    // Calculate task duration in days
    let duration_days = (task.end - task.start).num_days();
    let width = (duration_days as u32) * column_width;

    (x_position, width.max(column_width / 2)) // Minimum width for visibility
}

/// Returns true when `task` is a zero-duration milestone marker rather than
/// a duration bar or project container.
fn is_milestone(task: &GanttTask) -> bool {
    task.task_type == TaskType::Milestone
}

/// A milestone is considered "done" once its progress reaches 100%; this
/// drives filled (done) vs outline (not done) marker styling.
fn milestone_is_done(progress: f32) -> bool {
    progress >= 1.0
}

/// Side length (px) of the milestone diamond marker for a given bar height.
/// Slightly smaller than the row's bar height so it reads as a distinct
/// shape, with a floor so compact rows still render a visible marker.
fn milestone_marker_size(bar_height: u32) -> u32 {
    ((bar_height as f64 * 0.7).round() as u32).max(8)
}

/// Layout for a milestone marker: `(left_px, top_inset_px, size_px)`.
/// `left_px` is measured from the timeline's left edge, centered on the
/// task's start date (the same anchor used for zero-duration bars).
/// `top_inset_px` is the offset from the row's top needed to vertically
/// center the (smaller) marker within `bar_height`.
fn calculate_milestone_marker(
    task: &GanttTask,
    timeline_start: DateTime<Utc>,
    column_width: u32,
    bar_height: u32,
) -> (u32, u32, u32) {
    let center_x = timeline_x_position(task.start, timeline_start, column_width);
    let size = milestone_marker_size(bar_height);
    let left = center_x.saturating_sub(size / 2);
    let top_inset = bar_height.saturating_sub(size) / 2;
    (left, top_inset, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_calculate_bar_position() {
        let timeline_start = chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let task = GanttTask {
            id: "task1".to_string(),
            name: "Test Task".to_string(),
            start: chrono::Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap(),
            end: chrono::Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0).unwrap(),
            progress: 0.5,
            ..Default::default()
        };

        let (x, width) = calculate_bar_position(&task, timeline_start, 60);

        // Task starts 4 days after timeline start
        assert_eq!(x, 4 * 60);

        // Task duration is 5 days
        assert_eq!(width, 5 * 60);
    }

    #[test]
    fn test_calculate_bar_position_minimum_width() {
        let timeline_start = chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let task = GanttTask {
            id: "task1".to_string(),
            name: "Milestone".to_string(),
            start: chrono::Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap(),
            end: chrono::Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap(), // Same day
            progress: 0.0,
            ..Default::default()
        };

        let (_, width) = calculate_bar_position(&task, timeline_start, 60);

        // Minimum width is half of column width
        assert_eq!(width, 30);
    }

    fn milestone_task(start_day: u32) -> GanttTask {
        GanttTask {
            id: "m1".to_string(),
            name: "Launch".to_string(),
            start: chrono::Utc
                .with_ymd_and_hms(2024, 1, start_day, 0, 0, 0)
                .unwrap(),
            end: chrono::Utc
                .with_ymd_and_hms(2024, 1, start_day, 0, 0, 0)
                .unwrap(),
            task_type: crate::components::gantt::TaskType::Milestone,
            ..Default::default()
        }
    }

    #[test]
    fn test_is_milestone_true_for_milestone_type() {
        assert!(is_milestone(&milestone_task(5)));
    }

    #[test]
    fn test_is_milestone_false_for_regular_task() {
        let task = GanttTask {
            task_type: crate::components::gantt::TaskType::Task,
            ..Default::default()
        };
        assert!(!is_milestone(&task));
    }

    #[test]
    fn test_is_milestone_false_for_project() {
        let task = GanttTask {
            task_type: crate::components::gantt::TaskType::Project,
            ..Default::default()
        };
        assert!(!is_milestone(&task));
    }

    #[test]
    fn test_milestone_is_done_at_full_progress() {
        assert!(milestone_is_done(1.0));
    }

    #[test]
    fn test_milestone_is_done_false_when_incomplete() {
        assert!(!milestone_is_done(0.0));
        assert!(!milestone_is_done(0.99));
    }

    #[test]
    fn test_milestone_marker_size_scales_with_bar_height() {
        // 70% of 30, rounded
        assert_eq!(milestone_marker_size(30), 21);
    }

    #[test]
    fn test_milestone_marker_size_has_floor_for_compact_rows() {
        // 70% of 8 rounds to 6, but floor is 8
        assert_eq!(milestone_marker_size(8), 8);
    }

    #[test]
    fn test_calculate_milestone_marker_position() {
        let timeline_start = chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let task = milestone_task(5);

        let (left, top_inset, size) = calculate_milestone_marker(&task, timeline_start, 60, 30);

        // center_x = 4 days * 60px = 240; size = 21; left = 240 - 21/2 = 230
        assert_eq!(size, 21);
        assert_eq!(left, 230);
        // top_inset = (30 - 21) / 2 = 4
        assert_eq!(top_inset, 4);
    }

    #[test]
    fn test_calculate_milestone_marker_at_timeline_start() {
        let timeline_start = chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let task = milestone_task(1);

        let (left, _top_inset, size) = calculate_milestone_marker(&task, timeline_start, 60, 30);

        // center_x = 0, so left is clamped to 0 rather than underflowing
        assert_eq!(left, 0);
        assert_eq!(size, 21);
    }
}
