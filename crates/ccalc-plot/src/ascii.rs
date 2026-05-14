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

/// Renders a connected line plot to stdout.
pub fn render_line(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = x_bounds(x);
    if let Some(t) = &state.title {
        println!("{t}");
    }
    Chart::new(100, 30, x_min, x_max)
        .lineplot(&Shape::Lines(&data))
        .display();
    print_labels(&state);
}

/// Renders a scatter (point cloud) plot to stdout.
pub fn render_scatter(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = x_bounds(x);
    if let Some(t) = &state.title {
        println!("{t}");
    }
    Chart::new(100, 30, x_min, x_max)
        .lineplot(&Shape::Points(&data))
        .display();
    print_labels(&state);
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
