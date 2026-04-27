//! MAT-file read helpers backing `load('file.mat')`.
//!
//! Compiled only when the `mat` feature is enabled.

use indexmap::IndexMap;
use ndarray::Array2;

use crate::env::Value;

/// Loads a MATLAB Level 5/7 MAT file and returns a [`Value::Struct`] mapping
/// variable names to their values.
pub(crate) fn mat_load(path: &str) -> Result<Value, String> {
    let mat = matrw::load_matfile(path).map_err(|e| format!("load: cannot read '{path}': {e}"))?;
    let mut fields = IndexMap::new();
    for (name, var) in mat.iter() {
        let val = mat_var_to_value(var).map_err(|e| format!("load: variable '{name}': {e}"))?;
        fields.insert(name.clone(), val);
    }
    Ok(Value::Struct(fields))
}

fn mat_var_to_value(var: &matrw::MatVariable) -> Result<Value, String> {
    use matrw::MatlabType;
    match var {
        matrw::MatVariable::NumericArray(_) => {
            // Check for char arrays tagged with a string class
            match var.numeric_type() {
                Some(MatlabType::UTF8(chars)) => {
                    return Ok(Value::Str(chars.iter().collect::<String>()));
                }
                Some(MatlabType::UTF16(chars)) => {
                    return Ok(Value::Str(chars.iter().collect::<String>()));
                }
                _ => {}
            }
            let dim = var.dim();
            let data = var
                .to_vec_f64()
                .ok_or_else(|| "cannot convert to f64".to_string())?;
            match dim.as_slice() {
                [] | [1, 1] => Ok(Value::Scalar(data.first().copied().unwrap_or(0.0))),
                [rows, cols] => {
                    let (rows, cols) = (*rows, *cols);
                    if rows == 0 || cols == 0 {
                        return Ok(Value::Matrix(Array2::zeros((rows, cols))));
                    }
                    // matrw stores data in column-major order.
                    // Build a (cols × rows) row-major array, then transpose to (rows × cols).
                    let mat = Array2::from_shape_vec((cols, rows), data)
                        .map_err(|e| format!("matrix shape error: {e}"))?
                        .t()
                        .to_owned();
                    Ok(Value::Matrix(mat))
                }
                _ => Err("unsupported array dimensionality (>2D)".to_string()),
            }
        }
        matrw::MatVariable::Structure(s) => {
            let mut fields = IndexMap::new();
            for (name, field_var) in s.value.iter() {
                let val = mat_var_to_value(field_var)?;
                fields.insert(name.clone(), val);
            }
            Ok(Value::Struct(fields))
        }
        matrw::MatVariable::StructureArray(sa) => {
            let n = sa.value.len();
            if n == 1 {
                mat_var_to_value(&sa.value[0])
            } else {
                let items: Result<Vec<IndexMap<String, Value>>, String> = sa
                    .value
                    .iter()
                    .map(|elem| match mat_var_to_value(elem)? {
                        Value::Struct(f) => Ok(f),
                        _ => Err("StructureArray element is not a struct".to_string()),
                    })
                    .collect();
                Ok(Value::StructArray(items?))
            }
        }
        matrw::MatVariable::CellArray(ca) => {
            let cells: Result<Vec<Value>, String> = ca.value.iter().map(mat_var_to_value).collect();
            Ok(Value::Cell(cells?))
        }
        matrw::MatVariable::Null => Ok(Value::Scalar(f64::NAN)),
        _ => Err("unsupported MAT variable type (sparse/compressed not supported)".to_string()),
    }
}
