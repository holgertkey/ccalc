//! Integration tests for SVG/PNG file export (Phase 29b).
//! Run with: cargo test -p ccalc-plot --features plot-svg

#![cfg(feature = "plot-svg")]

use ccalc_engine::env::{Env, Value};
use ccalc_engine::plugin::Plugin;
use ccalc_plot::PlotPlugin;
use ndarray::Array2;

fn row_vec(vals: &[f64]) -> Value {
    Value::Matrix(Array2::from_shape_vec((1, vals.len()), vals.to_vec()).unwrap())
}

fn svg_path(name: &str) -> String {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.debug/TESTS");
    format!("{dir}/{name}")
}

// ── plot → SVG ───────────────────────────────────────────────────────────────

#[test]
fn test_plot_writes_svg_file() {
    let path = svg_path("test_plot.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = row_vec(&[1.0, 4.0, 9.0, 16.0, 25.0]);
    let file_arg = Value::Str(path.clone());
    plugin.call("plot", &[x, y, file_arg], &env).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"), "SVG file should contain <svg tag");
}

#[test]
fn test_plot_svg_inferred_x() {
    let path = svg_path("test_plot_inferred_x.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let y = row_vec(&[1.0, 2.0, 3.0]);
    let file_arg = Value::Str(path.clone());
    plugin.call("plot", &[y, file_arg], &env).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

// ── plot → PNG ───────────────────────────────────────────────────────────────

#[test]
fn test_plot_writes_png_file() {
    let path = svg_path("test_plot.png");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0, 3.0]);
    let y = row_vec(&[0.0, 1.0, 4.0, 9.0]);
    let file_arg = Value::Str(path.clone());
    plugin.call("plot", &[x, y, file_arg], &env).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "file should start with PNG magic bytes"
    );
}

// ── scatter → SVG ────────────────────────────────────────────────────────────

#[test]
fn test_scatter_writes_svg_file() {
    let path = svg_path("test_scatter.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0, 3.0, 4.0]);
    let y = row_vec(&[2.0, 1.0, 4.0, 3.0]);
    let file_arg = Value::Str(path.clone());
    plugin.call("scatter", &[x, y, file_arg], &env).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

// ── scatter → PNG ────────────────────────────────────────────────────────────

#[test]
fn test_scatter_writes_png_file() {
    let path = svg_path("test_scatter.png");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0, 3.0]);
    let y = row_vec(&[3.0, 1.0, 2.0]);
    let file_arg = Value::Str(path.clone());
    plugin.call("scatter", &[x, y, file_arg], &env).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

// ── FigureState labels are embedded in SVG ───────────────────────────────────

#[test]
fn test_labels_and_title_in_svg() {
    let path = svg_path("test_labels.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    plugin
        .call("title", &[Value::Str("My Chart".into())], &env)
        .unwrap();
    plugin
        .call("xlabel", &[Value::Str("time (s)".into())], &env)
        .unwrap();
    plugin
        .call("ylabel", &[Value::Str("amplitude".into())], &env)
        .unwrap();
    let x = row_vec(&[1.0, 2.0, 3.0]);
    let y = row_vec(&[1.0, 0.0, 1.0]);
    plugin
        .call("plot", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("My Chart"), "title should appear in SVG");
    assert!(content.contains("time (s)"), "xlabel should appear in SVG");
    assert!(content.contains("amplitude"), "ylabel should appear in SVG");
}

// ── FigureState is cleared after render (verified by second render) ───────────
//
// After the first plot the title should be gone, so the second SVG should not
// contain it.  We compare file contents to confirm the state was consumed.

#[test]
fn test_figure_state_cleared_after_file_render() {
    let path1 = svg_path("test_state_clear1.svg");
    let path2 = svg_path("test_state_clear2.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    // Set a unique title, render once — title consumed.
    plugin
        .call("title", &[Value::Str("UniqueTitle42".into())], &env)
        .unwrap();
    let y = row_vec(&[1.0, 2.0]);
    plugin
        .call("plot", &[y.clone(), Value::Str(path1.clone())], &env)
        .unwrap();
    // Second render without setting title — should not contain it.
    plugin
        .call("plot", &[y, Value::Str(path2.clone())], &env)
        .unwrap();
    let content2 = std::fs::read_to_string(&path2).unwrap();
    assert!(
        !content2.contains("UniqueTitle42"),
        "title from first call should not bleed into second render"
    );
}

// ── error: mismatch length ────────────────────────────────────────────────────

#[test]
fn test_length_mismatch_error() {
    let path = svg_path("test_mismatch.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0]);
    let y = row_vec(&[1.0, 2.0, 3.0]);
    let result = plugin.call("plot", &[x, y, Value::Str(path)], &env);
    assert!(result.is_err());
}

// ── single-point edge case ────────────────────────────────────────────────────

#[test]
fn test_single_point_svg() {
    let path = svg_path("test_single_point.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let y = Value::Scalar(42.0);
    plugin
        .call("plot", &[y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}
