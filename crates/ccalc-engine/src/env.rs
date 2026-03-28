use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Variable environment: maps names to scalar values.
///
/// `ans` is the reserved name for the result of the last expression
/// that was not assigned to a named variable (Octave/MATLAB convention).
pub type Env = HashMap<String, f64>;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ccalc")
}

fn workspace_path() -> PathBuf {
    config_dir().join("workspace.toml")
}

/// Saves all variables in `env` to `path`.
/// Each variable is written as `name = value\n`.
pub fn save_workspace(env: &Env, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create config dir: {e}"))?;
    }
    let mut pairs: Vec<(&String, &f64)> = env.iter().collect();
    pairs.sort_by_key(|(k, _)| k.as_str());
    let mut content = String::new();
    for (name, val) in pairs {
        content.push_str(&format!("{name} = {val}\n"));
    }
    std::fs::write(path, &content)
        .map_err(|e| format!("Cannot write {}: {e}", path.display()))
}

/// Loads variables from `path`, returning a new `Env`.
/// Lines that do not match `name = value` are silently skipped.
pub fn load_workspace(path: &Path) -> Result<Env, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let mut env = Env::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            if is_valid_ident(key) {
                if let Ok(v) = val.parse::<f64>() {
                    env.insert(key.to_string(), v);
                }
            }
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
        Some(c) if c.is_alphabetic() || c == '_' => {
            chars.all(|c| c.is_alphanumeric() || c == '_')
        }
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
        env.insert("x".to_string(), 42.0);
        env.insert("y".to_string(), -3.14);
        env.insert("ans".to_string(), 10.0);
        save_workspace(&env, &path).unwrap();

        let loaded = load_workspace(&path).unwrap();
        assert_eq!(loaded.get("x"), Some(&42.0));
        assert_eq!(loaded.get("y"), Some(&-3.14));
        assert_eq!(loaded.get("ans"), Some(&10.0));
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
        assert_eq!(env.get("x"), Some(&5.0));
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
}
