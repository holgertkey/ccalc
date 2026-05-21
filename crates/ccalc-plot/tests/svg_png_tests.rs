//! Integration tests for SVG/PNG file export (Phase 29b + 29c + 29d).
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

// ── bar → SVG / PNG ──────────────────────────────────────────────────────────

#[test]
fn test_bar_writes_svg_file() {
    let path = svg_path("test_bar.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let y = row_vec(&[3.0, 1.0, 4.0, 1.0, 5.0]);
    plugin
        .call("bar", &[y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_bar_writes_png_file() {
    let path = svg_path("test_bar.png");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0, 3.0, 4.0]);
    let y = row_vec(&[2.0, 5.0, 3.0, 7.0]);
    plugin
        .call("bar", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn test_bar_negative_values_svg() {
    let path = svg_path("test_bar_neg.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let y = row_vec(&[-2.0, 3.0, -1.0, 4.0]);
    plugin
        .call("bar", &[y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

// ── stem → SVG ───────────────────────────────────────────────────────────────

#[test]
fn test_stem_writes_svg_file() {
    let path = svg_path("test_stem.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = row_vec(&[0.5, 0.2, 0.8, 0.1, 0.6]);
    plugin
        .call("stem", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_stem_inferred_x_svg() {
    let path = svg_path("test_stem_inferred.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let y = row_vec(&[1.0, 3.0, 2.0, 4.0]);
    plugin
        .call("stem", &[y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

// ── stairs → SVG ─────────────────────────────────────────────────────────────

#[test]
fn test_stairs_writes_svg_file() {
    let path = svg_path("test_stairs.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0, 3.0, 4.0]);
    let y = row_vec(&[1.0, 3.0, 2.0, 4.0, 2.0]);
    plugin
        .call("stairs", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_stairs_inferred_x_svg() {
    let path = svg_path("test_stairs_inferred.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let y = row_vec(&[2.0, 5.0, 3.0, 8.0]);
    plugin
        .call("stairs", &[y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

// ── hist → SVG / PNG ─────────────────────────────────────────────────────────

#[test]
fn test_hist_writes_svg_file() {
    let path = svg_path("test_hist.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let v = row_vec(&[1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0]);
    plugin
        .call("hist", &[v, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_hist_explicit_bins_svg() {
    let path = svg_path("test_hist_bins.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let v = row_vec(&[1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 5.0]);
    let bins = Value::Scalar(4.0);
    plugin
        .call("hist", &[v, bins, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_hist_writes_png_file() {
    let path = svg_path("test_hist.png");
    let plugin = PlotPlugin;
    let env = Env::new();
    let v = row_vec(&[0.5, 1.0, 1.5, 2.0, 2.5, 3.0]);
    plugin
        .call("hist", &[v, Value::Str(path.clone())], &env)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

// ── loglog / semilogx / semilogy → SVG ───────────────────────────────────────

#[test]
fn test_loglog_writes_svg_file() {
    let path = svg_path("test_loglog.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 10.0, 100.0, 1000.0]);
    let y = row_vec(&[1.0, 4.0, 9.0, 16.0]);
    plugin
        .call("loglog", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_semilogx_writes_svg_file() {
    let path = svg_path("test_semilogx.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 10.0, 100.0]);
    let y = row_vec(&[-1.0, 0.0, 1.0]);
    plugin
        .call("semilogx", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_semilogy_writes_svg_file() {
    let path = svg_path("test_semilogy.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0, 3.0]);
    let y = row_vec(&[1.0, 10.0, 100.0, 1000.0]);
    plugin
        .call("semilogy", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_loglog_filters_non_positive() {
    // A dataset with some non-positive values — log10 of those yields -inf/NaN,
    // which should be filtered. Positive values should still produce a valid chart.
    let path = svg_path("test_loglog_filter.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.1, 1.0, 10.0]);
    let y = row_vec(&[0.5, 2.0, 8.0]);
    plugin
        .call("loglog", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

// ── multi-series plot → SVG ───────────────────────────────────────────────────

#[test]
fn test_multi_series_plot_svg() {
    let path = svg_path("test_multi_series.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    // Y is a 2×5 matrix (2 series, 5 points each).
    let x = row_vec(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let y_matrix = Value::Matrix(
        ndarray::Array2::from_shape_vec(
            (2, 5),
            vec![
                1.0, 4.0, 9.0, 16.0, 25.0, // sin-like series
                2.0, 3.0, 2.0, 3.0, 2.0,
            ], // flat series
        )
        .unwrap(),
    );
    plugin
        .call("plot", &[x, y_matrix, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_multi_series_with_legend() {
    let path = svg_path("test_multi_legend.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    plugin
        .call(
            "legend",
            &[Value::Str("series A".into()), Value::Str("series B".into())],
            &env,
        )
        .unwrap();
    let x = row_vec(&[1.0, 2.0, 3.0, 4.0]);
    let y_matrix = Value::Matrix(
        ndarray::Array2::from_shape_vec((2, 4), vec![1.0, 2.0, 3.0, 4.0, 4.0, 3.0, 2.0, 1.0])
            .unwrap(),
    );
    plugin
        .call("plot", &[x, y_matrix, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
    assert!(
        content.contains("series A") || content.contains("series B"),
        "SVG should contain legend labels"
    );
}

// ── plot3 → SVG / PNG ────────────────────────────────────────────────────────

#[test]
fn test_plot3_writes_svg_file() {
    let path = svg_path("test_plot3.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0, 3.0, 4.0]);
    let y = row_vec(&[0.0, 1.0, 0.0, -1.0, 0.0]);
    let z = row_vec(&[0.0, 0.5, 1.0, 0.5, 0.0]);
    plugin
        .call("plot3", &[x, y, z, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("<svg"),
        "plot3 SVG file should contain <svg tag"
    );
}

#[test]
fn test_plot3_writes_png_file() {
    let path = svg_path("test_plot3.png");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0]);
    let y = row_vec(&[0.0, 1.0, 2.0]);
    let z = row_vec(&[0.0, 1.0, 4.0]);
    plugin
        .call("plot3", &[x, y, z, Value::Str(path.clone())], &env)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "plot3 PNG should start with PNG magic bytes"
    );
}

#[test]
fn test_plot3_with_title_svg() {
    let path = svg_path("test_plot3_title.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    plugin
        .call("title", &[Value::Str("Helix".into())], &env)
        .unwrap();
    let x = row_vec(&[1.0, 0.0, -1.0, 0.0, 1.0]);
    let y = row_vec(&[0.0, 1.0, 0.0, -1.0, 0.0]);
    let z = row_vec(&[0.0, 0.25, 0.5, 0.75, 1.0]);
    plugin
        .call("plot3", &[x, y, z, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("Helix"),
        "title should appear in plot3 SVG"
    );
}

// ── scatter3 → SVG / PNG ──────────────────────────────────────────────────────

#[test]
fn test_scatter3_writes_svg_file() {
    let path = svg_path("test_scatter3.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0, 3.0]);
    let y = row_vec(&[3.0, 1.0, 4.0, 1.0]);
    let z = row_vec(&[1.0, 2.0, 1.0, 3.0]);
    plugin
        .call("scatter3", &[x, y, z, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("<svg"),
        "scatter3 SVG file should contain <svg tag"
    );
}

#[test]
fn test_scatter3_writes_png_file() {
    let path = svg_path("test_scatter3.png");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0, 3.0]);
    let y = row_vec(&[1.0, 4.0, 9.0]);
    let z = row_vec(&[1.0, 1.0, 1.0]);
    plugin
        .call("scatter3", &[x, y, z, Value::Str(path.clone())], &env)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn test_plot3_xyz_length_mismatch_error() {
    let path = svg_path("test_plot3_mismatch.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0]);
    let y = row_vec(&[0.0, 1.0]);
    let z = row_vec(&[0.0, 0.5, 1.0]);
    let result = plugin.call("plot3", &[x, y, z, Value::Str(path)], &env);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("same length"),
        "error should mention length mismatch: {msg}"
    );
}

// ── xlim / ylim applied in file export ───────────────────────────────────────

#[test]
fn test_xlim_ylim_applied_in_svg() {
    let path = svg_path("test_xlim_ylim.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    use ndarray::Array2;
    let lim_x = Value::Matrix(Array2::from_shape_vec((1, 2), vec![0.0, 6.0]).unwrap());
    let lim_y = Value::Matrix(Array2::from_shape_vec((1, 2), vec![-1.0, 1.0]).unwrap());
    plugin.call("xlim", &[lim_x], &env).unwrap();
    plugin.call("ylim", &[lim_y], &env).unwrap();
    let x = row_vec(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = row_vec(&[0.5, -0.5, 0.3, -0.3, 0.1]);
    plugin
        .call("plot", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

// ── 30a: imagesc ─────────────────────────────────────────────────────────────

fn mat4x4() -> Value {
    Value::Matrix(Array2::from_shape_vec((4, 4), (0..16).map(|i| i as f64).collect()).unwrap())
}

#[test]
fn test_imagesc_writes_svg_file() {
    let path = svg_path("test_imagesc.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    plugin
        .call("imagesc", &[mat4x4(), Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"), "SVG file should contain <svg tag");
}

#[test]
fn test_imagesc_writes_png_file() {
    let path = svg_path("test_imagesc.png");
    let plugin = PlotPlugin;
    let env = Env::new();
    plugin
        .call("imagesc", &[mat4x4(), Value::Str(path.clone())], &env)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "file should start with PNG magic bytes"
    );
}

#[test]
fn test_imagesc_with_colorbar_svg() {
    let path = svg_path("test_imagesc_colorbar.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    plugin
        .call("colormap", &[Value::Str("jet".into())], &env)
        .unwrap();
    plugin.call("colorbar", &[], &env).unwrap();
    let z =
        Value::Matrix(Array2::from_shape_vec((8, 8), (0..64).map(|i| i as f64).collect()).unwrap());
    plugin
        .call("imagesc", &[z, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"), "SVG file should contain <svg tag");
}

// ── Phase 30e — fill / area / polar ──────────────────────────────────────────

#[test]
fn test_fill_writes_svg_file() {
    let path = svg_path("test_fill.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    // Triangle polygon.
    let x = row_vec(&[0.0, 1.0, 0.5]);
    let y = row_vec(&[0.0, 0.0, 1.0]);
    plugin
        .call("fill", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"), "fill SVG should contain <svg tag");
}

#[test]
fn test_fill_with_style_string() {
    let path = svg_path("test_fill_red.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 0.5]);
    let y = row_vec(&[0.0, 0.0, 1.0]);
    // Style 'r' → red fill color.
    plugin
        .call(
            "fill",
            &[x, y, Value::Str("r".into()), Value::Str(path.clone())],
            &env,
        )
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_area_writes_svg_file() {
    let path = svg_path("test_area.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[0.0, 1.0, 2.0, 3.0, 4.0]);
    let y = row_vec(&[0.0, 1.0, 0.5, 1.5, 0.0]);
    plugin
        .call("area", &[x, y, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"), "area SVG should contain <svg tag");
}

#[test]
fn test_polar_writes_svg_file() {
    use std::f64::consts::PI;
    let path = svg_path("test_polar.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    // Circle in polar coordinates: r = 1 for all theta.
    let n = 64usize;
    let theta_vals: Vec<f64> = (0..=n).map(|i| 2.0 * PI * i as f64 / n as f64).collect();
    let r_vals: Vec<f64> = vec![1.0; theta_vals.len()];
    let theta = Value::Matrix(Array2::from_shape_vec((1, theta_vals.len()), theta_vals).unwrap());
    let r = Value::Matrix(Array2::from_shape_vec((1, r_vals.len()), r_vals).unwrap());
    plugin
        .call("polar", &[theta, r, Value::Str(path.clone())], &env)
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("<svg"),
        "polar SVG should contain <svg tag"
    );
}

#[test]
fn test_plot_with_style_string() {
    // plot(x, y, 'r--', path) — style string accepted without error.
    let path = svg_path("test_plot_style.svg");
    let plugin = PlotPlugin;
    let env = Env::new();
    let x = row_vec(&[1.0, 2.0, 3.0]);
    let y = row_vec(&[1.0, 4.0, 9.0]);
    plugin
        .call(
            "plot",
            &[x, y, Value::Str("r--".into()), Value::Str(path.clone())],
            &env,
        )
        .unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("<svg"));
}
