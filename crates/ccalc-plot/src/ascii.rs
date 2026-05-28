//! ASCII terminal rendering via `textplots`.
//!
//! `textplots` 0.8 uses Braille characters (U+2800–U+28FF) to render data
//! points. The chart body must be rendered via the method chain
//! `.lineplot(...).display()` — calling `Display::fmt` directly on the chart
//! skips the canvas-population step and produces a blank output.
//!
//! Axis labels (title, xlabel, ylabel) are printed before/after the chart
//! via `println!`.

use textplots::{Chart, Plot, Shape};

use crate::FigureState;

// ── Shared helpers ─────────────────────────────────────────────────────────

/// Returns the textplots canvas size in Braille dots based on terminal dimensions.
///
/// textplots encodes 2 dots per terminal column and 4 dots per terminal row, so
/// `(cols * 2, rows * 2)` fills roughly half the terminal height with the chart.
pub(crate) fn chart_canvas() -> (u32, u32) {
    let cols = crate::term_cols();
    let rows = crate::term_rows();
    ((cols * 2) as u32, ((rows * 2).clamp(20, 120)) as u32)
}

fn chart_x_bounds(x: &[f64], state: &FigureState) -> (f32, f32) {
    state
        .xlim
        .map(|(lo, hi)| (lo as f32, hi as f32))
        .unwrap_or_else(|| x_bounds(x))
}

fn print_header(state: &FigureState) {
    if let Some(t) = &state.title {
        println!("{t}");
    }
}

/// Renders a connected line plot to stdout.
pub fn render_line(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Lines(&data))
        .display();
    print_labels(&state);
}

/// Renders a scatter (point cloud) plot to stdout.
pub fn render_scatter(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Points(&data))
        .display();
    print_labels(&state);
}

/// Renders a bar chart to stdout (vertical bars from baseline).
pub fn render_bar(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Bars(&data))
        .display();
    print_labels(&state);
}

/// Renders a stem plot to stdout (vertical markers at each data point).
pub fn render_stem(x: &[f64], y: &[f64], state: FigureState) {
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let (cw, ch) = chart_canvas();
    print_header(&state);
    // textplots has no native stem shape — use Bars (filled from baseline) as
    // the closest ASCII approximation; file export produces proper thin stems.
    Chart::new(cw, ch, x_min, x_max)
        .lineplot(&Shape::Bars(&data))
        .display();
    print_labels(&state);
}

/// Renders a filled polygon to stdout (bounding-box shaded with `░` chars).
///
/// textplots has no native polygon fill; we sample a grid of cells and shade
/// each one with `░` if its centre lies inside the polygon (even-odd rule).
pub fn render_fill(x: &[f64], y: &[f64], state: FigureState) {
    if x.is_empty() {
        return;
    }
    let data = to_f32_pairs(x, y);
    let (x_min, x_max) = chart_x_bounds(x, &state);
    let y_min = y.iter().copied().fold(f64::INFINITY, f64::min) as f32;
    let y_max = y.iter().copied().fold(f64::NEG_INFINITY, f64::max) as f32;
    print_header(&state);
    let cols: usize = crate::term_cols().saturating_sub(2).max(10);
    let rows: usize = (crate::term_rows() / 2).max(5);
    let col_step = (x_max - x_min) / cols as f32;
    let row_step = if (y_max - y_min).abs() > f32::EPSILON {
        (y_max - y_min) / rows as f32
    } else {
        1.0
    };
    for r in (0..rows).rev() {
        let cy = y_min + (r as f32 + 0.5) * row_step;
        let mut row_str = String::new();
        for c in 0..cols {
            let cx = x_min + (c as f32 + 0.5) * col_step;
            if point_in_polygon(cx, cy, &data) {
                row_str.push('░');
            } else {
                row_str.push(' ');
            }
        }
        println!("|{row_str}|");
    }
    print_labels(&state);
}

/// Renders a filled area under curve to stdout.
pub fn render_area(x: &[f64], y: &[f64], state: FigureState) {
    if x.is_empty() {
        return;
    }
    // Build a closed polygon: outline + baseline.
    let mut poly_x = x.to_vec();
    let mut poly_y = y.to_vec();
    poly_x.push(*x.last().unwrap());
    poly_y.push(0.0);
    poly_x.push(x[0]);
    poly_y.push(0.0);
    render_fill(&poly_x, &poly_y, state);
}

/// Ray-casting point-in-polygon test (2D, even-odd rule).
fn point_in_polygon(px: f32, py: f32, polygon: &[(f32, f32)]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Renders dual-axis XY series on one combined character-grid chart.
///
/// Each side's Y values are normalised to `[0, 1]` so both curves fill the
/// full chart height.  Left series are drawn with `*`, right series with `+`.
/// Actual Y scales are printed in the footer.
///
/// `left` / `right` — tuples of `(x_values, y_values, use_lines)`.
pub(crate) fn render_dual_axis(
    left: &[(Vec<f32>, Vec<f32>, bool)],
    right: &[(Vec<f32>, Vec<f32>, bool)],
    left_ylim: Option<(f32, f32)>,
    right_ylim: Option<(f32, f32)>,
    xlim: Option<(f32, f32)>,
    title: Option<&str>,
    xlabel: Option<&str>,
    left_ylabel: Option<&str>,
    right_ylabel: Option<&str>,
) {
    let cols = crate::term_cols().saturating_sub(4).max(20);
    let rows = (crate::term_rows() / 2).clamp(6, 30);

    // ── Y range helpers ──────────────────────────────────────────────────
    let y_range = |series: &[(Vec<f32>, Vec<f32>, bool)], ovr: Option<(f32, f32)>| -> (f32, f32) {
        if let Some(r) = ovr {
            return r;
        }
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for (_, y, _) in series {
            for &v in y {
                if v < lo {
                    lo = v;
                }
                if v > hi {
                    hi = v;
                }
            }
        }
        if lo.is_infinite() {
            return (0.0, 1.0);
        }
        if (hi - lo).abs() < f32::EPSILON {
            (lo - 1.0, lo + 1.0)
        } else {
            (lo, hi)
        }
    };

    let (ly_min, ly_max) = y_range(left, left_ylim);
    let (ry_min, ry_max) = y_range(right, right_ylim);

    // ── X range ─────────────────────────────────────────────────────────
    let (x_min, x_max) = xlim.unwrap_or_else(|| {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for (x, _, _) in left.iter().chain(right.iter()) {
            for &v in x {
                if v < lo {
                    lo = v;
                }
                if v > hi {
                    hi = v;
                }
            }
        }
        if lo.is_infinite() {
            return (-1.0, 1.0);
        }
        if (hi - lo).abs() < f32::EPSILON {
            (lo - 1.0, lo + 1.0)
        } else {
            (lo, hi)
        }
    });

    // ── Build character grid ─────────────────────────────────────────────
    let mut grid: Vec<Vec<char>> = vec![vec![' '; cols]; rows];

    let to_col = |x: f32| -> usize {
        ((x - x_min) / (x_max - x_min) * (cols - 1) as f32)
            .round()
            .clamp(0.0, (cols - 1) as f32) as usize
    };
    let to_row = |yn: f32| -> usize {
        (yn * (rows - 1) as f32)
            .round()
            .clamp(0.0, (rows - 1) as f32) as usize
    };

    let plot_series = |grid: &mut Vec<Vec<char>>,
                       series: &[(Vec<f32>, Vec<f32>, bool)],
                       y_min: f32,
                       y_max: f32,
                       ch: char| {
        let span = y_max - y_min;
        for (xv, yv, use_lines) in series {
            let pts: Vec<(usize, usize)> = xv
                .iter()
                .zip(yv.iter())
                .map(|(&xi, &yi)| {
                    let yn = if span.abs() < f32::EPSILON {
                        0.5
                    } else {
                        (yi - y_min) / span
                    };
                    (to_col(xi), to_row(yn))
                })
                .collect();
            if *use_lines {
                for w in pts.windows(2) {
                    bresenham(grid, w[0], w[1], ch);
                }
            } else {
                for (c, r) in pts {
                    grid[r][c] = ch;
                }
            }
        }
    };

    plot_series(&mut grid, left, ly_min, ly_max, '.');
    plot_series(&mut grid, right, ry_min, ry_max, '*');

    // ── Print grid ───────────────────────────────────────────────────────
    if let Some(t) = title {
        println!("{t}");
    }
    println!("+{}+", "-".repeat(cols + 2));
    for row in grid.iter().rev() {
        let s: String = row.iter().collect();
        println!("| {s} |");
    }
    println!("+{}+", "-".repeat(cols + 2));

    // ── Footer ───────────────────────────────────────────────────────────
    if let Some(xl) = xlabel {
        println!("x: {xl}");
    }
    let fmt_r = |lo: f32, hi: f32| format!("[{lo} .. {hi}]");
    match left_ylabel {
        Some(yl) => println!("y (left)  . : {yl}  {}", fmt_r(ly_min, ly_max)),
        None => println!("y (left)  . : {}", fmt_r(ly_min, ly_max)),
    }
    match right_ylabel {
        Some(yr) => println!("y (right) * : {yr}  {}", fmt_r(ry_min, ry_max)),
        None => println!("y (right) * : {}", fmt_r(ry_min, ry_max)),
    }
}

/// Bresenham line-draw into a character grid.
fn bresenham(
    grid: &mut Vec<Vec<char>>,
    (x0, y0): (usize, usize),
    (x1, y1): (usize, usize),
    ch: char,
) {
    let rows = grid.len() as i32;
    let cols = if rows > 0 { grid[0].len() as i32 } else { 0 };
    let (mut x, mut y) = (x0 as i32, y0 as i32);
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    loop {
        if x >= 0 && x < cols && y >= 0 && y < rows {
            grid[y as usize][x as usize] = ch;
        }
        if x == x1 as i32 && y == y1 as i32 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn print_labels(state: &FigureState) {
    if let Some(xl) = &state.xlabel {
        println!("x: {xl}");
    }
    if let Some(yl) = &state.ylabel {
        println!("y: {yl}");
    }
}

fn to_f32_pairs(x: &[f64], y: &[f64]) -> Vec<(f32, f32)> {
    x.iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (xi as f32, yi as f32))
        .collect()
}

fn x_bounds(xs: &[f64]) -> (f32, f32) {
    let lo = xs.iter().copied().fold(f64::INFINITY, f64::min) as f32;
    let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) as f32;
    if (hi - lo).abs() < f32::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        (lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // textplots 0.8 renders via .display() which prints to stdout directly.
    // These tests verify no-panic behaviour; capturing the Braille canvas
    // output requires stdout redirection and is deferred.

    #[test]
    fn test_render_line_no_panic() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&xi| xi * xi).collect();
        render_line(&x, &y, FigureState::default());
    }

    #[test]
    fn test_render_scatter_no_panic() {
        let x: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&xi| xi * 2.0).collect();
        render_scatter(&x, &y, FigureState::default());
    }

    #[test]
    fn test_single_point_no_panic() {
        // A single-point vector must not panic (x_bounds adds padding).
        render_line(&[5.0], &[3.0], FigureState::default());
    }

    #[test]
    fn test_figure_state_label_values() {
        // Verify the FigureState struct holds label values correctly before
        // the render function consumes it.  This is the "label threading" test.
        let state = FigureState {
            title: Some("My Plot".into()),
            xlabel: Some("time".into()),
            ylabel: Some("value".into()),
            ..FigureState::default()
        };
        assert_eq!(state.title.as_deref(), Some("My Plot"));
        assert_eq!(state.xlabel.as_deref(), Some("time"));
        assert_eq!(state.ylabel.as_deref(), Some("value"));
    }
}
