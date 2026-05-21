//! Argument parsing helpers shared by all plot backends.

use ccalc_engine::env::Value;

use crate::style::{StyleSpec, looks_like_style_str, parse_style_str};

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

/// Splits off an optional trailing style string and/or file-path argument from `args`.
#[allow(clippy::type_complexity)]
///
/// Calls [`extract_file_arg`] first to separate any path, then checks whether the
/// last remaining argument looks like a MATLAB-style format string (e.g. `"r--"`,
/// `"b."`).  Returns `(data_args, style, path)`.
pub fn extract_style_and_file_arg(
    args: &[Value],
) -> Result<(Vec<Value>, Option<StyleSpec>, Option<String>), String> {
    let (mut data_args, path) = extract_file_arg(args);

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

fn as_str(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) | Value::StringObj(s) => Some(s.clone()),
        _ => None,
    }
}
