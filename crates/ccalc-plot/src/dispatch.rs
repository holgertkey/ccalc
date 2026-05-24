//! Argument parsing helpers shared by all plot backends.

use ccalc_engine::env::Value;

use crate::style::{
    StyleColor, StyleSpec, looks_like_style_str, parse_color_token, parse_style_str,
};

/// Splits off a trailing file-path argument from `args`.
///
/// Returns `(data_args, Some(path))` if the last argument is a string
/// whose value ends with `.svg`, `.png`, or equals `"ascii"` exactly.
/// Otherwise returns `(args.to_vec(), None)`.
pub fn extract_file_arg(args: &[Value]) -> (Vec<Value>, Option<String>) {
    if let Some(last) = args.last()
        && let Some(s) = as_str(last)
        && (s == "ascii" || s.ends_with(".svg") || s.ends_with(".png"))
    {
        return (args[..args.len() - 1].to_vec(), Some(s));
    }
    (args.to_vec(), None)
}

/// Extracts a flat `Vec<f64>` from any numeric [`Value`].
///
/// Accepts `Scalar` (promoted to a one-element vector) and any `Matrix`
/// regardless of shape (row-major order).  Unlike [`extract_vector`] this
/// does **not** require a vector layout, so it is suitable for meshgrid-style
/// 2-D inputs.
pub fn extract_flat(v: &Value) -> Result<Vec<f64>, String> {
    match v {
        Value::Scalar(f) => Ok(vec![*f]),
        Value::Matrix(m) => Ok(m.iter().copied().collect()),
        _ => Err("plot: numeric array argument required".into()),
    }
}

/// Extracts a flat `Vec<f64>` from a scalar or vector `Value`.
///
/// A `Scalar` is promoted to a one-element vector. A `Matrix` is accepted
/// only when it is a row or column vector (one dimension equals 1).
pub fn extract_vector(v: &Value) -> Result<Vec<f64>, String> {
    match v {
        Value::Scalar(f) => Ok(vec![*f]),
        Value::Matrix(m) => {
            let (r, c) = (m.nrows(), m.ncols());
            if r == 1 || c == 1 {
                Ok(m.iter().copied().collect())
            } else {
                Err(format!("plot: expected a vector, got {r}×{c} matrix"))
            }
        }
        _ => Err("plot: numeric vector argument required".into()),
    }
}

/// Extracts a 2D matrix from a `Value`, returning flat row-major data and
/// dimensions `(data, nrows, ncols)`.
///
/// A `Scalar` is promoted to a 1×1 matrix.  Any other type returns an error.
pub fn extract_matrix(v: &Value) -> Result<(Vec<f64>, usize, usize), String> {
    match v {
        Value::Matrix(m) => {
            let nrows = m.nrows();
            let ncols = m.ncols();
            let mut data = Vec::with_capacity(nrows * ncols);
            for r in 0..nrows {
                for c in 0..ncols {
                    data.push(m[[r, c]]);
                }
            }
            Ok((data, nrows, ncols))
        }
        Value::Scalar(f) => Ok((vec![*f], 1, 1)),
        _ => Err("imagesc: expected a numeric matrix".into()),
    }
}

/// Splits off an optional trailing style and/or file-path argument from `args`.
#[allow(clippy::type_complexity)]
///
/// Processing order (first match wins):
/// 1. Trailing `'color', <value>` named-argument pair.
/// 2. Trailing 1×3 RGB matrix (values in `[0, 1]`) — only when the number of
///    remaining data args would exceed `min_data` after stripping.
/// 3. Trailing MATLAB-style format string (`"r--"`, `"red"`, `"#FF4400"`, …).
/// 4. Trailing file path (`.svg`, `.png`, or `"ascii"`).
///
/// `min_data` sets the minimum number of data arguments that must remain after
/// removing the style element.  Pass `1` for most callers; pass a higher value
/// (e.g. `4` for `quiver`) to prevent ambiguous vector data from being consumed
/// as an RGB colour spec.
///
/// Returns `(data_args, style, path)`.
pub fn extract_style_and_file_arg(
    args: &[Value],
) -> Result<(Vec<Value>, Option<StyleSpec>, Option<String>), String> {
    extract_style_and_file_arg_min(args, 1)
}

/// Like [`extract_style_and_file_arg`] but with a caller-supplied `min_data` guard.
#[allow(clippy::type_complexity)]
pub fn extract_style_and_file_arg_min(
    args: &[Value],
    min_data: usize,
) -> Result<(Vec<Value>, Option<StyleSpec>, Option<String>), String> {
    let (mut data_args, path) = extract_file_arg(args);

    // ── 'color', <value> named-argument pair ─────────────────────────────
    let len = data_args.len();
    if len >= 2
        && let Some(key) = as_str(&data_args[len - 2])
        && key.eq_ignore_ascii_case("color")
    {
        let sc = value_to_style_color(&data_args[len - 1])?;
        data_args.truncate(len - 2);
        return Ok((
            data_args,
            Some(StyleSpec {
                color: Some(sc),
                ..StyleSpec::default()
            }),
            path,
        ));
    }

    // ── 1×3 RGB matrix (values must all be in [0, 1]) ────────────────────
    // Require at least `min_data + 1` args so at least `min_data` data args
    // remain after stripping the colour matrix.
    let rgb_style = if data_args.len() > min_data {
        if let Some(Value::Matrix(m)) = data_args.last() {
            if m.nrows() == 1 && m.ncols() == 3 && m.iter().all(|&v| (0.0..=1.0).contains(&v)) {
                let clamp = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                Some(StyleColor(
                    clamp(m[[0, 0]]),
                    clamp(m[[0, 1]]),
                    clamp(m[[0, 2]]),
                ))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    if let Some(sc) = rgb_style {
        data_args.pop();
        return Ok((
            data_args,
            Some(StyleSpec {
                color: Some(sc),
                ..StyleSpec::default()
            }),
            path,
        ));
    }

    // ── MATLAB-style format string ────────────────────────────────────────
    let mut style: Option<StyleSpec> = None;
    if let Some(last) = data_args.last()
        && let Some(s) = as_str(last)
        && looks_like_style_str(&s)
    {
        style = Some(parse_style_str(&s)?);
        data_args.pop();
    }

    Ok((data_args, style, path))
}

/// Converts a [`Value`] to a [`StyleColor`] for the `'color'` named argument.
fn value_to_style_color(v: &Value) -> Result<StyleColor, String> {
    match v {
        Value::Str(s) | Value::StringObj(s) => parse_color_token(s)
            .ok_or_else(|| format!("plot: '{s}' is not a recognised color name or hex code")),
        Value::Matrix(m) if m.nrows() == 1 && m.ncols() == 3 => {
            let clamp = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            Ok(StyleColor(
                clamp(m[[0, 0]]),
                clamp(m[[0, 1]]),
                clamp(m[[0, 2]]),
            ))
        }
        _ => Err("plot: 'color' value must be a color name string or 1×3 matrix".into()),
    }
}

fn as_str(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) | Value::StringObj(s) => Some(s.clone()),
        _ => None,
    }
}
