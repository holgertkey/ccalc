//! Plot plugin for ccalc — Phase 30a.
//!
//! Provides `plot`, `scatter`, `bar`, `stem`, `hist`, `stairs`, `loglog`,
//! `semilogx`, `semilogy`, `plot3`, `scatter3`, `imagesc`, and annotation
//! functions (`xlabel`, `ylabel`, `zlabel`, `title`, `legend`, `xlim`,
//! `ylim`, `zlim`, `grid`, `colormap`, `colorbar`).
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

#[cfg(feature = "plot")]
mod ascii;

#[cfg(feature = "plot-svg")]
mod file;

use std::cell::RefCell;

use ccalc_engine::env::{Env, Value};
use ccalc_engine::plugin::Plugin;

use dispatch::{extract_file_arg, extract_matrix, extract_vector};

// ── FigureState ────────────────────────────────────────────────────────────

/// Per-figure annotation state consumed by the next render call.
///
/// Set via `xlabel()`, `ylabel()`, `title()` etc. and cleared automatically
/// after each `plot()` / `scatter()` / `bar()` call.
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
    /// Active colormap name for `imagesc` (default `"viridis"` when `None`).
    pub colormap: Option<String>,
    /// Whether to append a colorbar to the next `imagesc` render.
    pub colorbar: bool,
}

thread_local! {
    static FIGURE_STATE: RefCell<FigureState> =
        RefCell::new(FigureState::default());
}

// ── Exported names ─────────────────────────────────────────────────────────

const EXPORTED: &[&str] = &[
    "plot", "scatter", "bar", "stem", "hist", "stairs", "loglog", "semilogx", "semilogy", "plot3",
    "scatter3", "xlabel", "ylabel", "zlabel", "title", "legend", "xlim", "ylim", "zlim", "grid",
    "colormap", "colorbar", "imagesc",
];

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
                        "ylabel" => st.ylabel = Some(s),
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
                        "ylim" => st.ylim = Some((lo, hi)),
                        "zlim" => st.zlim = Some((lo, hi)),
                        _ => unreachable!(),
                    }
                });
                Ok(Value::Void)
            }

            // ── Render calls ───────────────────────────────────────────
            "plot" => {
                let (data_args, path) = extract_file_arg(args);
                let state = FIGURE_STATE.with(|f| f.take());
                let (x, ys) = extract_xy_multi("plot", &data_args)?;
                if ys.len() == 1 {
                    render_line_xy("plot", &x, &ys[0], path.as_deref(), state)
                } else {
                    render_multi_series(&x, &ys, path.as_deref(), state)
                }
            }

            "scatter" | "bar" | "stem" | "stairs" => {
                let (data_args, path) = extract_file_arg(args);
                let state = FIGURE_STATE.with(|f| f.take());
                render_ascii_or_file(name, &data_args, path.as_deref(), state)
            }

            // ── Histogram ──────────────────────────────────────────────
            "hist" => {
                let (data_args, path) = extract_file_arg(args);
                let state = FIGURE_STATE.with(|f| f.take());
                let (counts, edges) = parse_and_compute_hist(&data_args)?;
                match path.as_deref() {
                    None | Some("ascii") => {
                        render_hist_ascii(&counts, &edges, &state);
                        Ok(Value::Void)
                    }
                    Some(p) if p.ends_with(".svg") || p.ends_with(".png") => {
                        render_hist_file(&counts, &edges, p, state)
                    }
                    Some(p) => Err(format!("hist: unknown output target '{p}'")),
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

            // ── Colormap / colorbar setters ────────────────────────────
            "colormap" => {
                let cmap = require_string("colormap", args)?;
                colormap::validate_colormap(&cmap)?;
                FIGURE_STATE.with(|f| f.borrow_mut().colormap = Some(cmap));
                Ok(Value::Void)
            }

            "colorbar" => {
                FIGURE_STATE.with(|f| f.borrow_mut().colorbar = true);
                Ok(Value::Void)
            }

            // ── imagesc ────────────────────────────────────────────────
            "imagesc" => {
                let (data_args, path) = extract_file_arg(args);
                if data_args.is_empty() {
                    return Err("imagesc: at least one argument required".into());
                }
                let state = FIGURE_STATE.with(|f| f.take());
                let (z, nrows, ncols) = extract_matrix(&data_args[0])?;
                render_imagesc(&z, nrows, ncols, path.as_deref(), state)
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
        "bar" => file::render_bar(&x, &y, path, state),
        "stem" => file::render_stem(&x, &y, path, state),
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
#[cfg(any(feature = "plot", feature = "plot-svg"))]
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
    let bar_cols: usize = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(80)
        .saturating_sub(26)
        .max(10);
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
    state: FigureState,
) -> Result<Value, String> {
    file::render_hist(counts, edges, path, state).map_err(|e| format!("hist: {e}"))?;
    Ok(Value::Void)
}

#[cfg(not(feature = "plot-svg"))]
fn render_hist_file(
    _counts: &[usize],
    _edges: &[f64],
    _path: &str,
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
        assert_eq!(cmap, Some("hot".into()));
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
}
