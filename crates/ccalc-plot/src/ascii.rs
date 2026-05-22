//! ASCII terminal rendering via `textplots`.
//!
//! `textplots` 0.8 uses Braille characters (U+2800–U+28FF) to render data
//! points. The chart body must be rendered via the method chain
//! `.lineplot(...).display()` — calling `Display::fmt` directly on the chart
//! skips the canvas-population step and produces a blank output.
//!
//! Axis labels (title, xlabel, ylabel) are printed before/after the chart
//! via `println!`.

use textplots::{Chart, Plot, Shape};

use crate::FigureState;

// ── Shared helpers ─────────────────────────────────────────────────────────

/// Returns the textplots canvas size in Braille dots based on terminal dimensions.
///
/// textplots encodes 2 dots per terminal column and 4 dots per terminal row, so
/// `(cols * 2, rows * 2)` fills roughly half the terminal height with the chart.
fn chart_canvas() -> (u32, u32) {
    let cols = crate::term_cols();
    let rows = crate::term_rows();
    ((cols * 2) as u32, ((rows * 2).clamp(20, 120)) as u32)
}

fn chart_x_bounds(x: &[f64], state: &FigureState) -> (f32, f32) {
    state
        .xlim
        .map(|(lo, hi)| (lo as f32, hi as f32))
        .unwrap_or_else(|| x_bounds(x))
}

fn print_header(state: &FigureState) {
    if let Some(t) = &state.title {
        println!("{t}");
    }
}

/// Renders a connected line plot to stdout.
pub fn render_line(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Lines(&data))
        .display();
    print_labels(&state);
}

/// Renders a scatter (point cloud) plot to stdout.
pub fn render_scatter(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Points(&data))
        .display();
    print_labels(&state);
}

/// Renders a bar chart to stdout (vertical bars from baseline).
pub fn render_bar(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Bars(&data))
        .display();
    print_labels(&state);
}

/// Renders a stem plot to stdout (vertical markers at each data point).
pub fn render_stem(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    // textplots has no native stem shape — use Bars (filled from baseline) as
    // the closest ASCII approximation; file export produces proper thin stems.
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Bars(&data))
        .display();
    print_labels(&state);
}

/// Renders a filled polygon to stdout (bounding-box shaded with `░` chars).
///
/// textplots has no native polygon fill; we sample a grid of cells and shade
/// each one with `░` if its centre lies inside the polygon (even-odd rule).
pub fn render_fill(x: &[f64], y: &[f64], state: FigureState) {
    if x.is_empty() {
        return;
    }
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let y_min = y.iter().copied().fold(f64::INFINITY, f64::min) as f32;
    let y_max = y.iter().copied().fold(f64::NEG_INFINITY, f64::max) as f32;
    print_header(&state);
    let cols: usize = crate::term_cols().saturating_sub(2).max(10);
    let rows: usize = (crate::term_rows() / 2).max(5);
    let col_step = (x_max - x_min) / cols as f32;
    let row_step = if (y_max - y_min).abs() > f32::EPSILON {
        (y_max - y_min) / rows as f32
    } else {
        1.0
    };
    for r in (0..rows).rev() {
        let cy = y_min + (r as f32 + 0.5) * row_step;
        let mut row_str = String::new();
        for c in 0..cols {
            let cx = x_min + (c as f32 + 0.5) * col_step;
            if point_in_polygon(cx, cy, &data) {
                row_str.push('░');
            } else {
                row_str.push(' ');
            }
        }
        println!("|{row_str}|");
    }
    print_labels(&state);
}

/// Renders a filled area under curve to stdout.
pub fn render_area(x: &[f64], y: &[f64], state: FigureState) {
    if x.is_empty() {
        return;
    }
    // Build a closed polygon: outline + baseline.
    let mut poly_x = x.to_vec();
    let mut poly_y = y.to_vec();
    poly_x.push(*x.last().unwrap());
    poly_y.push(0.0);
    poly_x.push(x[0]);
    poly_y.push(0.0);
    render_fill(&poly_x, &poly_y, state);
}

/// Ray-casting point-in-polygon test (2D, even-odd rule).
fn point_in_polygon(px: f32, py: f32, polygon: &[(f32, f32)]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn print_labels(state: &FigureState) {
    if let Some(xl) = &state.xlabel {
        println!("x: {xl}");
    }
    if let Some(yl) = &state.ylabel {
        println!("y: {yl}");
    }
}

fn to_f32_pairs(x: &[f64], y: &[f64]) -> Vec<(f32, f32)> {
    x.iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (xi as f32, yi as f32))
        .collect()
}

fn x_bounds(xs: &[f64]) -> (f32, f32) {
    let lo = xs.iter().copied().fold(f64::INFINITY, f64::min) as f32;
    let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) as f32;
    if (hi - lo).abs() < f32::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        (lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // textplots 0.8 renders via .display() which prints to stdout directly.
    // These tests verify no-panic behaviour; capturing the Braille canvas
    // output requires stdout redirection and is deferred.

    #[test]
    fn test_render_line_no_panic() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&xi| xi * xi).collect();
        render_line(&x, &y, FigureState::default());
    }

    #[test]
    fn test_render_scatter_no_panic() {
        let x: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&xi| xi * 2.0).collect();
        render_scatter(&x, &y, FigureState::default());
    }

    #[test]
    fn test_single_point_no_panic() {
        // A single-point vector must not panic (x_bounds adds padding).
        render_line(&[5.0], &[3.0], FigureState::default());
    }

    #[test]
    fn test_figure_state_label_values() {
        // Verify the FigureState struct holds label values correctly before
        // the render function consumes it.  This is the "label threading" test.
        let state = FigureState {
            title: Some("My Plot".into()),
            xlabel: Some("time".into()),
            ylabel: Some("value".into()),
            ..FigureState::default()
        };
        assert_eq!(state.title.as_deref(), Some("My Plot"));
        assert_eq!(state.xlabel.as_deref(), Some("time"));
        assert_eq!(state.ylabel.as_deref(), Some("value"));
    }
}
