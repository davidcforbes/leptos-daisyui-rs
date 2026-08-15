//! Lightweight SVG chart components — line, bar, pie, sparkline, stacked-bar,
//! and area. Pure Leptos + SVG with primitive-only props (no canvas, no JS
//! charting dependency), so they render server- or client-side and scale
//! crisply. Promoted from the euc frontend so any Leptos/daisyUI app can use
//! the same charts.

mod area_chart;
mod bar_chart;
mod heatmap;
mod line_chart;
// Not just charts: any SVG paint in the crate routes through here, because the
// presentation-attribute hazard belongs to the attribute, not to the component.
pub(crate) mod paint;
mod pie_chart;
mod sparkline;
mod stacked_area_chart;
mod stacked_bar_chart;

pub use area_chart::AreaChart;
pub use bar_chart::BarChart;
pub use heatmap::{HeatScale, Heatmap, HeatmapCell};
pub use line_chart::{
    LineCategory, LineChart, LineChartActivation, LineChartActivationSource,
    LineChartActivationValue, LineChartData, LineChartDataSource, LineChartModifiers,
    LineInteractionMode, LineLegendMode, LinePattern, LinePoint, LineSeries, MarkerShape,
    MarkerStyle,
};
pub use pie_chart::{PieChart, PieSlice};
pub use sparkline::Sparkline;
pub use stacked_area_chart::StackedAreaChart;
pub use stacked_bar_chart::{ChartSeries, StackedBarChart};
