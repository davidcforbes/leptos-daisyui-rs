//! Lightweight SVG chart components — line, bar, pie, sparkline, stacked-bar,
//! and area. Pure Leptos + SVG with primitive-only props (no canvas, no JS
//! charting dependency), so they render server- or client-side and scale
//! crisply. Promoted from the euc frontend so any Leptos/daisyUI app can use
//! the same charts.

mod area_chart;
mod bar_chart;
mod line_chart;
mod pie_chart;
mod sparkline;
mod stacked_bar_chart;

pub use area_chart::AreaChart;
pub use bar_chart::BarChart;
pub use line_chart::LineChart;
pub use pie_chart::{PieChart, PieSlice};
pub use sparkline::Sparkline;
pub use stacked_bar_chart::{ChartSeries, StackedBarChart};
