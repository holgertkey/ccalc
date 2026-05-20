//! SVG and PNG file export via `plotters`.

#![cfg(feature = "plot-svg")]

use plotters::prelude::*;
use plotters::series::LineSeries;

use crate::FigureState;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// Octave-style colour cycle for multi-series plots.
const SERIES_COLORS: [RGBColor; 7] = [
    RGBColor(0, 114, 189),  // blue
    RGBColor(217, 83, 25),  // orange
    RGBColor(237, 177, 32), // yellow
    RGBColor(126, 47, 142), // purple
    RGBColor(119, 172, 48), // green
    RGBColor(77, 190, 238), // cyan
    RGBColor(162, 20, 47),  // dark red
];

enum ChartKind {
    Line,
    Scatter,
    Bar,
    Stem,
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

/// Writes an SVG or PNG multi-series line plot to `path`.
///
/// Each element of `ys` is one series; colors cycle through [`SERIES_COLORS`].
/// If `state.legend` is set its entries label the corresponding series.
pub(crate) fn render_multi_line(
    x: &[f64],
    ys: &[Vec<f64>],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_multi_line_chart(x, ys, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_multi_line_chart(x, ys, &state, root)
    } else {
        Err(format!("plot: unsupported format '{path}'"))
    }
}

fn draw_multi_line_chart<DB: DrawingBackend>(
    x: &[f64],
    ys: &[Vec<f64>],
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| range_with_margin(x));

    // Y range spans all series.
    let all_y: Vec<f64> = ys.iter().flat_map(|v| v.iter().copied()).collect();
    let (y_min, y_max) = state.ylim.unwrap_or_else(|| range_with_margin(&all_y));

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

    if state.grid {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .draw()
            .map_err(|e| e.to_string())?;
    } else {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .disable_mesh()
            .draw()
            .map_err(|e| e.to_string())?;
    }

    let has_legend = !state.legend.is_empty();
    let clip_x = state.xlim.is_some();
    let clip_y = state.ylim.is_some();

    for (i, y_series) in ys.iter().enumerate() {
        let color = SERIES_COLORS[i % SERIES_COLORS.len()];
        let points: Vec<(f64, f64)> = x
            .iter()
            .zip(y_series)
            .filter_map(|(&xi, &yi)| {
                if clip_x && (xi < x_min || xi > x_max) {
                    return None;
                }
                if clip_y && (yi < y_min || yi > y_max) {
                    return None;
                }
                Some((xi, yi))
            })
            .collect();
        let series_ref = chart
            .draw_series(LineSeries::new(points, &color))
            .map_err(|e| e.to_string())?;
        if has_legend {
            let label = state.legend.get(i).map(|s| s.as_str()).unwrap_or("");
            if !label.is_empty() {
                series_ref
                    .label(label)
                    .legend(move |(lx, ly)| PathElement::new(vec![(lx, ly), (lx + 20, ly)], color));
            }
        }
    }

    if has_legend {
        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| e.to_string())?;
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

/// Writes an SVG or PNG histogram to `path`, routing on the file extension.
pub(crate) fn render_hist(
    counts: &[usize],
    edges: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_hist_chart(counts, edges, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_hist_chart(counts, edges, &state, root)
    } else {
        Err(format!("hist: unsupported format '{path}'"))
    }
}

fn draw_hist_chart<DB: DrawingBackend>(
    counts: &[usize],
    edges: &[f64],
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let x_min = state
        .xlim
        .map(|(lo, _)| lo)
        .unwrap_or_else(|| *edges.first().unwrap_or(&0.0));
    let x_max = state
        .xlim
        .map(|(_, hi)| hi)
        .unwrap_or_else(|| *edges.last().unwrap_or(&1.0));
    let max_count = counts.iter().copied().max().unwrap_or(1).max(1) as f64;
    let y_min = state.ylim.map(|(lo, _)| lo).unwrap_or(0.0);
    let y_max = state.ylim.map(|(_, hi)| hi).unwrap_or(max_count * 1.05);

    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("count");

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    if state.grid {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .draw()
            .map_err(|e| e.to_string())?;
    } else {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .disable_mesh()
            .draw()
            .map_err(|e| e.to_string())?;
    }

    // Edge-to-edge bars (no gap between adjacent bins).
    chart
        .draw_series((0..counts.len()).map(|i| {
            Rectangle::new(
                [(edges[i], 0.0), (edges[i + 1], counts[i] as f64)],
                BLUE.filled(),
            )
        }))
        .map_err(|e| e.to_string())?;

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

/// Writes an SVG or PNG bar chart to `path`, routing on the file extension.
pub(crate) fn render_bar(
    x: &[f64],
    y: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    render_file(ChartKind::Bar, x, y, path, state)
}

/// Writes an SVG or PNG stem plot to `path`, routing on the file extension.
pub(crate) fn render_stem(
    x: &[f64],
    y: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    render_file(ChartKind::Stem, x, y, path, state)
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

    // Bar and stem charts always include y = 0 in the y axis.
    let zero_baseline = matches!(kind, ChartKind::Bar | ChartKind::Stem);

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| range_with_margin(x));
    let (y_min, y_max) = state.ylim.unwrap_or_else(|| {
        if zero_baseline {
            range_with_zero_baseline(y)
        } else {
            range_with_margin(y)
        }
    });

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

    if state.grid {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .draw()
            .map_err(|e| e.to_string())?;
    } else {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .disable_mesh()
            .draw()
            .map_err(|e| e.to_string())?;
    }

    // When xlim/ylim is set explicitly, clip data to the visible range so
    // that no segment is drawn through the chart boundary (which would create
    // a visible hard cut-off artefact at the edge).
    let clip_x = state.xlim.is_some();
    let clip_y = state.ylim.is_some() && !zero_baseline;
    let points: Vec<(f64, f64)> = x
        .iter()
        .zip(y.iter())
        .filter_map(|(&xi, &yi)| {
            if clip_x && (xi < x_min || xi > x_max) {
                return None;
            }
            if clip_y && (yi < y_min || yi > y_max) {
                return None;
            }
            Some((xi, yi))
        })
        .collect();

    match kind {
        ChartKind::Line => {
            chart
                .draw_series(LineSeries::new(points, &BLUE))
                .map_err(|e| e.to_string())?;
        }
        ChartKind::Scatter => {
            chart
                .draw_series(
                    points
                        .iter()
                        .map(|&(xi, yi)| Circle::new((xi, yi), 3, BLUE.filled())),
                )
                .map_err(|e| e.to_string())?;
        }
        ChartKind::Bar => {
            let bar_w = bar_half_width(x, x_min, x_max);
            chart
                .draw_series(x.iter().zip(y.iter()).map(|(&xi, &yi)| {
                    let (y_lo, y_hi) = if yi >= 0.0 { (0.0, yi) } else { (yi, 0.0) };
                    Rectangle::new([(xi - bar_w, y_lo), (xi + bar_w, y_hi)], BLUE.filled())
                }))
                .map_err(|e| e.to_string())?;
        }
        ChartKind::Stem => {
            // Vertical lines from y=0 to each tip.
            for (&xi, &yi) in x.iter().zip(y.iter()) {
                chart
                    .draw_series(std::iter::once(PathElement::new(
                        vec![(xi, 0.0), (xi, yi)],
                        BLUE,
                    )))
                    .map_err(|e| e.to_string())?;
            }
            // Tip circles.
            chart
                .draw_series(
                    x.iter()
                        .zip(y.iter())
                        .map(|(&xi, &yi)| Circle::new((xi, yi), 4, BLUE.filled())),
                )
                .map_err(|e| e.to_string())?;
        }
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

// ── 3D file rendering ──────────────────────────────────────────────────────

/// Writes an SVG or PNG 3D line plot to `path`, routing on the file extension.
pub(crate) fn render_plot3(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_3d_chart(false, x, y, z, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_3d_chart(false, x, y, z, &state, root)
    } else {
        Err(format!("plot3: unsupported format '{path}'"))
    }
}

/// Writes an SVG or PNG 3D scatter plot to `path`, routing on the file extension.
pub(crate) fn render_scatter3(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_3d_chart(true, x, y, z, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_3d_chart(true, x, y, z, &state, root)
    } else {
        Err(format!("scatter3: unsupported format '{path}'"))
    }
}

fn draw_3d_chart<DB: DrawingBackend>(
    scatter: bool,
    x: &[f64],
    y: &[f64],
    z: &[f64],
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| range_with_margin(x));
    let (y_min, y_max) = state.ylim.unwrap_or_else(|| range_with_margin(y));
    let (z_min, z_max) = state.zlim.unwrap_or_else(|| range_with_margin(z));

    let title = state.title.as_deref().unwrap_or("");

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(30)
        .build_cartesian_3d(x_min..x_max, y_min..y_max, z_min..z_max)
        .map_err(|e| e.to_string())?;

    chart.configure_axes().draw().map_err(|e| e.to_string())?;

    if scatter {
        chart
            .draw_series(
                x.iter()
                    .zip(y.iter())
                    .zip(z.iter())
                    .map(|((&xi, &yi), &zi)| Circle::new((xi, yi, zi), 3, BLUE.filled())),
            )
            .map_err(|e| e.to_string())?;
    } else {
        chart
            .draw_series(LineSeries::new(
                x.iter()
                    .zip(y.iter())
                    .zip(z.iter())
                    .map(|((&xi, &yi), &zi)| (xi, yi, zi)),
                &BLUE,
            ))
            .map_err(|e| e.to_string())?;
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

// ── subplot / savefig rendering ────────────────────────────────────────────

/// Renders a list of [`Panel`]s into a single SVG or PNG file.
///
/// Grid dimensions are inferred from the maximum row/column indices found in
/// panel layouts. A single panel without a layout fills the whole canvas.
pub(crate) fn render_subplot_panels(panels: &[crate::Panel], path: &str) -> Result<(), String> {
    let rows = panels
        .iter()
        .filter_map(|p| p.layout.map(|(r, _, _)| r))
        .max()
        .unwrap_or(1);
    let cols = panels
        .iter()
        .filter_map(|p| p.layout.map(|(_, c, _)| c))
        .max()
        .unwrap_or(1);

    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_panels(panels, rows, cols, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_panels(panels, rows, cols, root)
    } else {
        Err(format!("savefig: unsupported format '{path}'"))
    }
}

fn draw_panels<DB: DrawingBackend>(
    panels: &[crate::Panel],
    rows: u32,
    cols: u32,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;
    let sub_areas = root.split_evenly((rows as usize, cols as usize));

    for panel in panels {
        let idx = panel
            .layout
            .map(|(_, _, k)| k.saturating_sub(1) as usize)
            .unwrap_or(0);
        let area = &sub_areas[idx.min(sub_areas.len().saturating_sub(1))];
        draw_panel(panel, area)?;
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

fn draw_panel<DB: DrawingBackend>(
    panel: &crate::Panel,
    area: &DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    use crate::PendingSeries;

    if panel.series.is_empty() {
        return Ok(());
    }

    // Collect coordinate bounds across all series.
    let mut all_x: Vec<f64> = Vec::new();
    let mut all_y: Vec<f64> = Vec::new();
    let mut has_zero_baseline = false;

    for series in &panel.series {
        match series {
            PendingSeries::Line(x, y) | PendingSeries::Scatter(x, y) => {
                all_x.extend_from_slice(x);
                all_y.extend_from_slice(y);
            }
            PendingSeries::Bar(x, y) | PendingSeries::Stem(x, y) => {
                all_x.extend_from_slice(x);
                all_y.extend_from_slice(y);
                has_zero_baseline = true;
            }
            PendingSeries::Hist { counts, edges } => {
                all_x.extend_from_slice(edges);
                all_y.push(0.0);
                all_y.push(counts.iter().copied().max().unwrap_or(0) as f64);
                has_zero_baseline = true;
            }
        }
    }

    let (x_min, x_max) = panel.xlim.unwrap_or_else(|| range_with_margin(&all_x));
    let (y_min, y_max) = panel.ylim.unwrap_or_else(|| {
        if has_zero_baseline {
            range_with_zero_baseline(&all_y)
        } else {
            range_with_margin(&all_y)
        }
    });

    let title = panel.title.as_deref().unwrap_or("");
    let xlabel = panel.xlabel.as_deref().unwrap_or("");
    let ylabel = panel.ylabel.as_deref().unwrap_or("");

    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 16))
        .margin(15)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    if panel.grid {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .draw()
            .map_err(|e| e.to_string())?;
    } else {
        chart
            .configure_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .disable_mesh()
            .draw()
            .map_err(|e| e.to_string())?;
    }

    for (i, series) in panel.series.iter().enumerate() {
        let color = SERIES_COLORS[i % SERIES_COLORS.len()];
        match series {
            PendingSeries::Line(x, y) => {
                chart
                    .draw_series(LineSeries::new(
                        x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)),
                        &color,
                    ))
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Scatter(x, y) => {
                chart
                    .draw_series(
                        x.iter()
                            .zip(y.iter())
                            .map(|(&xi, &yi)| Circle::new((xi, yi), 3, color.filled())),
                    )
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Bar(x, y) => {
                let bar_w = bar_half_width(x, x_min, x_max);
                chart
                    .draw_series(x.iter().zip(y.iter()).map(|(&xi, &yi)| {
                        let (y_lo, y_hi) = if yi >= 0.0 { (0.0, yi) } else { (yi, 0.0) };
                        Rectangle::new([(xi - bar_w, y_lo), (xi + bar_w, y_hi)], color.filled())
                    }))
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Stem(x, y) => {
                for (&xi, &yi) in x.iter().zip(y.iter()) {
                    chart
                        .draw_series(std::iter::once(PathElement::new(
                            vec![(xi, 0.0), (xi, yi)],
                            color,
                        )))
                        .map_err(|e| e.to_string())?;
                }
                chart
                    .draw_series(
                        x.iter()
                            .zip(y.iter())
                            .map(|(&xi, &yi)| Circle::new((xi, yi), 3, color.filled())),
                    )
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Hist { counts, edges } => {
                chart
                    .draw_series((0..counts.len()).map(|j| {
                        Rectangle::new(
                            [(edges[j], 0.0), (edges[j + 1], counts[j] as f64)],
                            color.filled(),
                        )
                    }))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

/// Y range for bar/stem: always includes y = 0 as the baseline.
fn range_with_zero_baseline(vals: &[f64]) -> (f64, f64) {
    let lo = vals.iter().copied().fold(f64::INFINITY, f64::min).min(0.0);
    let hi = vals
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);
    let span = hi - lo;
    if span.abs() < f64::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        let margin = span * 0.05;
        (lo - margin, hi + margin)
    }
}

/// Half-width of a bar column: 40% of the minimum x-spacing.
fn bar_half_width(x: &[f64], x_min: f64, x_max: f64) -> f64 {
    if x.len() > 1 {
        x.windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(f64::INFINITY, f64::min)
            * 0.4
    } else {
        ((x_max - x_min).abs() * 0.1).max(0.4)
    }
}
