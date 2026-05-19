//! contour and contourf 2D contour rendering (ASCII and SVG/PNG).
//!
//! Uses the marching squares algorithm to compute isolines.  The saddle-point
//! cases (case 5 and 10) are resolved with the simple split convention: the two
//! above-level corners are treated as separate islands rather than bridged.

#[cfg(feature = "plot-svg")]
use plotters::prelude::*;
#[cfg(feature = "plot-svg")]
use plotters::series::LineSeries;

#[cfg(any(feature = "plot", feature = "plot-svg"))]
use crate::FigureState;
#[cfg(any(feature = "plot", feature = "plot-svg"))]
use crate::colormap::apply_colormap;

// ── Level computation ──────────────────────────────────────────────────────

#[cfg_attr(not(any(feature = "plot", feature = "plot-svg")), allow(dead_code))]
/// Computes `n` evenly-spaced contour levels strictly inside `[z_min, z_max]`.
///
/// Levels are placed at positions `z_min + (z_max - z_min) * k / (n + 1)`
/// for `k = 1..=n`, so the returned values never equal the data extrema.
pub(crate) fn compute_levels(z_min: f64, z_max: f64, n: usize) -> Vec<f64> {
    if n == 0 {
        return vec![];
    }
    let range = z_max - z_min;
    (1..=n)
        .map(|k| z_min + range * k as f64 / (n + 1) as f64)
        .collect()
}

// ── Marching squares ───────────────────────────────────────────────────────

#[cfg_attr(not(any(feature = "plot", feature = "plot-svg")), allow(dead_code))]
type Segment = ((f64, f64), (f64, f64));

#[cfg_attr(not(any(feature = "plot", feature = "plot-svg")), allow(dead_code))]
/// Computes all contour line segments for a single isoline at `level`.
///
/// Iterates over every 2×2 cell of the grid and applies the marching squares
/// lookup table.  Returns a flat list of unconnected segments; caller is
/// responsible for any polyline assembly.
///
/// **Bit assignment** for the 4-bit case index:
/// - bit 0 → bottom-left corner `z[r][c]`
/// - bit 1 → bottom-right corner `z[r][c+1]`
/// - bit 2 → top-right corner `z[r+1][c+1]`
/// - bit 3 → top-left corner `z[r+1][c]`
///
/// **Saddle cases 5 and 10** use the simple split: each pair of same-value
/// diagonal corners is treated as a separate island (no centre-value disambiguation).
pub(crate) fn marching_squares(
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    level: f64,
) -> Vec<Segment> {
    let mut segs = Vec::new();
    if nrows < 2 || ncols < 2 {
        return segs;
    }

    for r in 0..nrows - 1 {
        for c in 0..ncols - 1 {
            let z00 = z[r * ncols + c];
            let z01 = z[r * ncols + c + 1];
            let z10 = z[(r + 1) * ncols + c];
            let z11 = z[(r + 1) * ncols + c + 1];

            let idx = ((z00 >= level) as u8)
                | (((z01 >= level) as u8) << 1)
                | (((z11 >= level) as u8) << 2)
                | (((z10 >= level) as u8) << 3);

            if idx == 0 || idx == 15 {
                continue;
            }

            let xc = x_vals[c];
            let xc1 = x_vals[c + 1];
            let yr = y_vals[r];
            let yr1 = y_vals[r + 1];

            // Linear interpolation along an edge: returns the crossing position.
            let lerp = |a_z: f64, b_z: f64, a_pos: f64, b_pos: f64| -> f64 {
                let dz = b_z - a_z;
                if dz.abs() < f64::EPSILON {
                    (a_pos + b_pos) * 0.5
                } else {
                    a_pos + (level - a_z) / dz * (b_pos - a_pos)
                }
            };

            // Four edge crossing points (computed lazily via closures,
            // but materialised here for clarity in the match below).
            let e0 = (lerp(z00, z01, xc, xc1), yr); // bottom edge
            let e1 = (xc1, lerp(z01, z11, yr, yr1)); // right edge
            let e2 = (lerp(z10, z11, xc, xc1), yr1); // top edge
            let e3 = (xc, lerp(z00, z10, yr, yr1)); // left edge

            match idx {
                1 => segs.push((e0, e3)),
                2 => segs.push((e0, e1)),
                3 => segs.push((e3, e1)),
                4 => segs.push((e1, e2)),
                // saddle case 5: BL + TR above → two islands
                5 => {
                    segs.push((e0, e3));
                    segs.push((e1, e2));
                }
                6 => segs.push((e0, e2)),
                7 => segs.push((e3, e2)),
                8 => segs.push((e3, e2)),
                9 => segs.push((e0, e2)),
                // saddle case 10: BR + TL above → two islands
                10 => {
                    segs.push((e0, e1));
                    segs.push((e2, e3));
                }
                11 => segs.push((e1, e2)),
                12 => segs.push((e3, e1)),
                13 => segs.push((e0, e1)),
                14 => segs.push((e0, e3)),
                _ => {}
            }
        }
    }
    segs
}

// ── ASCII renderer ─────────────────────────────────────────────────────────

/// Renders a contour or contourf map as character art to stdout.
///
/// The Z matrix is sampled onto an 80×24 character grid.  Each character
/// encodes the level band of the sampled Z value using the palette
/// `" .:-=+*#"` (low → high).
#[cfg(feature = "plot")]
pub fn render_contour_ascii(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    levels: &[f64],
    state: &FigureState,
) {
    const DENSITY: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#'];
    const CHART_W: usize = 80;
    const CHART_H: usize = 24;

    if nrows == 0 || ncols == 0 {
        return;
    }

    if let Some(t) = &state.title {
        println!("{t}");
    }

    let n_chars = DENSITY.len();

    for row in (0..CHART_H).rev() {
        // Sample row index in Z (row 0 = bottom, CHART_H-1 = top).
        let r = (row * nrows / CHART_H).min(nrows - 1);
        let line: String = (0..CHART_W)
            .map(|col| {
                let c = (col * ncols / CHART_W).min(ncols - 1);
                let v = z[r * ncols + c];
                // band ∈ [0, levels.len()]
                let band = levels.iter().filter(|&&lev| v >= lev).count();
                let char_idx = if levels.is_empty() {
                    0
                } else {
                    (band * (n_chars - 1) / levels.len()).min(n_chars - 1)
                };
                DENSITY[char_idx]
            })
            .collect();
        println!("{line}");
    }

    if let Some(xl) = &state.xlabel {
        println!("x: {xl}");
    }
    if let Some(yl) = &state.ylabel {
        println!("y: {yl}");
    }
}

// ── SVG/PNG renderers ──────────────────────────────────────────────────────

/// Writes a `contour` isoline chart to an SVG or PNG file.
///
/// Each level is drawn as a `LineSeries` colored by the active colormap.
#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
pub fn render_contour_file(
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    levels: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
        draw_contour(x_vals, y_vals, z, nrows, ncols, levels, &state, root, false)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
        draw_contour(x_vals, y_vals, z, nrows, ncols, levels, &state, root, false)
    } else {
        Err(format!("contour: unsupported format '{path}'"))
    }
}

/// Writes a `contourf` filled contour chart to an SVG or PNG file.
///
/// Cells are flood-colored by Z band (discrete colormap), then contour
/// isolines are drawn on top.
#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
pub fn render_contourf_file(
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    levels: &[f64],
    path: &str,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
        draw_contour(x_vals, y_vals, z, nrows, ncols, levels, &state, root, true)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
        draw_contour(x_vals, y_vals, z, nrows, ncols, levels, &state, root, true)
    } else {
        Err(format!("contourf: unsupported format '{path}'"))
    }
}

/// Core 2D contour drawing.
///
/// When `filled = true`, each grid cell is pre-colored by Z band before the
/// isoline segments are drawn on top.
#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
fn draw_contour<DB: DrawingBackend>(
    x_vals: &[f64],
    y_vals: &[f64],
    z: &[f64],
    nrows: usize,
    ncols: usize,
    levels: &[f64],
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
    filled: bool,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    if nrows == 0 || ncols == 0 || levels.is_empty() {
        return root.present().map_err(|e| e.to_string());
    }

    let x_lo = x_vals.iter().copied().fold(f64::INFINITY, f64::min);
    let x_hi = x_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_lo = y_vals.iter().copied().fold(f64::INFINITY, f64::min);
    let y_hi = y_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let (x_lo, x_hi) = state.xlim.unwrap_or((x_lo, x_hi));
    let (y_lo, y_hi) = state.ylim.unwrap_or((y_lo, y_hi));

    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("");
    let cmap = state.colormap.as_deref().unwrap_or("viridis");

    let n_levels = levels.len();

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_lo..x_hi, y_lo..y_hi)
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

    // ── Filled background: one colored Rectangle per grid cell ─────────────
    if filled && nrows >= 2 && ncols >= 2 {
        for r in 0..nrows - 1 {
            for c in 0..ncols - 1 {
                let z_mean = (z[r * ncols + c]
                    + z[r * ncols + c + 1]
                    + z[(r + 1) * ncols + c]
                    + z[(r + 1) * ncols + c + 1])
                    / 4.0;

                // band ∈ [0, n_levels]; map to [0, 1] for colormap
                let band = levels.iter().filter(|&&lev| z_mean >= lev).count();
                let t = band as f64 / n_levels as f64;
                let (rr, gg, bb) = apply_colormap(t, cmap);

                let x0 = x_vals[c];
                let x1 = x_vals[c + 1];
                let y0 = y_vals[r];
                let y1 = y_vals[r + 1];

                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(x0.min(x1), y0.min(y1)), (x0.max(x1), y0.max(y1))],
                        RGBColor(rr, gg, bb).filled(),
                    )))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    // ── Isoline segments ───────────────────────────────────────────────────
    for (i, &level) in levels.iter().enumerate() {
        let t = if n_levels > 1 {
            i as f64 / (n_levels - 1) as f64
        } else {
            0.5
        };
        let (rr, gg, bb) = apply_colormap(t, cmap);
        let color = RGBColor(rr, gg, bb);

        for ((x0, y0), (x1, y1)) in marching_squares(x_vals, y_vals, z, nrows, ncols, level) {
            chart
                .draw_series(LineSeries::new(vec![(x0, y0), (x1, y1)], color))
                .map_err(|e| e.to_string())?;
        }
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── compute_levels ────────────────────────────────────────────────

    #[test]
    fn test_compute_levels_zero_count() {
        assert!(compute_levels(0.0, 1.0, 0).is_empty());
    }

    #[test]
    fn test_compute_levels_one() {
        let lvls = compute_levels(0.0, 2.0, 1);
        assert_eq!(lvls.len(), 1);
        assert!(
            (lvls[0] - 1.0).abs() < 1e-12,
            "single level should be midpoint"
        );
    }

    #[test]
    fn test_compute_levels_five_interior() {
        let lvls = compute_levels(0.0, 6.0, 5);
        assert_eq!(lvls.len(), 5);
        // Levels: 1, 2, 3, 4, 5 (step = 1.0 = 6/6)
        for (k, &v) in lvls.iter().enumerate() {
            let expected = (k + 1) as f64;
            assert!(
                (v - expected).abs() < 1e-12,
                "level[{k}] expected {expected}, got {v}"
            );
        }
        // All strictly inside (0, 6)
        assert!(lvls.iter().all(|&v| v > 0.0 && v < 6.0));
    }

    // ── marching_squares ─────────────────────────────────────────────

    /// A 2×2 grid with only the bottom-left corner above the level should
    /// produce one segment from the bottom edge to the left edge.
    #[test]
    fn test_marching_squares_case1_bl_above() {
        // z row-major: z[0][0]=1.0(BL), z[0][1]=0.0(BR), z[1][0]=0.0(TL), z[1][1]=0.0(TR)
        let x_vals = vec![0.0, 1.0];
        let y_vals = vec![0.0, 1.0];
        let z = vec![1.0, 0.0, 0.0, 0.0];
        let segs = marching_squares(&x_vals, &y_vals, &z, 2, 2, 0.5);

        assert_eq!(segs.len(), 1, "case 1 should produce exactly one segment");
        let ((x0, y0), (x1, y1)) = segs[0];
        // E0 (bottom): t = (0.5-1.0)/(0.0-1.0) = 0.5 → x=0.5, y=0.0
        // E3 (left):   t = (0.5-1.0)/(0.0-1.0) = 0.5 → x=0.0, y=0.5
        let pa = (x0, y0);
        let pb = (x1, y1);
        let ea = (0.5_f64, 0.0_f64);
        let eb = (0.0_f64, 0.5_f64);
        assert!(
            (pa == ea && pb == eb) || (pa == eb && pb == ea),
            "segment should be (0.5,0)↔(0,0.5), got {pa:?}↔{pb:?}"
        );
    }

    /// An all-below or all-above cell must produce zero segments.
    #[test]
    fn test_marching_squares_no_crossing() {
        let x_vals = vec![0.0, 1.0];
        let y_vals = vec![0.0, 1.0];
        // All below 0.5
        let z_below = vec![0.0, 0.1, 0.2, 0.3];
        assert!(marching_squares(&x_vals, &y_vals, &z_below, 2, 2, 0.5).is_empty());
        // All above 0.5
        let z_above = vec![1.0, 0.9, 0.8, 0.7];
        assert!(marching_squares(&x_vals, &y_vals, &z_above, 2, 2, 0.5).is_empty());
    }

    /// A saddle case (case 5: BL and TR above) should produce two segments.
    #[test]
    fn test_marching_squares_saddle_case5() {
        let x_vals = vec![0.0, 1.0];
        let y_vals = vec![0.0, 1.0];
        // z00=1(BL above), z01=0(BR below), z11=1(TR above), z10=0(TL below)
        let z = vec![1.0, 0.0, 0.0, 1.0];
        let segs = marching_squares(&x_vals, &y_vals, &z, 2, 2, 0.5);
        assert_eq!(segs.len(), 2, "saddle case 5 should produce 2 segments");
    }

    /// Grid smaller than 2×2 returns empty.
    #[test]
    fn test_marching_squares_too_small() {
        assert!(marching_squares(&[0.0], &[0.0], &[1.0], 1, 1, 0.5).is_empty());
    }
}
