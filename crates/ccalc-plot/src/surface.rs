//! surf and mesh 3D surface rendering (ASCII and SVG/PNG).

#[cfg(feature = "plot-svg")]
use plotters::prelude::*;
#[cfg(feature = "plot-svg")]
use plotters::series::LineSeries;

#[cfg(any(feature = "plot", feature = "plot-svg"))]
use crate::FigureState;
#[cfg(any(feature = "plot", feature = "plot-svg"))]
use crate::colormap::apply_colormap;
#[cfg(any(feature = "plot", feature = "plot-svg"))]
use crate::colormap::data_range;

// ── ASCII renderers ────────────────────────────────────────────────────────

/// Renders `surf` or `mesh` as a side-view ASCII elevation map.
///
/// For each column index (X position), the maximum Z value across all rows is
/// projected as a vertical bar.  Both `surf` and `mesh` use the same ASCII
/// representation.
///
/// `x_vals` are the unique x coordinates (first row of the X meshgrid matrix).
/// `z` is the full Z matrix in row-major order with `nrows × ncols` shape.
#[cfg(feature = "plot")]
pub fn render_surf_ascii(
    x_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    state: &FigureState,
) {
    const CHART_HEIGHT: usize = 20;

    if nrows == 0 || ncols == 0 {
        return;
    }

    // Max Z per column (across all rows).
    let col_max: Vec<f64> = (0..ncols)
        .map(|c| {
            (0..nrows)
                .map(|r| z[r * ncols + c])
                .filter(|v| v.is_finite())
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .collect();

    let (z_min, z_max) = data_range(z);
    let z_range = z_max - z_min;

    if let Some(t) = &state.title {
        println!("{t}");
    }

    // Print character grid from top (row CHART_HEIGHT-1 = highest Z) down.
    for row in (0..CHART_HEIGHT).rev() {
        let threshold = z_min + z_range * (row as f64 / CHART_HEIGHT as f64);
        let line: String = col_max
            .iter()
            .map(|&v| if v >= threshold { '#' } else { ' ' })
            .collect();
        println!("{line}");
    }

    // X-axis tick: first and last x value.
    if !x_vals.is_empty() {
        let first = x_vals[0];
        let last = x_vals[x_vals.len() - 1];
        let width = ncols.saturating_sub(16);
        println!("{first:<8.4}{:>width$}{last:>8.4}", "");
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
}

// ── SVG/PNG renderers ──────────────────────────────────────────────────────

/// Writes a `surf` (colored surface) to an SVG or PNG file.
///
/// `x_vals` and `y_vals` are the unique coordinate vectors (first row of X,
/// first column of Y from the meshgrid call).  `z` is row-major Z data with
/// `nrows × ncols` elements.
///
/// The surface is drawn as a dense colored grid: row lines and column lines
/// are both rendered, each colored by the local Z value.
#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
pub fn render_surf_file(
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, false)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, false)
    } else {
        Err(format!("surf: unsupported format '{path}'"))
    }
}

/// Writes a `mesh` (wireframe surface) to an SVG or PNG file.
///
/// Like `surf` but draws only row lines (no column fill lines), giving a
/// sparser wireframe appearance.
#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
pub fn render_mesh_file(
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    let canvas = state.canvas_size();
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, canvas).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, true)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, canvas).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, true)
    } else {
        Err(format!("mesh: unsupported format '{path}'"))
    }
}

/// Core 3D surface drawing using colored `LineSeries` grid lines.
///
/// Axis mapping — chart `(X, Y, Z)` = our `(X, Z_height, Y_depth)`:
/// - chart first dim  (X)      = our X values (horizontal, left–right)
/// - chart second dim (Y, up)  = our Z values (height, color axis)
/// - chart third dim  (Z, back)= our Y values (spatial depth, into the page)
///
/// This keeps our Z as the visual height axis and matches the standard
/// `surf(X, Y, Z)` convention.
///
/// `wireframe = true`  → draw only row lines (sparse, mesh style).
/// `wireframe = false` → draw both row and column lines (denser, surf style).
#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
fn draw_surface<DB: DrawingBackend>(
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
    wireframe: bool,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    if nrows == 0 || ncols == 0 {
        return root.present().map_err(|e| e.to_string());
    }

    let (z_min, z_max) = data_range(z);

    let x_lo = x_vals.iter().copied().fold(f64::INFINITY, f64::min);
    let x_hi = x_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_lo = y_vals.iter().copied().fold(f64::INFINITY, f64::min);
    let y_hi = y_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let (x_lo, x_hi) = state.xlim.unwrap_or((x_lo, x_hi));
    let (y_lo, y_hi) = state.ylim.unwrap_or((y_lo, y_hi));
    let (z_lo, z_hi) = state.zlim.unwrap_or((z_min, z_max));

    let title = state.title.as_deref().unwrap_or("");
    let cmap = state.colormap.as_deref().unwrap_or("viridis");
    let z_range = (z_hi - z_lo).max(f64::EPSILON);

    // Chart: X = our X, Y (height) = our Z, Z (depth) = our Y.
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(30)
        .build_cartesian_3d(x_lo..x_hi, z_lo..z_hi, y_lo..y_hi)
        .map_err(|e| e.to_string())?;

    chart.configure_axes().draw().map_err(|e| e.to_string())?;

    // Row lines: fixed Y (depth), varying X (horizontal) — colored by row mean Z.
    for r in 0..nrows {
        let y_val = y_vals[r];
        let points: Vec<(f64, f64, f64)> = (0..ncols)
            .map(|c| (x_vals[c], z[r * ncols + c], y_val))
            .collect();
        let z_avg = z_row_avg(z, r, ncols);
        let t = ((z_avg - z_lo) / z_range).clamp(0.0, 1.0);
        let (rr, gg, bb) = apply_colormap(t, cmap);
        chart
            .draw_series(LineSeries::new(points, RGBColor(rr, gg, bb)))
            .map_err(|e| e.to_string())?;
    }

    // Column lines: fixed X (horizontal), varying Y (depth) — colored by col mean Z.
    // Drawn for surf only; mesh shows just the row lines as a wireframe.
    if !wireframe {
        for c in 0..ncols {
            let x_val = x_vals[c];
            let points: Vec<(f64, f64, f64)> = (0..nrows)
                .map(|r| (x_val, z[r * ncols + c], y_vals[r]))
                .collect();
            let z_avg = z_col_avg(z, c, nrows, ncols);
            let t = ((z_avg - z_lo) / z_range).clamp(0.0, 1.0);
            let (rr, gg, bb) = apply_colormap(t, cmap);
            chart
                .draw_series(LineSeries::new(points, RGBColor(rr, gg, bb)))
                .map_err(|e| e.to_string())?;
        }
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

/// Mean Z value across row `r`.
#[cfg(feature = "plot-svg")]
fn z_row_avg(z: &[f64], r: usize, ncols: usize) -> f64 {
    let sum: f64 = (0..ncols).map(|c| z[r * ncols + c]).sum();
    sum / ncols.max(1) as f64
}

/// Mean Z value down column `c`.
#[cfg(feature = "plot-svg")]
fn z_col_avg(z: &[f64], c: usize, nrows: usize, ncols: usize) -> f64 {
    let sum: f64 = (0..nrows).map(|r| z[r * ncols + c]).sum();
    sum / nrows.max(1) as f64
}
