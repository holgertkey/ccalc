//! Argument parsing helpers shared by all plot backends.

use ccalc_engine::env::Value;

/// Splits off a trailing file-path argument from `args`.
///
/// Returns `(data_args, Some(path))` if the last argument is a string
/// whose value ends with `.svg`, `.png`, or equals `"ascii"` exactly.
/// Otherwise returns `(args.to_vec(), None)`.
pub fn extract_file_arg(args: &[Value]) -> (Vec<Value>, Option<String>) {
    if let Some(last) = args.last() {
        if let Some(s) = as_str(last) {
            if s == "ascii" || s.ends_with(".svg") || s.ends_with(".png") {
                return (args[..args.len() - 1].to_vec(), Some(s));
            }
        }
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

fn as_str(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) | Value::StringObj(s) => Some(s.clone()),
        _ => None,
    }
}
