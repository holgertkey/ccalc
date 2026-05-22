//! SVG and PNG file export via `plotters`.

#![cfg(feature = "plot-svg")]

use plotters::prelude::*;
use plotters::series::LineSeries;

use crate::FigureState;
use crate::style::{LinestyleKind, StyleSpec};

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

/// Writes an SVG or PNG filled-polygon plot to `path`.
pub(crate) fn render_fill(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_polygon_chart(x, y, false, style, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_polygon_chart(x, y, false, style, &state, root)
    } else {
        Err(format!("fill: unsupported format '{path}'"))
    }
}

/// Writes an SVG or PNG area-under-curve plot to `path`.
pub(crate) fn render_area(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_polygon_chart(x, y, true, style, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_polygon_chart(x, y, true, style, &state, root)
    } else {
        Err(format!("area: unsupported format '{path}'"))
    }
}

fn draw_polygon_chart<DB: DrawingBackend>(
    x: &[f64],
    y: &[f64],
    area_mode: bool,
    style: Option<StyleSpec>,
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| range_with_margin(x));
    let y_with_zero: Vec<f64> = y.iter().copied().chain(std::iter::once(0.0)).collect();
    let (y_min, y_max) = state
        .ylim
        .unwrap_or_else(|| range_with_zero_baseline(&y_with_zero));

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
        .disable_mesh()
        .draw()
        .map_err(|e| e.to_string())?;

    let fill_color = style_to_rgb(&style).unwrap_or(RGBColor(0, 114, 189));

    // Build polygon vertices: outline + closing segment along y=0 for area mode.
    let mut pts: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)).collect();
    if area_mode && !pts.is_empty() {
        pts.push((*x.last().unwrap(), 0.0));
        pts.push((x[0], 0.0));
    }

    chart
        .draw_series(std::iter::once(Polygon::new(pts, fill_color.mix(0.4))))
        .map_err(|e| e.to_string())?;

    // Overlay outline.
    let outline_pts: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)).collect();
    chart
        .draw_series(LineSeries::new(outline_pts, &fill_color))
        .map_err(|e| e.to_string())?;

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

/// Converts an optional [`StyleSpec`] color to a plotters [`RGBColor`].
fn style_to_rgb(style: &Option<StyleSpec>) -> Option<RGBColor> {
    style
        .as_ref()
        .and_then(|s| s.color.as_ref())
        .map(|c| RGBColor(c.0, c.1, c.2))
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
            PendingSeries::Line(x, y, _) | PendingSeries::Scatter(x, y, _) => {
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
            PendingSeries::Fill(x, y, _) | PendingSeries::Area(x, y, _) => {
                all_x.extend_from_slice(x);
                all_y.extend_from_slice(y);
                all_y.push(0.0);
                has_zero_baseline = true;
            }
            PendingSeries::Quiver(x, y, u, v) => {
                all_x.extend_from_slice(x);
                all_x.extend(x.iter().zip(u.iter()).map(|(&xi, &ui)| xi + ui));
                all_y.extend_from_slice(y);
                all_y.extend(y.iter().zip(v.iter()).map(|(&yi, &vi)| yi + vi));
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
        let default_color = SERIES_COLORS[i % SERIES_COLORS.len()];
        match series {
            PendingSeries::Line(x, y, style) => {
                let color = style_to_rgb(style).unwrap_or(default_color);
                let linestyle = style
                    .as_ref()
                    .map(|s| s.linestyle)
                    .unwrap_or(LinestyleKind::Solid);
                let pts: Vec<(f64, f64)> =
                    x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)).collect();
                let n = pts.len();
                match linestyle {
                    LinestyleKind::Solid => {
                        chart
                            .draw_series(LineSeries::new(pts.iter().copied(), &color))
                            .map_err(|e| e.to_string())?;
                    }
                    LinestyleKind::Dashed => {
                        let dash = (n / 8).max(3);
                        let gap = (n / 16).max(2);
                        let mut i = 0;
                        while i < n {
                            let end = (i + dash).min(n);
                            if end > i + 1 {
                                chart
                                    .draw_series(LineSeries::new(
                                        pts[i..end].iter().copied(),
                                        &color,
                                    ))
                                    .map_err(|e| e.to_string())?;
                            }
                            i = end + gap;
                        }
                    }
                    LinestyleKind::DashDot => {
                        let dash = (n / 8).max(3);
                        let gap = (n / 25).max(1);
                        let mut i = 0;
                        while i < n {
                            let end = (i + dash).min(n);
                            if end > i + 1 {
                                chart
                                    .draw_series(LineSeries::new(
                                        pts[i..end].iter().copied(),
                                        &color,
                                    ))
                                    .map_err(|e| e.to_string())?;
                            }
                            i = end + gap;
                            if i < n {
                                chart
                                    .draw_series(std::iter::once(Circle::new(
                                        pts[i],
                                        3,
                                        color.filled(),
                                    )))
                                    .map_err(|e| e.to_string())?;
                            }
                            i += gap + 1;
                        }
                    }
                    LinestyleKind::Dotted => {
                        let step = (n / 25).max(1);
                        chart
                            .draw_series(
                                pts.iter()
                                    .step_by(step)
                                    .map(|&p| Circle::new(p, 2, color.filled())),
                            )
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
            PendingSeries::Scatter(x, y, style) => {
                let color = style_to_rgb(style).unwrap_or(default_color);
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
                        Rectangle::new(
                            [(xi - bar_w, y_lo), (xi + bar_w, y_hi)],
                            default_color.filled(),
                        )
                    }))
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Stem(x, y) => {
                for (&xi, &yi) in x.iter().zip(y.iter()) {
                    chart
                        .draw_series(std::iter::once(PathElement::new(
                            vec![(xi, 0.0), (xi, yi)],
                            default_color,
                        )))
                        .map_err(|e| e.to_string())?;
                }
                chart
                    .draw_series(
                        x.iter()
                            .zip(y.iter())
                            .map(|(&xi, &yi)| Circle::new((xi, yi), 3, default_color.filled())),
                    )
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Hist { counts, edges } => {
                chart
                    .draw_series((0..counts.len()).map(|j| {
                        Rectangle::new(
                            [(edges[j], 0.0), (edges[j + 1], counts[j] as f64)],
                            default_color.filled(),
                        )
                    }))
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Fill(x, y, style) | PendingSeries::Area(x, y, style) => {
                let is_area = matches!(series, PendingSeries::Area(..));
                let fill_color = style_to_rgb(style).unwrap_or(default_color);
                let mut pts: Vec<(f64, f64)> =
                    x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)).collect();
                if is_area && !pts.is_empty() {
                    pts.push((*x.last().unwrap(), 0.0));
                    pts.push((x[0], 0.0));
                }
                chart
                    .draw_series(std::iter::once(Polygon::new(pts, fill_color.mix(0.4))))
                    .map_err(|e| e.to_string())?;
                let outline: Vec<(f64, f64)> =
                    x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)).collect();
                chart
                    .draw_series(LineSeries::new(outline, &fill_color))
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Quiver(x, y, u, v) => {
                let scale = arrow_scale(x, y, u, v);
                let n = x.len();
                for j in 0..n {
                    let x0 = x[j];
                    let y0 = y[j];
                    let dx = u[j] * scale;
                    let dy = v[j] * scale;
                    let x1 = x0 + dx;
                    let y1 = y0 + dy;
                    chart
                        .draw_series(std::iter::once(PathElement::new(
                            vec![(x0, y0), (x1, y1)],
                            default_color,
                        )))
                        .map_err(|e| e.to_string())?;
                    let len = (dx * dx + dy * dy).sqrt();
                    if len > 1e-12 {
                        let ux = dx / len;
                        let uy = dy / len;
                        let head_len = len * 0.3;
                        let head_w = len * 0.15;
                        let bx = x1 - ux * head_len;
                        let by = y1 - uy * head_len;
                        chart
                            .draw_series(std::iter::once(Polygon::new(
                                vec![
                                    (x1, y1),
                                    (bx - uy * head_w, by + ux * head_w),
                                    (bx + uy * head_w, by - ux * head_w),
                                ],
                                default_color.filled(),
                            )))
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    draw_text_annotations(&mut chart, &panel.annotations)?;

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

// ── quiver rendering ───────────────────────────────────────────────────────

/// Writes an SVG or PNG quiver (vector-field) plot to `path`.
pub(crate) fn render_quiver(
    xs: &[f64],
    ys: &[f64],
    us: &[f64],
    vs: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_quiver_chart(xs, ys, us, vs, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
        draw_quiver_chart(xs, ys, us, vs, &state, root)
    } else {
        Err(format!("quiver: unsupported format '{path}'"))
    }
}

fn draw_quiver_chart<DB: DrawingBackend>(
    xs: &[f64],
    ys: &[f64],
    us: &[f64],
    vs: &[f64],
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    // Axis ranges span both origins and scaled tips.
    let scale = arrow_scale(xs, ys, us, vs);
    let tip_x: Vec<f64> = xs
        .iter()
        .zip(us.iter())
        .map(|(&x, &u)| x + u * scale)
        .collect();
    let tip_y: Vec<f64> = ys
        .iter()
        .zip(vs.iter())
        .map(|(&y, &v)| y + v * scale)
        .collect();
    let all_x: Vec<f64> = xs.iter().copied().chain(tip_x.iter().copied()).collect();
    let all_y: Vec<f64> = ys.iter().copied().chain(tip_y.iter().copied()).collect();
    let (x_min, x_max) = state.xlim.unwrap_or_else(|| range_with_margin(&all_x));
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

    chart
        .configure_mesh()
        .x_desc(xlabel)
        .y_desc(ylabel)
        .disable_mesh()
        .draw()
        .map_err(|e| e.to_string())?;

    let n = xs.len();
    for i in 0..n {
        let x0 = xs[i];
        let y0 = ys[i];
        let dx = us[i] * scale;
        let dy = vs[i] * scale;
        let x1 = x0 + dx;
        let y1 = y0 + dy;

        // Arrow shaft.
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(x0, y0), (x1, y1)],
                BLUE,
            )))
            .map_err(|e| e.to_string())?;

        // Triangular arrowhead at tip.
        let len = (dx * dx + dy * dy).sqrt();
        if len > 1e-12 {
            let ux = dx / len;
            let uy = dy / len;
            let head_len = len * 0.3;
            let head_w = len * 0.15;
            let bx = x1 - ux * head_len;
            let by = y1 - uy * head_len;
            chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (x1, y1),
                        (bx - uy * head_w, by + ux * head_w),
                        (bx + uy * head_w, by - ux * head_w),
                    ],
                    BLUE.filled(),
                )))
                .map_err(|e| e.to_string())?;
        }
    }

    draw_text_annotations(&mut chart, &state.annotations)?;

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

/// Computes the arrow scale factor: 80% of the minimum grid spacing divided
/// by the maximum arrow magnitude, so the longest arrow fills ~80% of a cell.
fn arrow_scale(xs: &[f64], ys: &[f64], us: &[f64], vs: &[f64]) -> f64 {
    let x_sp = min_vec_spacing(xs);
    let y_sp = min_vec_spacing(ys);
    let grid_sp = x_sp.min(y_sp);
    let max_mag = us
        .iter()
        .zip(vs.iter())
        .map(|(&u, &v)| (u * u + v * v).sqrt())
        .fold(0.0f64, f64::max);
    if max_mag > 1e-12 {
        grid_sp * 0.8 / max_mag
    } else {
        grid_sp * 0.8
    }
}

/// Minimum non-zero gap between adjacent sorted values.
fn min_vec_spacing(vals: &[f64]) -> f64 {
    if vals.len() < 2 {
        return 1.0;
    }
    let mut sorted = vals.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less));
    let min_sp = sorted
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .filter(|&d| d > 1e-12)
        .fold(f64::INFINITY, f64::min);
    if min_sp.is_infinite() {
        1.0
    } else {
        min_sp.max(1e-6)
    }
}

/// Draws text annotations onto a 2-D chart using data coordinates.
fn draw_text_annotations<DB: DrawingBackend>(
    chart: &mut plotters::prelude::ChartContext<
        '_,
        DB,
        plotters::coord::cartesian::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    annotations: &[(f64, f64, String)],
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    for (ax, ay, label) in annotations {
        // Text::new requires an owned String so the element has no lifetime
        // dependency on `annotations` (which avoids a 'static bound from draw_series).
        chart
            .draw_series(std::iter::once(Text::new(
                label.clone(),
                (*ax, *ay),
                ("sans-serif", 12),
            )))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
