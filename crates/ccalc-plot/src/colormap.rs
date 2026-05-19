//! Colormap LUT data and imagesc rendering (ASCII and SVG/PNG).

#[cfg(feature = "plot-svg")]
use plotters::prelude::*;

#[cfg(any(feature = "plot", feature = "plot-svg"))]
use crate::FigureState;

// ── Public API ─────────────────────────────────────────────────────────────

/// All supported colormap names.
pub const VALID_COLORMAPS: &[&str] = &[
    "viridis", "inferno", "magma", "plasma", "hot", "cool", "jet", "gray",
];

/// Validates a colormap name.
///
/// Returns `Ok(())` when `name` is a recognised colormap, otherwise returns an
/// error string listing the valid choices.
pub fn validate_colormap(name: &str) -> Result<(), String> {
    if VALID_COLORMAPS.contains(&name) {
        Ok(())
    } else {
        Err(format!(
            "colormap: '{}' is not a recognised colormap. Valid colormaps: {}",
            name,
            VALID_COLORMAPS.join(", ")
        ))
    }
}

/// Maps a normalised value `t ∈ [0, 1]` to an `(R, G, B)` triple.
///
/// Values outside `[0, 1]` are clamped.  Unrecognised names fall back to
/// `"viridis"`.
///
/// # Examples
///
/// ```
/// use ccalc_plot::colormap::apply_colormap;
/// let (r, g, b) = apply_colormap(0.0, "gray");
/// assert_eq!((r, g, b), (0, 0, 0));
/// let (r, g, b) = apply_colormap(1.0, "gray");
/// assert_eq!((r, g, b), (255, 255, 255));
/// ```
pub fn apply_colormap(t: f64, name: &str) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    match name {
        "viridis" => lut_lerp(t, &VIRIDIS),
        "inferno" => lut_lerp(t, &INFERNO),
        "magma" => lut_lerp(t, &MAGMA),
        "plasma" => lut_lerp(t, &PLASMA),
        "hot" => lut_lerp(t, &HOT),
        "cool" => lut_lerp(t, &COOL),
        "jet" => lut_lerp(t, &JET),
        "gray" => {
            let v = (t * 255.0).round() as u8;
            (v, v, v)
        }
        _ => lut_lerp(t, &VIRIDIS),
    }
}

// ── LUT interpolation ──────────────────────────────────────────────────────

fn lut_lerp(t: f64, lut: &[(u8, u8, u8)]) -> (u8, u8, u8) {
    let n = lut.len();
    if n == 1 {
        return lut[0];
    }
    let ts = t * (n - 1) as f64;
    let lo = (ts as usize).min(n - 2);
    let hi = lo + 1;
    let f = ts - lo as f64;
    let lerp = |a: u8, b: u8| (a as f64 + f * (b as f64 - a as f64)).round() as u8;
    (
        lerp(lut[lo].0, lut[hi].0),
        lerp(lut[lo].1, lut[hi].1),
        lerp(lut[lo].2, lut[hi].2),
    )
}

// ── LUT data ───────────────────────────────────────────────────────────────

const VIRIDIS: [(u8, u8, u8); 8] = [
    (68, 1, 84),
    (72, 40, 120),
    (62, 83, 160),
    (49, 104, 142),
    (53, 183, 121),
    (101, 203, 94),
    (180, 222, 44),
    (253, 231, 37),
];
const INFERNO: [(u8, u8, u8); 8] = [
    (0, 0, 4),
    (40, 11, 84),
    (101, 21, 110),
    (159, 42, 99),
    (212, 72, 66),
    (245, 125, 21),
    (252, 190, 44),
    (252, 255, 164),
];
const MAGMA: [(u8, u8, u8); 8] = [
    (0, 0, 4),
    (28, 16, 68),
    (79, 18, 123),
    (129, 37, 129),
    (181, 55, 122),
    (229, 89, 104),
    (251, 143, 107),
    (252, 253, 191),
];
const PLASMA: [(u8, u8, u8); 8] = [
    (13, 8, 135),
    (84, 2, 163),
    (139, 10, 165),
    (185, 50, 137),
    (219, 92, 104),
    (243, 135, 72),
    (253, 182, 44),
    (240, 249, 33),
];
const HOT: [(u8, u8, u8); 8] = [
    (0, 0, 0),
    (96, 0, 0),
    (192, 0, 0),
    (255, 48, 0),
    (255, 144, 0),
    (255, 216, 0),
    (255, 255, 96),
    (255, 255, 255),
];
const COOL: [(u8, u8, u8); 8] = [
    (0, 255, 255),
    (36, 219, 255),
    (73, 182, 255),
    (109, 146, 255),
    (146, 109, 255),
    (182, 73, 255),
    (219, 36, 255),
    (255, 0, 255),
];
const JET: [(u8, u8, u8); 8] = [
    (0, 0, 143),
    (0, 0, 255),
    (0, 218, 255),
    (0, 255, 36),
    (146, 255, 0),
    (255, 218, 0),
    (255, 36, 0),
    (143, 0, 0),
];

// ── Data helpers ───────────────────────────────────────────────────────────

/// Returns `(min, max)` of finite values in `z`.  Falls back to `(0, 1)` on
/// all-NaN input; expands a degenerate range by 1.
#[cfg(any(feature = "plot", feature = "plot-svg"))]
pub(crate) fn data_range(z: &[f64]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &v in z {
        if v.is_finite() {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    if !lo.is_finite() {
        lo = 0.0;
        hi = 1.0;
    }
    if (hi - lo).abs() < f64::EPSILON {
        hi = lo + 1.0;
    }
    (lo, hi)
}

// ── ASCII renderer ─────────────────────────────────────────────────────────

/// Renders `imagesc` as character art to stdout.
///
/// Uses a 10-level density palette `" .:-=+*#@█"` to approximate intensity.
/// A one-line colorbar showing the data range is appended when
/// `state.colorbar` is `true`.
#[cfg(feature = "plot")]
pub fn render_imagesc_ascii(z: &[f64], nrows: usize, ncols: usize, state: &FigureState) {
    const DENSITY: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '@', '█'];

    if nrows == 0 || ncols == 0 {
        return;
    }

    let (z_min, z_max) = data_range(z);
    let range = z_max - z_min;

    if let Some(t) = &state.title {
        println!("{t}");
    }

    for r in 0..nrows {
        for c in 0..ncols {
            let v = z[r * ncols + c];
            let t = if range > 0.0 {
                ((v - z_min) / range).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let idx = ((t * 9.0) as usize).min(9);
            print!("{}", DENSITY[idx]);
        }
        println!();
    }

    if state.colorbar {
        let steps = 20_usize;
        let gradient: String = (0..steps)
            .map(|i| {
                let t = i as f64 / (steps - 1).max(1) as f64;
                let idx = ((t * 9.0) as usize).min(9);
                DENSITY[idx]
            })
            .collect();
        println!("{z_min:.4} [{gradient}] {z_max:.4}");
    }
    if let Some(xl) = &state.xlabel {
        println!("x: {xl}");
    }
    if let Some(yl) = &state.ylabel {
        println!("y: {yl}");
    }
}

// ── SVG/PNG file renderer ──────────────────────────────────────────────────

/// Width reserved for the colorbar strip (pixels).
#[cfg(feature = "plot-svg")]
const CB_WIDTH: u32 = 80;

/// Writes a false-colour image of `z` to an SVG or PNG file.
///
/// The active colormap is taken from `state.colormap` (default `"viridis"`).
/// If `state.colorbar` is `true`, a gradient strip with value labels is
/// appended on the right side of the image.
/// `width` and `height` set the canvas size in pixels.
#[cfg(feature = "plot-svg")]
pub fn render_imagesc_file(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    path: &str,
    width: u32,
    height: u32,
    state: FigureState,
) -> Result<(), String> {
    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (width, height)).into_drawing_area();
        draw_imagesc(z, nrows, ncols, &state, root, width)
    } else if path.ends_with(".png") {
        let root = BitMapBackend::new(path, (width, height)).into_drawing_area();
        draw_imagesc(z, nrows, ncols, &state, root, width)
    } else {
        Err(format!("imagesc: unsupported format '{path}'"))
    }
}

#[cfg(feature = "plot-svg")]
fn draw_imagesc<DB: DrawingBackend>(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    state: &FigureState,
    root: DrawingArea<DB, plotters::coord::Shift>,
    width: u32,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    if nrows == 0 || ncols == 0 {
        return root.present().map_err(|e| e.to_string());
    }

    let cmap = state.colormap.as_deref().unwrap_or("viridis");
    let (z_min, z_max) = data_range(z);
    let range = z_max - z_min;

    if state.colorbar {
        let split = (width.saturating_sub(CB_WIDTH)) as i32;
        let (img_area, cb_area) = root.split_horizontally(split);
        draw_imagesc_cells(&img_area, z, nrows, ncols, state, cmap, z_min, range)?;
        draw_colorbar(&cb_area, z_min, z_max, cmap)?;
    } else {
        draw_imagesc_cells(&root, z, nrows, ncols, state, cmap, z_min, range)?;
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(feature = "plot-svg")]
#[allow(clippy::too_many_arguments)]
fn draw_imagesc_cells<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    z: &[f64],
    nrows: usize,
    ncols: usize,
    state: &FigureState,
    cmap: &str,
    z_min: f64,
    range: f64,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    let title = state.title.as_deref().unwrap_or("");
    let xlabel = state.xlabel.as_deref().unwrap_or("");
    let ylabel = state.ylabel.as_deref().unwrap_or("");

    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 20))
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..(ncols as f64), 0.0..(nrows as f64))
        .map_err(|e| e.to_string())?;

    chart
        .configure_mesh()
        .x_desc(xlabel)
        .y_desc(ylabel)
        .disable_mesh()
        .draw()
        .map_err(|e| e.to_string())?;

    // Row 0 of Z is the top row; map it to y ∈ [nrows-1, nrows].
    for r in 0..nrows {
        let y_lo = (nrows - 1 - r) as f64;
        let y_hi = y_lo + 1.0;
        for c in 0..ncols {
            let v = z[r * ncols + c];
            let t = if range > 0.0 {
                ((v - z_min) / range).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let (rr, gg, bb) = apply_colormap(t, cmap);
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(c as f64, y_lo), ((c + 1) as f64, y_hi)],
                    RGBColor(rr, gg, bb).filled(),
                )))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(feature = "plot-svg")]
fn draw_colorbar<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    z_min: f64,
    z_max: f64,
    cmap: &str,
) -> Result<(), String>
where
    DB::ErrorType: std::fmt::Display,
{
    let n_steps: usize = 64;
    let step_h = (z_max - z_min) / n_steps as f64;

    // Horizontal margins must be small: CB_WIDTH = 80 px, y_label_area = 40 px.
    // margin_left=0 + margin_right=4 + y_label_area=40 → 36 px for the gradient strip.
    let mut chart = ChartBuilder::on(area)
        .margin_top(30)
        .margin_bottom(30)
        .margin_left(0)
        .margin_right(4)
        .x_label_area_size(0)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..1.0, z_min..z_max)
        .map_err(|e| e.to_string())?;

    // Draw the axis ticks / labels first (fills chart area with white background).
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .map_err(|e| e.to_string())?;

    // Draw gradient on top of the white background.
    chart
        .draw_series((0..n_steps).map(|i| {
            let t = i as f64 / (n_steps - 1).max(1) as f64;
            let y_lo = z_min + i as f64 * step_h;
            let y_hi = (y_lo + step_h).min(z_max);
            let (r, g, b) = apply_colormap(t, cmap);
            Rectangle::new([(0.0, y_lo), (1.0, y_hi)], RGBColor(r, g, b).filled())
        }))
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_colormap_gray_extremes() {
        assert_eq!(apply_colormap(0.0, "gray"), (0, 0, 0));
        assert_eq!(apply_colormap(1.0, "gray"), (255, 255, 255));
    }

    #[test]
    fn test_apply_colormap_clamp() {
        // Values outside [0,1] are clamped, not panicked.
        let lo = apply_colormap(-1.0, "hot");
        let hi = apply_colormap(2.0, "hot");
        assert_eq!(lo, apply_colormap(0.0, "hot"));
        assert_eq!(hi, apply_colormap(1.0, "hot"));
    }

    #[test]
    fn test_apply_colormap_fallback() {
        // Unknown colormap falls back to viridis — no panic.
        let _ = apply_colormap(0.5, "unknown_colormap_xyz");
    }

    #[test]
    fn test_validate_colormap_valid() {
        for name in VALID_COLORMAPS {
            assert!(validate_colormap(name).is_ok(), "'{name}' should be valid");
        }
    }

    #[test]
    fn test_validate_colormap_invalid() {
        let result = validate_colormap("rainbow");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("colormap"),
            "error should mention colormap: {msg}"
        );
    }

    #[cfg(any(feature = "plot", feature = "plot-svg"))]
    #[test]
    fn test_data_range_normal() {
        let (lo, hi) = data_range(&[3.0, 1.0, 4.0, 1.5]);
        assert!((lo - 1.0).abs() < 1e-9);
        assert!((hi - 4.0).abs() < 1e-9);
    }

    #[cfg(any(feature = "plot", feature = "plot-svg"))]
    #[test]
    fn test_data_range_all_nan() {
        let (lo, hi) = data_range(&[f64::NAN]);
        assert_eq!((lo, hi), (0.0, 1.0));
    }

    #[cfg(any(feature = "plot", feature = "plot-svg"))]
    #[test]
    fn test_data_range_constant() {
        // Constant input gets expanded so range > 0.
        let (lo, hi) = data_range(&[5.0, 5.0, 5.0]);
        assert!(hi > lo);
    }
}
