//! Plot plugin for ccalc — Phase 29a.
//!
//! Provides `plot`, `scatter`, `xlabel`, `ylabel`, `title` (and stub entries
//! for `bar`, `stem`). Rendering requires the `plot` or `plot-svg` feature
//! flags; annotation-only calls work in every build configuration.
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

pub mod dispatch;
pub mod proj3d;

#[cfg(feature = "plot")]
mod ascii;

use std::cell::RefCell;

use ccalc_engine::env::{Env, Value};
use ccalc_engine::plugin::Plugin;

use dispatch::{extract_file_arg, extract_vector};

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
}

thread_local! {
    static FIGURE_STATE: RefCell<FigureState> =
        RefCell::new(FigureState::default());
}

// ── Exported names ─────────────────────────────────────────────────────────

const EXPORTED: &[&str] = &[
    "plot", "scatter", "bar", "stem", "xlabel", "ylabel", "title",
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
            // ── Annotation setters ─────────────────────────────────────
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

            // ── Render calls ───────────────────────────────────────────
            "plot" | "scatter" => {
                let (data_args, path) = extract_file_arg(args);
                let state = FIGURE_STATE.with(|f| f.take());
                render_ascii_or_file(name, &data_args, path.as_deref(), state)
            }

            // ── Stubs (Phase 29c) ──────────────────────────────────────
            "bar" | "stem" => Err(format!("{name}: not yet implemented — coming in Phase 29c")),

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
        // No path or explicit 'ascii' → terminal rendering
        None | Some("ascii") => render_ascii(name, data_args, state),
        // SVG / PNG → file export (Phase 29b)
        Some(p) if p.ends_with(".svg") || p.ends_with(".png") => Err(format!(
            "{name}: SVG/PNG export requires the 'plot-svg' feature (Phase 29b)"
        )),
        Some(p) => Err(format!("{name}: unknown output target '{p}'")),
    }
}

#[cfg(feature = "plot")]
fn render_ascii(name: &str, data_args: &[Value], state: FigureState) -> Result<Value, String> {
    let (x, y) = extract_xy(name, data_args)?;
    match name {
        "plot" => ascii::render_line(&x, &y, state),
        "scatter" => ascii::render_scatter(&x, &y, state),
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

// ── Argument helpers ───────────────────────────────────────────────────────

fn require_string(name: &str, args: &[Value]) -> Result<String, String> {
    match args {
        [Value::Str(s)] | [Value::StringObj(s)] => Ok(s.clone()),
        [_] => Err(format!("{name}: argument must be a string")),
        _ => Err(format!("{name}: expected exactly one string argument")),
    }
}

#[cfg_attr(not(feature = "plot"), allow(dead_code))]
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
    fn test_bar_stub_error() {
        let plugin = PlotPlugin;
        let env = Env::new();
        let result = plugin.call("bar", &[Value::Scalar(1.0)], &env);
        assert!(result.is_err());
    }

    #[test]
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
}
