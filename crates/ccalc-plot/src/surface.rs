//! surf and mesh 3D surface rendering (ASCII and SVG/PNG).

#[cfg(feature = "plot-svg")]
use plotters::prelude::*;
#[cfg(feature = "plot-svg")]
use plotters::series::SurfaceSeries;

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

/// Writes a `surf` (filled color surface) to an SVG or PNG file.
///
/// `x_vals` and `y_vals` are the unique coordinate vectors (first row of X,
/// first column of Y from the meshgrid call).  `z` is row-major Z data with
/// `nrows × ncols` elements.
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
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, false)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, false)
    } else {
        Err(format!("surf: unsupported format '{path}'"))
    }
}

/// Writes a `mesh` (wireframe surface) to an SVG or PNG file.
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
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, true)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
        draw_surface(x_vals, y_vals, z, nrows, ncols, &state, root, true)
    } else {
        Err(format!("mesh: unsupported format '{path}'"))
    }
}

/// Core 3D surface drawing via `SurfaceSeries::xoy`.
///
/// `wireframe = true` → thin colored border strokes only (mesh style).
/// `wireframe = false` → filled quads colored by the active colormap (surf style).
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

    // Build owned copies for the closure.
    let xv = x_vals.to_vec();
    let yv = y_vals.to_vec();
    let zv = z.to_vec();
    let nc = ncols;
    let nr = nrows;

    // Z lookup: find nearest grid indices for (xi, yi) and return Z value.
    let z_lookup = move |xi: f64, yi: f64| -> f64 {
        let col = xv
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| ((*a - xi).abs()).partial_cmp(&((*b - xi).abs())).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
            .min(nc.saturating_sub(1));
        let row = yv
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| ((*a - yi).abs()).partial_cmp(&((*b - yi).abs())).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
            .min(nr.saturating_sub(1));
        zv[row * nc + col]
    };

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(30)
        // plotters 3D: x=depth (Y axis in math), y=height (Z), z=horizontal (X).
        .build_cartesian_3d(x_lo..x_hi, z_lo..z_hi, y_lo..y_hi)
        .map_err(|e| e.to_string())?;

    chart.configure_axes().draw().map_err(|e| e.to_string())?;

    chart
        .draw_series(
            SurfaceSeries::xoy(
                x_vals.iter().copied(),
                y_vals.iter().copied(),
                z_lookup,
            )
            .style_func(&|&v| {
                let t = ((v - z_lo) / z_range).clamp(0.0, 1.0);
                let (r, g, b) = apply_colormap(t, cmap);
                let color = RGBColor(r, g, b);
                if wireframe {
                    color.stroke_width(1)
                } else {
                    color.filled()
                }
            }),
        )
        .map_err(|e| e.to_string())?;

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}
