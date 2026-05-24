//! SVG and PNG file export via `plotters`.

#![cfg(feature = "plot-svg")]

use plotters::prelude::*;
use plotters::series::LineSeries;

use crate::FigureState;
use crate::style::{AxisMode, LinestyleKind, StyleColor, StyleSpec, Theme};

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
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_multi_line_chart(x, ys, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
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
    let (bg_c, text_c, axis_c, grid_bold_c, grid_light_c) = resolve_colors(state);
    root.fill(&bg_c).map_err(|e| e.to_string())?;
    let (grid_bold_style, grid_light_style) = resolve_grid_styles(
        grid_bold_c,
        grid_light_c,
        state.grid_color,
        state.grid_width,
    );

    let axis_mode = state.axis_mode;
    let (canvas_w, canvas_h) = root.dim_in_pixel();

    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("");

    let title_sz = eff_title_size(state.font_size, 20);
    let axis_desc_sz = eff_axis_desc_size(state.font_size, 12);
    let tick_sz = eff_axis_desc_size(state.font_size, 11);
    let lw = eff_line_width(None, state.line_width);

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| {
        if axis_mode == Some(AxisMode::Tight) {
            range_exact(x)
        } else {
            range_with_margin(x)
        }
    });

    // Y range spans all series.
    let all_y: Vec<f64> = ys.iter().flat_map(|v| v.iter().copied()).collect();
    let (y_min, y_max) = state.ylim.unwrap_or_else(|| {
        if axis_mode == Some(AxisMode::Tight) {
            range_exact(&all_y)
        } else {
            range_with_margin(&all_y)
        }
    });

    let (x_min, x_max, y_min, y_max) =
        if axis_mode == Some(AxisMode::Equal) && state.xlim.is_none() && state.ylim.is_none() {
            let (pw, ph) = plot_area_px(canvas_w, canvas_h, 30, 40, 50, title_sz);
            apply_equal_scale(x_min, x_max, y_min, y_max, pw, ph)
        } else {
            (x_min, x_max, y_min, y_max)
        };

    let (xa, ya) = if axis_mode == Some(AxisMode::Off) {
        (0, 0)
    } else {
        (40, 50)
    };
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", title_sz).into_font().color(&text_c))
        .margin(30)
        .x_label_area_size(xa)
        .y_label_area_size(ya)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    let mut mesh_binding = chart.configure_mesh();
    let mut mesh = mesh_binding
        .axis_style(ShapeStyle::from(&axis_c))
        .axis_desc_style(("sans-serif", axis_desc_sz).into_font().color(&text_c))
        .label_style(("sans-serif", tick_sz).into_font().color(&text_c))
        .bold_line_style(grid_bold_style)
        .light_line_style(grid_light_style)
        .x_desc(xlabel)
        .y_desc(ylabel);
    if axis_mode == Some(AxisMode::Off) {
        mesh = mesh.disable_axes().disable_mesh();
    } else if !state.grid {
        mesh = mesh.disable_mesh();
    }
    mesh.draw().map_err(|e| e.to_string())?;

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
            .draw_series(LineSeries::new(
                points,
                ShapeStyle::from(&color).stroke_width(lw),
            ))
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
    style: Option<crate::style::StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_hist_chart(counts, edges, &style, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
        draw_hist_chart(counts, edges, &style, &state, root)
    } else {
        Err(format!("hist: unsupported format '{path}'"))
    }
}

fn draw_hist_chart<DB: DrawingBackend>(
    counts: &[usize],
    edges: &[f64],
    style: &Option<crate::style::StyleSpec>,
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    let (bg_c, text_c, axis_c, grid_bold_c, grid_light_c) = resolve_colors(state);
    root.fill(&bg_c).map_err(|e| e.to_string())?;
    let (grid_bold_style, grid_light_style) = resolve_grid_styles(
        grid_bold_c,
        grid_light_c,
        state.grid_color,
        state.grid_width,
    );

    let axis_mode = state.axis_mode;
    let (canvas_w, canvas_h) = root.dim_in_pixel();

    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("count");

    let title_sz = eff_title_size(state.font_size, 20);
    let axis_desc_sz = eff_axis_desc_size(state.font_size, 12);
    let tick_sz = eff_axis_desc_size(state.font_size, 11);

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
    let y_max = state.ylim.map(|(_, hi)| hi).unwrap_or_else(|| {
        if axis_mode == Some(AxisMode::Tight) {
            max_count
        } else {
            max_count * 1.05
        }
    });

    let (x_min, x_max, y_min, y_max) =
        if axis_mode == Some(AxisMode::Equal) && state.xlim.is_none() && state.ylim.is_none() {
            let (pw, ph) = plot_area_px(canvas_w, canvas_h, 30, 40, 50, title_sz);
            apply_equal_scale(x_min, x_max, y_min, y_max, pw, ph)
        } else {
            (x_min, x_max, y_min, y_max)
        };

    let (xa, ya) = if axis_mode == Some(AxisMode::Off) {
        (0, 0)
    } else {
        (40, 50)
    };
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", title_sz).into_font().color(&text_c))
        .margin(30)
        .x_label_area_size(xa)
        .y_label_area_size(ya)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    let mut mesh_binding = chart.configure_mesh();
    let mut mesh = mesh_binding
        .axis_style(ShapeStyle::from(&axis_c))
        .axis_desc_style(("sans-serif", axis_desc_sz).into_font().color(&text_c))
        .label_style(("sans-serif", tick_sz).into_font().color(&text_c))
        .bold_line_style(grid_bold_style)
        .light_line_style(grid_light_style)
        .x_desc(xlabel)
        .y_desc(ylabel);
    if axis_mode == Some(AxisMode::Off) {
        mesh = mesh.disable_axes().disable_mesh();
    } else if !state.grid {
        mesh = mesh.disable_mesh();
    }
    mesh.draw().map_err(|e| e.to_string())?;

    let bar_color = style_to_rgb(style).unwrap_or(SERIES_COLORS[0]);
    // Edge-to-edge bars (no gap between adjacent bins).
    chart
        .draw_series((0..counts.len()).map(|i| {
            Rectangle::new(
                [(edges[i], 0.0), (edges[i + 1], counts[i] as f64)],
                bar_color.filled(),
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
    style: Option<crate::style::StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    render_file_styled(ChartKind::Bar, x, y, path, style, state)
}

/// Writes an SVG or PNG stem plot to `path`, routing on the file extension.
pub(crate) fn render_stem(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<crate::style::StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    render_file_styled(ChartKind::Stem, x, y, path, style, state)
}

/// Writes an SVG or PNG filled-polygon plot to `path`.
pub(crate) fn render_fill(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_polygon_chart(x, y, false, style, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
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
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_polygon_chart(x, y, true, style, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
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
    let (bg_c, text_c, axis_c, grid_bold_c, grid_light_c) = resolve_colors(state);
    root.fill(&bg_c).map_err(|e| e.to_string())?;

    let axis_mode = state.axis_mode;
    let (canvas_w, canvas_h) = root.dim_in_pixel();

    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("");

    let title_sz = eff_title_size(state.font_size, 20);
    let axis_desc_sz = eff_axis_desc_size(state.font_size, 12);
    let tick_sz = eff_axis_desc_size(state.font_size, 11);

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| {
        if axis_mode == Some(AxisMode::Tight) {
            range_exact(x)
        } else {
            range_with_margin(x)
        }
    });
    let y_with_zero: Vec<f64> = y.iter().copied().chain(std::iter::once(0.0)).collect();
    let (y_min, y_max) = state.ylim.unwrap_or_else(|| {
        if axis_mode == Some(AxisMode::Tight) {
            range_exact_zero_baseline(&y_with_zero)
        } else {
            range_with_zero_baseline(&y_with_zero)
        }
    });

    let (x_min, x_max, y_min, y_max) =
        if axis_mode == Some(AxisMode::Equal) && state.xlim.is_none() && state.ylim.is_none() {
            let (pw, ph) = plot_area_px(canvas_w, canvas_h, 30, 40, 50, title_sz);
            apply_equal_scale(x_min, x_max, y_min, y_max, pw, ph)
        } else {
            (x_min, x_max, y_min, y_max)
        };

    let (xa, ya) = if axis_mode == Some(AxisMode::Off) {
        (0, 0)
    } else {
        (40, 50)
    };
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", title_sz).into_font().color(&text_c))
        .margin(30)
        .x_label_area_size(xa)
        .y_label_area_size(ya)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    let mut mesh_binding = chart.configure_mesh();
    let mut mesh = mesh_binding
        .axis_style(ShapeStyle::from(&axis_c))
        .axis_desc_style(("sans-serif", axis_desc_sz).into_font().color(&text_c))
        .label_style(("sans-serif", tick_sz).into_font().color(&text_c))
        .bold_line_style(ShapeStyle::from(&grid_bold_c))
        .light_line_style(ShapeStyle::from(&grid_light_c))
        .x_desc(xlabel)
        .y_desc(ylabel);
    if axis_mode == Some(AxisMode::Off) {
        mesh = mesh.disable_axes();
    }
    mesh.disable_mesh().draw().map_err(|e| e.to_string())?;

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

/// Resolves the active [`Theme`] for `state`: explicit theme > light default.
fn resolve_theme(state: &FigureState) -> Theme {
    state.theme.clone().unwrap_or_else(Theme::light)
}

/// Returns the effective background colour as a plotters [`RGBColor`].
///
/// Resolution order: per-figure `bg_color` override > active theme background.
fn effective_bg(state: &FigureState) -> RGBColor {
    let c: StyleColor = state.bg_color.unwrap_or_else(|| resolve_theme(state).bg);
    RGBColor(c.0, c.1, c.2)
}

/// Converts a [`StyleColor`] to a plotters [`RGBColor`].
fn sc_to_rgb(c: StyleColor) -> RGBColor {
    RGBColor(c.0, c.1, c.2)
}

/// Resolves all five theme colours for a state in one call.
///
/// Returns `(bg, text, axis, grid_bold, grid_light)`.
fn resolve_colors(state: &FigureState) -> (RGBColor, RGBColor, RGBColor, RGBColor, RGBColor) {
    let theme = resolve_theme(state);
    let bg = effective_bg(state);
    let text = sc_to_rgb(theme.text);
    let axis = sc_to_rgb(theme.axis);
    let grid_bold = sc_to_rgb(theme.grid_bold);
    let grid_light = sc_to_rgb(theme.grid_light);
    (bg, text, axis, grid_bold, grid_light)
}

/// Resolves theme colours from an already-resolved [`Theme`] (no state access).
fn theme_to_colors(theme: &Theme) -> (RGBColor, RGBColor, RGBColor, RGBColor) {
    (
        sc_to_rgb(theme.text),
        sc_to_rgb(theme.axis),
        sc_to_rgb(theme.grid_bold),
        sc_to_rgb(theme.grid_light),
    )
}

/// Resolves bold and light grid line styles with optional colour/width overrides.
///
/// `grid_color` overrides the colour of both styles.  `grid_width` applies
/// **only to the bold (major) lines**; the light (minor subdivision) lines are
/// always kept at the theme's default stroke width (1) so they remain visually
/// thin regardless of the user-specified width.
///
/// For `grid_width < 1.0` the bold style uses `stroke_width = 1` with an alpha
/// channel proportional to the width, so 0.3 renders visibly lighter than 0.7.
/// For `grid_width >= 1.0` the width is rounded to the nearest integer.
fn resolve_grid_styles(
    theme_bold: RGBColor,
    theme_light: RGBColor,
    grid_color: Option<crate::style::StyleColor>,
    grid_width: Option<f32>,
) -> (ShapeStyle, ShapeStyle) {
    let bold_c = grid_color
        .map(|sc| RGBColor(sc.0, sc.1, sc.2))
        .unwrap_or(theme_bold);
    let light_c = grid_color
        .map(|sc| RGBColor(sc.0, sc.1, sc.2))
        .unwrap_or(theme_light);

    let bold_style = match grid_width {
        Some(gw) if gw > 0.0 && gw < 1.0 => {
            // Sub-pixel: keep stroke_width=1, reduce alpha so thin values look
            // visibly lighter than the full-opacity default.
            let alpha = gw as f64;
            let rgba = RGBAColor(bold_c.0, bold_c.1, bold_c.2, alpha);
            ShapeStyle::from(&rgba).stroke_width(1)
        }
        Some(gw) => ShapeStyle::from(&bold_c).stroke_width(gw.round().max(1.0) as u32),
        None => ShapeStyle::from(&bold_c).stroke_width(1),
    };

    // Minor (light) lines always stay thin — grid_width must not thicken them.
    let light_style = ShapeStyle::from(&light_c).stroke_width(1);

    (bold_style, light_style)
}

/// Effective title/caption font size: session override → given default, min 8.
fn eff_title_size(session: Option<u32>, default: u32) -> u32 {
    session.map(|f| f.max(8)).unwrap_or(default)
}

/// Effective axis-descriptor font size: proportional to title, min 8.
fn eff_axis_desc_size(session: Option<u32>, default: u32) -> u32 {
    session
        .map(|f| ((f as f32 * 0.65).round() as u32).max(8))
        .unwrap_or(default)
}

/// Effective line stroke width (pixels): per-series → session override → 1.
fn eff_line_width(series_style: Option<&crate::style::StyleSpec>, session: Option<f32>) -> u32 {
    series_style
        .and_then(|s| s.line_width)
        .or(session)
        .map(|f| f.round().max(1.0) as u32)
        .unwrap_or(1)
}

/// Effective marker radius (pixels): per-series → session override → 3.
fn eff_marker_size(series_style: Option<&crate::style::StyleSpec>, session: Option<u32>) -> u32 {
    series_style
        .and_then(|s| s.marker_size)
        .or(session)
        .unwrap_or(3)
}

fn render_file(
    kind: ChartKind,
    x: &[f64],
    y: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    render_file_styled(kind, x, y, path, None, state)
}

fn render_file_styled(
    kind: ChartKind,
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<crate::style::StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_chart(kind, x, y, &style, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
        draw_chart(kind, x, y, &style, &state, root)
    } else {
        Err(format!("file: unsupported format '{path}'"))
    }
}

fn draw_chart<DB: DrawingBackend>(
    kind: ChartKind,
    x: &[f64],
    y: &[f64],
    style: &Option<crate::style::StyleSpec>,
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    let (bg_c, text_c, axis_c, grid_bold_c, grid_light_c) = resolve_colors(state);
    root.fill(&bg_c).map_err(|e| e.to_string())?;
    let (grid_bold_style, grid_light_style) = resolve_grid_styles(
        grid_bold_c,
        grid_light_c,
        state.grid_color,
        state.grid_width,
    );

    let axis_mode = state.axis_mode;
    let (canvas_w, canvas_h) = root.dim_in_pixel();

    // Bar and stem charts always include y = 0 in the y axis.
    let zero_baseline = matches!(kind, ChartKind::Bar | ChartKind::Stem);

    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("");

    let title_sz = eff_title_size(state.font_size, 20);
    let axis_desc_sz = eff_axis_desc_size(state.font_size, 12);
    let tick_sz = eff_axis_desc_size(state.font_size, 11);

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| {
        if axis_mode == Some(AxisMode::Tight) {
            range_exact(x)
        } else {
            range_with_margin(x)
        }
    });
    let (y_min, y_max) = state.ylim.unwrap_or_else(|| {
        if zero_baseline {
            if axis_mode == Some(AxisMode::Tight) {
                range_exact_zero_baseline(y)
            } else {
                range_with_zero_baseline(y)
            }
        } else if axis_mode == Some(AxisMode::Tight) {
            range_exact(y)
        } else {
            range_with_margin(y)
        }
    });

    let (x_min, x_max, y_min, y_max) =
        if axis_mode == Some(AxisMode::Equal) && state.xlim.is_none() && state.ylim.is_none() {
            let (pw, ph) = plot_area_px(canvas_w, canvas_h, 30, 40, 50, title_sz);
            apply_equal_scale(x_min, x_max, y_min, y_max, pw, ph)
        } else {
            (x_min, x_max, y_min, y_max)
        };

    let (xa, ya) = if axis_mode == Some(AxisMode::Off) {
        (0, 0)
    } else {
        (40, 50)
    };
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", title_sz).into_font().color(&text_c))
        .margin(30)
        .x_label_area_size(xa)
        .y_label_area_size(ya)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    let mut mesh_binding = chart.configure_mesh();
    let mut mesh = mesh_binding
        .axis_style(ShapeStyle::from(&axis_c))
        .axis_desc_style(("sans-serif", axis_desc_sz).into_font().color(&text_c))
        .label_style(("sans-serif", tick_sz).into_font().color(&text_c))
        .bold_line_style(grid_bold_style)
        .light_line_style(grid_light_style)
        .x_desc(xlabel)
        .y_desc(ylabel);
    if axis_mode == Some(AxisMode::Off) {
        mesh = mesh.disable_axes().disable_mesh();
    } else if !state.grid {
        mesh = mesh.disable_mesh();
    }
    mesh.draw().map_err(|e| e.to_string())?;

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

    let chart_color = style_to_rgb(style).unwrap_or(SERIES_COLORS[0]);
    let lw = eff_line_width(style.as_ref(), state.line_width);
    let ms = eff_marker_size(style.as_ref(), state.marker_size) as i32;
    match kind {
        ChartKind::Line => {
            chart
                .draw_series(LineSeries::new(
                    points,
                    ShapeStyle::from(&chart_color).stroke_width(lw),
                ))
                .map_err(|e| e.to_string())?;
        }
        ChartKind::Scatter => {
            chart
                .draw_series(
                    points
                        .iter()
                        .map(|&(xi, yi)| Circle::new((xi, yi), ms, chart_color.filled())),
                )
                .map_err(|e| e.to_string())?;
        }
        ChartKind::Bar => {
            let bar_w = bar_half_width(x, x_min, x_max);
            chart
                .draw_series(x.iter().zip(y.iter()).map(|(&xi, &yi)| {
                    let (y_lo, y_hi) = if yi >= 0.0 { (0.0, yi) } else { (yi, 0.0) };
                    Rectangle::new(
                        [(xi - bar_w, y_lo), (xi + bar_w, y_hi)],
                        chart_color.filled(),
                    )
                }))
                .map_err(|e| e.to_string())?;
        }
        ChartKind::Stem => {
            // Vertical lines from y=0 to each tip.
            for (&xi, &yi) in x.iter().zip(y.iter()) {
                chart
                    .draw_series(std::iter::once(PathElement::new(
                        vec![(xi, 0.0), (xi, yi)],
                        ShapeStyle::from(&chart_color).stroke_width(lw),
                    )))
                    .map_err(|e| e.to_string())?;
            }
            // Tip circles.
            chart
                .draw_series(
                    x.iter()
                        .zip(y.iter())
                        .map(|(&xi, &yi)| Circle::new((xi, yi), ms, chart_color.filled())),
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
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_3d_chart(false, x, y, z, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
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
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_3d_chart(true, x, y, z, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
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
    let text_c = sc_to_rgb(resolve_theme(state).text);
    root.fill(&effective_bg(state)).map_err(|e| e.to_string())?;

    let (x_min, x_max) = state.xlim.unwrap_or_else(|| range_with_margin(x));
    let (y_min, y_max) = state.ylim.unwrap_or_else(|| range_with_margin(y));
    let (z_min, z_max) = state.zlim.unwrap_or_else(|| range_with_margin(z));

    let title = state.title.as_deref().unwrap_or("");
    let title_sz = eff_title_size(state.font_size, 20);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", title_sz).into_font().color(&text_c))
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
pub(crate) fn render_subplot_panels(
    panels: &[crate::Panel],
    path: &str,
    canvas: (u32, u32),
    theme: &Theme,
    bg: RGBColor,
) -> Result<(), String> {
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
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_panels(panels, rows, cols, theme, bg, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
        draw_panels(panels, rows, cols, theme, bg, root)
    } else {
        Err(format!("savefig: unsupported format '{path}'"))
    }
}

fn draw_panels<DB: DrawingBackend>(
    panels: &[crate::Panel],
    rows: u32,
    cols: u32,
    theme: &Theme,
    bg: RGBColor,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&bg).map_err(|e| e.to_string())?;
    let sub_areas = root.split_evenly((rows as usize, cols as usize));

    for panel in panels {
        let idx = panel
            .layout
            .map(|(_, _, k)| k.saturating_sub(1) as usize)
            .unwrap_or(0);
        let area = &sub_areas[idx.min(sub_areas.len().saturating_sub(1))];
        draw_panel(panel, theme, area)?;
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

fn draw_panel<DB: DrawingBackend>(
    panel: &crate::Panel,
    theme: &Theme,
    area: &DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    use crate::PendingSeries;
    let (text_c, axis_c, grid_bold_c, grid_light_c) = theme_to_colors(theme);
    let (grid_bold_style, grid_light_style) = resolve_grid_styles(
        grid_bold_c,
        grid_light_c,
        panel.grid_color,
        panel.grid_width,
    );

    if panel.series.is_empty() {
        return Ok(());
    }

    let axis_mode = panel.axis_mode;
    let (area_w, area_h) = area.dim_in_pixel();

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
            PendingSeries::Bar(x, y, _) | PendingSeries::Stem(x, y, _) => {
                all_x.extend_from_slice(x);
                all_y.extend_from_slice(y);
                has_zero_baseline = true;
            }
            PendingSeries::Hist {
                counts,
                edges,
                style: _,
            } => {
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
            PendingSeries::Quiver(x, y, u, v, _) => {
                all_x.extend_from_slice(x);
                all_x.extend(x.iter().zip(u.iter()).map(|(&xi, &ui)| xi + ui));
                all_y.extend_from_slice(y);
                all_y.extend(y.iter().zip(v.iter()).map(|(&yi, &vi)| yi + vi));
            }
        }
    }

    let (x_min, x_max) = panel.xlim.unwrap_or_else(|| {
        if axis_mode == Some(AxisMode::Tight) {
            range_exact(&all_x)
        } else {
            range_with_margin(&all_x)
        }
    });
    let (y_min, y_max) = panel.ylim.unwrap_or_else(|| {
        if has_zero_baseline {
            if axis_mode == Some(AxisMode::Tight) {
                range_exact_zero_baseline(&all_y)
            } else {
                range_with_zero_baseline(&all_y)
            }
        } else if axis_mode == Some(AxisMode::Tight) {
            range_exact(&all_y)
        } else {
            range_with_margin(&all_y)
        }
    });

    let title = panel.title.as_deref().unwrap_or("");
    let xlabel = panel.xlabel.as_deref().unwrap_or("");
    let ylabel = panel.ylabel.as_deref().unwrap_or("");

    let title_sz = eff_title_size(panel.font_size, 16);
    let axis_desc_sz = eff_axis_desc_size(panel.font_size, 11);
    let tick_sz = eff_axis_desc_size(panel.font_size, 10);

    let (x_min, x_max, y_min, y_max) =
        if axis_mode == Some(AxisMode::Equal) && panel.xlim.is_none() && panel.ylim.is_none() {
            let (pw, ph) = plot_area_px(area_w, area_h, 15, 30, 40, title_sz);
            apply_equal_scale(x_min, x_max, y_min, y_max, pw, ph)
        } else {
            (x_min, x_max, y_min, y_max)
        };

    let (xa, ya) = if axis_mode == Some(AxisMode::Off) {
        (0, 0)
    } else {
        (30, 40)
    };
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", title_sz).into_font().color(&text_c))
        .margin(15)
        .x_label_area_size(xa)
        .y_label_area_size(ya)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    let mut mesh_binding = chart.configure_mesh();
    let mut mesh = mesh_binding
        .axis_style(ShapeStyle::from(&axis_c))
        .axis_desc_style(("sans-serif", axis_desc_sz).into_font().color(&text_c))
        .label_style(("sans-serif", tick_sz).into_font().color(&text_c))
        .bold_line_style(grid_bold_style)
        .light_line_style(grid_light_style)
        .x_desc(xlabel)
        .y_desc(ylabel);
    if axis_mode == Some(AxisMode::Off) {
        mesh = mesh.disable_axes().disable_mesh();
    } else if !panel.grid {
        mesh = mesh.disable_mesh();
    }
    mesh.draw().map_err(|e| e.to_string())?;

    for (i, series) in panel.series.iter().enumerate() {
        let default_color = SERIES_COLORS[i % SERIES_COLORS.len()];
        match series {
            PendingSeries::Line(x, y, style) => {
                let color = style_to_rgb(style).unwrap_or(default_color);
                let lw = eff_line_width(style.as_ref(), panel.line_width);
                let ms = eff_marker_size(style.as_ref(), panel.marker_size) as i32;
                let linestyle = style
                    .as_ref()
                    .map(|s| s.linestyle)
                    .unwrap_or(LinestyleKind::Solid);
                let pts: Vec<(f64, f64)> =
                    x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi, yi)).collect();
                let n = pts.len();
                let line_style = ShapeStyle::from(&color).stroke_width(lw);
                match linestyle {
                    LinestyleKind::Solid => {
                        chart
                            .draw_series(LineSeries::new(pts.iter().copied(), line_style))
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
                                        line_style,
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
                                        line_style,
                                    ))
                                    .map_err(|e| e.to_string())?;
                            }
                            i = end + gap;
                            if i < n {
                                chart
                                    .draw_series(std::iter::once(Circle::new(
                                        pts[i],
                                        ms,
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
                                    .map(|&p| Circle::new(p, ms, color.filled())),
                            )
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
            PendingSeries::Scatter(x, y, style) => {
                let color = style_to_rgb(style).unwrap_or(default_color);
                let ms = eff_marker_size(style.as_ref(), panel.marker_size) as i32;
                chart
                    .draw_series(
                        x.iter()
                            .zip(y.iter())
                            .map(|(&xi, &yi)| Circle::new((xi, yi), ms, color.filled())),
                    )
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Bar(x, y, style) => {
                let color = style_to_rgb(style).unwrap_or(default_color);
                let bar_w = bar_half_width(x, x_min, x_max);
                chart
                    .draw_series(x.iter().zip(y.iter()).map(|(&xi, &yi)| {
                        let (y_lo, y_hi) = if yi >= 0.0 { (0.0, yi) } else { (yi, 0.0) };
                        Rectangle::new([(xi - bar_w, y_lo), (xi + bar_w, y_hi)], color.filled())
                    }))
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Stem(x, y, style) => {
                let color = style_to_rgb(style).unwrap_or(default_color);
                let lw = eff_line_width(style.as_ref(), panel.line_width);
                let ms = eff_marker_size(style.as_ref(), panel.marker_size) as i32;
                for (&xi, &yi) in x.iter().zip(y.iter()) {
                    chart
                        .draw_series(std::iter::once(PathElement::new(
                            vec![(xi, 0.0), (xi, yi)],
                            ShapeStyle::from(&color).stroke_width(lw),
                        )))
                        .map_err(|e| e.to_string())?;
                }
                chart
                    .draw_series(
                        x.iter()
                            .zip(y.iter())
                            .map(|(&xi, &yi)| Circle::new((xi, yi), ms, color.filled())),
                    )
                    .map_err(|e| e.to_string())?;
            }
            PendingSeries::Hist {
                counts,
                edges,
                style,
            } => {
                let color = style_to_rgb(style).unwrap_or(default_color);
                chart
                    .draw_series((0..counts.len()).map(|j| {
                        Rectangle::new(
                            [(edges[j], 0.0), (edges[j + 1], counts[j] as f64)],
                            color.filled(),
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
            PendingSeries::Quiver(x, y, u, v, style) => {
                let color = style_to_rgb(style).unwrap_or(default_color);
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
                            color,
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
                                color.filled(),
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

/// Exact range with no added margin — used by `axis('tight')`.
fn range_exact(vals: &[f64]) -> (f64, f64) {
    let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (hi - lo).abs() < f64::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        (lo, hi)
    }
}

/// Tight range that always includes y = 0 — used by `axis('tight')` on bar/stem.
fn range_exact_zero_baseline(vals: &[f64]) -> (f64, f64) {
    let lo = vals.iter().copied().fold(f64::INFINITY, f64::min).min(0.0);
    let hi = vals
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);
    if (hi - lo).abs() < f64::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        (lo, hi)
    }
}

/// Estimated plot area pixel dimensions for `axis('equal')` aspect-ratio maths.
///
/// Parameters mirror the `ChartBuilder` calls: `margin(mg)`,
/// `x_label_area_size(xa)`, `y_label_area_size(ya)`, `title_sz` for the
/// caption overhead.  Returns `(plot_w, plot_h)`.
fn plot_area_px(
    total_w: u32,
    total_h: u32,
    mg: u32,
    xa: u32,
    ya: u32,
    title_sz: u32,
) -> (u32, u32) {
    let title_h = title_sz + 15;
    let w = total_w.saturating_sub(2 * mg + ya);
    let h = total_h.saturating_sub(2 * mg + xa + title_h);
    (w.max(1), h.max(1))
}

/// Adjusts `(x_min, x_max, y_min, y_max)` so that data-units per pixel are
/// equal on both axes.
///
/// Always expands the tighter axis (never clips data).  `plot_w` / `plot_h`
/// are the pixel dimensions of the drawable chart area.
fn apply_equal_scale(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    plot_w: u32,
    plot_h: u32,
) -> (f64, f64, f64, f64) {
    let x_span = x_max - x_min;
    let y_span = y_max - y_min;
    let px_aspect = plot_w as f64 / plot_h as f64;
    // Required x_span for equal scale given current y_span.
    let target_x = y_span * px_aspect;
    if target_x > x_span {
        let cx = (x_min + x_max) / 2.0;
        return (cx - target_x / 2.0, cx + target_x / 2.0, y_min, y_max);
    }
    // Required y_span for equal scale given current x_span.
    let target_y = x_span / px_aspect;
    if target_y > y_span {
        let cy = (y_min + y_max) / 2.0;
        return (x_min, x_max, cy - target_y / 2.0, cy + target_y / 2.0);
    }
    (x_min, x_max, y_min, y_max)
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
    style: Option<crate::style::StyleSpec>,
    state: FigureState,
) -> Result<(), String> {
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_quiver_chart(xs, ys, us, vs, &style, &state, root)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
        draw_quiver_chart(xs, ys, us, vs, &style, &state, root)
    } else {
        Err(format!("quiver: unsupported format '{path}'"))
    }
}

fn draw_quiver_chart<DB: DrawingBackend>(
    xs: &[f64],
    ys: &[f64],
    us: &[f64],
    vs: &[f64],
    style: &Option<crate::style::StyleSpec>,
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    let (bg_c, text_c, axis_c, grid_bold_c, grid_light_c) = resolve_colors(state);
    root.fill(&bg_c).map_err(|e| e.to_string())?;

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
        .caption(title, ("sans-serif", 20).into_font().color(&text_c))
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| e.to_string())?;

    chart
        .configure_mesh()
        .axis_style(ShapeStyle::from(&axis_c))
        .axis_desc_style(("sans-serif", 12).into_font().color(&text_c))
        .label_style(("sans-serif", 11).into_font().color(&text_c))
        .bold_line_style(ShapeStyle::from(&grid_bold_c))
        .light_line_style(ShapeStyle::from(&grid_light_c))
        .x_desc(xlabel)
        .y_desc(ylabel)
        .disable_mesh()
        .draw()
        .map_err(|e| e.to_string())?;

    let arrow_color = style_to_rgb(style).unwrap_or(SERIES_COLORS[0]);
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
                arrow_color,
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
                    arrow_color.filled(),
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

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_light_style_always_thin() {
        let bold = RGBColor(100, 100, 200);
        let light = RGBColor(200, 200, 240);
        // Light lines must always be stroke_width=1 regardless of grid_width.
        let (_, ls_none) = resolve_grid_styles(bold, light, None, None);
        assert_eq!(ls_none.stroke_width, 1);
        let (_, ls_4) = resolve_grid_styles(bold, light, None, Some(4.0));
        assert_eq!(ls_4.stroke_width, 1, "grid_width must not thicken minor lines");
        let (_, ls_sub) = resolve_grid_styles(bold, light, None, Some(0.3));
        assert_eq!(ls_sub.stroke_width, 1, "sub-pixel grid_width must not affect minor lines");
    }

    #[test]
    fn grid_bold_style_default_width_and_alpha() {
        let bold = RGBColor(100, 100, 200);
        let light = RGBColor(200, 200, 240);
        let (bold_s, _) = resolve_grid_styles(bold, light, None, None);
        assert_eq!(bold_s.stroke_width, 1);
        assert!((bold_s.color.3 - 1.0_f64).abs() < 1e-6, "default alpha must be 1.0");
    }

    #[test]
    fn grid_sub_pixel_widths_use_alpha() {
        let bold = RGBColor(100, 150, 200);
        let light = RGBColor(200, 200, 240);

        let (s03, ls) = resolve_grid_styles(bold, light, None, Some(0.3));
        assert_eq!(s03.stroke_width, 1);
        assert!((s03.color.3 - 0.3_f64).abs() < 0.01, "gridwidth 0.3 must give alpha ≈ 0.3");
        assert_eq!(ls.stroke_width, 1, "light lines stay thin regardless of grid_width");

        let (s07, _) = resolve_grid_styles(bold, light, None, Some(0.7));
        assert_eq!(s07.stroke_width, 1);
        assert!((s07.color.3 - 0.7_f64).abs() < 0.01, "gridwidth 0.7 must give alpha ≈ 0.7");

        assert!(s03.color.3 < s07.color.3, "0.3 must be less opaque than 0.7");
    }

    #[test]
    fn grid_integer_widths_use_stroke_width() {
        let bold = RGBColor(100, 100, 200);
        let light = RGBColor(200, 200, 240);

        let (s15, _) = resolve_grid_styles(bold, light, None, Some(1.5));
        assert_eq!(s15.stroke_width, 2);
        assert!((s15.color.3 - 1.0_f64).abs() < 1e-6, "width≥1 must have full alpha");

        let (s3, _) = resolve_grid_styles(bold, light, None, Some(3.0));
        assert_eq!(s3.stroke_width, 3);

        let (s4, _) = resolve_grid_styles(bold, light, None, Some(4.0));
        assert_eq!(s4.stroke_width, 4);
    }

    #[test]
    fn grid_custom_color_applied_to_bold() {
        let bold = RGBColor(100, 100, 200);
        let light = RGBColor(200, 200, 240);
        let custom = crate::style::StyleColor(255, 0, 128);
        let (bold_s, _) = resolve_grid_styles(bold, light, Some(custom), None);
        assert_eq!(bold_s.color.0, 255);
        assert_eq!(bold_s.color.1, 0);
        assert_eq!(bold_s.color.2, 128);
    }
}
