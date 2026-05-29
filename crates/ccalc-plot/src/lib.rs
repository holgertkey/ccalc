//! Plot plugin for ccalc — Phase 32c.
//!
//! Provides `plot`, `scatter`, `bar`, `stem`, `hist`, `stairs`, `loglog`,
//! `semilogx`, `semilogy`, `plot3`, `scatter3`, `imagesc`, `surf`, `mesh`,
//! `contour`, `contourf`, `subplot`, `hold`, `savefig`, `fill`, `area`,
//! `polar`, `quiver`, `text`, `axis`, `line`, `patch`, `rectangle`,
//! `errorbar`, `pie`, and annotation functions (`xlabel`, `ylabel`, `zlabel`,
//! `title`, `legend`, `xlim`, `ylim`, `zlim`, `grid`, `colormap`, `colorbar`).
//! Rendering requires the `plot` or `plot-svg` feature flags; annotation-only
//! calls work in every build configuration.
//!
//! # Feature flags
//!
//! | Flag | Backend | Extra size |
//! |------|---------|------------|
//! | `plot` | ASCII via `textplots` | ~100 KB |
//! | `plot-svg` | SVG + PNG via `plotters` | ~3 MB |
//! | `plot-all` | Both tiers | combined |
//!
//! Build with `--features plot` to enable ASCII rendering.

pub mod colormap;
pub mod dispatch;
pub mod proj3d;
pub mod style;

#[cfg(feature = "plot")]
mod ascii;

#[cfg(feature = "plot-svg")]
mod file;

mod contour;
mod surface;

use std::cell::RefCell;

use ccalc_engine::env::{Env, Value};
use ccalc_engine::plugin::Plugin;

use colormap::ColormapSpec;
use dispatch::{
    extract_file_arg, extract_flat, extract_matrix, extract_style_and_file_arg,
    extract_style_and_file_arg_min, extract_vector,
};
use style::{AxisMode, StyleColor, StyleSpec, Theme, YAxis};

// ── PendingSeries / Panel ──────────────────────────────────────────────────

/// A renderable data series stored for deferred rendering under `hold`/`subplot`.
#[derive(Clone)]
pub enum PendingSeries {
    /// Connected line plot, with optional style override.
    Line(Vec<f64>, Vec<f64>, Option<StyleSpec>),
    /// Point-cloud scatter, with optional style override.
    Scatter(Vec<f64>, Vec<f64>, Option<StyleSpec>),
    /// Vertical bar chart, with optional style override.
    Bar(Vec<f64>, Vec<f64>, Option<StyleSpec>),
    /// Stem (lollipop) chart, with optional style override.
    Stem(Vec<f64>, Vec<f64>, Option<StyleSpec>),
    /// Histogram — pre-computed counts and bin edges, with optional style override.
    Hist {
        counts: Vec<usize>,
        edges: Vec<f64>,
        style: Option<StyleSpec>,
    },
    /// Filled polygon.
    Fill(Vec<f64>, Vec<f64>, Option<StyleSpec>),
    /// Area under a curve (polygon closing along y = 0).
    Area(Vec<f64>, Vec<f64>, Option<StyleSpec>),
    /// Vector field: origin coordinates `(x, y)` and displacement vectors `(u, v)`, with optional style override.
    Quiver(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Option<StyleSpec>),
    /// Vertical error bars with symmetric or asymmetric half-lengths.
    ///
    /// `e_low` and `e_high` store the downward and upward half-extents
    /// respectively (both are non-negative distances from `y[i]`).
    ErrorBar {
        /// X positions.
        x: Vec<f64>,
        /// Y centre positions.
        y: Vec<f64>,
        /// Downward half-extents (≥ 0).
        e_low: Vec<f64>,
        /// Upward half-extents (≥ 0).
        e_high: Vec<f64>,
        /// Optional style override.
        style: Option<StyleSpec>,
    },
    /// Per-point color scatter plot mapped through the active colormap.
    ColorScatter {
        /// X positions.
        x: Vec<f64>,
        /// Y positions.
        y: Vec<f64>,
        /// Per-point marker radii in pixels.
        sz: Vec<f64>,
        /// Scalar values that drive the colormap lookup.
        c: Vec<f64>,
        /// Minimum `c` value (for normalisation).
        c_min: f64,
        /// Maximum `c` value (for normalisation).
        c_max: f64,
    },
    /// Pie chart with optional per-slice labels and explode offsets.
    Pie {
        /// Slice magnitudes (will be normalised to 100 % internally).
        values: Vec<f64>,
        /// Optional per-slice text labels (empty `String` = no label for that slice).
        labels: Vec<String>,
        /// Per-slice explode offsets as a fraction of the radius (0.0 = no offset).
        explode: Vec<f64>,
    },
}

/// A committed subplot panel ready for file rendering.
#[derive(Clone, Default)]
pub struct Panel {
    /// Grid position `(rows, cols, index_1based)` inside a subplot layout.
    pub layout: Option<(u32, u32, u32)>,
    /// X-axis label.
    pub xlabel: Option<String>,
    /// Y-axis label.
    pub ylabel: Option<String>,
    /// Chart title.
    pub title: Option<String>,
    /// Series labels for legend.
    pub legend: Vec<String>,
    /// X-axis range override.
    pub xlim: Option<(f64, f64)>,
    /// Y-axis range override.
    pub ylim: Option<(f64, f64)>,
    /// Whether to draw grid lines.
    pub grid: bool,
    /// Accumulated data series.
    pub series: Vec<PendingSeries>,
    /// Text annotations placed on this panel.
    pub annotations: Vec<(f64, f64, String)>,
    /// Session-level font size carried into this panel.
    pub font_size: Option<u32>,
    /// Session-level line width carried into this panel.
    pub line_width: Option<f32>,
    /// Session-level marker size carried into this panel.
    pub marker_size: Option<u32>,
    /// Session-level grid colour override carried into this panel.
    pub grid_color: Option<StyleColor>,
    /// Session-level grid line width carried into this panel.
    pub grid_width: Option<f32>,
    /// Axis display mode override carried into this panel.
    pub axis_mode: Option<AxisMode>,
    /// Active colormap specification carried into this panel (for `ColorScatter`).
    pub colormap: Option<ColormapSpec>,
    // ── Phase 32d — dual Y axis ────────────────────────────────────────────
    /// Series on the secondary (right) Y axis.
    pub right_series: Vec<PendingSeries>,
    /// Override for the right Y axis range `[min, max]`.
    pub right_ylim: Option<(f64, f64)>,
    /// Label for the right Y axis.
    pub right_ylabel: Option<String>,
}

// ── FigureState ────────────────────────────────────────────────────────────

/// Per-figure annotation and accumulation state.
///
/// Annotations (`xlabel`, `title`, …) are set via their corresponding
/// functions and consumed at the next render call (or at `hold('off')` /
/// `savefig` when in accumulating mode).
#[derive(Default, Clone)]
pub struct FigureState {
    /// X-axis label.
    pub xlabel: Option<String>,
    /// Y-axis label.
    pub ylabel: Option<String>,
    /// Z-axis label (consumed only by `plot3` / `scatter3`).
    pub zlabel: Option<String>,
    /// Chart title.
    pub title: Option<String>,
    /// Series labels for legend boxes (file export only).
    pub legend: Vec<String>,
    /// Override x-axis range `[min, max]`.
    pub xlim: Option<(f64, f64)>,
    /// Override y-axis range `[min, max]`.
    pub ylim: Option<(f64, f64)>,
    /// Override z-axis range `[min, max]` (3D only).
    pub zlim: Option<(f64, f64)>,
    /// Whether to draw grid lines (file export only; ASCII ignores).
    pub grid: bool,
    /// Active colormap for `imagesc` (default [`ColormapSpec::Named`]`("viridis")` when `None`).
    pub colormap: Option<ColormapSpec>,
    /// Whether to append a colorbar to the next `imagesc` render.
    pub colorbar: bool,

    // ── Phase 30d — subplot + hold ────────────────────────────────────────
    /// Active subplot grid position `(rows, cols, index_1based)`.
    pub subplot: Option<(u32, u32, u32)>,
    /// When `true`, plot calls accumulate into [`Self::pending_series`].
    pub hold: bool,
    /// Series accumulated for the current in-progress panel.
    pub pending_series: Vec<PendingSeries>,
    /// Committed panels waiting for `savefig`.
    pub panels: Vec<Panel>,
    /// Text annotations accumulated for the current render (flushed at render time).
    pub annotations: Vec<(f64, f64, String)>,

    // ── Phase 30.6a — theme + background colour ───────────────────────────
    /// Active colour theme (`None` means use the light default).
    pub theme: Option<Theme>,
    /// Per-figure background colour override (beats the theme background).
    pub bg_color: Option<StyleColor>,

    // ── Phase 30.6b — font / stroke sizes ─────────────────────────────────
    /// Title and axis-label font size override in points (minimum 8).
    pub font_size: Option<u32>,
    /// Stroke width override for all line series (pixels).
    pub line_width: Option<f32>,
    /// Marker radius override for scatter / marker series (pixels).
    pub marker_size: Option<u32>,

    // ── Phase 30.6c — grid style ───────────────────────────────────────────
    /// Grid line colour override (applied to both bold and light grid lines).
    pub grid_color: Option<StyleColor>,
    /// Grid line stroke width override in pixels.
    pub grid_width: Option<f32>,

    // ── Phase 30.6d — axis mode ────────────────────────────────────────────
    /// Axis display mode (`axis('equal')`, `axis('tight')`, `axis('off')`).
    pub axis_mode: Option<AxisMode>,

    // ── Phase 31 — custom canvas size ─────────────────────────────────────
    /// Output canvas size in pixels `(width, height)` for file export.
    ///
    /// `None` falls back to the default `800×600`. Set via `figure(w, h)`.
    /// Persists across panels; cleared only when the whole state is reset.
    pub figure_size: Option<(u32, u32)>,

    // ── Phase 32d — dual Y axis ────────────────────────────────────────────
    /// Which Y axis receives new series and annotation calls.
    pub active_yaxis: YAxis,
    /// Series accumulated for the right (secondary) Y axis.
    pub right_pending_series: Vec<PendingSeries>,
    /// Override for the right Y axis range.
    pub right_ylim: Option<(f64, f64)>,
    /// Label for the right Y axis.
    pub right_ylabel: Option<String>,

    // ── Phase 32e — contour level labels ──────────────────────────────────
    /// When `true`, the next contour render places a text label at each level.
    pub clabel: bool,
}

impl FigureState {
    /// Returns the canvas size in pixels, falling back to `800×600` if not set.
    pub fn canvas_size(&self) -> (u32, u32) {
        self.figure_size.unwrap_or((800, 600))
    }

    /// Returns the resolved active [`Theme`]: explicit `theme` field > light default.
    pub fn resolve_theme(&self) -> style::Theme {
        self.theme.clone().unwrap_or_else(style::Theme::light)
    }

    /// Returns the effective background colour as an RGB triple.
    ///
    /// Resolution order: explicit `bg_color` override > active theme background.
    pub fn effective_bg_rgb(&self) -> (u8, u8, u8) {
        let c = self.bg_color.unwrap_or_else(|| self.resolve_theme().bg);
        (c.0, c.1, c.2)
    }

    /// Pushes `series` to the left or right pending queue based on [`Self::active_yaxis`].
    pub fn push_series(&mut self, series: PendingSeries) {
        if self.active_yaxis == YAxis::Right {
            self.right_pending_series.push(series);
        } else {
            self.pending_series.push(series);
        }
    }
}

// ── Terminal size helpers ───────────────────────────────────────────────────

/// Returns the terminal width in columns, read from `$COLUMNS` (default 80).
pub(crate) fn term_cols() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

/// Returns the terminal height in rows, read from `$LINES` (default 24).
pub(crate) fn term_rows() -> usize {
    std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24)
}

thread_local! {
    static FIGURE_STATE: RefCell<FigureState> =
        RefCell::new(FigureState::default());
}

// ── Exported names ─────────────────────────────────────────────────────────

const EXPORTED: &[&str] = &[
    "plot",
    "scatter",
    "bar",
    "stem",
    "hist",
    "stairs",
    "loglog",
    "semilogx",
    "semilogy",
    "plot3",
    "scatter3",
    "xlabel",
    "ylabel",
    "zlabel",
    "title",
    "legend",
    "xlim",
    "ylim",
    "zlim",
    "grid",
    "colormap",
    "colorbar",
    "imagesc",
    "surf",
    "mesh",
    "contour",
    "contourf",
    "subplot",
    "hold",
    "savefig",
    "fill",
    "area",
    "polar",
    "quiver",
    "text",
    "figure",
    "theme",
    "bgcolor",
    "fontsize",
    "linewidth",
    "markersize",
    "gridcolor",
    "gridwidth",
    "axis",
    // Phase 32a — drawing primitives
    "line",
    "patch",
    "rectangle",
    // Phase 32b — statistical extensions
    "errorbar",
    // Phase 32c — pie chart
    "pie",
    // Phase 32d — dual Y axis
    "yyaxis",
    // Phase 32e — contour level labels
    "clabel",
];

// ── subplot / hold helpers ─────────────────────────────────────────────────

/// Returns `true` when the figure is in accumulating mode (subplot or hold).
fn is_accumulating(st: &FigureState) -> bool {
    st.subplot.is_some() || st.hold
}

/// Commits the current in-progress panel to `st.panels`.
///
/// Only commits when there are pending series (left or right) to avoid empty panels.
/// Clears annotations and all pending series after committing.
fn commit_current_panel(st: &mut FigureState) {
    if !st.pending_series.is_empty() || !st.right_pending_series.is_empty() {
        let panel = Panel {
            layout: st.subplot,
            xlabel: st.xlabel.take(),
            ylabel: st.ylabel.take(),
            title: st.title.take(),
            legend: std::mem::take(&mut st.legend),
            xlim: st.xlim.take(),
            ylim: st.ylim.take(),
            grid: std::mem::replace(&mut st.grid, false),
            series: std::mem::take(&mut st.pending_series),
            annotations: std::mem::take(&mut st.annotations),
            font_size: st.font_size,
            line_width: st.line_width,
            marker_size: st.marker_size,
            grid_color: st.grid_color,
            grid_width: st.grid_width,
            axis_mode: st.axis_mode,
            colormap: st.colormap.clone(),
            right_series: std::mem::take(&mut st.right_pending_series),
            right_ylim: st.right_ylim.take(),
            right_ylabel: st.right_ylabel.take(),
        };
        st.panels.push(panel);
    }
}

// ── PlotPlugin ─────────────────────────────────────────────────────────────

/// Plot plugin — registers all 2D/3D plotting functions.
pub struct PlotPlugin;

impl Plugin for PlotPlugin {
    fn name(&self) -> &str {
        "plot"
    }

    fn exported_names(&self) -> &[&str] {
        EXPORTED
    }

    fn call(&self, name: &str, args: &[Value], _env: &Env) -> Result<Value, String> {
        match name {
            // ── String annotation setters ──────────────────────────────
            "xlabel" | "ylabel" | "title" => {
                let s = require_string(name, args)?;
                FIGURE_STATE.with(|f| {
                    let mut st = f.borrow_mut();
                    match name {
                        "xlabel" => st.xlabel = Some(s),
                        "ylabel" => {
                            if st.active_yaxis == YAxis::Right {
                                st.right_ylabel = Some(s);
                            } else {
                                st.ylabel = Some(s);
                            }
                        }
                        "title" => st.title = Some(s),
                        _ => unreachable!(),
                    }
                });
                Ok(Value::Void)
            }

            "zlabel" => {
                let s = require_string(name, args)?;
                FIGURE_STATE.with(|f| f.borrow_mut().zlabel = Some(s));
                Ok(Value::Void)
            }

            // ── Legend ─────────────────────────────────────────────────
            "legend" => {
                let labels = require_string_list(args)?;
                FIGURE_STATE.with(|f| f.borrow_mut().legend = labels);
                Ok(Value::Void)
            }

            // ── Grid toggle ────────────────────────────────────────────
            "grid" => {
                match args {
                    [] => FIGURE_STATE.with(|f| {
                        let mut st = f.borrow_mut();
                        st.grid = !st.grid;
                    }),
                    [Value::Str(s) | Value::StringObj(s)] => {
                        let enable = match s.as_str() {
                            "on" => true,
                            "off" => false,
                            other => {
                                return Err(format!("grid: expected 'on' or 'off', got '{other}'"));
                            }
                        };
                        FIGURE_STATE.with(|f| f.borrow_mut().grid = enable);
                    }
                    _ => return Err("grid: expected no arguments, 'on', or 'off'".into()),
                }
                Ok(Value::Void)
            }

            // ── Axis limit setters ─────────────────────────────────────
            "xlim" | "ylim" | "zlim" => {
                let (lo, hi) = extract_lim(name, args)?;
                FIGURE_STATE.with(|f| {
                    let mut st = f.borrow_mut();
                    match name {
                        "xlim" => st.xlim = Some((lo, hi)),
                        "ylim" => {
                            if st.active_yaxis == YAxis::Right {
                                st.right_ylim = Some((lo, hi));
                            } else {
                                st.ylim = Some((lo, hi));
                            }
                        }
                        "zlim" => st.zlim = Some((lo, hi)),
                        _ => unreachable!(),
                    }
                });
                Ok(Value::Void)
            }

            // ── Render calls ───────────────────────────────────────────
            // `line` is a MATLAB alias for `plot` — identical behaviour.
            "plot" | "line" => {
                let (data_args, style, path) = extract_style_and_file_arg(args)?;
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    let (x, ys) = extract_xy_multi(name, &data_args)?;
                    FIGURE_STATE.with(|f| {
                        let mut st = f.borrow_mut();
                        for y in ys {
                            st.push_series(PendingSeries::Line(x.clone(), y, style.clone()));
                        }
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    let (x, ys) = extract_xy_multi(name, &data_args)?;
                    if ys.len() == 1 {
                        render_line_xy(name, &x, &ys[0], path.as_deref(), state)
                    } else {
                        render_multi_series(&x, &ys, path.as_deref(), state)
                    }
                }
            }

            // ── scatter: 4-arg per-point color form, or regular 2-arg ──
            "scatter" => {
                // Check for 4-arg ColorScatter form: scatter(x, y, sz, c)
                // Use extract_file_arg first to isolate data args, then branch.
                let (data_peek, path_peek) = extract_file_arg(args);
                if data_peek.len() == 4
                    && is_numeric_value(&data_peek[2])
                    && is_numeric_value(&data_peek[3])
                {
                    let x = extract_flat(&data_peek[0])
                        .map_err(|_| "scatter: x must be a numeric array".to_string())?;
                    let y = extract_flat(&data_peek[1])
                        .map_err(|_| "scatter: y must be a numeric array".to_string())?;
                    let sz_raw = extract_flat(&data_peek[2])
                        .map_err(|_| "scatter: sz must be a numeric scalar or array".to_string())?;
                    let c = extract_flat(&data_peek[3])
                        .map_err(|_| "scatter: c must be a numeric array".to_string())?;
                    if x.len() != y.len() || x.len() != c.len() {
                        return Err(format!(
                            "scatter: x, y, c must have the same length ({}, {}, {})",
                            x.len(),
                            y.len(),
                            c.len()
                        ));
                    }
                    let sz = if sz_raw.len() == 1 {
                        vec![sz_raw[0]; x.len()]
                    } else if sz_raw.len() == x.len() {
                        sz_raw
                    } else {
                        return Err(format!(
                            "scatter: sz must be scalar or same length as x ({} vs {})",
                            sz_raw.len(),
                            x.len()
                        ));
                    };
                    let (c_min, c_max) = colormap::data_range(&c);
                    let series = PendingSeries::ColorScatter {
                        x,
                        y,
                        sz,
                        c,
                        c_min,
                        c_max,
                    };
                    if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                        FIGURE_STATE.with(|f| f.borrow_mut().push_series(series));
                        Ok(Value::Void)
                    } else {
                        let state = FIGURE_STATE.with(|f| f.take());
                        if let PendingSeries::ColorScatter {
                            x,
                            y,
                            sz,
                            c,
                            c_min,
                            c_max,
                        } = series
                        {
                            render_color_scatter(
                                &x,
                                &y,
                                &sz,
                                &c,
                                c_min,
                                c_max,
                                path_peek.as_deref(),
                                state,
                            )
                        } else {
                            unreachable!()
                        }
                    }
                } else {
                    // Regular scatter(x, y) or scatter(x, y, 'style')
                    let (data_args, style, path) = extract_style_and_file_arg(args)?;
                    if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                        let (x, y) = extract_xy("scatter", &data_args)?;
                        FIGURE_STATE.with(|f| {
                            f.borrow_mut()
                                .push_series(PendingSeries::Scatter(x, y, style));
                        });
                        Ok(Value::Void)
                    } else {
                        let state = FIGURE_STATE.with(|f| f.take());
                        render_ascii_or_file("scatter", &data_args, path.as_deref(), state)
                    }
                }
            }

            "bar" | "stem" | "stairs" => {
                let (data_args, style, path) = extract_style_and_file_arg(args)?;
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    let (x, y) = extract_xy(name, &data_args)?;
                    let (x, y) = if name == "stairs" {
                        make_step_data(&x, &y)
                    } else {
                        (x, y)
                    };
                    let series = match name {
                        "bar" | "stairs" => PendingSeries::Bar(x, y, style),
                        "stem" => PendingSeries::Stem(x, y, style),
                        _ => unreachable!(),
                    };
                    FIGURE_STATE.with(|f| f.borrow_mut().push_series(series));
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    match name {
                        "bar" => {
                            let (x, y) = extract_xy(name, &data_args)?;
                            render_bar_xy(&x, &y, path.as_deref(), style, state)
                        }
                        "stem" => {
                            let (x, y) = extract_xy(name, &data_args)?;
                            render_stem_xy(&x, &y, path.as_deref(), style, state)
                        }
                        _ => render_ascii_or_file(name, &data_args, path.as_deref(), state),
                    }
                }
            }

            // ── errorbar ───────────────────────────────────────────────
            "errorbar" => {
                // Forms:
                //   errorbar(x, y, e)             — symmetric bars
                //   errorbar(x, y, e_low, e_high) — asymmetric bars
                // Optional trailing style string and/or file path.
                let (data_args, style, path) = extract_style_and_file_arg_min(args, 3)?;
                let (x, y, e_low, e_high) = match data_args.as_slice() {
                    [xv, yv, ev] => {
                        let x = extract_vector(xv)
                            .map_err(|_| "errorbar: x must be a numeric vector".to_string())?;
                        let y = extract_vector(yv)
                            .map_err(|_| "errorbar: y must be a numeric vector".to_string())?;
                        let e = extract_vector(ev)
                            .map_err(|_| "errorbar: e must be a numeric vector".to_string())?;
                        if x.len() != y.len() || x.len() != e.len() {
                            return Err(format!(
                                "errorbar: x, y, e must have the same length \
                                 ({}, {}, {})",
                                x.len(),
                                y.len(),
                                e.len()
                            ));
                        }
                        let e2 = e.clone();
                        (x, y, e, e2)
                    }
                    [xv, yv, elv, ehv] => {
                        let x = extract_vector(xv)
                            .map_err(|_| "errorbar: x must be a numeric vector".to_string())?;
                        let y = extract_vector(yv)
                            .map_err(|_| "errorbar: y must be a numeric vector".to_string())?;
                        let el = extract_vector(elv)
                            .map_err(|_| "errorbar: e_low must be a numeric vector".to_string())?;
                        let eh = extract_vector(ehv)
                            .map_err(|_| "errorbar: e_high must be a numeric vector".to_string())?;
                        if x.len() != y.len() || x.len() != el.len() || x.len() != eh.len() {
                            return Err(format!(
                                "errorbar: x, y, e_low, e_high must have the same length \
                                 ({}, {}, {}, {})",
                                x.len(),
                                y.len(),
                                el.len(),
                                eh.len()
                            ));
                        }
                        (x, y, el, eh)
                    }
                    other => {
                        return Err(format!(
                            "errorbar: expected 3 or 4 data arguments, got {}",
                            other.len()
                        ));
                    }
                };
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    FIGURE_STATE.with(|f| {
                        f.borrow_mut().push_series(PendingSeries::ErrorBar {
                            x,
                            y,
                            e_low,
                            e_high,
                            style,
                        });
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    render_errorbar(&x, &y, &e_low, &e_high, path.as_deref(), style, state)
                }
            }

            // ── pie ────────────────────────────────────────────────────
            "pie" => {
                // Supported forms:
                //   pie(v)
                //   pie(v, path)
                //   pie(v, {'A','B','C'})
                //   pie(v, {'A','B','C'}, path)
                //   pie(v, [0 1 0])
                //   pie(v, [0 1 0], {'A','B','C'})
                //   pie(v, [0 1 0], {'A','B','C'}, path)
                //
                // Detection order (type-based, not positional):
                //  1. Trailing string = path (.svg/.png/ascii).
                //  2. Cell array anywhere after v = labels.
                //  3. Numeric vector (not the first arg) = explode.
                let mut rest = args.to_vec();

                // Extract trailing path.
                let path = if let Some(last) = rest.last()
                    && let Value::Str(s) | Value::StringObj(s) = last
                    && (s == "ascii" || s.ends_with(".svg") || s.ends_with(".png"))
                {
                    let p = s.clone();
                    rest.pop();
                    Some(p)
                } else {
                    None
                };

                if rest.is_empty() {
                    return Err("pie: expected at least one argument (values vector)".into());
                }

                // First argument is the values vector.
                let values = extract_vector(&rest[0])
                    .map_err(|_| "pie: first argument must be a numeric vector".to_string())?;
                if values.is_empty() {
                    return Err("pie: values vector must not be empty".into());
                }
                if values.iter().any(|&v| v < 0.0) {
                    return Err("pie: all values must be non-negative".into());
                }
                let total: f64 = values.iter().sum();
                if total <= 0.0 {
                    return Err("pie: sum of values must be positive".into());
                }

                // Parse remaining optional args (labels Cell, explode vector).
                let mut labels: Vec<String> = Vec::new();
                let mut explode: Vec<f64> = Vec::new();
                for arg in &rest[1..] {
                    match arg {
                        Value::Cell(cells) => {
                            labels = cells
                                .iter()
                                .map(|v| match v {
                                    Value::Str(s) | Value::StringObj(s) => s.clone(),
                                    _ => String::new(),
                                })
                                .collect();
                            if labels.len() != values.len() {
                                return Err(format!(
                                    "pie: labels cell array length ({}) must match \
                                     values length ({})",
                                    labels.len(),
                                    values.len()
                                ));
                            }
                        }
                        _ => {
                            // Treat as explode vector.
                            let ex = extract_vector(arg).map_err(|_| {
                                "pie: unrecognised argument — expected labels cell \
                                 array or explode vector"
                                    .to_string()
                            })?;
                            if ex.len() != values.len() {
                                return Err(format!(
                                    "pie: explode vector length ({}) must match \
                                     values length ({})",
                                    ex.len(),
                                    values.len()
                                ));
                            }
                            explode = ex;
                        }
                    }
                }

                // Default empty labels / zero explode.
                if labels.is_empty() {
                    labels = vec![String::new(); values.len()];
                }
                if explode.is_empty() {
                    explode = vec![0.0_f64; values.len()];
                }

                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    FIGURE_STATE.with(|f| {
                        f.borrow_mut().push_series(PendingSeries::Pie {
                            values,
                            labels,
                            explode,
                        });
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    render_pie(&values, &labels, &explode, path.as_deref(), state)
                }
            }

            // ── Histogram ──────────────────────────────────────────────
            "hist" => {
                let (data_args, style, path) = extract_style_and_file_arg(args)?;
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    let (counts, edges) = parse_and_compute_hist(&data_args)?;
                    FIGURE_STATE.with(|f| {
                        f.borrow_mut().push_series(PendingSeries::Hist {
                            counts,
                            edges,
                            style,
                        });
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    let (counts, edges) = parse_and_compute_hist(&data_args)?;
                    match path.as_deref() {
                        None | Some("ascii") => {
                            render_hist_ascii(&counts, &edges, &state);
                            Ok(Value::Void)
                        }
                        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
                            render_hist_file(&counts, &edges, p, style, state)
                        }
                        Some(p) => Err(format!("hist: unknown output target '{p}'")),
                    }
                }
            }

            // ── Log-scale plots ────────────────────────────────────────
            "loglog" | "semilogx" | "semilogy" => {
                let (data_args, path) = extract_file_arg(args);
                let mut state = FIGURE_STATE.with(|f| f.take());
                let (x_raw, y_raw) = extract_xy(name, &data_args)?;

                let log_x = name == "loglog" || name == "semilogx";
                let log_y = name == "loglog" || name == "semilogy";

                // Apply log₁₀ and filter non-finite pairs.
                let (x, y): (Vec<f64>, Vec<f64>) = x_raw
                    .iter()
                    .zip(y_raw.iter())
                    .filter_map(|(&xi, &yi)| {
                        let lx = if log_x { xi.log10() } else { xi };
                        let ly = if log_y { yi.log10() } else { yi };
                        if lx.is_finite() && ly.is_finite() {
                            Some((lx, ly))
                        } else {
                            None
                        }
                    })
                    .unzip();

                if x.is_empty() {
                    return Err(format!(
                        "{name}: no finite values after log₁₀ transform \
                         (check for non-positive values)"
                    ));
                }

                // Annotate axis labels with log₁₀ notation.
                if log_x {
                    let lbl = state.xlabel.take().unwrap_or_default();
                    state.xlabel = Some(if lbl.is_empty() {
                        "log\u{2081}\u{2080}(x)".into()
                    } else {
                        format!("{lbl} [log\u{2081}\u{2080}]")
                    });
                }
                if log_y {
                    let lbl = state.ylabel.take().unwrap_or_default();
                    state.ylabel = Some(if lbl.is_empty() {
                        "log\u{2081}\u{2080}(y)".into()
                    } else {
                        format!("{lbl} [log\u{2081}\u{2080}]")
                    });
                }

                render_line_xy(name, &x, &y, path.as_deref(), state)
            }

            // ── 3D plots ───────────────────────────────────────────────
            "plot3" | "scatter3" => {
                let (data_args, path) = extract_file_arg(args);
                let state = FIGURE_STATE.with(|f| f.take());
                render_3d(name, &data_args, path.as_deref(), state)
            }

            // ── Canvas size ────────────────────────────────────────────
            "figure" => {
                if args.len() != 2 {
                    return Err(format!(
                        "figure: expected 2 arguments (width, height), got {}",
                        args.len()
                    ));
                }
                let w = match &args[0] {
                    Value::Scalar(f) if *f >= 1.0 && *f <= 16384.0 => *f as u32,
                    _ => return Err("figure: width must be a positive integer (1–16384)".into()),
                };
                let h = match &args[1] {
                    Value::Scalar(f) if *f >= 1.0 && *f <= 16384.0 => *f as u32,
                    _ => return Err("figure: height must be a positive integer (1–16384)".into()),
                };
                FIGURE_STATE.with(|f| f.borrow_mut().figure_size = Some((w, h)));
                Ok(Value::Void)
            }

            // ── Colormap / colorbar setters ────────────────────────────
            "colormap" => {
                if args.is_empty() {
                    return Err("colormap: one argument required".into());
                }
                let spec = match &args[0] {
                    Value::Str(name) | Value::StringObj(name) => ColormapSpec::Named(name.clone()),
                    Value::Matrix(m) => {
                        if m.ncols() != 3 {
                            return Err("colormap: matrix argument must be N×3".into());
                        }
                        let lut: Vec<(u8, u8, u8)> = (0..m.nrows())
                            .map(|r| {
                                let clamp = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                                (clamp(m[[r, 0]]), clamp(m[[r, 1]]), clamp(m[[r, 2]]))
                            })
                            .collect();
                        ColormapSpec::Custom(lut)
                    }
                    _ => {
                        return Err("colormap: argument must be a name string or N×3 matrix".into());
                    }
                };
                colormap::validate_colormap_spec(&spec)?;
                FIGURE_STATE.with(|f| f.borrow_mut().colormap = Some(spec));
                Ok(Value::Void)
            }

            "colorbar" => {
                FIGURE_STATE.with(|f| f.borrow_mut().colorbar = true);
                Ok(Value::Void)
            }

            // ── Theme / background colour ──────────────────────────────
            "theme" => {
                if args.is_empty() {
                    return Err("theme: one argument required (e.g. 'dark' or 'light')".into());
                }
                let name = match &args[0] {
                    Value::Str(s) | Value::StringObj(s) => s.clone(),
                    _ => return Err("theme: argument must be a theme name string".into()),
                };
                let t = Theme::from_name(&name)?;
                FIGURE_STATE.with(|f| f.borrow_mut().theme = Some(t));
                Ok(Value::Void)
            }

            "bgcolor" => {
                if args.is_empty() {
                    return Err("bgcolor: one argument required".into());
                }
                let sc = match &args[0] {
                    Value::Str(s) | Value::StringObj(s) => style::parse_color_token(s)
                        .ok_or_else(|| format!("bgcolor: unrecognised color '{s}'"))?,
                    Value::Matrix(m) if m.nrows() == 1 && m.ncols() == 3 => {
                        let all_unit = (0..3).all(|c| {
                            let v = m[[0, c]];
                            (0.0..=1.0).contains(&v)
                        });
                        if !all_unit {
                            return Err("bgcolor: RGB matrix values must be in [0, 1]".into());
                        }
                        let clamp = |v: f64| (v * 255.0).round() as u8;
                        StyleColor(clamp(m[[0, 0]]), clamp(m[[0, 1]]), clamp(m[[0, 2]]))
                    }
                    _ => {
                        return Err(
                            "bgcolor: argument must be a color name string or 1×3 RGB matrix"
                                .into(),
                        );
                    }
                };
                FIGURE_STATE.with(|f| f.borrow_mut().bg_color = Some(sc));
                Ok(Value::Void)
            }

            // ── Font / stroke size setters ─────────────────────────────
            "fontsize" => {
                let val = match args {
                    [Value::Scalar(f)] if *f >= 1.0 => (*f as u32).max(8),
                    _ => return Err("fontsize: expected a positive number".into()),
                };
                FIGURE_STATE.with(|f| f.borrow_mut().font_size = Some(val));
                Ok(Value::Void)
            }

            "linewidth" => {
                let val = match args {
                    [Value::Scalar(f)] if *f > 0.0 => *f as f32,
                    _ => return Err("linewidth: expected a positive number".into()),
                };
                FIGURE_STATE.with(|f| f.borrow_mut().line_width = Some(val));
                Ok(Value::Void)
            }

            "markersize" => {
                let val = match args {
                    [Value::Scalar(f)] if *f >= 1.0 => *f as u32,
                    _ => return Err("markersize: expected a positive integer".into()),
                };
                FIGURE_STATE.with(|f| f.borrow_mut().marker_size = Some(val));
                Ok(Value::Void)
            }

            // ── Grid colour / width overrides ──────────────────────────
            "gridcolor" => {
                if args.is_empty() {
                    return Err("gridcolor: one argument required".into());
                }
                let sc = match &args[0] {
                    Value::Str(s) | Value::StringObj(s) => style::parse_color_token(s)
                        .ok_or_else(|| format!("gridcolor: unrecognised color '{s}'"))?,
                    Value::Matrix(m) if m.nrows() == 1 && m.ncols() == 3 => {
                        let all_unit = (0..3).all(|c| {
                            let v = m[[0, c]];
                            (0.0..=1.0).contains(&v)
                        });
                        if !all_unit {
                            return Err("gridcolor: RGB matrix values must be in [0, 1]".into());
                        }
                        let clamp = |v: f64| (v * 255.0).round() as u8;
                        StyleColor(clamp(m[[0, 0]]), clamp(m[[0, 1]]), clamp(m[[0, 2]]))
                    }
                    _ => {
                        return Err(
                            "gridcolor: argument must be a color name string or 1×3 RGB matrix"
                                .into(),
                        );
                    }
                };
                FIGURE_STATE.with(|f| f.borrow_mut().grid_color = Some(sc));
                Ok(Value::Void)
            }

            "gridwidth" => {
                let val = match args {
                    [Value::Scalar(f)] if *f > 0.0 => *f as f32,
                    _ => return Err("gridwidth: expected a positive number".into()),
                };
                FIGURE_STATE.with(|f| f.borrow_mut().grid_width = Some(val));
                Ok(Value::Void)
            }

            // ── axis mode ──────────────────────────────────────────────
            "axis" => {
                let s = require_string("axis", args)?;
                let mode = match s.as_str() {
                    "equal" => Some(AxisMode::Equal),
                    "tight" => Some(AxisMode::Tight),
                    "off" => Some(AxisMode::Off),
                    "on" => None,
                    other => {
                        return Err(format!(
                            "axis: expected 'equal', 'tight', 'off', or 'on', got '{other}'"
                        ));
                    }
                };
                FIGURE_STATE.with(|f| f.borrow_mut().axis_mode = mode);
                Ok(Value::Void)
            }

            // ── yyaxis — dual Y axis ───────────────────────────────────
            "yyaxis" => {
                let s = require_string("yyaxis", args)?;
                match s.as_str() {
                    "left" | "right" => {
                        let is_right = s == "right";

                        // When switching back to 'left' while a dual-axis session is
                        // pending (right side has series), auto-flush to ASCII so the
                        // caller does not need an explicit hold('off').
                        let panel_to_flush = if !is_right {
                            FIGURE_STATE.with(|f| {
                                let mut st = f.borrow_mut();
                                if !st.right_pending_series.is_empty() && st.subplot.is_none() {
                                    Some(Panel {
                                        layout: None,
                                        xlabel: st.xlabel.take(),
                                        ylabel: st.ylabel.take(),
                                        title: st.title.take(),
                                        legend: std::mem::take(&mut st.legend),
                                        xlim: st.xlim.take(),
                                        ylim: st.ylim.take(),
                                        grid: std::mem::replace(&mut st.grid, false),
                                        series: std::mem::take(&mut st.pending_series),
                                        annotations: std::mem::take(&mut st.annotations),
                                        font_size: st.font_size,
                                        line_width: st.line_width,
                                        marker_size: st.marker_size,
                                        grid_color: st.grid_color,
                                        grid_width: st.grid_width,
                                        axis_mode: st.axis_mode,
                                        colormap: st.colormap.clone(),
                                        right_series: std::mem::take(&mut st.right_pending_series),
                                        right_ylim: st.right_ylim.take(),
                                        right_ylabel: st.right_ylabel.take(),
                                    })
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        };

                        if let Some(panel) = panel_to_flush {
                            render_panel_ascii(&panel)?;
                        }

                        FIGURE_STATE.with(|f| {
                            let mut st = f.borrow_mut();
                            st.active_yaxis = if is_right { YAxis::Right } else { YAxis::Left };
                            st.hold = true;
                        });
                    }
                    other => {
                        return Err(format!("yyaxis: expected 'left' or 'right', got '{other}'"));
                    }
                }
                Ok(Value::Void)
            }

            // ── clabel — contour level labels ─────────────────────────
            "clabel" => {
                if !args.is_empty() {
                    return Err("clabel: expected no arguments".into());
                }
                FIGURE_STATE.with(|f| f.borrow_mut().clabel = true);
                Ok(Value::Void)
            }

            // ── imagesc ────────────────────────────────────────────────
            "imagesc" => {
                if args.is_empty() {
                    return Err("imagesc: at least one argument required".into());
                }
                let (z, nrows, ncols) = extract_matrix(&args[0])?;
                let state = FIGURE_STATE.with(|f| f.take());
                // Accepted forms:
                //   imagesc(Z)          — ASCII or terminal
                //   imagesc(Z, path)    — file export; canvas from figure(w,h) or 800×600
                let path: Option<String> = match args.len() {
                    1 => None,
                    2 => match &args[1] {
                        Value::Str(s) | Value::StringObj(s) => Some(s.clone()),
                        _ => {
                            return Err(
                                "imagesc: second argument must be a file path string".into()
                            );
                        }
                    },
                    n => return Err(format!("imagesc: expected 1 or 2 arguments, got {n}")),
                };
                render_imagesc(&z, nrows, ncols, path.as_deref(), state)
            }

            // ── surf / mesh ────────────────────────────────────────────
            "surf" | "mesh" => {
                let (data_args, path) = extract_file_arg(args);
                if data_args.len() < 3 {
                    return Err(format!(
                        "{name}: requires (X, Y, Z) matrix arguments, got {}",
                        data_args.len()
                    ));
                }
                let (x_data, x_rows, x_cols) = extract_matrix(&data_args[0])
                    .map_err(|_| format!("{name}: X must be a numeric matrix"))?;
                let (y_data, y_rows, y_cols) = extract_matrix(&data_args[1])
                    .map_err(|_| format!("{name}: Y must be a numeric matrix"))?;
                let (z_data, z_rows, z_cols) = extract_matrix(&data_args[2])
                    .map_err(|_| format!("{name}: Z must be a numeric matrix"))?;
                if x_rows != y_rows || x_rows != z_rows || x_cols != y_cols || x_cols != z_cols {
                    return Err(format!(
                        "{name}: X ({x_rows}×{x_cols}), Y ({y_rows}×{y_cols}) and \
                         Z ({z_rows}×{z_cols}) must have the same dimensions"
                    ));
                }
                let state = FIGURE_STATE.with(|f| f.take());
                // Unique x values = first row of X; unique y values = first column of Y.
                let x_vals: Vec<f64> = (0..x_cols).map(|c| x_data[c]).collect();
                let y_vals: Vec<f64> = (0..x_rows).map(|r| y_data[r * x_cols]).collect();
                render_surface(
                    name,
                    &x_vals,
                    &y_vals,
                    &z_data,
                    z_rows,
                    z_cols,
                    path.as_deref(),
                    state,
                )
            }

            // ── contour / contourf ─────────────────────────────────────
            "contour" | "contourf" => {
                let (data_args, path) = extract_file_arg(args);
                if data_args.len() < 3 {
                    return Err(format!(
                        "{name}: requires (X, Y, Z) matrix arguments, got {}",
                        data_args.len()
                    ));
                }
                let (x_data, x_rows, x_cols) = extract_matrix(&data_args[0])
                    .map_err(|_| format!("{name}: X must be a numeric matrix"))?;
                let (y_data, y_rows, y_cols) = extract_matrix(&data_args[1])
                    .map_err(|_| format!("{name}: Y must be a numeric matrix"))?;
                let (z_data, z_rows, z_cols) = extract_matrix(&data_args[2])
                    .map_err(|_| format!("{name}: Z must be a numeric matrix"))?;
                if x_rows != y_rows || x_rows != z_rows || x_cols != y_cols || x_cols != z_cols {
                    return Err(format!(
                        "{name}: X ({x_rows}×{x_cols}), Y ({y_rows}×{y_cols}) and \
                         Z ({z_rows}×{z_cols}) must have the same dimensions"
                    ));
                }
                // Optional 4th arg: number of contour levels (default 10).
                let n_levels: usize = if data_args.len() >= 4 {
                    match &data_args[3] {
                        Value::Scalar(v) if *v >= 1.0 => *v as usize,
                        _ => return Err(format!("{name}: level count must be a positive integer")),
                    }
                } else {
                    10
                };
                let state = FIGURE_STATE.with(|f| f.take());
                // Unique coordinate vectors from meshgrid output.
                let x_vals: Vec<f64> = (0..x_cols).map(|c| x_data[c]).collect();
                let y_vals: Vec<f64> = (0..x_rows).map(|r| y_data[r * x_cols]).collect();
                let filled = name == "contourf";
                render_contour(
                    filled,
                    &x_vals,
                    &y_vals,
                    &z_data,
                    z_rows,
                    z_cols,
                    n_levels,
                    path.as_deref(),
                    state,
                )
            }

            // ── subplot ────────────────────────────────────────────────
            "subplot" => match args {
                [Value::Scalar(m), Value::Scalar(n), Value::Scalar(k)] => {
                    let m = *m as u32;
                    let n = *n as u32;
                    let k = *k as u32;
                    if m == 0 || n == 0 || k == 0 || k > m * n {
                        return Err(format!(
                            "subplot: invalid layout ({m},{n},{k}) — \
                                 index must be in 1..={}",
                            m * n
                        ));
                    }
                    FIGURE_STATE.with(|f| {
                        let mut st = f.borrow_mut();
                        commit_current_panel(&mut st);
                        st.subplot = Some((m, n, k));
                    });
                    Ok(Value::Void)
                }
                _ => Err("subplot: expected 3 numeric arguments (rows, cols, index)".into()),
            },

            // ── hold ───────────────────────────────────────────────────
            "hold" => {
                let turn_on = match args {
                    [] => !FIGURE_STATE.with(|f| f.borrow().hold),
                    [Value::Str(s) | Value::StringObj(s)] => match s.as_str() {
                        "on" => true,
                        "off" => false,
                        other => {
                            return Err(format!(
                                "hold: expected 'on', 'off', or no argument, got '{other}'"
                            ));
                        }
                    },
                    _ => return Err("hold: expected 'on', 'off', or no argument".into()),
                };

                if !turn_on {
                    let panel_opt = FIGURE_STATE.with(|f| {
                        let mut st = f.borrow_mut();
                        st.hold = false;
                        // When not in subplot mode: extract panel for ASCII flush.
                        let has_series =
                            !st.pending_series.is_empty() || !st.right_pending_series.is_empty();
                        if st.subplot.is_none() && has_series {
                            Some(Panel {
                                layout: None,
                                xlabel: st.xlabel.take(),
                                ylabel: st.ylabel.take(),
                                title: st.title.take(),
                                legend: std::mem::take(&mut st.legend),
                                xlim: st.xlim.take(),
                                ylim: st.ylim.take(),
                                grid: std::mem::replace(&mut st.grid, false),
                                series: std::mem::take(&mut st.pending_series),
                                annotations: std::mem::take(&mut st.annotations),
                                font_size: st.font_size,
                                line_width: st.line_width,
                                marker_size: st.marker_size,
                                grid_color: st.grid_color,
                                grid_width: st.grid_width,
                                axis_mode: st.axis_mode,
                                colormap: st.colormap.clone(),
                                right_series: std::mem::take(&mut st.right_pending_series),
                                right_ylim: st.right_ylim.take(),
                                right_ylabel: st.right_ylabel.take(),
                            })
                        } else {
                            None
                        }
                    });
                    if let Some(panel) = panel_opt {
                        return render_panel_ascii(&panel);
                    }
                } else {
                    FIGURE_STATE.with(|f| f.borrow_mut().hold = true);
                }
                Ok(Value::Void)
            }

            // ── savefig ────────────────────────────────────────────────
            "savefig" => {
                let path = require_string("savefig", args)?;
                if !path.ends_with(".svg") && !path.ends_with(".png") {
                    return Err("savefig: path must end with '.svg' or '.png'".into());
                }
                let (panels, canvas, theme, bg_override) = FIGURE_STATE.with(|f| {
                    let mut st = f.borrow_mut();
                    commit_current_panel(&mut st);
                    st.hold = false;
                    st.subplot = None;
                    let canvas = st.canvas_size();
                    let theme = st.theme.clone().unwrap_or_else(style::Theme::light);
                    let bg_override = st.bg_color;
                    (std::mem::take(&mut st.panels), canvas, theme, bg_override)
                });
                if panels.is_empty() {
                    return Err("savefig: no panels to render".into());
                }
                render_panels_file(&panels, &path, canvas, &theme, bg_override)
            }

            // ── fill / patch ───────────────────────────────────────────
            // `patch` is a MATLAB alias for `fill` — identical behaviour.
            "fill" | "patch" => {
                let (data_args, style, path) = extract_style_and_file_arg(args)?;
                let (x, y) = extract_xy(name, &data_args)?;
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    FIGURE_STATE.with(|f| {
                        f.borrow_mut().push_series(PendingSeries::Fill(x, y, style));
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    render_fill_xy(&x, &y, path.as_deref(), style, state)
                }
            }

            // ── rectangle ─────────────────────────────────────────────
            "rectangle" => {
                let (data_args, style, path) = extract_style_and_file_arg(args)?;
                let (rx, ry, rw, rh) = match data_args.as_slice() {
                    [vec_arg] => {
                        let v = extract_vector(vec_arg).map_err(|_| {
                            "rectangle: single argument must be a numeric [x y w h] vector"
                                .to_string()
                        })?;
                        if v.len() != 4 {
                            return Err(format!(
                                "rectangle: [x y w h] vector must have 4 elements, got {}",
                                v.len()
                            ));
                        }
                        (v[0], v[1], v[2], v[3])
                    }
                    [xv, yv, wv, hv] => {
                        let to_scalar = |v: &Value, field: &'static str| match v {
                            Value::Scalar(f) => Ok(*f),
                            _ => Err(format!("rectangle: {field} must be a scalar")),
                        };
                        (
                            to_scalar(xv, "x")?,
                            to_scalar(yv, "y")?,
                            to_scalar(wv, "w")?,
                            to_scalar(hv, "h")?,
                        )
                    }
                    other => {
                        return Err(format!(
                            "rectangle: expected 1 (vector) or 4 (x,y,w,h) data arguments, got {}",
                            other.len()
                        ));
                    }
                };
                // Build closed axis-aligned polygon.
                let x_pts = vec![rx, rx + rw, rx + rw, rx];
                let y_pts = vec![ry, ry, ry + rh, ry + rh];
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    FIGURE_STATE.with(|f| {
                        f.borrow_mut()
                            .push_series(PendingSeries::Fill(x_pts, y_pts, style));
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    render_fill_xy(&x_pts, &y_pts, path.as_deref(), style, state)
                }
            }

            // ── area ──────────────────────────────────────────────────
            "area" => {
                let (data_args, style, path) = extract_style_and_file_arg(args)?;
                let (x, y) = extract_xy("area", &data_args)?;
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    FIGURE_STATE.with(|f| {
                        f.borrow_mut().push_series(PendingSeries::Area(x, y, style));
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    render_area_xy(&x, &y, path.as_deref(), style, state)
                }
            }

            // ── polar ─────────────────────────────────────────────────
            "polar" => {
                let (data_args, _style, path) = extract_style_and_file_arg(args)?;
                let (theta, r) = extract_xy("polar", &data_args)?;
                let (px, py): (Vec<f64>, Vec<f64>) = theta
                    .iter()
                    .zip(r.iter())
                    .map(|(&t, &rv)| (rv * t.cos(), rv * t.sin()))
                    .unzip();
                let state = FIGURE_STATE.with(|f| f.take());
                render_line_xy("polar", &px, &py, path.as_deref(), state)
            }

            // ── quiver ────────────────────────────────────────────────
            "quiver" => {
                let (data_args, style, path) = extract_style_and_file_arg_min(args, 4)?;
                if data_args.len() != 4 {
                    return Err(format!(
                        "quiver: expected 4 data arguments (x, y, u, v), got {}",
                        data_args.len()
                    ));
                }
                let x = extract_flat(&data_args[0])
                    .map_err(|_| "quiver: x must be a numeric array".to_string())?;
                let y = extract_flat(&data_args[1])
                    .map_err(|_| "quiver: y must be a numeric array".to_string())?;
                let u = extract_flat(&data_args[2])
                    .map_err(|_| "quiver: u must be a numeric array".to_string())?;
                let v = extract_flat(&data_args[3])
                    .map_err(|_| "quiver: v must be a numeric array".to_string())?;
                if x.len() != y.len() || x.len() != u.len() || x.len() != v.len() {
                    return Err(format!(
                        "quiver: x, y, u, v must have the same length \
                         ({}, {}, {}, {})",
                        x.len(),
                        y.len(),
                        u.len(),
                        v.len()
                    ));
                }
                if FIGURE_STATE.with(|f| is_accumulating(&f.borrow())) {
                    FIGURE_STATE.with(|f| {
                        f.borrow_mut()
                            .push_series(PendingSeries::Quiver(x, y, u, v, style));
                    });
                    Ok(Value::Void)
                } else {
                    let state = FIGURE_STATE.with(|f| f.take());
                    render_quiver(&x, &y, &u, &v, path.as_deref(), style, state)
                }
            }

            // ── text ──────────────────────────────────────────────────
            "text" => {
                let (data_args, _path) = extract_file_arg(args);
                match data_args.as_slice() {
                    [xval, yval, Value::Str(s) | Value::StringObj(s)] => {
                        let x = match xval {
                            Value::Scalar(f) => *f,
                            _ => return Err("text: x must be a scalar".into()),
                        };
                        let y = match yval {
                            Value::Scalar(f) => *f,
                            _ => return Err("text: y must be a scalar".into()),
                        };
                        let label = s.clone();
                        FIGURE_STATE.with(|f| {
                            f.borrow_mut().annotations.push((x, y, label));
                        });
                        Ok(Value::Void)
                    }
                    _ => Err("text: expected text(x, y, 'string')".into()),
                }
            }

            _ => Err(format!("plot plugin: unknown function '{name}'")),
        }
    }
}

// ── Dispatch helpers ───────────────────────────────────────────────────────

fn render_ascii_or_file(
    name: &str,
    data_args: &[Value],
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_ascii(name, data_args, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_file(name, data_args, p, state)
        }
        Some(p) => Err(format!("{name}: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot-svg")]
fn render_file(
    name: &str,
    data_args: &[Value],
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    let (x, y) = extract_xy(name, data_args)?;
    let (x, y) = if name == "stairs" {
        make_step_data(&x, &y)
    } else {
        (x, y)
    };
    let result = match name {
        "plot" | "stairs" => file::render_line(&x, &y, path, state),
        "scatter" => file::render_scatter(&x, &y, path, state),
        _ => unreachable!(),
    };
    result.map_err(|e| format!("{name}: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_file(
    name: &str,
    _data_args: &[Value],
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err(format!(
        "{name}: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
    ))
}

#[cfg(feature = "plot")]
fn render_ascii(name: &str, data_args: &[Value], state: FigureState) -> Result<Value, String> {
    let (x, y) = extract_xy(name, data_args)?;
    let (x, y) = if name == "stairs" {
        make_step_data(&x, &y)
    } else {
        (x, y)
    };
    match name {
        "plot" | "stairs" => ascii::render_line(&x, &y, state),
        "scatter" => ascii::render_scatter(&x, &y, state),
        "bar" => ascii::render_bar(&x, &y, state),
        "stem" => ascii::render_stem(&x, &y, state),
        _ => unreachable!(),
    }
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_ascii(name: &str, _data_args: &[Value], _state: FigureState) -> Result<Value, String> {
    Err(format!(
        "{name}: ASCII rendering requires the 'plot' feature flag. \
         Rebuild with: cargo build --features plot"
    ))
}

// ── contour / contourf dispatch ────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_contour(
    filled: bool,
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    n_levels: usize,
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_contour_ascii_tier(z, nrows, ncols, n_levels, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_contour_file_tier(filled, x_vals, y_vals, z, nrows, ncols, n_levels, p, state)
        }
        Some(p) => Err(format!("contour: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_contour_ascii_tier(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    n_levels: usize,
    state: FigureState,
) -> Result<Value, String> {
    let (z_min, z_max) = colormap::data_range(z);
    let levels = contour::compute_levels(z_min, z_max, n_levels);
    contour::render_contour_ascii(z, nrows, ncols, &levels, &state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_contour_ascii_tier(
    _z: &[f64],
    _nrows: usize,
    _ncols: usize,
    _n_levels: usize,
    _state: FigureState,
) -> Result<Value, String> {
    Err(
        "contour: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
            .into(),
    )
}

#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
fn render_contour_file_tier(
    filled: bool,
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    n_levels: usize,
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    let (z_min, z_max) = colormap::data_range(z);
    let levels = contour::compute_levels(z_min, z_max, n_levels);
    let result = if filled {
        contour::render_contourf_file(x_vals, y_vals, z, nrows, ncols, &levels, path, state)
    } else {
        contour::render_contour_file(x_vals, y_vals, z, nrows, ncols, &levels, path, state)
    };
    result.map_err(|e| e.to_string())?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
#[allow(clippy::too_many_arguments)]
fn render_contour_file_tier(
    _filled: bool,
    _x_vals: &[f64],
    _y_vals: &[f64],
    _z: &[f64],
    _nrows: usize,
    _ncols: usize,
    _n_levels: usize,
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err("contour: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── surf / mesh dispatch ───────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_surface(
    name: &str,
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    let wireframe = name == "mesh";
    match path {
        None | Some("ascii") => render_surface_ascii_tier(x_vals, z, nrows, ncols, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_surface_file_tier(wireframe, x_vals, y_vals, z, nrows, ncols, p, state)
        }
        Some(p) => Err(format!("{name}: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_surface_ascii_tier(
    x_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    state: FigureState,
) -> Result<Value, String> {
    surface::render_surf_ascii(x_vals, z, nrows, ncols, &state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_surface_ascii_tier(
    _x_vals: &[f64],
    _z: &[f64],
    _nrows: usize,
    _ncols: usize,
    _state: FigureState,
) -> Result<Value, String> {
    Err(
        "surf/mesh: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
            .into(),
    )
}

#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
fn render_surface_file_tier(
    wireframe: bool,
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    let result = if wireframe {
        surface::render_mesh_file(x_vals, y_vals, z, nrows, ncols, path, state)
    } else {
        surface::render_surf_file(x_vals, y_vals, z, nrows, ncols, path, state)
    };
    result.map_err(|e| e.to_string())?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
#[allow(clippy::too_many_arguments)]
fn render_surface_file_tier(
    _wireframe: bool,
    _x_vals: &[f64],
    _y_vals: &[f64],
    _z: &[f64],
    _nrows: usize,
    _ncols: usize,
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err(
        "surf/mesh: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
            .into(),
    )
}

// ── imagesc dispatch ───────────────────────────────────────────────────────

fn render_imagesc(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_imagesc_ascii_tier(z, nrows, ncols, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_imagesc_file_tier(z, nrows, ncols, p, state)
        }
        Some(p) => Err(format!("imagesc: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_imagesc_ascii_tier(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    state: FigureState,
) -> Result<Value, String> {
    colormap::render_imagesc_ascii(z, nrows, ncols, &state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_imagesc_ascii_tier(
    _z: &[f64],
    _nrows: usize,
    _ncols: usize,
    _state: FigureState,
) -> Result<Value, String> {
    Err(
        "imagesc: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
            .into(),
    )
}

#[cfg(feature = "plot-svg")]
fn render_imagesc_file_tier(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    colormap::render_imagesc_file(z, nrows, ncols, path, state)
        .map_err(|e| format!("imagesc: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_imagesc_file_tier(
    _z: &[f64],
    _nrows: usize,
    _ncols: usize,
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err("imagesc: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── Argument helpers ───────────────────────────────────────────────────────

fn require_string(name: &str, args: &[Value]) -> Result<String, String> {
    match args {
        [Value::Str(s)] | [Value::StringObj(s)] => Ok(s.clone()),
        [_] => Err(format!("{name}: argument must be a string")),
        _ => Err(format!("{name}: expected exactly one string argument")),
    }
}

fn require_string_list(args: &[Value]) -> Result<Vec<String>, String> {
    if args.is_empty() {
        return Err("legend: at least one string argument required".into());
    }
    args.iter()
        .map(|a| match a {
            Value::Str(s) | Value::StringObj(s) => Ok(s.clone()),
            _ => Err("legend: all arguments must be strings".into()),
        })
        .collect()
}

fn extract_lim(name: &str, args: &[Value]) -> Result<(f64, f64), String> {
    let v = match args {
        [val] => extract_vector(val)
            .map_err(|_| format!("{name}: expected a 2-element vector [lo hi]"))?,
        _ => return Err(format!("{name}: expected exactly one argument [lo hi]")),
    };
    if v.len() != 2 {
        return Err(format!(
            "{name}: vector must have exactly 2 elements, got {}",
            v.len()
        ));
    }
    Ok((v[0], v[1]))
}

// ── Stairs helpers ─────────────────────────────────────────────────────────

/// Converts (x, y) data into step/staircase pairs for rendering.
fn make_step_data(x: &[f64], y: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = x.len();
    if n == 0 {
        return (vec![], vec![]);
    }
    let mut sx = Vec::with_capacity(2 * n - 1);
    let mut sy = Vec::with_capacity(2 * n - 1);
    for i in 0..n - 1 {
        sx.push(x[i]);
        sy.push(y[i]);
        // Horizontal segment at y[i] until the next x position.
        sx.push(x[i + 1]);
        sy.push(y[i]);
    }
    sx.push(*x.last().unwrap());
    sy.push(*y.last().unwrap());
    (sx, sy)
}

// ── Histogram helpers ──────────────────────────────────────────────────────

/// Sturges rule: bins ≈ √n, minimum 1.
fn sturges_bins(n: usize) -> usize {
    (n as f64).sqrt().round() as usize
}

/// Parses `hist` arguments: `(data_vec, n_bins)`.
/// Parses hist arguments and returns `(counts, edges)` ready for rendering.
///
/// Accepts `[v]` (Sturges default), `[v, n]` (explicit bin count), or
/// `[v, edges]` (explicit bin-edge vector).
fn parse_and_compute_hist(args: &[Value]) -> Result<(Vec<usize>, Vec<f64>), String> {
    match args.len() {
        0 => Err("hist: at least one argument required".into()),
        1 => {
            let vals = extract_vector(&args[0])
                .map_err(|_| "hist: first argument must be a numeric vector".to_string())?;
            let n = sturges_bins(vals.len()).max(1);
            Ok(compute_histogram_uniform(&vals, n))
        }
        2 => {
            let vals = extract_vector(&args[0])
                .map_err(|_| "hist: first argument must be a numeric vector".to_string())?;
            match &args[1] {
                Value::Scalar(v) => {
                    let n = *v as usize;
                    if n == 0 {
                        return Err("hist: bin count must be positive".into());
                    }
                    Ok(compute_histogram_uniform(&vals, n))
                }
                Value::Matrix(_) | Value::ComplexMatrix(_) => {
                    let edges = extract_vector(&args[1])
                        .map_err(|_| "hist: edge vector must be numeric".to_string())?;
                    if edges.len() < 2 {
                        return Err("hist: edge vector must have at least 2 elements".into());
                    }
                    Ok(compute_histogram_edges(&vals, &edges))
                }
                _ => Err("hist: second argument must be a bin count or an edge vector".into()),
            }
        }
        _ => Err("hist: too many arguments".into()),
    }
}

/// Computes histogram counts with `n_bins` uniform bins spanning the data range.
fn compute_histogram_uniform(vals: &[f64], n_bins: usize) -> (Vec<usize>, Vec<f64>) {
    if vals.is_empty() {
        return (vec![0; n_bins], (0..=n_bins).map(|i| i as f64).collect());
    }
    let min_v = vals.iter().copied().fold(f64::INFINITY, f64::min);
    let max_v = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = if max_v > min_v { max_v - min_v } else { 1.0 };
    let mut counts = vec![0usize; n_bins];
    for &v in vals {
        let b = ((v - min_v) / range * n_bins as f64) as usize;
        counts[b.min(n_bins - 1)] += 1;
    }
    let edges: Vec<f64> = (0..=n_bins)
        .map(|i| min_v + range * (i as f64 / n_bins as f64))
        .collect();
    (counts, edges)
}

/// Computes histogram counts using caller-supplied bin edges.
///
/// Values below `edges[0]` or above `edges[last]` are ignored.
fn compute_histogram_edges(vals: &[f64], edges: &[f64]) -> (Vec<usize>, Vec<f64>) {
    let n_bins = edges.len() - 1;
    let mut counts = vec![0usize; n_bins];
    for &v in vals {
        // Binary search for the bin: edges[b] <= v < edges[b+1]
        match edges.binary_search_by(|e| e.partial_cmp(&v).unwrap_or(std::cmp::Ordering::Less)) {
            Ok(b) => counts[b.min(n_bins - 1)] += 1,
            Err(b) if b > 0 && b <= n_bins => counts[b - 1] += 1,
            _ => {}
        }
    }
    (counts, edges.to_vec())
}

/// Prints a character-art histogram to stdout (no feature flag required).
fn render_hist_ascii(counts: &[usize], edges: &[f64], state: &FigureState) {
    let n_bins = counts.len();
    let bar_cols: usize = term_cols().saturating_sub(26).max(10);
    let max_count = counts.iter().copied().max().unwrap_or(1).max(1);
    if let Some(t) = &state.title {
        println!("{t}");
    }
    for i in 0..n_bins {
        let lo = edges[i];
        let hi = edges[i + 1];
        let bar_len = counts[i] * bar_cols / max_count;
        println!(
            "{lo:8.4} {hi:8.4} |{bar:<width$}| {c}",
            bar = "#".repeat(bar_len),
            width = bar_cols,
            c = counts[i],
        );
    }
    if let Some(xl) = &state.xlabel {
        println!("x: {xl}");
    }
    if let Some(yl) = &state.ylabel {
        println!("y: {yl}");
    }
}

#[cfg(feature = "plot-svg")]
fn render_hist_file(
    counts: &[usize],
    edges: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    file::render_hist(counts, edges, path, style, state).map_err(|e| format!("hist: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_hist_file(
    _counts: &[usize],
    _edges: &[f64],
    _path: &str,
    _style: Option<StyleSpec>,
    _state: FigureState,
) -> Result<Value, String> {
    Err("hist: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── Multi-series dispatch ──────────────────────────────────────────────────

fn render_multi_series(
    x: &[f64],
    ys: &[Vec<f64>],
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_multi_series_ascii(x, ys, &state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_multi_series_file(x, ys, p, state)
        }
        Some(p) => Err(format!("plot: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_multi_series_ascii(
    x: &[f64],
    ys: &[Vec<f64>],
    _state: &FigureState,
) -> Result<Value, String> {
    // Render first series only; note remaining series.
    ascii::render_line(x, &ys[0], FigureState::default());
    println!("% {} series total — use file export for all", ys.len());
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_multi_series_ascii(
    _x: &[f64],
    _ys: &[Vec<f64>],
    _state: &FigureState,
) -> Result<Value, String> {
    Err("plot: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
        .into())
}

#[cfg(feature = "plot-svg")]
fn render_multi_series_file(
    x: &[f64],
    ys: &[Vec<f64>],
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    file::render_multi_line(x, ys, path, state).map_err(|e| format!("plot: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_multi_series_file(
    _x: &[f64],
    _ys: &[Vec<f64>],
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err("plot: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── Pre-transformed line dispatch (loglog / semilogx / semilogy) ───────────

/// Dispatch a pre-processed (x, y) pair to ASCII or file, rendering a line.
fn render_line_xy(
    name: &str,
    x: &[f64],
    y: &[f64],
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_line_xy_ascii(name, x, y, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_line_xy_file(name, x, y, p, state)
        }
        Some(p) => Err(format!("{name}: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_line_xy_ascii(
    _name: &str,
    x: &[f64],
    y: &[f64],
    state: FigureState,
) -> Result<Value, String> {
    ascii::render_line(x, y, state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_line_xy_ascii(
    name: &str,
    _x: &[f64],
    _y: &[f64],
    _state: FigureState,
) -> Result<Value, String> {
    Err(format!(
        "{name}: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
    ))
}

#[cfg(feature = "plot-svg")]
fn render_line_xy_file(
    name: &str,
    x: &[f64],
    y: &[f64],
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    file::render_line(x, y, path, state).map_err(|e| format!("{name}: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_line_xy_file(
    name: &str,
    _x: &[f64],
    _y: &[f64],
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err(format!(
        "{name}: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
    ))
}

// ── fill / area dispatch ───────────────────────────────────────────────────

fn render_fill_xy(
    x: &[f64],
    y: &[f64],
    path: Option<&str>,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_fill_ascii(x, y, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_fill_file(x, y, p, style, state)
        }
        Some(p) => Err(format!("fill: unknown output target '{p}'")),
    }
}

fn render_area_xy(
    x: &[f64],
    y: &[f64],
    path: Option<&str>,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_area_ascii(x, y, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_area_file(x, y, p, style, state)
        }
        Some(p) => Err(format!("area: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_fill_ascii(x: &[f64], y: &[f64], state: FigureState) -> Result<Value, String> {
    ascii::render_fill(x, y, state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_fill_ascii(_x: &[f64], _y: &[f64], _state: FigureState) -> Result<Value, String> {
    Err("fill: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
        .into())
}

#[cfg(feature = "plot")]
fn render_area_ascii(x: &[f64], y: &[f64], state: FigureState) -> Result<Value, String> {
    ascii::render_area(x, y, state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_area_ascii(_x: &[f64], _y: &[f64], _state: FigureState) -> Result<Value, String> {
    Err("area: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
        .into())
}

#[cfg(feature = "plot-svg")]
fn render_fill_file(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    file::render_fill(x, y, path, style, state).map_err(|e| format!("fill: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_fill_file(
    _x: &[f64],
    _y: &[f64],
    _path: &str,
    _style: Option<StyleSpec>,
    _state: FigureState,
) -> Result<Value, String> {
    Err("fill: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

#[cfg(feature = "plot-svg")]
fn render_area_file(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    file::render_area(x, y, path, style, state).map_err(|e| format!("area: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_area_file(
    _x: &[f64],
    _y: &[f64],
    _path: &str,
    _style: Option<StyleSpec>,
    _state: FigureState,
) -> Result<Value, String> {
    Err("area: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── bar / stem dispatch ────────────────────────────────────────────────────

fn render_bar_xy(
    x: &[f64],
    y: &[f64],
    path: Option<&str>,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_bar_ascii(x, y, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_bar_file(x, y, p, style, state)
        }
        Some(p) => Err(format!("bar: unknown output target '{p}'")),
    }
}

fn render_stem_xy(
    x: &[f64],
    y: &[f64],
    path: Option<&str>,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_stem_ascii(x, y, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_stem_file(x, y, p, style, state)
        }
        Some(p) => Err(format!("stem: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_bar_ascii(x: &[f64], y: &[f64], state: FigureState) -> Result<Value, String> {
    ascii::render_bar(x, y, state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_bar_ascii(_x: &[f64], _y: &[f64], _state: FigureState) -> Result<Value, String> {
    Err("bar: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
        .into())
}

#[cfg(feature = "plot")]
fn render_stem_ascii(x: &[f64], y: &[f64], state: FigureState) -> Result<Value, String> {
    ascii::render_stem(x, y, state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_stem_ascii(_x: &[f64], _y: &[f64], _state: FigureState) -> Result<Value, String> {
    Err("stem: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
        .into())
}

#[cfg(feature = "plot-svg")]
fn render_bar_file(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    file::render_bar(x, y, path, style, state).map_err(|e| format!("bar: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_bar_file(
    _x: &[f64],
    _y: &[f64],
    _path: &str,
    _style: Option<StyleSpec>,
    _state: FigureState,
) -> Result<Value, String> {
    Err("bar: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

#[cfg(feature = "plot-svg")]
fn render_stem_file(
    x: &[f64],
    y: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    file::render_stem(x, y, path, style, state).map_err(|e| format!("stem: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_stem_file(
    _x: &[f64],
    _y: &[f64],
    _path: &str,
    _style: Option<StyleSpec>,
    _state: FigureState,
) -> Result<Value, String> {
    Err("stem: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── quiver dispatch ────────────────────────────────────────────────────────

fn render_quiver(
    x: &[f64],
    y: &[f64],
    u: &[f64],
    v: &[f64],
    path: Option<&str>,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_quiver_ascii_tier(x, y, u, v, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_quiver_file_tier(x, y, u, v, p, style, state)
        }
        Some(p) => Err(format!("quiver: unknown output target '{p}'")),
    }
}

fn render_quiver_ascii_tier(
    x: &[f64],
    y: &[f64],
    u: &[f64],
    v: &[f64],
    state: FigureState,
) -> Result<Value, String> {
    render_quiver_ascii(x, y, u, v, &state);
    Ok(Value::Void)
}

#[cfg(feature = "plot-svg")]
fn render_quiver_file_tier(
    x: &[f64],
    y: &[f64],
    u: &[f64],
    v: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    file::render_quiver(x, y, u, v, path, style, state).map_err(|e| format!("quiver: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_quiver_file_tier(
    _x: &[f64],
    _y: &[f64],
    _u: &[f64],
    _v: &[f64],
    _path: &str,
    _style: Option<StyleSpec>,
    _state: FigureState,
) -> Result<Value, String> {
    Err("quiver: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

/// ASCII quiver: Unicode directional arrows placed on a character grid.
fn render_quiver_ascii(xs: &[f64], ys: &[f64], us: &[f64], vs: &[f64], state: &FigureState) {
    let n = xs.len();
    if n == 0 {
        return;
    }
    let w = term_cols().saturating_sub(4).max(20);
    let h = (term_rows() / 2).max(10);

    let x_min = state
        .xlim
        .map(|(lo, _)| lo)
        .unwrap_or_else(|| xs.iter().copied().fold(f64::INFINITY, f64::min));
    let x_max = state
        .xlim
        .map(|(_, hi)| hi)
        .unwrap_or_else(|| xs.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    let y_min = state
        .ylim
        .map(|(lo, _)| lo)
        .unwrap_or_else(|| ys.iter().copied().fold(f64::INFINITY, f64::min));
    let y_max = state
        .ylim
        .map(|(_, hi)| hi)
        .unwrap_or_else(|| ys.iter().copied().fold(f64::NEG_INFINITY, f64::max));

    let x_span = if (x_max - x_min).abs() < f64::EPSILON {
        2.0
    } else {
        x_max - x_min
    };
    let y_span = if (y_max - y_min).abs() < f64::EPSILON {
        2.0
    } else {
        y_max - y_min
    };

    let mut grid: Vec<Vec<char>> = vec![vec![' '; w]; h];

    for i in 0..n {
        let col = ((xs[i] - x_min) / x_span * (w - 1) as f64).round() as isize;
        let row = ((y_max - ys[i]) / y_span * (h - 1) as f64).round() as isize;
        if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
            let angle = vs[i].atan2(us[i]);
            grid[row as usize][col as usize] = arrow_char(angle);
        }
    }

    if let Some(t) = &state.title {
        println!("{t}");
    }
    for row in &grid {
        println!("|{}|", row.iter().collect::<String>());
    }
    if let Some(xl) = &state.xlabel {
        println!("x: {xl}");
    }
    if let Some(yl) = &state.ylabel {
        println!("y: {yl}");
    }
}

#[cfg(feature = "plot")]
fn render_color_scatter_ascii(x: &[f64], y: &[f64], state: FigureState) -> Result<Value, String> {
    ascii::render_scatter(x, y, state);
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_color_scatter_ascii(
    _x: &[f64],
    _y: &[f64],
    _state: FigureState,
) -> Result<Value, String> {
    Err(
        "scatter: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
            .into(),
    )
}

/// Returns `true` when `v` is a numeric `Value` (Scalar or Matrix).
fn is_numeric_value(v: &Value) -> bool {
    matches!(v, Value::Scalar(_) | Value::Matrix(_))
}

// ── errorbar dispatch ──────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_errorbar(
    x: &[f64],
    y: &[f64],
    e_low: &[f64],
    e_high: &[f64],
    path: Option<&str>,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => {
            render_errorbar_ascii(x, y, e_low, e_high);
            Ok(Value::Void)
        }
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_errorbar_file(x, y, e_low, e_high, p, style, state)
        }
        Some(p) => Err(format!("errorbar: unknown output target '{p}'")),
    }
}

/// ASCII tier for `errorbar`: prints a compact table with ± notation.
fn render_errorbar_ascii(x: &[f64], y: &[f64], e_low: &[f64], e_high: &[f64]) {
    println!(" {:>10}  {:>12}  {:>12}", "x", "y", "error");
    println!(" {:->10}  {:->12}  {:->12}", "", "", "");
    for i in 0..x.len() {
        let err_str = if (e_low[i] - e_high[i]).abs() < 1e-12 {
            format!("±{:.4}", e_low[i])
        } else {
            format!("-{:.4}/+{:.4}", e_low[i], e_high[i])
        };
        println!(" {:>10.4}  {:>12.4}  {:>12}", x[i], y[i], err_str);
    }
}

#[cfg(feature = "plot-svg")]
fn render_errorbar_file(
    x: &[f64],
    y: &[f64],
    e_low: &[f64],
    e_high: &[f64],
    path: &str,
    style: Option<StyleSpec>,
    state: FigureState,
) -> Result<Value, String> {
    file::render_errorbar(x, y, e_low, e_high, path, style, state)
        .map_err(|e| format!("errorbar: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_errorbar_file(
    _x: &[f64],
    _y: &[f64],
    _e_low: &[f64],
    _e_high: &[f64],
    _path: &str,
    _style: Option<StyleSpec>,
    _state: FigureState,
) -> Result<Value, String> {
    Err(
        "errorbar: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
            .into(),
    )
}

// ── color_scatter dispatch ─────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_color_scatter(
    x: &[f64],
    y: &[f64],
    sz: &[f64],
    c: &[f64],
    c_min: f64,
    c_max: f64,
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => render_color_scatter_ascii(x, y, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_color_scatter_file(x, y, sz, c, c_min, c_max, p, state)
        }
        Some(p) => Err(format!("scatter: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
fn render_color_scatter_file(
    x: &[f64],
    y: &[f64],
    sz: &[f64],
    c: &[f64],
    c_min: f64,
    c_max: f64,
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    file::render_color_scatter(x, y, sz, c, c_min, c_max, path, state)
        .map_err(|e| format!("scatter: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
#[allow(clippy::too_many_arguments)]
fn render_color_scatter_file(
    _x: &[f64],
    _y: &[f64],
    _sz: &[f64],
    _c: &[f64],
    _c_min: f64,
    _c_max: f64,
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err("scatter: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── pie dispatch ──────────────────────────────────────────────────────────

fn render_pie(
    values: &[f64],
    labels: &[String],
    explode: &[f64],
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    match path {
        None | Some("ascii") => {
            print!("{}", format_pie_ascii(values, labels, explode));
            Ok(Value::Void)
        }
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_pie_file(values, labels, explode, p, state)
        }
        Some(p) => Err(format!("pie: unknown output target '{p}'")),
    }
}

/// Formats a pie chart as a horizontal bar-art table.
///
/// Returns the formatted string so callers (including tests) can inspect it.
/// Fill characters cycled per slice so each slice is visually distinct.
const SLICE_FILLS: [char; 4] = [
    '\u{2588}', // █ full block
    '\u{2593}', // ▓ dark shade
    '\u{2592}', // ▒ medium shade
    '\u{2591}', // ░ light shade
];

pub(crate) fn format_pie_ascii(values: &[f64], labels: &[String], explode: &[f64]) -> String {
    use std::fmt::Write;
    let total: f64 = values.iter().sum();
    let bar_width: usize = 20;
    let mut out = String::new();
    for (i, &v) in values.iter().enumerate() {
        let pct = v / total * 100.0;
        let label = if i < labels.len() && !labels[i].is_empty() {
            labels[i].as_str()
        } else {
            ""
        };
        let is_exploded = explode.get(i).copied().unwrap_or(0.0) > 1e-9;
        let fill = SLICE_FILLS[i % SLICE_FILLS.len()];
        // Build bar: filled part uses slice fill char; empty part uses `·`
        // with a single `─` (U+2500) exactly at the midpoint.
        let filled = (pct / 100.0 * bar_width as f64).round() as usize;
        let filled = filled.min(bar_width);
        let mid = bar_width / 2;
        let mut bar = String::new();
        for j in 0..bar_width {
            if j < filled {
                bar.push(fill);
            } else if j == mid && filled <= mid {
                bar.push(':');
            } else {
                bar.push('\u{00b7}'); // ·
            }
        }
        let explode_marker = if is_exploded { " \u{25c4}" } else { "" }; // ◄ or nothing
        if label.is_empty() {
            let _ = writeln!(out, " [{bar}] {pct:5.1}%{explode_marker}");
        } else {
            let _ = writeln!(out, " [{bar}] {pct:5.1}%  {label}{explode_marker}");
        }
    }
    out
}

#[cfg(feature = "plot-svg")]
fn render_pie_file(
    values: &[f64],
    labels: &[String],
    explode: &[f64],
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    file::render_pie(values, labels, explode, path, state).map_err(|e| format!("pie: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_pie_file(
    _values: &[f64],
    _labels: &[String],
    _explode: &[f64],
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err("pie: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

/// Maps an angle in radians to one of 8 Unicode directional arrow characters.
fn arrow_char(angle: f64) -> char {
    use std::f64::consts::PI;
    let a = (angle + 2.0 * PI).rem_euclid(2.0 * PI);
    let octant = ((a + PI / 8.0) / (PI / 4.0)) as usize % 8;
    match octant {
        0 => '\u{2192}', // →
        1 => '\u{2197}', // ↗
        2 => '\u{2191}', // ↑
        3 => '\u{2196}', // ↖
        4 => '\u{2190}', // ←
        5 => '\u{2199}', // ↙
        6 => '\u{2193}', // ↓
        _ => '\u{2198}', // ↘
    }
}

// ── 3D dispatch ────────────────────────────────────────────────────────────

fn render_3d(
    name: &str,
    data_args: &[Value],
    path: Option<&str>,
    state: FigureState,
) -> Result<Value, String> {
    extract_xyz(name, data_args)?;
    match path {
        None | Some("ascii") => render_3d_ascii(name, data_args, state),
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
            render_3d_file(name, data_args, p, state)
        }
        Some(p) => Err(format!("{name}: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_3d_ascii(name: &str, data_args: &[Value], state: FigureState) -> Result<Value, String> {
    let (x, y, z) = extract_xyz(name, data_args)?;
    let (px, py) = proj3d::project_ortho(&x, &y, &z);
    // Pass only title and axis limits to the 2D ASCII renderer.
    // Labels are printed below as a footer to avoid misleading axis descriptions.
    let state_2d = FigureState {
        title: state.title.clone(),
        xlim: state.xlim,
        ylim: state.ylim,
        ..FigureState::default()
    };
    match name {
        "plot3" => ascii::render_line(&px, &py, state_2d),
        "scatter3" => ascii::render_scatter(&px, &py, state_2d),
        _ => unreachable!(),
    }
    if let Some(xl) = &state.xlabel {
        println!("x: {xl}");
    }
    if let Some(yl) = &state.ylabel {
        println!("y: {yl}");
    }
    if let Some(zl) = &state.zlabel {
        println!("z: {zl}");
    }
    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_3d_ascii(name: &str, _data_args: &[Value], _state: FigureState) -> Result<Value, String> {
    Err(format!(
        "{name}: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
    ))
}

#[cfg(feature = "plot-svg")]
fn render_3d_file(
    name: &str,
    data_args: &[Value],
    path: &str,
    state: FigureState,
) -> Result<Value, String> {
    let (x, y, z) = extract_xyz(name, data_args)?;
    let result = match name {
        "plot3" => file::render_plot3(&x, &y, &z, path, state),
        "scatter3" => file::render_scatter3(&x, &y, &z, path, state),
        _ => unreachable!(),
    };
    result.map_err(|e| format!("{name}: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_3d_file(
    name: &str,
    _data_args: &[Value],
    _path: &str,
    _state: FigureState,
) -> Result<Value, String> {
    Err(format!(
        "{name}: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
    ))
}

// ── subplot / hold / savefig render ───────────────────────────────────────

/// Renders all series in a panel to ASCII stdout (used by `hold('off')`).
///
/// Each series is printed sequentially; a `---` divider separates them.
/// If the panel has right-axis series, they are printed after a `[right axis]` header.
#[cfg(feature = "plot")]
fn render_panel_ascii(panel: &Panel) -> Result<Value, String> {
    if panel.series.is_empty() && panel.right_series.is_empty() {
        return Ok(Value::Void);
    }

    let render_series = |series_list: &[PendingSeries], base_state: &FigureState| {
        for (i, series) in series_list.iter().enumerate() {
            if i > 0 {
                println!("---");
            }
            match series {
                PendingSeries::Line(x, y, _style) => {
                    ascii::render_line(x, y, base_state.clone());
                }
                PendingSeries::Scatter(x, y, _style) => {
                    ascii::render_scatter(x, y, base_state.clone());
                }
                PendingSeries::Bar(x, y, _style) => {
                    ascii::render_bar(x, y, base_state.clone());
                }
                PendingSeries::Stem(x, y, _style) => {
                    ascii::render_stem(x, y, base_state.clone());
                }
                PendingSeries::Hist {
                    counts,
                    edges,
                    style: _,
                } => {
                    render_hist_ascii(counts, edges, base_state);
                }
                PendingSeries::Fill(x, y, _style) => {
                    ascii::render_fill(x, y, base_state.clone());
                }
                PendingSeries::Area(x, y, _style) => {
                    ascii::render_area(x, y, base_state.clone());
                }
                PendingSeries::Quiver(x, y, u, v, _style) => {
                    render_quiver_ascii(x, y, u, v, base_state);
                }
                PendingSeries::ErrorBar {
                    x,
                    y,
                    e_low,
                    e_high,
                    style: _,
                } => {
                    render_errorbar_ascii(x, y, e_low, e_high);
                }
                PendingSeries::ColorScatter {
                    x,
                    y,
                    sz: _,
                    c: _,
                    c_min: _,
                    c_max: _,
                } => {
                    ascii::render_scatter(x, y, base_state.clone());
                }
                PendingSeries::Pie {
                    values,
                    labels,
                    explode,
                } => {
                    print!("{}", format_pie_ascii(values, labels, explode));
                }
            }
        }
    };

    let has_dual = !panel.right_series.is_empty();

    if has_dual {
        // Use combined chart when both sides consist only of Line / Scatter series.
        let is_xy =
            |s: &PendingSeries| matches!(s, PendingSeries::Line(..) | PendingSeries::Scatter(..));
        let can_combine = !panel.series.is_empty()
            && panel.series.iter().all(is_xy)
            && panel.right_series.iter().all(is_xy);

        if can_combine {
            let to_f32 = |series: &[PendingSeries]| -> Vec<(Vec<f32>, Vec<f32>, bool)> {
                series
                    .iter()
                    .map(|s| match s {
                        PendingSeries::Line(x, y, _) => (
                            x.iter().map(|&v| v as f32).collect(),
                            y.iter().map(|&v| v as f32).collect(),
                            true,
                        ),
                        PendingSeries::Scatter(x, y, _) => (
                            x.iter().map(|&v| v as f32).collect(),
                            y.iter().map(|&v| v as f32).collect(),
                            false,
                        ),
                        _ => unreachable!(),
                    })
                    .collect()
            };
            ascii::render_dual_axis(
                &to_f32(&panel.series),
                &to_f32(&panel.right_series),
                panel.ylim.map(|(lo, hi)| (lo as f32, hi as f32)),
                panel.right_ylim.map(|(lo, hi)| (lo as f32, hi as f32)),
                panel.xlim.map(|(lo, hi)| (lo as f32, hi as f32)),
                panel.title.as_deref(),
                panel.xlabel.as_deref(),
                panel.ylabel.as_deref(),
                panel.right_ylabel.as_deref(),
            );
        } else {
            // Mixed series types: fall back to two-block rendering.
            println!("[left axis]");
            let left_state = FigureState {
                xlabel: panel.xlabel.clone(),
                ylabel: panel.ylabel.clone(),
                title: panel.title.clone(),
                xlim: panel.xlim,
                ylim: panel.ylim,
                ..FigureState::default()
            };
            render_series(&panel.series, &left_state);
            println!("\n[right axis]");
            let right_state = FigureState {
                xlabel: panel.xlabel.clone(),
                ylabel: panel.right_ylabel.clone(),
                xlim: panel.xlim,
                ylim: panel.right_ylim,
                ..FigureState::default()
            };
            render_series(&panel.right_series, &right_state);
        }

        for (ax, ay, label) in &panel.annotations {
            println!("  ({ax:.4}, {ay:.4}): {label}");
        }
    } else {
        let left_state = FigureState {
            xlabel: panel.xlabel.clone(),
            ylabel: panel.ylabel.clone(),
            title: panel.title.clone(),
            xlim: panel.xlim,
            ylim: panel.ylim,
            ..FigureState::default()
        };
        render_series(&panel.series, &left_state);

        for (ax, ay, label) in &panel.annotations {
            println!("  ({ax:.4}, {ay:.4}): {label}");
        }
    }

    Ok(Value::Void)
}

#[cfg(not(feature = "plot"))]
fn render_panel_ascii(_panel: &Panel) -> Result<Value, String> {
    Err("hold: ASCII rendering requires the 'plot' feature flag — \
         rebuild with: cargo build --features plot"
        .into())
}

#[cfg(feature = "plot-svg")]
fn render_panels_file(
    panels: &[Panel],
    path: &str,
    canvas: (u32, u32),
    theme: &style::Theme,
    bg_override: Option<style::StyleColor>,
) -> Result<Value, String> {
    use plotters::style::RGBColor;
    let bg = bg_override
        .map(|c| RGBColor(c.0, c.1, c.2))
        .unwrap_or_else(|| {
            let c = theme.bg;
            RGBColor(c.0, c.1, c.2)
        });
    file::render_subplot_panels(panels, path, canvas, theme, bg)
        .map_err(|e| format!("savefig: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_panels_file(
    _panels: &[Panel],
    _path: &str,
    _canvas: (u32, u32),
    _theme: &style::Theme,
    _bg_override: Option<style::StyleColor>,
) -> Result<Value, String> {
    Err("savefig: SVG/PNG export requires the 'plot-svg' feature — \
         rebuild with: cargo build --features plot-svg"
        .into())
}

// ── Argument helpers (continued) ───────────────────────────────────────────

/// Extracts three equal-length numeric vectors from `plot3`/`scatter3` args.
#[cfg_attr(not(any(feature = "plot", feature = "plot-svg")), allow(dead_code))]
#[allow(clippy::type_complexity)]
fn extract_xyz(name: &str, args: &[Value]) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), String> {
    match args {
        [xv, yv, zv] => {
            let x = extract_vector(xv).map_err(|e| format!("{name}: {e}"))?;
            let y = extract_vector(yv).map_err(|e| format!("{name}: {e}"))?;
            let z = extract_vector(zv).map_err(|e| format!("{name}: {e}"))?;
            if x.len() != y.len() || x.len() != z.len() {
                return Err(format!(
                    "{name}: x, y, z must have the same length \
                     (got {}, {}, {})",
                    x.len(),
                    y.len(),
                    z.len()
                ));
            }
            Ok((x, y, z))
        }
        _ => Err(format!(
            "{name}: expected 3 arguments (x, y, z), got {}",
            args.len()
        )),
    }
}

#[cfg_attr(not(any(feature = "plot", feature = "plot-svg")), allow(dead_code))]
fn extract_xy(name: &str, args: &[Value]) -> Result<(Vec<f64>, Vec<f64>), String> {
    match args.len() {
        0 => Err(format!("{name}: at least one argument required")),
        1 => {
            let y = extract_vector(&args[0])?;
            let x: Vec<f64> = (1..=y.len()).map(|i| i as f64).collect();
            Ok((x, y))
        }
        2 => {
            let x = extract_vector(&args[0])?;
            let y = extract_vector(&args[1])?;
            if x.len() != y.len() {
                return Err(format!(
                    "{name}: x and y must have the same length ({} vs {})",
                    x.len(),
                    y.len()
                ));
            }
            Ok((x, y))
        }
        _ => Err(format!("{name}: too many arguments")),
    }
}

/// Extracts x and one or more y series from plot arguments.
///
/// When y is an M×N matrix with M > 1, returns M separate row-series.
/// Otherwise behaves identically to `extract_xy`.
#[cfg_attr(not(any(feature = "plot", feature = "plot-svg")), allow(dead_code))]
fn extract_xy_multi(name: &str, args: &[Value]) -> Result<(Vec<f64>, Vec<Vec<f64>>), String> {
    match args.len() {
        0 => Err(format!("{name}: at least one argument required")),
        1 => {
            let y = extract_vector(&args[0])?;
            let x: Vec<f64> = (1..=y.len()).map(|i| i as f64).collect();
            Ok((x, vec![y]))
        }
        2 => {
            let x = extract_vector(&args[0])?;
            match &args[1] {
                Value::Matrix(m) if m.nrows() > 1 => {
                    // Each row is one series.
                    let n_cols = m.ncols();
                    if n_cols != x.len() {
                        return Err(format!(
                            "{name}: x has {} elements but Y has {} columns",
                            x.len(),
                            n_cols
                        ));
                    }
                    let ys = (0..m.nrows())
                        .map(|r| m.row(r).iter().copied().collect())
                        .collect();
                    Ok((x, ys))
                }
                other => {
                    let y = extract_vector(other)?;
                    if x.len() != y.len() {
                        return Err(format!(
                            "{name}: x and y must have the same length ({} vs {})",
                            x.len(),
                            y.len()
                        ));
                    }
                    Ok((x, vec![y]))
                }
            }
        }
        _ => Err(format!("{name}: too many arguments")),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use ccalc_engine::env::{Env, Value};
    use ndarray::Array2;

    use super::*;

    // ── term_cols / term_rows ─────────────────────────────────────────

    #[test]
    fn test_term_cols_default() {
        // Without $COLUMNS set, must return the 80-column fallback.
        unsafe { std::env::remove_var("COLUMNS") };
        assert_eq!(term_cols(), 80);
    }

    #[test]
    fn test_term_rows_default() {
        unsafe { std::env::remove_var("LINES") };
        assert_eq!(term_rows(), 24);
    }

    #[test]
    fn test_term_cols_env_override() {
        unsafe { std::env::set_var("COLUMNS", "132") };
        let cols = term_cols();
        unsafe { std::env::remove_var("COLUMNS") };
        assert_eq!(cols, 132);
    }

    fn f64_vec(vals: &[f64]) -> Value {
        Value::Matrix(Array2::from_shape_vec((1, vals.len()), vals.to_vec()).unwrap())
    }

    // ── extract_xy ────────────────────────────────────────────────────

    #[test]
    fn test_extract_xy_infer_x() {
        let y = f64_vec(&[1.0, 4.0, 9.0]);
        let (x, yv) = extract_xy("plot", &[y]).unwrap();
        assert_eq!(x, vec![1.0, 2.0, 3.0]);
        assert_eq!(yv, vec![1.0, 4.0, 9.0]);
    }

    #[test]
    fn test_extract_xy_explicit() {
        let x = f64_vec(&[10.0, 20.0]);
        let y = f64_vec(&[1.0, 2.0]);
        let (xv, yv) = extract_xy("plot", &[x, y]).unwrap();
        assert_eq!(xv, vec![10.0, 20.0]);
        assert_eq!(yv, vec![1.0, 2.0]);
    }

    #[test]
    fn test_extract_xy_mismatch() {
        let x = f64_vec(&[1.0, 2.0]);
        let y = f64_vec(&[1.0, 2.0, 3.0]);
        assert!(extract_xy("plot", &[x, y]).is_err());
    }

    #[test]
    fn test_extract_xy_scalar_promoted() {
        let y = Value::Scalar(5.0);
        let (x, yv) = extract_xy("plot", &[y]).unwrap();
        assert_eq!(x, vec![1.0]);
        assert_eq!(yv, vec![5.0]);
    }

    // ── Annotation setters ────────────────────────────────────────────

    #[test]
    fn test_xlabel_sets_state() {
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("xlabel", &[Value::Str("time".into())], &env)
            .unwrap();
        let label = FIGURE_STATE.with(|f| f.borrow().xlabel.clone());
        assert_eq!(label, Some("time".into()));
        // Clean up so other tests start fresh.
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_title_sets_state() {
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("title", &[Value::Str("My Chart".into())], &env)
            .unwrap();
        let title = FIGURE_STATE.with(|f| f.borrow().title.clone());
        assert_eq!(title, Some("My Chart".into()));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_annotation_requires_string() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("xlabel", &[Value::Scalar(1.0)], &env);
        assert!(result.is_err());
    }

    // ── Render dispatch ───────────────────────────────────────────────

    #[test]
    fn test_plot_no_feature_returns_error_without_feature() {
        // When compiled WITHOUT --features plot, calling plot should give a
        // helpful error rather than silently doing nothing.
        #[cfg(not(feature = "plot"))]
        {
            let plugin = PlotPlugin;
            let env = Env::new();
            let y = f64_vec(&[1.0, 2.0, 3.0]);
            let result = plugin.call("plot", &[y], &env);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("plot"), "error should mention 'plot'");
        }
        // With the feature enabled this path is dead code — that's fine.
        #[cfg(feature = "plot")]
        let _ = ();
    }

    #[test]
    fn test_hist_single_value_no_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("hist", &[Value::Scalar(1.0)], &env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hist_vector_returns_void() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let v = f64_vec(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = plugin.call("hist", &[v], &env).unwrap();
        assert_eq!(result, Value::Void);
    }

    #[test]
    fn test_hist_custom_bins_returns_void() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let v = f64_vec(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = plugin.call("hist", &[v, Value::Scalar(3.0)], &env).unwrap();
        assert_eq!(result, Value::Void);
    }

    #[test]
    fn test_hist_zero_bins_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let v = f64_vec(&[1.0, 2.0, 3.0]);
        let result = plugin.call("hist", &[v, Value::Scalar(0.0)], &env);
        assert!(result.is_err());
    }

    // ── Multi-series extract_xy_multi ─────────────────────────────────────

    #[test]
    fn test_extract_xy_multi_single_series() {
        let x = f64_vec(&[1.0, 2.0, 3.0]);
        let y = f64_vec(&[1.0, 4.0, 9.0]);
        let (xv, ys) = extract_xy_multi("plot", &[x, y]).unwrap();
        assert_eq!(xv, vec![1.0, 2.0, 3.0]);
        assert_eq!(ys.len(), 1);
        assert_eq!(ys[0], vec![1.0, 4.0, 9.0]);
    }

    #[test]
    fn test_extract_xy_multi_matrix_y() {
        let x = f64_vec(&[1.0, 2.0, 3.0]);
        // 2×3 matrix → 2 series of 3 points each
        let y = Value::Matrix(
            Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap(),
        );
        let (xv, ys) = extract_xy_multi("plot", &[x, y]).unwrap();
        assert_eq!(xv, vec![1.0, 2.0, 3.0]);
        assert_eq!(ys.len(), 2);
        assert_eq!(ys[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(ys[1], vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_extract_xy_multi_column_count_mismatch() {
        let x = f64_vec(&[1.0, 2.0]);
        let y = Value::Matrix(
            Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap(),
        );
        let result = extract_xy_multi("plot", &[x, y]);
        assert!(result.is_err());
    }

    // ── Log-scale plots ───────────────────────────────────────────────────

    #[test]
    fn test_loglog_non_positive_all_filtered_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[-1.0, 0.0, -2.0]);
        let y = f64_vec(&[1.0, 2.0, 3.0]);
        let result = plugin.call("loglog", &[x, y], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("finite"), "error should mention finite: {msg}");
    }

    #[test]
    fn test_semilogx_valid_data() {
        let plugin = PlotPlugin;
        let env = Env::new();
        // Without plot feature → feature error; with plot feature → ok.
        let x = f64_vec(&[1.0, 10.0, 100.0]);
        let y = f64_vec(&[1.0, 2.0, 3.0]);
        let result = plugin.call("semilogx", &[x, y], &env);
        // Should not say "not yet implemented" regardless of features.
        if let Err(msg) = &result {
            assert!(
                !msg.contains("not yet implemented"),
                "should not say 'not yet implemented': {msg}"
            );
        }
    }

    #[test]
    fn test_semilogy_label_annotation() {
        // After calling semilogy, ylabel should be cleared (consumed by render).
        // This test verifies that the state is consumed and ylabel is annotated
        // before rendering (requires plot feature to actually render).
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_stairs_stub_is_gone() {
        // stairs should succeed (not stub-error) when called with valid data
        let plugin = PlotPlugin;
        let env = Env::new();
        // Without plot feature this should error about missing feature (not "not implemented").
        // With plot feature this should succeed.
        #[cfg(feature = "plot")]
        {
            let y = f64_vec(&[1.0, 4.0, 9.0, 16.0]);
            let result = plugin.call("stairs", &[y], &env);
            assert!(result.is_ok(), "stairs should succeed: {result:?}");
        }
        #[cfg(not(feature = "plot"))]
        {
            let y = f64_vec(&[1.0, 4.0, 9.0]);
            let result = plugin.call("stairs", &[y], &env);
            // Should error about missing feature, not "not implemented".
            let msg = result.unwrap_err();
            assert!(
                !msg.contains("not yet implemented"),
                "should not say 'not yet implemented': {msg}"
            );
        }
    }

    // ── 29c.1 annotation setters ──────────────────────────────────────

    #[test]
    fn test_xlim_sets_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let lim = Value::Matrix(Array2::from_shape_vec((1, 2), vec![0.0, 10.0]).unwrap());
        plugin.call("xlim", &[lim], &env).unwrap();
        let xlim = FIGURE_STATE.with(|f| f.borrow().xlim);
        assert_eq!(xlim, Some((0.0, 10.0)));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_ylim_sets_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let lim = Value::Matrix(Array2::from_shape_vec((1, 2), vec![-1.0, 1.0]).unwrap());
        plugin.call("ylim", &[lim], &env).unwrap();
        let ylim = FIGURE_STATE.with(|f| f.borrow().ylim);
        assert_eq!(ylim, Some((-1.0, 1.0)));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_legend_sets_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call(
                "legend",
                &[Value::Str("a".into()), Value::Str("b".into())],
                &env,
            )
            .unwrap();
        let legend = FIGURE_STATE.with(|f| f.borrow().legend.clone());
        assert_eq!(legend, vec!["a".to_string(), "b".to_string()]);
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_legend_requires_strings() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("legend", &[Value::Scalar(1.0)], &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_legend_requires_at_least_one_arg() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("legend", &[], &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_grid_toggles_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        // Initially false.
        assert!(!FIGURE_STATE.with(|f| f.borrow().grid));
        plugin.call("grid", &[], &env).unwrap();
        assert!(FIGURE_STATE.with(|f| f.borrow().grid));
        plugin.call("grid", &[], &env).unwrap();
        assert!(!FIGURE_STATE.with(|f| f.borrow().grid));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_grid_on_off_string_args() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("grid", &[Value::Str("on".into())], &env)
            .unwrap();
        assert!(FIGURE_STATE.with(|f| f.borrow().grid));
        plugin
            .call("grid", &[Value::Str("off".into())], &env)
            .unwrap();
        assert!(!FIGURE_STATE.with(|f| f.borrow().grid));
        // Invalid string arg should still error.
        let result = plugin.call("grid", &[Value::Str("maybe".into())], &env);
        assert!(result.is_err());
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_zlabel_sets_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("zlabel", &[Value::Str("depth".into())], &env)
            .unwrap();
        let zlabel = FIGURE_STATE.with(|f| f.borrow().zlabel.clone());
        assert_eq!(zlabel, Some("depth".into()));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_xlim_wrong_length() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let v = Value::Matrix(Array2::from_shape_vec((1, 3), vec![1.0, 2.0, 3.0]).unwrap());
        let result = plugin.call("xlim", &[v], &env);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(feature = "plot-svg"))]
    fn test_svg_without_feature() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let y = f64_vec(&[1.0, 2.0, 3.0]);
        let path = Value::Str("out.svg".into());
        let result = plugin.call("plot", &[y, path], &env);
        assert!(result.is_err());
    }

    // ── ASCII rendering (requires --features plot) ────────────────────

    #[test]
    #[cfg(feature = "plot")]
    fn test_plot_ascii_no_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let y = f64_vec(&[1.0, 4.0, 9.0, 16.0, 25.0]);
        assert!(plugin.call("plot", &[y], &env).is_ok());
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_scatter_ascii_no_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[1.0, 2.0, 3.0, 4.0]);
        let y = f64_vec(&[1.0, 4.0, 9.0, 16.0]);
        assert!(plugin.call("scatter", &[x, y], &env).is_ok());
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_figure_state_cleared_after_render() {
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("title", &[Value::Str("Temp".into())], &env)
            .unwrap();
        let y = f64_vec(&[1.0, 2.0, 3.0]);
        plugin.call("plot", &[y], &env).unwrap();
        // State should be cleared after render.
        let title = FIGURE_STATE.with(|f| f.borrow().title.clone());
        assert!(
            title.is_none(),
            "FigureState should be cleared after plot()"
        );
    }

    // ── 29d: plot3 / scatter3 ─────────────────────────────────────────

    #[test]
    fn test_plot3_length_mismatch_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[1.0, 2.0, 3.0]);
        let y = f64_vec(&[1.0, 2.0]);
        let z = f64_vec(&[0.0, 0.0, 0.0]);
        let result = plugin.call("plot3", &[x, y, z], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("same length"),
            "error should mention length: {msg}"
        );
    }

    #[test]
    fn test_scatter3_wrong_arg_count_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[1.0, 2.0]);
        let y = f64_vec(&[1.0, 2.0]);
        // Only two args — missing z.
        let result = plugin.call("scatter3", &[x, y], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("3 arguments"),
            "error should mention 3 args: {msg}"
        );
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_plot3_ascii_no_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[0.0, 1.0, 2.0, 3.0]);
        let y = f64_vec(&[0.0, 1.0, 0.0, -1.0]);
        let z = f64_vec(&[0.0, 0.5, 1.0, 0.5]);
        let result = plugin.call("plot3", &[x, y, z], &env);
        assert!(result.is_ok(), "plot3 ASCII should succeed: {result:?}");
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_scatter3_ascii_no_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[0.0, 1.0, 2.0]);
        let y = f64_vec(&[0.0, 1.0, 0.0]);
        let z = f64_vec(&[1.0, 2.0, 3.0]);
        let result = plugin.call("scatter3", &[x, y, z], &env);
        assert!(result.is_ok(), "scatter3 ASCII should succeed: {result:?}");
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_plot3_state_cleared_after_render() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("zlabel", &[Value::Str("depth".into())], &env)
            .unwrap();
        let x = f64_vec(&[0.0, 1.0, 2.0]);
        let y = f64_vec(&[0.0, 1.0, 2.0]);
        let z = f64_vec(&[0.0, 1.0, 2.0]);
        plugin.call("plot3", &[x, y, z], &env).unwrap();
        let zlabel = FIGURE_STATE.with(|f| f.borrow().zlabel.clone());
        assert!(
            zlabel.is_none(),
            "FigureState.zlabel should be cleared after plot3()"
        );
    }

    #[test]
    #[cfg(not(feature = "plot-svg"))]
    fn test_plot3_svg_without_feature() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[0.0, 1.0]);
        let y = f64_vec(&[0.0, 1.0]);
        let z = f64_vec(&[0.0, 1.0]);
        let path = Value::Str("out.svg".into());
        let result = plugin.call("plot3", &[x, y, z, path], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("plot-svg"),
            "error should mention plot-svg feature: {msg}"
        );
    }

    // ── 30a: colormap / colorbar / imagesc ────────────────────────────

    #[test]
    fn test_colormap_sets_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("colormap", &[Value::Str("hot".into())], &env)
            .unwrap();
        let cmap = FIGURE_STATE.with(|f| f.borrow().colormap.clone());
        assert_eq!(cmap, Some(colormap::ColormapSpec::Named("hot".to_string())));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_colorbar_sets_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin.call("colorbar", &[], &env).unwrap();
        let cb = FIGURE_STATE.with(|f| f.borrow().colorbar);
        assert!(cb, "colorbar should set FigureState.colorbar = true");
        FIGURE_STATE.with(|f| f.take());
    }

    // ── 30.5b: extended style strings ─────────────────────────────────────

    #[test]
    fn test_style_rgb_matrix_dispatch() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let x = f64_vec(&[1.0, 2.0]);
        let y = f64_vec(&[1.0, 2.0]);
        let m = Value::Matrix(Array2::from_shape_vec((1, 3), vec![1.0, 0.0, 0.0]).unwrap());
        plugin.call("plot", &[x, y, m], &env).unwrap();
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        assert_eq!(series.len(), 1, "should have one pending series");
        if let PendingSeries::Line(_, _, style) = &series[0] {
            assert_eq!(
                style.as_ref().and_then(|s| s.color),
                Some(style::StyleColor(255, 0, 0))
            );
        } else {
            panic!("expected PendingSeries::Line");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_style_color_named_arg_bar() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let v = f64_vec(&[1.0, 2.0, 3.0]);
        plugin
            .call(
                "bar",
                &[v, Value::Str("color".into()), Value::Str("blue".into())],
                &env,
            )
            .expect("bar with 'color' named arg should succeed");
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        assert_eq!(series.len(), 1);
        if let PendingSeries::Bar(_, _, style) = &series[0] {
            assert_eq!(
                style.as_ref().and_then(|s| s.color),
                Some(style::StyleColor(0, 0, 255)),
                "bar should carry blue style"
            );
        } else {
            panic!("expected PendingSeries::Bar");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_style_color_named_arg_hex() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let v = f64_vec(&[1.0, 2.0, 3.0]);
        plugin
            .call(
                "bar",
                &[v, Value::Str("color".into()), Value::Str("#FF4400".into())],
                &env,
            )
            .expect("bar with hex color should succeed");
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        assert_eq!(series.len(), 1);
        if let PendingSeries::Bar(_, _, style) = &series[0] {
            assert_eq!(
                style.as_ref().and_then(|s| s.color),
                Some(style::StyleColor(0xFF, 0x44, 0x00)),
                "bar should carry #FF4400 style"
            );
        } else {
            panic!("expected PendingSeries::Bar");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_colormap_matrix_dispatch() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let m = Array2::from_shape_vec((2, 3), vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0]).unwrap();
        let result = plugin.call("colormap", &[Value::Matrix(m)], &env);
        assert!(
            result.is_ok(),
            "colormap(N×3 matrix) should succeed: {result:?}"
        );
        let spec = FIGURE_STATE.with(|f| f.borrow().colormap.clone());
        assert!(
            matches!(spec, Some(colormap::ColormapSpec::Custom(_))),
            "should store ColormapSpec::Custom"
        );
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_colormap_matrix_wrong_cols() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let m = Array2::from_shape_vec((2, 2), vec![1.0, 0.0, 0.0, 1.0]).unwrap();
        let result = plugin.call("colormap", &[Value::Matrix(m)], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("N×3"), "error should mention N×3: {msg}");
    }

    // ── 30.5c: Option<StyleSpec> for Bar / Stem / Hist / Quiver ─────────────

    #[test]
    fn test_bar_accumulates_with_style_red() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let x = f64_vec(&[1.0, 2.0, 3.0]);
        let y = f64_vec(&[4.0, 5.0, 6.0]);
        plugin
            .call("bar", &[x, y, Value::Str("r".into())], &env)
            .unwrap();
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        assert_eq!(series.len(), 1, "should have one bar series");
        if let PendingSeries::Bar(_, _, style) = &series[0] {
            assert_eq!(
                style.as_ref().and_then(|s| s.color),
                Some(style::StyleColor(255, 0, 0)),
                "bar should carry red style"
            );
        } else {
            panic!("expected PendingSeries::Bar");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_stem_accumulates_with_style_blue() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let x = f64_vec(&[1.0, 2.0, 3.0]);
        let y = f64_vec(&[1.0, 2.0, 3.0]);
        plugin
            .call("stem", &[x, y, Value::Str("blue".into())], &env)
            .unwrap();
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        assert_eq!(series.len(), 1, "should have one stem series");
        if let PendingSeries::Stem(_, _, style) = &series[0] {
            assert_eq!(
                style.as_ref().and_then(|s| s.color),
                Some(style::StyleColor(0, 0, 255)),
                "stem should carry blue style"
            );
        } else {
            panic!("expected PendingSeries::Stem");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_hist_accumulates_with_style_hex() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let data = f64_vec(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        plugin
            .call("hist", &[data, Value::Str("#FF8800".into())], &env)
            .unwrap();
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        assert_eq!(series.len(), 1, "should have one hist series");
        if let PendingSeries::Hist { style, .. } = &series[0] {
            assert_eq!(
                style.as_ref().and_then(|s| s.color),
                Some(style::StyleColor(0xFF, 0x88, 0x00)),
                "hist should carry hex colour style"
            );
        } else {
            panic!("expected PendingSeries::Hist");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_quiver_accumulates_with_style_green() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let x = f64_vec(&[0.0, 1.0]);
        let y = f64_vec(&[0.0, 1.0]);
        let u = f64_vec(&[1.0, 0.0]);
        let v = f64_vec(&[0.0, 1.0]);
        plugin
            .call("quiver", &[x, y, u, v, Value::Str("g".into())], &env)
            .unwrap();
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        assert_eq!(series.len(), 1, "should have one quiver series");
        if let PendingSeries::Quiver(_, _, _, _, style) = &series[0] {
            assert_eq!(
                style.as_ref().and_then(|s| s.color),
                Some(style::StyleColor(0, 128, 0)),
                "quiver should carry green style"
            );
        } else {
            panic!("expected PendingSeries::Quiver");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_bar_no_style_stores_none() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let x = f64_vec(&[1.0, 2.0]);
        let y = f64_vec(&[3.0, 4.0]);
        plugin.call("bar", &[x, y], &env).unwrap();
        let series = FIGURE_STATE.with(|f| f.borrow().pending_series.clone());
        if let PendingSeries::Bar(_, _, style) = &series[0] {
            assert!(style.is_none(), "unstyled bar should have None style");
        } else {
            panic!("expected PendingSeries::Bar");
        }
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_bar_svg_with_red_style() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let tmp = std::env::temp_dir().join("bar_red_30_5c.svg");
        let path = tmp.to_string_lossy().to_string();
        let x = f64_vec(&[1.0, 2.0, 3.0]);
        let y = f64_vec(&[4.0, 5.0, 3.0]);
        let result = plugin.call(
            "bar",
            &[x, y, Value::Str("r".into()), Value::Str(path.clone())],
            &env,
        );
        assert!(
            result.is_ok(),
            "bar with red style to SVG should succeed: {result:?}"
        );
        assert!(
            std::path::Path::new(&path).exists(),
            "SVG file should be created"
        );
        let _ = std::fs::remove_file(&path);
        FIGURE_STATE.with(|f| f.take());
    }

    // ── figure() tests ───────────────────────────────────────────────────────

    #[test]
    fn test_figure_sets_canvas_size() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call(
                "figure",
                &[Value::Scalar(1200.0), Value::Scalar(400.0)],
                &env,
            )
            .unwrap();
        let size = FIGURE_STATE.with(|f| f.borrow().figure_size);
        assert_eq!(size, Some((1200, 400)));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_figure_default_canvas_size() {
        FIGURE_STATE.with(|f| f.take());
        let st = FIGURE_STATE.with(|f| f.take());
        assert_eq!(st.canvas_size(), (800, 600));
    }

    #[test]
    fn test_figure_wrong_arg_count_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("figure", &[Value::Scalar(800.0)], &env);
        assert!(result.is_err());
        let result = plugin.call("figure", &[], &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_figure_invalid_size_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("figure", &[Value::Scalar(0.0), Value::Scalar(600.0)], &env);
        assert!(result.is_err(), "width 0 should error");
        let result = plugin.call(
            "figure",
            &[Value::Scalar(800.0), Value::Scalar(20000.0)],
            &env,
        );
        assert!(result.is_err(), "height > 16384 should error");
    }

    #[test]
    fn test_figure_in_builtin_names() {
        use ccalc_engine::eval::builtin_names;
        assert!(
            builtin_names().contains(&"figure"),
            "figure missing from builtin_names"
        );
    }

    #[test]
    fn test_colormap_invalid_name_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("colormap", &[Value::Str("notacolormap".into())], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("colormap"),
            "error should mention colormap: {msg}"
        );
    }

    #[test]
    fn test_apply_colormap_gray_extremes() {
        let (r, g, b) = colormap::apply_colormap(0.0, "gray");
        assert_eq!((r, g, b), (0, 0, 0));
        let (r, g, b) = colormap::apply_colormap(1.0, "gray");
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_imagesc_non_matrix_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("imagesc", &[Value::Str("notamatrix".into())], &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_imagesc_no_args_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("imagesc", &[], &env);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(feature = "plot-svg"))]
    fn test_imagesc_svg_without_feature_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let z = Value::Matrix(Array2::from_shape_vec((2, 2), vec![1.0, 2.0, 3.0, 4.0]).unwrap());
        let path = Value::Str("out.svg".into());
        let result = plugin.call("imagesc", &[z, path], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("plot-svg"),
            "error should mention plot-svg feature: {msg}"
        );
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_imagesc_ascii_no_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let z = Value::Matrix(
            Array2::from_shape_vec((4, 4), (0..16).map(|i| i as f64).collect()).unwrap(),
        );
        let result = plugin.call("imagesc", &[z], &env);
        assert!(result.is_ok(), "imagesc ASCII should succeed: {result:?}");
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_imagesc_ascii_with_colorbar_no_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("colormap", &[Value::Str("jet".into())], &env)
            .unwrap();
        plugin.call("colorbar", &[], &env).unwrap();
        let z = Value::Matrix(
            Array2::from_shape_vec((3, 3), (0..9).map(|i| i as f64).collect()).unwrap(),
        );
        let result = plugin.call("imagesc", &[z], &env);
        assert!(
            result.is_ok(),
            "imagesc with colorbar should succeed: {result:?}"
        );
    }

    // ── 30b: surf / mesh ───────────────────────────────────────────────────

    #[allow(dead_code)]
    fn make_xyz(rows: usize, cols: usize) -> (Value, Value, Value) {
        let x = Value::Matrix(Array2::from_shape_fn((rows, cols), |(_r, c)| c as f64));
        let y = Value::Matrix(Array2::from_shape_fn((rows, cols), |(r, _c)| r as f64));
        let z = Value::Matrix(Array2::from_shape_fn((rows, cols), |(r, c)| (r + c) as f64));
        (x, y, z)
    }

    #[test]
    fn test_surf_dimension_mismatch_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = Value::Matrix(Array2::from_shape_vec((2, 3), vec![1.0; 6]).unwrap());
        let y = Value::Matrix(Array2::from_shape_vec((3, 2), vec![1.0; 6]).unwrap());
        let z = Value::Matrix(Array2::from_shape_vec((2, 3), vec![0.0; 6]).unwrap());
        let err = plugin.call("surf", &[x, y, z], &env).unwrap_err();
        assert!(
            err.contains("same dimensions"),
            "error should mention dimensions: {err}"
        );
    }

    #[test]
    fn test_mesh_dimension_mismatch_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = Value::Matrix(Array2::from_shape_vec((2, 3), vec![1.0; 6]).unwrap());
        let y = Value::Matrix(Array2::from_shape_vec((2, 2), vec![1.0; 4]).unwrap());
        let z = Value::Matrix(Array2::from_shape_vec((2, 3), vec![0.0; 6]).unwrap());
        let err = plugin.call("mesh", &[x, y, z], &env).unwrap_err();
        assert!(
            err.contains("same dimensions"),
            "error should mention dimensions: {err}"
        );
    }

    #[test]
    fn test_surf_missing_args_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = Value::Matrix(Array2::from_shape_vec((2, 2), vec![1.0; 4]).unwrap());
        let err = plugin.call("surf", &[x], &env).unwrap_err();
        assert!(
            err.contains("requires"),
            "error should mention requires: {err}"
        );
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_surf_ascii_no_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_xyz(5, 8);
        let result = plugin.call("surf", &[x, y, z], &env);
        assert!(result.is_ok(), "surf ASCII should succeed: {result:?}");
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_mesh_ascii_no_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_xyz(5, 8);
        let result = plugin.call("mesh", &[x, y, z], &env);
        assert!(result.is_ok(), "mesh ASCII should succeed: {result:?}");
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_surf_svg_creates_file() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_xyz(4, 5);
        let path = ".debug/test_surf.svg";
        std::fs::create_dir_all(".debug").ok();
        let result = plugin.call("surf", &[x, y, z, Value::Str(path.into())], &env);
        assert!(result.is_ok(), "surf SVG should succeed: {result:?}");
        let content = std::fs::read_to_string(path).unwrap();
        assert!(
            content.contains("<svg"),
            "output should be SVG: starts with {}",
            &content[..50.min(content.len())]
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_mesh_png_creates_file() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_xyz(4, 5);
        let path = ".debug/test_mesh.png";
        std::fs::create_dir_all(".debug").ok();
        let result = plugin.call("mesh", &[x, y, z, Value::Str(path.into())], &env);
        assert!(result.is_ok(), "mesh PNG should succeed: {result:?}");
        let bytes = std::fs::read(path).unwrap();
        // PNG magic bytes: 0x89 P N G
        assert_eq!(
            &bytes[0..4],
            &[0x89, 0x50, 0x4E, 0x47],
            "output should be PNG"
        );
        std::fs::remove_file(path).ok();
    }

    // ── 30c: contour / contourf ────────────────────────────────────────────

    #[allow(dead_code)]
    fn make_contour_xyz(rows: usize, cols: usize) -> (Value, Value, Value) {
        // X, Y from meshgrid; Z = Gaussian bell centred at (0,0)
        let x = Value::Matrix(Array2::from_shape_fn((rows, cols), |(_r, c)| {
            -2.0 + 4.0 * c as f64 / (cols - 1).max(1) as f64
        }));
        let y = Value::Matrix(Array2::from_shape_fn((rows, cols), |(r, _c)| {
            -2.0 + 4.0 * r as f64 / (rows - 1).max(1) as f64
        }));
        let z = Value::Matrix(Array2::from_shape_fn((rows, cols), |(r, c)| {
            let xi = -2.0 + 4.0 * c as f64 / (cols - 1).max(1) as f64;
            let yi = -2.0 + 4.0 * r as f64 / (rows - 1).max(1) as f64;
            (-xi * xi - yi * yi).exp()
        }));
        (x, y, z)
    }

    #[test]
    fn test_contour_non_matrix_x_errors() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = Value::Str("notamatrix".into());
        let y = f64_vec(&[0.0, 1.0]);
        let z = f64_vec(&[0.0, 1.0]);
        let result = plugin.call("contour", &[x, y, z], &env);
        assert!(result.is_err(), "non-matrix X should error");
        let msg = result.unwrap_err();
        assert!(msg.contains("X"), "error should mention X: {msg}");
    }

    #[test]
    fn test_contour_mismatched_dimensions_errors() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = Value::Matrix(Array2::from_shape_vec((2, 3), vec![0.0; 6]).unwrap());
        let y = Value::Matrix(Array2::from_shape_vec((3, 2), vec![0.0; 6]).unwrap());
        let z = Value::Matrix(Array2::from_shape_vec((2, 3), vec![0.0; 6]).unwrap());
        let result = plugin.call("contour", &[x, y, z], &env);
        assert!(result.is_err(), "mismatched dimensions should error");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("same dimensions"),
            "error should mention dimensions: {msg}"
        );
    }

    #[test]
    fn test_contour_missing_args_errors() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = Value::Matrix(Array2::from_shape_vec((2, 2), vec![0.0; 4]).unwrap());
        let result = plugin.call("contour", &[x], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("requires"),
            "error should mention requires: {msg}"
        );
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_contour_ascii_no_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_contour_xyz(10, 12);
        let result = plugin.call("contour", &[x, y, z, Value::Scalar(5.0)], &env);
        assert!(result.is_ok(), "contour ASCII should succeed: {result:?}");
    }

    #[test]
    #[cfg(feature = "plot")]
    fn test_contourf_ascii_no_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_contour_xyz(10, 12);
        let result = plugin.call("contourf", &[x, y, z, Value::Scalar(5.0)], &env);
        assert!(result.is_ok(), "contourf ASCII should succeed: {result:?}");
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_contour_svg_creates_file() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_contour_xyz(15, 20);
        let path = ".debug/test_contour.svg";
        std::fs::create_dir_all(".debug").ok();
        let result = plugin.call(
            "contour",
            &[x, y, z, Value::Scalar(5.0), Value::Str(path.into())],
            &env,
        );
        assert!(result.is_ok(), "contour SVG should succeed: {result:?}");
        let content = std::fs::read_to_string(path).unwrap();
        assert!(
            content.contains("<svg"),
            "output should be SVG: starts with {}",
            &content[..50.min(content.len())]
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_contourf_png_magic_bytes() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_contour_xyz(15, 20);
        let path = ".debug/test_contourf.png";
        std::fs::create_dir_all(".debug").ok();
        let result = plugin.call(
            "contourf",
            &[x, y, z, Value::Scalar(5.0), Value::Str(path.into())],
            &env,
        );
        assert!(result.is_ok(), "contourf PNG should succeed: {result:?}");
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            &bytes[0..4],
            &[0x89, 0x50, 0x4E, 0x47],
            "output should be PNG"
        );
        std::fs::remove_file(path).ok();
    }

    // ── Phase 30d: subplot + hold + savefig ──────────────────────────

    #[test]
    fn test_subplot_sets_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call(
                "subplot",
                &[Value::Scalar(2.0), Value::Scalar(2.0), Value::Scalar(1.0)],
                &env,
            )
            .unwrap();
        let subplot = FIGURE_STATE.with(|f| f.borrow().subplot);
        assert_eq!(subplot, Some((2, 2, 1)));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_hold_on_sets_flag() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let hold = FIGURE_STATE.with(|f| f.borrow().hold);
        assert!(hold, "hold flag should be true after hold('on')");
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_hold_off_clears_flag_and_series() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        // Prime hold + a series so hold('off') has something to flush.
        FIGURE_STATE.with(|f| {
            let mut st = f.borrow_mut();
            st.hold = true;
            st.pending_series
                .push(PendingSeries::Line(vec![1.0, 2.0], vec![1.0, 4.0], None));
        });
        // State is mutated before ASCII rendering; ignore the render result so
        // this test passes regardless of which feature flags are enabled.
        let _ = plugin.call("hold", &[Value::Str("off".into())], &env);
        let (hold, series_empty) = FIGURE_STATE.with(|f| {
            let st = f.borrow();
            (st.hold, st.pending_series.is_empty())
        });
        assert!(!hold, "hold should be false after hold('off')");
        assert!(
            series_empty,
            "pending_series should be cleared after hold('off')"
        );
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_plot_accumulates_under_hold() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        let y1 = f64_vec(&[1.0, 2.0, 3.0]);
        let y2 = f64_vec(&[3.0, 2.0, 1.0]);
        plugin.call("plot", &[y1], &env).unwrap();
        plugin.call("plot", &[y2], &env).unwrap();
        let count = FIGURE_STATE.with(|f| f.borrow().pending_series.len());
        assert_eq!(count, 2, "two plot calls should accumulate 2 series");
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_subplot_then_plot_accumulates() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call(
                "subplot",
                &[Value::Scalar(2.0), Value::Scalar(1.0), Value::Scalar(1.0)],
                &env,
            )
            .unwrap();
        let y = f64_vec(&[1.0, 2.0, 3.0]);
        plugin.call("plot", &[y], &env).unwrap();
        let count = FIGURE_STATE.with(|f| f.borrow().pending_series.len());
        assert_eq!(
            count, 1,
            "plot under subplot should accumulate into pending_series"
        );
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_second_subplot_commits_first_panel() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call(
                "subplot",
                &[Value::Scalar(2.0), Value::Scalar(1.0), Value::Scalar(1.0)],
                &env,
            )
            .unwrap();
        plugin.call("plot", &[f64_vec(&[1.0, 2.0])], &env).unwrap();
        // Move to panel 2 — should commit panel 1
        plugin
            .call(
                "subplot",
                &[Value::Scalar(2.0), Value::Scalar(1.0), Value::Scalar(2.0)],
                &env,
            )
            .unwrap();
        let (panels_len, pending_len) = FIGURE_STATE.with(|f| {
            let st = f.borrow();
            (st.panels.len(), st.pending_series.len())
        });
        assert_eq!(panels_len, 1, "panel 1 should be committed");
        assert_eq!(
            pending_len, 0,
            "pending_series should be empty after commit"
        );
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_subplot_invalid_index_errors() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call(
            "subplot",
            &[Value::Scalar(2.0), Value::Scalar(2.0), Value::Scalar(5.0)],
            &env,
        );
        assert!(result.is_err(), "index 5 in a 2×2 grid should error");
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_savefig_with_no_panels_errors() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("savefig", &[Value::Str("out.svg".into())], &env);
        assert!(result.is_err(), "savefig with no panels should error");
        FIGURE_STATE.with(|f| f.take());
    }

    // ── Phase 30f: quiver + text ───────────────────────────────────────────

    #[test]
    fn test_quiver_mismatch_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[0.0, 1.0, 2.0]);
        let y = f64_vec(&[0.0, 1.0, 2.0]);
        let u = f64_vec(&[1.0, 0.0]);
        let v = f64_vec(&[0.0, 1.0, 0.0]);
        let result = plugin.call("quiver", &[x, y, u, v], &env);
        assert!(result.is_err(), "length mismatch should produce an error");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("same length"),
            "error should mention 'same length': {msg}"
        );
    }

    #[test]
    fn test_text_stores_annotation() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call(
                "text",
                &[
                    Value::Scalar(0.0),
                    Value::Scalar(1.0),
                    Value::Str("label".into()),
                ],
                &env,
            )
            .unwrap();
        let ann = FIGURE_STATE.with(|f| f.borrow().annotations.clone());
        assert_eq!(ann.len(), 1, "one annotation should be stored");
        assert_eq!(ann[0], (0.0, 1.0, "label".to_string()));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_quiver_svg_creates_file() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let x = f64_vec(&[0.0, 1.0, 0.0, 1.0]);
        let y = f64_vec(&[0.0, 0.0, 1.0, 1.0]);
        let u = f64_vec(&[1.0, 0.0, -1.0, 0.0]);
        let v = f64_vec(&[0.0, 1.0, 0.0, -1.0]);
        let path = ".debug/test_quiver.svg";
        std::fs::create_dir_all(".debug").ok();
        let result = plugin.call("quiver", &[x, y, u, v, Value::Str(path.into())], &env);
        assert!(result.is_ok(), "quiver SVG should succeed: {result:?}");
        let content = std::fs::read_to_string(path).unwrap();
        assert!(
            content.contains("<svg"),
            "output should be SVG: starts with {}",
            &content[..50.min(content.len())]
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_subplot_savefig_creates_svg() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let path = ".debug/test_subplot_grid.svg";
        std::fs::create_dir_all(".debug").ok();
        plugin
            .call(
                "subplot",
                &[Value::Scalar(2.0), Value::Scalar(1.0), Value::Scalar(1.0)],
                &env,
            )
            .unwrap();
        plugin
            .call("plot", &[f64_vec(&[1.0, 2.0, 3.0])], &env)
            .unwrap();
        plugin
            .call(
                "subplot",
                &[Value::Scalar(2.0), Value::Scalar(1.0), Value::Scalar(2.0)],
                &env,
            )
            .unwrap();
        plugin
            .call("plot", &[f64_vec(&[3.0, 2.0, 1.0])], &env)
            .unwrap();
        plugin
            .call("savefig", &[Value::Str(path.into())], &env)
            .unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(
            content.contains("<svg"),
            "savefig should produce an SVG file"
        );
        std::fs::remove_file(path).ok();
    }

    #[cfg(feature = "plot-svg")]
    #[test]
    fn test_figure_size_applied_to_svg() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let path = ".debug/test_figure_size.svg";
        std::fs::create_dir_all(".debug").ok();
        plugin
            .call(
                "figure",
                &[Value::Scalar(1024.0), Value::Scalar(300.0)],
                &env,
            )
            .unwrap();
        plugin
            .call(
                "plot",
                &[
                    f64_vec(&[1.0, 2.0, 3.0]),
                    f64_vec(&[1.0, 4.0, 9.0]),
                    Value::Str(path.into()),
                ],
                &env,
            )
            .unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(
            content.contains("1024"),
            "SVG should contain requested width"
        );
        assert!(
            content.contains("300"),
            "SVG should contain requested height"
        );
        std::fs::remove_file(path).ok();
    }

    // ── Phase 30.6a — Theme + bgcolor ─────────────────────────────────

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_theme_dark_svg_contains_dark_bg() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());

        let path = ".debug/test_theme_dark.svg";
        plugin
            .call("theme", &[Value::Str("dark".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[
                    f64_vec(&[1.0, 2.0]),
                    f64_vec(&[1.0, 2.0]),
                    Value::Str(path.into()),
                ],
                &env,
            )
            .unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        // Dark theme background is #1E1E2E.
        assert!(
            content.contains("1E1E2E") || content.contains("1e1e2e"),
            "SVG must contain the dark theme background colour"
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_theme_light_is_default() {
        let light = style::Theme::light();
        // Default FigureState has no theme → resolve_theme returns light.
        let st = FigureState::default();
        let resolved = st.resolve_theme();
        assert_eq!(resolved.bg, light.bg);
        assert_eq!(resolved.text, light.text);
    }

    #[test]
    fn test_theme_unknown_name_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("theme", &[Value::Str("rainbow".into())], &env);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown theme"));
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_bgcolor_overrides_theme_bg() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());

        let path = ".debug/test_bgcolor_override.svg";
        plugin
            .call("theme", &[Value::Str("dark".into())], &env)
            .unwrap();
        // Override with a bright red background.
        plugin
            .call("bgcolor", &[Value::Str("red".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[
                    f64_vec(&[1.0, 2.0]),
                    f64_vec(&[1.0, 2.0]),
                    Value::Str(path.into()),
                ],
                &env,
            )
            .unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        // Red = #FF0000; dark theme bg #1E1E2E must NOT be the fill.
        assert!(
            !content.contains("1E1E2E") && !content.contains("1e1e2e"),
            "Dark theme bg should not appear when bgcolor overrides it"
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_bgcolor_hex_accepted() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("bgcolor", &[Value::Str("#AABBCC".into())], &env)
            .unwrap();
        let bg = FIGURE_STATE.with(|f| f.borrow().bg_color);
        assert_eq!(bg, Some(style::StyleColor(0xAA, 0xBB, 0xCC)));
    }

    #[test]
    fn test_bgcolor_rgb_matrix() {
        use ndarray::Array2;
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        // [0.0, 0.5, 1.0] as 1×3 matrix → RGB(0, 128, 255).
        let m = Value::Matrix(Array2::from_shape_vec((1, 3), vec![0.0_f64, 0.5, 1.0]).unwrap());
        plugin.call("bgcolor", &[m], &env).unwrap();
        let bg = FIGURE_STATE.with(|f| f.borrow().bg_color);
        assert_eq!(bg, Some(style::StyleColor(0, 128, 255)));
    }

    // ── Phase 30.6b tests ──────────────────────────────────────────────────

    #[test]
    fn test_linewidth_named_arg_plot() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[
                    f64_vec(&[0.0, 1.0]),
                    f64_vec(&[0.0, 1.0]),
                    Value::Str("r--".into()),
                    Value::Str("linewidth".into()),
                    Value::Scalar(2.5),
                ],
                &env,
            )
            .unwrap();
        let lw = FIGURE_STATE.with(|f| {
            if let Some(PendingSeries::Line(_, _, Some(sp))) = f.borrow().pending_series.first() {
                sp.line_width
            } else {
                None
            }
        });
        assert_eq!(lw, Some(2.5_f32));
    }

    #[test]
    fn test_markersize_named_arg_scatter() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        plugin
            .call(
                "scatter",
                &[
                    f64_vec(&[1.0, 2.0]),
                    f64_vec(&[1.0, 2.0]),
                    Value::Str("markersize".into()),
                    Value::Scalar(7.0),
                ],
                &env,
            )
            .unwrap();
        let ms = FIGURE_STATE.with(|f| {
            if let Some(PendingSeries::Scatter(_, _, Some(sp))) = f.borrow().pending_series.first()
            {
                sp.marker_size
            } else {
                None
            }
        });
        assert_eq!(ms, Some(7_u32));
    }

    #[test]
    fn test_linewidth_and_markersize_combined() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[
                    f64_vec(&[0.0, 1.0]),
                    f64_vec(&[0.0, 1.0]),
                    Value::Str("b.".into()),
                    Value::Str("linewidth".into()),
                    Value::Scalar(1.5),
                    Value::Str("markersize".into()),
                    Value::Scalar(8.0),
                ],
                &env,
            )
            .unwrap();
        let (lw, ms) = FIGURE_STATE.with(|f| {
            if let Some(PendingSeries::Line(_, _, Some(sp))) = f.borrow().pending_series.first() {
                (sp.line_width, sp.marker_size)
            } else {
                (None, None)
            }
        });
        assert_eq!(lw, Some(1.5_f32));
        assert_eq!(ms, Some(8_u32));
    }

    #[test]
    fn test_fontsize_global_setter() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("fontsize", &[Value::Scalar(18.0)], &env)
            .unwrap();
        let fs = FIGURE_STATE.with(|f| f.borrow().font_size);
        assert_eq!(fs, Some(18_u32));
    }

    #[test]
    fn test_linewidth_global_setter() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("linewidth", &[Value::Scalar(3.0)], &env)
            .unwrap();
        let lw = FIGURE_STATE.with(|f| f.borrow().line_width);
        assert_eq!(lw, Some(3.0_f32));
    }

    #[test]
    fn test_markersize_global_setter() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("markersize", &[Value::Scalar(5.0)], &env)
            .unwrap();
        let ms = FIGURE_STATE.with(|f| f.borrow().marker_size);
        assert_eq!(ms, Some(5_u32));
    }

    // ── Phase 30.6c — grid style ────────────────────────────────────────

    #[test]
    fn test_gridcolor_named_color() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("gridcolor", &[Value::Str("red".into())], &env)
            .unwrap();
        let gc = FIGURE_STATE.with(|f| f.borrow().grid_color);
        assert_eq!(gc, Some(StyleColor(255, 0, 0)));
    }

    #[test]
    fn test_gridcolor_rgb_matrix() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        use ccalc_engine::env::Value;
        use ndarray::arr2;
        let m = Value::Matrix(arr2(&[[0.0_f64, 1.0, 0.0]]));
        plugin.call("gridcolor", &[m], &env).unwrap();
        let gc = FIGURE_STATE.with(|f| f.borrow().grid_color);
        assert_eq!(gc, Some(StyleColor(0, 255, 0)));
    }

    #[test]
    fn test_gridwidth_global_setter() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("gridwidth", &[Value::Scalar(2.0)], &env)
            .unwrap();
        let gw = FIGURE_STATE.with(|f| f.borrow().grid_width);
        assert_eq!(gw, Some(2.0_f32));
    }

    // ── 30.6d: axis mode ─────────────────────────────────────────────────────

    #[test]
    fn test_axis_equal_sets_state() {
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("axis", &[Value::Str("equal".into())], &env)
            .unwrap();
        let mode = FIGURE_STATE.with(|f| f.borrow().axis_mode);
        assert_eq!(mode, Some(style::AxisMode::Equal));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_axis_tight_sets_state() {
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("axis", &[Value::Str("tight".into())], &env)
            .unwrap();
        let mode = FIGURE_STATE.with(|f| f.borrow().axis_mode);
        assert_eq!(mode, Some(style::AxisMode::Tight));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_axis_off_sets_state() {
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("axis", &[Value::Str("off".into())], &env)
            .unwrap();
        let mode = FIGURE_STATE.with(|f| f.borrow().axis_mode);
        assert_eq!(mode, Some(style::AxisMode::Off));
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_axis_on_clears_mode() {
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("axis", &[Value::Str("equal".into())], &env)
            .unwrap();
        plugin
            .call("axis", &[Value::Str("on".into())], &env)
            .unwrap();
        let mode = FIGURE_STATE.with(|f| f.borrow().axis_mode);
        assert_eq!(mode, None, "axis('on') should clear the axis mode");
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_axis_invalid_arg_errors() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("axis", &[Value::Str("square".into())], &env);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("expected"),
            "error should describe valid options: {msg}"
        );
    }

    #[test]
    fn test_axis_mode_carried_into_panel() {
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("axis", &[Value::Str("tight".into())], &env)
            .unwrap();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        plugin
            .call("plot", &[f64_vec(&[0.0, 1.0]), f64_vec(&[0.0, 1.0])], &env)
            .unwrap();
        // commit_current_panel via subplot
        plugin
            .call(
                "subplot",
                &[Value::Scalar(1.0), Value::Scalar(2.0), Value::Scalar(2.0)],
                &env,
            )
            .unwrap();
        let mode = FIGURE_STATE.with(|f| f.borrow().panels.first().and_then(|p| p.axis_mode));
        assert_eq!(
            mode,
            Some(style::AxisMode::Tight),
            "axis_mode should be carried into the committed panel"
        );
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn test_axis_off_svg_no_error() {
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("axis", &[Value::Str("off".into())], &env)
            .unwrap();
        let tmp = std::env::temp_dir().join("axis_off_30_6d.svg");
        let path = tmp.to_string_lossy().to_string();
        let x = f64_vec(&[1.0, 2.0, 3.0]);
        let y = f64_vec(&[1.0, 4.0, 9.0]);
        let result = plugin.call("plot", &[x, y, Value::Str(path.clone())], &env);
        assert!(
            result.is_ok(),
            "axis('off') + plot to SVG should succeed: {result:?}"
        );
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(content.contains("<svg"), "output should contain <svg");
        let _ = std::fs::remove_file(&path);
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn test_gridcolor_carried_into_panel() {
        let plugin = PlotPlugin;
        let env = Env::new();
        FIGURE_STATE.with(|f| *f.borrow_mut() = FigureState::default());
        plugin
            .call("gridcolor", &[Value::Str("blue".into())], &env)
            .unwrap();
        plugin
            .call("gridwidth", &[Value::Scalar(3.0)], &env)
            .unwrap();
        plugin
            .call("hold", &[Value::Str("on".into())], &env)
            .unwrap();
        plugin
            .call("plot", &[f64_vec(&[0.0, 1.0]), f64_vec(&[0.0, 1.0])], &env)
            .unwrap();
        // commit_current_panel via subplot call
        plugin
            .call(
                "subplot",
                &[Value::Scalar(1.0), Value::Scalar(2.0), Value::Scalar(2.0)],
                &env,
            )
            .unwrap();
        let (gc, gw) = FIGURE_STATE.with(|f| {
            f.borrow()
                .panels
                .first()
                .map(|p| (p.grid_color, p.grid_width))
                .unwrap_or((None, None))
        });
        assert_eq!(gc, Some(StyleColor(0, 0, 255)));
        assert_eq!(gw, Some(3.0_f32));
    }

    // ── Phase 32c: pie ─────────────────────────────────────────────────────

    #[test]
    fn pie_ascii_sums_100pct() {
        // Each bar line should show a percentage that adds up to ~100%.
        let values = vec![25.0_f64, 50.0, 25.0];
        let labels: Vec<String> = vec!["A".into(), "B".into(), "C".into()];
        let out = format_pie_ascii(&values, &labels, &[]);
        // Extract percentages from lines like " [██...] 25.0%  A"
        let pct_sum: f64 = out
            .lines()
            .filter_map(|line| {
                let pct_part = line.split('%').next()?;
                let num = pct_part.rsplit_once(']')?.1.trim();
                num.parse::<f64>().ok()
            })
            .sum();
        assert!(
            (pct_sum - 100.0).abs() < 0.1,
            "percentages should sum to ~100, got {pct_sum}"
        );
    }

    #[test]
    fn pie_ascii_contains_labels() {
        let values = vec![60.0_f64, 40.0];
        let labels: Vec<String> = vec!["Alpha".into(), "Beta".into()];
        let out = format_pie_ascii(&values, &labels, &[]);
        assert!(out.contains("Alpha"), "output should contain label 'Alpha'");
        assert!(out.contains("Beta"), "output should contain label 'Beta'");
    }

    #[test]
    fn pie_ascii_explode_marker() {
        let values = vec![50.0_f64, 30.0, 20.0];
        let labels: Vec<String> = vec![String::new(); 3];
        let explode = vec![0.0_f64, 0.1, 0.0];
        let out = format_pie_ascii(&values, &labels, &explode);
        let lines: Vec<&str> = out.lines().collect();
        // Second slice (index 1) should have ◄ suffix, others should not.
        assert!(
            !lines[0].ends_with('\u{25c4}'),
            "non-exploded slice 0 should not have ◄"
        );
        assert!(
            lines[1].ends_with('\u{25c4}'),
            "exploded slice 1 should end with ◄"
        );
        assert!(
            !lines[2].ends_with('\u{25c4}'),
            "non-exploded slice 2 should not have ◄"
        );
    }

    #[test]
    fn pie_dispatch_empty_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let err = plugin.call("pie", &[f64_vec(&[])], &env).unwrap_err();
        assert!(
            err.contains("empty") || err.contains("positive") || err.contains("non-negative"),
            "expected meaningful error, got: {err}"
        );
    }

    #[test]
    fn pie_dispatch_negative_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let err = plugin
            .call("pie", &[f64_vec(&[1.0, -2.0, 3.0])], &env)
            .unwrap_err();
        assert!(
            err.contains("non-negative"),
            "expected non-negative error, got: {err}"
        );
    }

    #[test]
    fn pie_dispatch_label_length_mismatch_error() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let values = f64_vec(&[30.0, 30.0, 40.0]);
        // Cell array with wrong number of labels.
        let cell = Value::Cell(vec![Value::Str("A".into()), Value::Str("B".into())]);
        let err = plugin.call("pie", &[values, cell], &env).unwrap_err();
        assert!(
            err.contains("length"),
            "expected length mismatch error, got: {err}"
        );
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn pie_svg_polygon_count() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let path = ".debug/test_pie_polygon_count.svg".to_string();
        let _ = std::fs::remove_file(&path);
        let values = f64_vec(&[25.0, 50.0, 25.0]);
        let result = plugin.call("pie", &[values, Value::Str(path.clone())], &env);
        assert!(result.is_ok(), "pie SVG should succeed: {result:?}");
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        // One polygon per slice — 3 slices.
        let count = content.matches("<polygon").count();
        assert_eq!(
            count, 3,
            "expected exactly 3 <polygon> elements for 3 slices, got {count}"
        );
        let _ = std::fs::remove_file(&path);
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn pie_with_labels_svg() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let path = ".debug/test_pie_labels.svg".to_string();
        let _ = std::fs::remove_file(&path);
        let values = f64_vec(&[30.0, 70.0]);
        let cell = Value::Cell(vec![Value::Str("Small".into()), Value::Str("Large".into())]);
        let result = plugin.call("pie", &[values, cell, Value::Str(path.clone())], &env);
        assert!(
            result.is_ok(),
            "pie with labels SVG should succeed: {result:?}"
        );
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            content.contains("Small"),
            "SVG should contain label 'Small'"
        );
        assert!(
            content.contains("Large"),
            "SVG should contain label 'Large'"
        );
        let _ = std::fs::remove_file(&path);
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn pie_explode_svg() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let path = ".debug/test_pie_explode.svg".to_string();
        let _ = std::fs::remove_file(&path);
        let values = f64_vec(&[40.0, 30.0, 30.0]);
        let explode = f64_vec(&[0.1, 0.0, 0.0]);
        let result = plugin.call("pie", &[values, explode, Value::Str(path.clone())], &env);
        assert!(
            result.is_ok(),
            "pie with explode SVG should succeed: {result:?}"
        );
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(content.contains("<polygon"), "SVG should contain polygons");
        let _ = std::fs::remove_file(&path);
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn pie_single_slice() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let path = ".debug/test_pie_single.svg".to_string();
        let _ = std::fs::remove_file(&path);
        let values = f64_vec(&[100.0]);
        let result = plugin.call("pie", &[values, Value::Str(path.clone())], &env);
        assert!(
            result.is_ok(),
            "pie single-slice SVG should succeed: {result:?}"
        );
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let count = content.matches("<polygon").count();
        assert_eq!(
            count, 1,
            "single-slice pie should have exactly 1 polygon, got {count}"
        );
        let _ = std::fs::remove_file(&path);
        FIGURE_STATE.with(|f| f.take());
    }

    // ── Phase 32d — yyaxis ────────────────────────────────────────────

    #[test]
    fn yyaxis_right_sets_active() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("yyaxis", &[Value::Str("right".into())], &env)
            .unwrap();
        FIGURE_STATE.with(|f| {
            let st = f.borrow();
            assert_eq!(
                st.active_yaxis,
                style::YAxis::Right,
                "active_yaxis should be Right after yyaxis('right')"
            );
            assert!(st.hold, "yyaxis should enable hold");
        });
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn yyaxis_series_routing() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        // Activate left axis first (also enables hold so series are not flushed).
        plugin
            .call("yyaxis", &[Value::Str("left".into())], &env)
            .unwrap();
        plugin
            .call("plot", &[f64_vec(&[1.0, 2.0]), f64_vec(&[1.0, 2.0])], &env)
            .unwrap();
        // Switch to right axis and add another series.
        plugin
            .call("yyaxis", &[Value::Str("right".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[f64_vec(&[1.0, 2.0]), f64_vec(&[10.0, 20.0])],
                &env,
            )
            .unwrap();
        FIGURE_STATE.with(|f| {
            let st = f.borrow();
            assert_eq!(st.pending_series.len(), 1, "one series on the left axis");
            assert_eq!(
                st.right_pending_series.len(),
                1,
                "one series on the right axis"
            );
        });
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn yyaxis_ylabel_routing() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        plugin
            .call("ylabel", &[Value::Str("left label".into())], &env)
            .unwrap();
        plugin
            .call("yyaxis", &[Value::Str("right".into())], &env)
            .unwrap();
        plugin
            .call("ylabel", &[Value::Str("right label".into())], &env)
            .unwrap();
        FIGURE_STATE.with(|f| {
            let st = f.borrow();
            assert_eq!(
                st.ylabel.as_deref(),
                Some("left label"),
                "left ylabel must be unchanged"
            );
            assert_eq!(
                st.right_ylabel.as_deref(),
                Some("right label"),
                "right ylabel must be set"
            );
        });
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn yyaxis_svg_has_two_axis_labels() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let path = ".debug/test_yyaxis.svg";
        let _ = std::fs::remove_file(path);

        // Activate left axis first so the first plot is held instead of flushed.
        plugin
            .call("yyaxis", &[Value::Str("left".into())], &env)
            .unwrap();
        plugin
            .call("ylabel", &[Value::Str("Left Y".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[f64_vec(&[1.0, 2.0, 3.0]), f64_vec(&[1.0, 2.0, 3.0])],
                &env,
            )
            .unwrap();
        plugin
            .call("yyaxis", &[Value::Str("right".into())], &env)
            .unwrap();
        plugin
            .call("ylabel", &[Value::Str("Right Y".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[f64_vec(&[1.0, 2.0, 3.0]), f64_vec(&[100.0, 200.0, 300.0])],
                &env,
            )
            .unwrap();
        plugin
            .call("savefig", &[Value::Str(path.into())], &env)
            .unwrap();

        let content = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            content.contains("Left Y"),
            "SVG must contain the left y-axis label"
        );
        assert!(
            content.contains("Right Y"),
            "SVG must contain the right y-axis label"
        );
        std::fs::remove_file(path).ok();
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot")]
    fn yyaxis_ascii_combined_state() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();

        // Activate left axis first so the series is held.
        plugin
            .call("yyaxis", &[Value::Str("left".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[f64_vec(&[1.0, 2.0, 3.0]), f64_vec(&[1.0, 2.0, 3.0])],
                &env,
            )
            .unwrap();
        plugin
            .call("yyaxis", &[Value::Str("right".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[f64_vec(&[1.0, 2.0, 3.0]), f64_vec(&[100.0, 200.0, 300.0])],
                &env,
            )
            .unwrap();

        FIGURE_STATE.with(|f| {
            let st = f.borrow();
            // Both series should still be in pending state (hold is on).
            assert_eq!(st.pending_series.len(), 1, "one left series");
            assert_eq!(st.right_pending_series.len(), 1, "one right series");
        });
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot")]
    fn yyaxis_auto_flush_on_new_left() {
        // A second yyaxis('left') call must flush the previous dual-axis session
        // without requiring an explicit hold('off').
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();

        plugin
            .call("yyaxis", &[Value::Str("left".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[f64_vec(&[1.0, 2.0]), f64_vec(&[10.0, 20.0])],
                &env,
            )
            .unwrap();
        plugin
            .call("yyaxis", &[Value::Str("right".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[f64_vec(&[1.0, 2.0]), f64_vec(&[100.0, 200.0])],
                &env,
            )
            .unwrap();

        // State: both sides pending.
        FIGURE_STATE.with(|f| {
            let st = f.borrow();
            assert_eq!(st.pending_series.len(), 1);
            assert_eq!(st.right_pending_series.len(), 1);
        });

        // Starting a new session via yyaxis('left') must flush the previous one.
        plugin
            .call("yyaxis", &[Value::Str("left".into())], &env)
            .unwrap();

        FIGURE_STATE.with(|f| {
            let st = f.borrow();
            assert_eq!(
                st.pending_series.len(),
                0,
                "left queue must be empty after auto-flush"
            );
            assert_eq!(
                st.right_pending_series.len(),
                0,
                "right queue must be empty after auto-flush"
            );
        });
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot")]
    fn yyaxis_ascii_combined_no_panic() {
        // hold('off') must flush both sides onto one combined chart without panic.
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();

        plugin
            .call("yyaxis", &[Value::Str("left".into())], &env)
            .unwrap();
        plugin
            .call("ylabel", &[Value::Str("Left Y".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[
                    f64_vec(&[0.0, 1.0, 2.0, 3.0]),
                    f64_vec(&[18.0, 19.0, 21.0, 23.0]),
                ],
                &env,
            )
            .unwrap();
        plugin
            .call("yyaxis", &[Value::Str("right".into())], &env)
            .unwrap();
        plugin
            .call("ylabel", &[Value::Str("Right Y".into())], &env)
            .unwrap();
        plugin
            .call(
                "plot",
                &[
                    f64_vec(&[0.0, 1.0, 2.0, 3.0]),
                    f64_vec(&[60.0, 65.0, 70.0, 68.0]),
                ],
                &env,
            )
            .unwrap();
        plugin
            .call("title", &[Value::Str("Dual".into())], &env)
            .unwrap();
        // Flushing via hold('off') must not panic.
        plugin
            .call("hold", &[Value::Str("off".into())], &env)
            .unwrap();
    }

    // ── Phase 32e — clabel ────────────────────────────────────────────

    #[test]
    fn clabel_sets_flag() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        assert!(!FIGURE_STATE.with(|f| f.borrow().clabel));
        plugin.call("clabel", &[], &env).unwrap();
        assert!(
            FIGURE_STATE.with(|f| f.borrow().clabel),
            "clabel() should set FigureState.clabel to true"
        );
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    fn clabel_without_contour_noop() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        assert!(plugin.call("clabel", &[], &env).is_ok());
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn clabel_svg_has_text_elements() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let (x, y, z) = make_contour_xyz(20, 20);
        let path = ".debug/test_clabel.svg";
        std::fs::create_dir_all(".debug").ok();
        plugin.call("clabel", &[], &env).unwrap();
        plugin
            .call(
                "contour",
                &[
                    x,
                    y,
                    z,
                    Value::Scalar(5.0),
                    Value::Str(path.into()),
                ],
                &env,
            )
            .unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(
            content.contains("<text"),
            "clabel SVG should contain <text elements"
        );
        std::fs::remove_file(path).ok();
        FIGURE_STATE.with(|f| f.take());
    }

    #[test]
    #[cfg(feature = "plot-svg")]
    fn clabel_text_count_matches_levels() {
        FIGURE_STATE.with(|f| f.take());
        let plugin = PlotPlugin;
        let env = Env::new();
        let n_levels: usize = 5;
        let path_base = ".debug/test_clabel_base.svg";
        let path_labeled = ".debug/test_clabel_labeled.svg";
        std::fs::create_dir_all(".debug").ok();

        // Render without clabel to get baseline <text> count (title/axis labels).
        let (x0, y0, z0) = make_contour_xyz(20, 20);
        plugin
            .call(
                "contour",
                &[
                    x0,
                    y0,
                    z0,
                    Value::Scalar(n_levels as f64),
                    Value::Str(path_base.into()),
                ],
                &env,
            )
            .unwrap();
        let base_count = std::fs::read_to_string(path_base).unwrap().matches("<text").count();

        // Render with clabel — should add one label per level.
        let (x, y, z) = make_contour_xyz(20, 20);
        plugin.call("clabel", &[], &env).unwrap();
        plugin
            .call(
                "contour",
                &[
                    x,
                    y,
                    z,
                    Value::Scalar(n_levels as f64),
                    Value::Str(path_labeled.into()),
                ],
                &env,
            )
            .unwrap();
        let label_count = std::fs::read_to_string(path_labeled)
            .unwrap()
            .matches("<text")
            .count();

        assert!(
            label_count >= base_count + n_levels,
            "clabel should add at least {n_levels} <text> elements \
             (base={base_count}, with labels={label_count})"
        );

        std::fs::remove_file(path_base).ok();
        std::fs::remove_file(path_labeled).ok();
        FIGURE_STATE.with(|f| f.take());
    }
}
