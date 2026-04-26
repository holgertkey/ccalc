//! JSON encode/decode helpers backing the `jsondecode` and `jsonencode` built-ins.
//!
//! Compiled only when the `json` feature is enabled.

use indexmap::IndexMap;
use ndarray::Array2;

use crate::env::Value;

/// Converts a `serde_json::Value` to a ccalc [`Value`].
///
/// | JSON type | ccalc `Value` |
/// |-----------|---------------|
/// | `null`    | `Scalar(NaN)` |
/// | `bool`    | `Scalar(1.0 / 0.0)` |
/// | number    | `Scalar` |
/// | string    | `Str` |
/// | all-numeric array (or nulls) | `Matrix` (1×N row vector) |
/// | mixed array | `Cell` |
/// | object    | `Struct` |
pub(crate) fn json_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Scalar(f64::NAN),
        serde_json::Value::Bool(b) => Value::Scalar(if *b { 1.0 } else { 0.0 }),
        serde_json::Value::Number(n) => Value::Scalar(n.as_f64().unwrap_or(f64::NAN)),
        serde_json::Value::String(s) => Value::Str(s.clone()),
        serde_json::Value::Array(arr) => decode_array(arr),
        serde_json::Value::Object(map) => {
            let mut fields = IndexMap::new();
            for (k, v) in map {
                fields.insert(k.clone(), json_to_value(v));
            }
            Value::Struct(fields)
        }
    }
}

fn decode_array(arr: &[serde_json::Value]) -> Value {
    if arr.is_empty() {
        return Value::Matrix(Array2::zeros((1, 0)));
    }
    // Attempt all-numeric row vector (null → NaN, numbers → f64, rest fails)
    let maybe_nums: Option<Vec<f64>> = arr
        .iter()
        .map(|item| match item {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::Null => Some(f64::NAN),
            _ => None,
        })
        .collect();
    if let Some(nums) = maybe_nums {
        let ncols = nums.len();
        Value::Matrix(Array2::from_shape_vec((1, ncols), nums).unwrap())
    } else {
        Value::Cell(arr.iter().map(json_to_value).collect())
    }
}

/// Converts a ccalc [`Value`] to a `serde_json::Value`.
///
/// Returns an error for types with no JSON representation
/// (`Complex`, `Lambda`, `Function`, `Void`, `Tuple`).
pub(crate) fn value_to_json(v: &Value) -> Result<serde_json::Value, String> {
    match v {
        Value::Scalar(x) => encode_f64(*x),
        Value::Matrix(m) => {
            if m.nrows() == 0 || m.ncols() == 0 {
                return Ok(serde_json::Value::Array(vec![]));
            }
            if m.nrows() == 1 {
                // Row vector → flat JSON array
                let arr: Result<Vec<serde_json::Value>, String> =
                    m.iter().map(|x| encode_f64(*x)).collect();
                Ok(serde_json::Value::Array(arr?))
            } else {
                // M×N matrix → array of row arrays
                let rows: Result<Vec<serde_json::Value>, String> = m
                    .rows()
                    .into_iter()
                    .map(|row| {
                        let arr: Result<Vec<serde_json::Value>, String> =
                            row.iter().map(|x| encode_f64(*x)).collect();
                        Ok(serde_json::Value::Array(arr?))
                    })
                    .collect();
                Ok(serde_json::Value::Array(rows?))
            }
        }
        Value::Str(s) | Value::StringObj(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Cell(cells) => {
            let arr: Result<Vec<serde_json::Value>, String> =
                cells.iter().map(value_to_json).collect();
            Ok(serde_json::Value::Array(arr?))
        }
        Value::Struct(fields) => {
            let mut map = serde_json::Map::new();
            for (k, v) in fields {
                map.insert(k.clone(), value_to_json(v)?);
            }
            Ok(serde_json::Value::Object(map))
        }
        Value::StructArray(arr) => {
            // Array of structs → JSON array of objects
            let items: Result<Vec<serde_json::Value>, String> = arr
                .iter()
                .map(|fields| {
                    let mut map = serde_json::Map::new();
                    for (k, v) in fields {
                        map.insert(k.clone(), value_to_json(v)?);
                    }
                    Ok(serde_json::Value::Object(map))
                })
                .collect();
            Ok(serde_json::Value::Array(items?))
        }
        Value::Complex(re, im) => Err(format!(
            "jsonencode: cannot represent complex {re}+{im}i in JSON"
        )),
        Value::Lambda(_) | Value::Function { .. } => {
            Err("jsonencode: cannot encode function values".to_string())
        }
        Value::Void => Err("jsonencode: cannot encode Void".to_string()),
        Value::Tuple(_) => Err("jsonencode: cannot encode multi-output tuple".to_string()),
    }
}

fn encode_f64(x: f64) -> Result<serde_json::Value, String> {
    if x.is_nan() {
        Ok(serde_json::Value::Null)
    } else if x.is_infinite() {
        Err(format!("jsonencode: cannot represent {x} in JSON"))
    } else {
        Ok(serde_json::json!(x))
    }
}
