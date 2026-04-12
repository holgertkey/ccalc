use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ndarray::Array2;

use crate::io::IoContext;

/// A type-erased callable for anonymous functions (lambdas).
///
/// Stores a heap-allocated closure that captures the lambda's body expression
/// and the lexical environment at the point of definition.
/// Two `LambdaFn` values are equal only if they are the exact same allocation.
#[derive(Clone)]
pub struct LambdaFn(pub Rc<dyn Fn(&[Value], Option<&mut IoContext>) -> Result<Value, String>>);

impl std::fmt::Debug for LambdaFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@<lambda>")
    }
}

impl PartialEq for LambdaFn {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// A value held in the variable environment.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// No display value — returned by side-effectful functions like `fprintf`.
    Void,
    Scalar(f64),
    Matrix(Array2<f64>),
    /// Complex number `re + im*i`.
    Complex(f64, f64),
    /// Character array (single-quoted string). Represents a 1×N row of char values.
    Str(String),
    /// String object (double-quoted string).
    StringObj(String),
    /// Anonymous function: `@(params) expr`. Stores a pre-compiled closure
    /// that captures the lexical environment at definition time.
    Lambda(LambdaFn),
    /// Named user-defined function: `function [outputs] = name(params) ... end`.
    ///
    /// The body is stored as raw source text and re-parsed on each call.
    /// Named functions execute in an isolated scope (only params are visible,
    /// plus built-in constants `i`, `j`).
    Function {
        outputs: Vec<String>,
        params: Vec<String>,
        body_source: String,
    },
    /// Multiple return values from a multi-output function call (internal use).
    ///
    /// Produced by calling a function with `outputs.len() > 1`.
    /// Consumed by `Stmt::MultiAssign` in exec.rs. Not directly user-visible.
    Tuple(Vec<Value>),
}

impl Value {
    pub fn as_scalar(&self) -> Option<f64> {
        match self {
            Value::Scalar(n) => Some(*n),
            Value::Void
            | Value::Matrix(_)
            | Value::Complex(_, _)
            | Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_) => None,
        }
    }
}

/// Variable environment: maps names to values.
///
/// `ans` is the reserved name for the result of the last expression
/// that was not assigned to a named variable (Octave/MATLAB convention).
pub type Env = HashMap<String, Value>;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ccalc")
}

fn workspace_path() -> PathBuf {
    config_dir().join("workspace.toml")
}

/// Serializes one `Value` to a workspace line value string.
/// Returns `None` for types that cannot be persisted (Matrix, Complex, Void,
/// or strings containing characters that would break the format).
fn serialize_value(v: &Value) -> Option<String> {
    match v {
        Value::Scalar(n) => Some(format!("{n}")),
        // Char arrays: wrap in single quotes; skip if contains ' or newline
        Value::Str(s) if !s.contains('\'') && !s.contains('\n') => Some(format!("'{s}'")),
        // String objects: wrap in double quotes; skip if contains " or newline
        Value::StringObj(s) if !s.contains('"') && !s.contains('\n') => Some(format!("\"{s}\"")),
        _ => None,
    }
}

/// Saves scalars and strings from `env` to `path`.
/// Matrices, complex values, and strings with unsafe characters are skipped.
/// Format: `name = value` per line, where value is a raw f64, `'str'`, or `"strobj"`.
pub fn save_workspace(env: &Env, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create config dir: {e}"))?;
    }
    let mut entries: Vec<(&String, String)> = env
        .iter()
        .filter_map(|(k, v)| serialize_value(v).map(|s| (k, s)))
        .collect();
    entries.sort_by_key(|(k, _)| k.as_str());
    let mut content = String::new();
    for (name, val) in entries {
        content.push_str(&format!("{name} = {val}\n"));
    }
    std::fs::write(path, &content).map_err(|e| format!("Cannot write {}: {e}", path.display()))
}

/// Saves only the named variables from `env` to `path`.
/// Variables not present in `env` are silently ignored.
pub fn save_workspace_vars(env: &Env, path: &Path, vars: &[&str]) -> Result<(), String> {
    let filtered: Env = env
        .iter()
        .filter(|(k, _)| vars.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    save_workspace(&filtered, path)
}

/// Loads variables from `path` into a new `Env`.
/// Recognises: `name = 3.14` (Scalar), `name = 'str'` (Str), `name = "str"` (StringObj).
/// Unrecognised lines are silently skipped.
pub fn load_workspace(path: &Path) -> Result<Env, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let mut env = Env::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            if !is_valid_ident(key) {
                continue;
            }
            let value = if val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2 {
                Value::Str(val[1..val.len() - 1].to_string())
            } else if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                Value::StringObj(val[1..val.len() - 1].to_string())
            } else if let Ok(n) = val.parse::<f64>() {
                Value::Scalar(n)
            } else {
                continue;
            };
            env.insert(key.to_string(), value);
        }
    }
    Ok(env)
}

pub fn save_workspace_default(env: &Env) -> Result<(), String> {
    save_workspace(env, &workspace_path())
}

pub fn load_workspace_default() -> Result<Env, String> {
    load_workspace(&workspace_path())
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => chars.all(|c| c.is_alphanumeric() || c == '_'),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_roundtrip() {
        let path = std::env::temp_dir().join("ccalc_test_workspace_roundtrip.toml");
        let mut env = Env::new();
        env.insert("x".to_string(), Value::Scalar(42.0));
        env.insert("y".to_string(), Value::Scalar(-3.14));
        env.insert("ans".to_string(), Value::Scalar(10.0));
        save_workspace(&env, &path).unwrap();

        let loaded = load_workspace(&path).unwrap();
        assert_eq!(loaded.get("x"), Some(&Value::Scalar(42.0)));
        assert_eq!(loaded.get("y"), Some(&Value::Scalar(-3.14)));
        assert_eq!(loaded.get("ans"), Some(&Value::Scalar(10.0)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_empty_workspace() {
        let path = std::env::temp_dir().join("ccalc_test_workspace_empty.toml");
        save_workspace(&Env::new(), &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.is_empty());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_nonexistent_returns_error() {
        let path = std::env::temp_dir().join("ccalc_test_workspace_nonexistent_xyz.toml");
        let _ = std::fs::remove_file(&path);
        assert!(load_workspace(&path).is_err());
    }

    #[test]
    fn test_load_ignores_invalid_lines() {
        let path = std::env::temp_dir().join("ccalc_test_workspace_invalid.toml");
        std::fs::write(&path, "# comment\n\nx = 5\n1bad = 9\ngood = abc\n").unwrap();
        let env = load_workspace(&path).unwrap();
        assert_eq!(env.get("x"), Some(&Value::Scalar(5.0)));
        assert!(!env.contains_key("1bad"));
        assert!(!env.contains_key("good")); // value not a float
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_is_valid_ident() {
        assert!(is_valid_ident("x"));
        assert!(is_valid_ident("my_var"));
        assert!(is_valid_ident("_private"));
        assert!(is_valid_ident("var1"));
        assert!(is_valid_ident("ans"));
        assert!(!is_valid_ident("1x"));
        assert!(!is_valid_ident(""));
        assert!(!is_valid_ident("a b"));
        assert!(!is_valid_ident("a-b"));
    }

    #[test]
    fn test_save_skips_matrices() {
        use ndarray::array;
        let path = std::env::temp_dir().join("ccalc_test_workspace_matrix_skip.toml");
        let mut env = Env::new();
        env.insert("x".to_string(), Value::Scalar(5.0));
        env.insert(
            "m".to_string(),
            Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]),
        );
        save_workspace(&env, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("x = 5"));
        assert!(!content.contains("m"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_load_strings() {
        let path = std::env::temp_dir().join("ccalc_test_workspace_strings.toml");
        let mut env = Env::new();
        env.insert("name".to_string(), Value::Str("hello".to_string()));
        env.insert("tag".to_string(), Value::StringObj("world".to_string()));
        env.insert("n".to_string(), Value::Scalar(1.0));
        save_workspace(&env, &path).unwrap();

        let loaded = load_workspace(&path).unwrap();
        assert_eq!(loaded.get("name"), Some(&Value::Str("hello".to_string())));
        assert_eq!(
            loaded.get("tag"),
            Some(&Value::StringObj("world".to_string()))
        );
        assert_eq!(loaded.get("n"), Some(&Value::Scalar(1.0)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_skips_string_with_unsafe_chars() {
        let path = std::env::temp_dir().join("ccalc_test_workspace_unsafe_str.toml");
        let mut env = Env::new();
        env.insert("s".to_string(), Value::Str("it's".to_string())); // embedded quote
        env.insert("x".to_string(), Value::Scalar(5.0));
        save_workspace(&env, &path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("x = 5"));
        assert!(!content.contains("it's")); // unsafe string skipped
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_workspace_vars_selective() {
        let path = std::env::temp_dir().join("ccalc_test_workspace_vars.toml");
        let mut env = Env::new();
        env.insert("x".to_string(), Value::Scalar(1.0));
        env.insert("y".to_string(), Value::Scalar(2.0));
        env.insert("z".to_string(), Value::Scalar(3.0));
        save_workspace_vars(&env, &path, &["x", "z"]).unwrap();

        let loaded = load_workspace(&path).unwrap();
        assert_eq!(loaded.get("x"), Some(&Value::Scalar(1.0)));
        assert_eq!(loaded.get("z"), Some(&Value::Scalar(3.0)));
        assert!(!loaded.contains_key("y")); // not in the list
        std::fs::remove_file(&path).ok();
    }
}
