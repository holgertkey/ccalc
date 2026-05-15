//! SVG and PNG file export via `plotters`.

#![cfg(feature = "plot-svg")]

use plotters::prelude::*;
use plotters::series::LineSeries;

use crate::FigureState;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

enum ChartKind {
    Line,
    Scatter,
}

/// Writes an SVG or PNG line plot to `path`, routing on the file extension.
pub(crate) fn render_line(
    x: &[f64],
    y: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    render_file(ChartKind::Line, x, y, path, state)
}

/// Writes an SVG or PNG scatter plot to `path`, routing on the file extension.
pub(crate) fn render_scatter(
    x: &[f64],
    y: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    render_file(ChartKind::Scatter, x, y, path, state)
}

fn render_file(
    kind: ChartKind,
    x: &[f64],
    y: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_chart(kind, x, y, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_chart(kind, x, y, &state, root)
    } else {
        Err(format!("file: unsupported format '{path}'"))
    }
}

fn draw_chart<DB: DrawingBackend>(
    kind: ChartKind,
    x: &[f64],
    y: &[f64],
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let (x_min, x_max) = range_with_margin(x);
    let (y_min, y_max) = range_with_margin(y);

    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("");

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    chart
        .configure_mesh()
        .x_desc(xlabel)
        .y_desc(ylabel)
        .draw()
        .map_err(|e| e.to_string())?;

    let points: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)).collect();

    match kind {
        ChartKind::Line => {
            chart
                .draw_series(LineSeries::new(points, &BLUE))
                .map_err(|e| e.to_string())?;
        }
        ChartKind::Scatter => {
            chart
                .draw_series(points.iter().map(|&(xi, yi)| {
                    Circle::new((xi, yi), 3, BLUE.filled())
                }))
                .map_err(|e| e.to_string())?;
        }
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

fn range_with_margin(vals: &[f64]) -> (f64, f64) {
    let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = hi - lo;
    if span.abs() < f64::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        let margin = span * 0.05;
        (lo - margin, hi + margin)
    }
}
