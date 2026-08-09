use super::paint::{paint_attrs, stroke_attrs};
use leptos::prelude::*;

/// SVG-based area chart component.
///
/// Renders a line with a filled polygon underneath for area visualization.
#[component]
pub fn AreaChart(
    /// Data points as (x, y) pairs.
    data: Vec<(f64, f64)>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 400)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 200)]
    height: u32,
    /// Fill color for the area beneath the line.
    #[prop(default = "oklch(0.65 0.2 250)".to_string())]
    fill_color: String,
    /// Stroke color for the line on top of the area.
    #[prop(default = "oklch(0.55 0.2 250)".to_string())]
    stroke_color: String,
    /// Opacity of the filled area (0.0 - 1.0).
    #[prop(default = 0.3)]
    fill_opacity: f64,
) -> impl IntoView {
    let viewbox = format!("0 0 {width} {height}");

    if data.is_empty() {
        return view! {
            <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                <text x=format!("{}", width / 2) y=format!("{}", height / 2)
                    text-anchor="middle" fill="currentColor" font-size="14">
                    "No data"
                </text>
            </svg>
        }
        .into_any();
    }

    let pad_left: f64 = 40.0;
    let pad_right: f64 = 20.0;
    let pad_top: f64 = 15.0;
    let pad_bottom: f64 = 35.0;

    let chart_w = width as f64 - pad_left - pad_right;
    let chart_h = height as f64 - pad_top - pad_bottom;

    let x_min = data.iter().map(|d| d.0).fold(f64::INFINITY, f64::min);
    let x_max = data.iter().map(|d| d.0).fold(f64::NEG_INFINITY, f64::max);
    let y_min = data.iter().map(|d| d.1).fold(f64::INFINITY, f64::min);
    let y_max = data.iter().map(|d| d.1).fold(f64::NEG_INFINITY, f64::max);

    let x_range = if (x_max - x_min).abs() < f64::EPSILON {
        1.0
    } else {
        x_max - x_min
    };
    let y_range = if (y_max - y_min).abs() < f64::EPSILON {
        1.0
    } else {
        y_max - y_min
    };

    let to_svg = |x: f64, y: f64| -> (f64, f64) {
        let sx = pad_left + (x - x_min) / x_range * chart_w;
        let sy = pad_top + chart_h - (y - y_min) / y_range * chart_h;
        (sx, sy)
    };

    // Build the polyline points for the stroke
    let line_points: String = data
        .iter()
        .map(|&(x, y)| {
            let (sx, sy) = to_svg(x, y);
            format!("{sx:.2},{sy:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Build polygon points: line points + two bottom corners to close the area
    let baseline_y = pad_top + chart_h;
    let first_sx = pad_left + (data[0].0 - x_min) / x_range * chart_w;
    let last_sx = pad_left + (data[data.len() - 1].0 - x_min) / x_range * chart_w;

    let polygon_points =
        format!("{first_sx:.2},{baseline_y:.2} {line_points} {last_sx:.2},{baseline_y:.2}");

    // Axis ticks
    let y_tick_views = (0..=4)
        .map(|i| {
            let frac = i as f64 / 4.0;
            let val = y_min + frac * y_range;
            let sy = pad_top + chart_h - frac * chart_h;
            let x = format!("{:.2}", pad_left - 5.0);
            let y = format!("{sy:.2}");
            let label = format!("{val:.1}");
            view! {
                <text x=x y=y text-anchor="end" dominant-baseline="middle"
                    fill="currentColor" font-size="10" opacity="0.6">
                    {label}
                </text>
            }
        })
        .collect_view();

    let x_tick_views = (0..=4)
        .map(|i| {
            let frac = i as f64 / 4.0;
            let val = x_min + frac * x_range;
            let sx = pad_left + frac * chart_w;
            let x = format!("{sx:.2}");
            let y = format!("{:.2}", baseline_y + 15.0);
            let label = format!("{val:.1}");
            view! {
                <text x=x y=y text-anchor="middle"
                    fill="currentColor" font-size="10" opacity="0.6">
                    {label}
                </text>
            }
        })
        .collect_view();

    // Neither colour may ride on its presentation attribute when it carries a
    // theme token — see `super::paint` (ldui-1g5). They land on two different
    // elements, so the two `style` attributes never collide.
    let (area_fill, area_style) = paint_attrs(fill_color);
    let (line_stroke, line_style) = stroke_attrs(stroke_color);

    let fill_opacity_str = format!("{fill_opacity}");
    let axis_y_end = format!("{baseline_y:.2}");
    let pad_left_str = format!("{pad_left:.2}");
    let pad_top_str = format!("{pad_top:.2}");
    let axis_x_end = format!("{:.2}", pad_left + chart_w);

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            <line
                x1=pad_left_str.clone()
                y1=pad_top_str
                x2=pad_left_str.clone()
                y2=axis_y_end.clone()
                stroke="currentColor"
                stroke-opacity="0.3"
                stroke-width="1"
            />
            <line
                x1=pad_left_str
                y1=axis_y_end.clone()
                x2=axis_x_end
                y2=axis_y_end
                stroke="currentColor"
                stroke-opacity="0.3"
                stroke-width="1"
            />
            {y_tick_views}
            {x_tick_views}
            <polygon
                points=polygon_points
                fill=area_fill
                style=area_style
                opacity=fill_opacity_str
            />
            <polyline
                points=line_points
                fill="none"
                stroke=line_stroke
                style=line_style
                stroke-width="2"
                stroke-linejoin="round"
                stroke-linecap="round"
            />
        </svg>
    }
    .into_any()
}
