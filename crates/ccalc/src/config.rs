use std::path::Path;

use serde::{Deserialize, Serialize};

use ccalc_engine::eval::Base;

// ---------------------------------------------------------------------------
// Default config file template — written when no config.toml exists
// ---------------------------------------------------------------------------

const DEFAULT_CONFIG: &str = r#"# ccalc configuration
# Edit this file and run 'config reload' in the REPL to apply changes.

# Search path for run() / source() — directories checked after the current working directory.
# Tilde (~) is expanded to the home directory.
# Trailing slash means the directory AND all its subdirectories are added (genpath semantics).
# On Windows use forward slashes or escaped backslashes:
#   path = ["C:/Users/me/scripts", "D:/work/calc/"]
# path = ["~/.config/ccalc/lib/"]

[display]
# Default decimal precision (number of digits after the decimal point, 0–15).
precision = 10

# Default number base for output: "dec", "hex", "bin", "oct"
base = "dec"
"#;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    /// Directories added to the script search path at startup.
    #[serde(default)]
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub precision: usize,
    pub base: String,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("DEFAULT_CONFIG is valid TOML")
    }
}

impl Config {
    /// Decimal precision clamped to 0–15.
    pub fn precision(&self) -> usize {
        self.display.precision.min(15)
    }

    /// Parsed `Base` from the `base` string; falls back to `Base::Dec` for
    /// unknown values.
    pub fn base(&self) -> Base {
        match self.display.base.as_str() {
            "hex" => Base::Hex,
            "bin" => Base::Bin,
            "oct" => Base::Oct,
            _ => Base::Dec,
        }
    }

    /// Returns the search path as `PathBuf`s with `~` expanded.
    ///
    /// A trailing `/` or `\` triggers genpath semantics: the directory and all
    /// its subdirectories (recursively, sorted) are added to the path.
    pub fn search_path(&self) -> Vec<std::path::PathBuf> {
        let mut result = Vec::new();
        for s in &self.path {
            let recursive = s.ends_with('/') || s.ends_with('\\');
            let trimmed = if recursive { s.trim_end_matches(['/', '\\']) } else { s.as_str() };
            let expanded = expand_tilde(trimmed);
            let root = std::path::PathBuf::from(&expanded);
            if recursive {
                collect_dirs_recursive(&root, &mut result);
            } else {
                result.push(root);
            }
        }
        result
    }
}

fn collect_dirs_recursive(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if !root.is_dir() {
        return;
    }
    out.push(root.to_path_buf());
    if let Ok(entries) = std::fs::read_dir(root) {
        let mut children: Vec<std::path::PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        children.sort();
        for child in children {
            collect_dirs_recursive(&child, out);
        }
    }
}

fn expand_tilde(path: &str) -> String {
    if path == "~" || path.starts_with("~/") || path.starts_with("~\\") {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();
        if home.is_empty() {
            return path.to_string();
        }
        if path == "~" {
            home
        } else {
            format!("{}{}", home, &path[1..])
        }
    } else {
        path.to_string()
    }
}

// ---------------------------------------------------------------------------
// Load / create
// ---------------------------------------------------------------------------

/// Loads the config file at `path`.
///
/// - If the file does not exist, it is created with the default template and
///   the defaults are returned.
/// - If the file exists but cannot be parsed, an error is printed and the
///   defaults are returned.
pub fn load_or_create(path: &Path) -> Config {
    if !path.exists() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(path, DEFAULT_CONFIG) {
            eprintln!(
                "Warning: could not create config file '{}': {e}",
                path.display()
            );
        }
        return Config::default();
    }

    load(path).unwrap_or_else(|e| {
        eprintln!("Warning: could not read config '{}': {e}", path.display());
        Config::default()
    })
}

/// Reads and parses the config file. Returns an error string on failure.
pub fn load(path: &Path) -> Result<Config, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("parse error in '{}': {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_parses() {
        let cfg = Config::default();
        assert_eq!(cfg.precision(), 10);
        assert!(matches!(cfg.base(), Base::Dec));
    }

    #[test]
    fn load_or_create_makes_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert!(!path.exists());
        let cfg = load_or_create(&path);
        assert!(path.exists(), "config.toml should have been created");
        assert_eq!(cfg.precision(), 10);
        assert!(matches!(cfg.base(), Base::Dec));
    }

    #[test]
    fn load_custom_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[display]\nprecision = 4\nbase = \"hex\"\n").unwrap();
        let cfg = load(&path).unwrap();
        assert_eq!(cfg.precision(), 4);
        assert!(matches!(cfg.base(), Base::Hex));
    }

    #[test]
    fn precision_clamped_to_15() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[display]\nprecision = 99\nbase = \"dec\"\n").unwrap();
        let cfg = load(&path).unwrap();
        assert_eq!(cfg.precision(), 15);
    }

    #[test]
    fn unknown_base_falls_back_to_dec() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[display]\nprecision = 10\nbase = \"invalid\"\n").unwrap();
        let cfg = load(&path).unwrap();
        assert!(matches!(cfg.base(), Base::Dec));
    }

    #[test]
    fn search_path_loaded_from_top_level() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        // path must be at the TOML root, not under [display]
        std::fs::write(
            &path,
            "path = [\"/my/scripts\", \"/home/user/calc\"]\n\n[display]\nprecision = 10\nbase = \"dec\"\n",
        )
        .unwrap();
        let cfg = load(&path).unwrap();
        let sp = cfg.search_path();
        assert_eq!(sp.len(), 2);
        assert_eq!(sp[0], std::path::PathBuf::from("/my/scripts"));
        assert_eq!(sp[1], std::path::PathBuf::from("/home/user/calc"));
    }

    #[test]
    fn search_path_under_display_is_ignored() {
        // Regression: old DEFAULT_CONFIG placed the path comment under [display].
        // If a user uncommented it there, path was silently ignored.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[display]\nprecision = 10\nbase = \"dec\"\npath = [\"/wrong\"]\n",
        )
        .unwrap();
        // TOML parses this as display.path (unknown field) — serde should ignore it
        // and cfg.path (root level) stays empty.
        let cfg = load(&path).unwrap();
        assert!(cfg.search_path().is_empty());
    }

    #[test]
    fn search_path_trailing_slash_includes_subdirs() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();

        let path = dir.path().join("config.toml");
        let root_with_slash = format!("{}/", dir.path().to_string_lossy().replace('\\', "/"));
        std::fs::write(
            &path,
            format!(
                "path = [\"{root_with_slash}\"]\n\n[display]\nprecision = 10\nbase = \"dec\"\n"
            ),
        )
        .unwrap();
        let cfg = load(&path).unwrap();
        let sp = cfg.search_path();
        assert!(sp.len() >= 2, "root + at least one subdir expected");
        assert_eq!(sp[0], dir.path());
        assert!(sp.iter().any(|p| p == &sub));
    }

    #[test]
    fn search_path_no_trailing_slash_exact_only() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();

        let path = dir.path().join("config.toml");
        let root = dir.path().to_string_lossy().replace('\\', "/");
        std::fs::write(
            &path,
            format!("path = [\"{root}\"]\n\n[display]\nprecision = 10\nbase = \"dec\"\n"),
        )
        .unwrap();
        let cfg = load(&path).unwrap();
        let sp = cfg.search_path();
        assert_eq!(sp.len(), 1);
        assert_eq!(sp[0], dir.path());
    }

    #[test]
    fn search_path_windows_forward_slashes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        // Windows paths with forward slashes are valid TOML and valid PathBuf on Windows
        std::fs::write(
            &path,
            "path = [\"e:/github.com/holgertkey/ccalc/examples\"]\n\n[display]\nprecision = 10\nbase = \"dec\"\n",
        )
        .unwrap();
        let cfg = load(&path).unwrap();
        let sp = cfg.search_path();
        assert_eq!(sp.len(), 1);
        assert_eq!(
            sp[0],
            std::path::PathBuf::from("e:/github.com/holgertkey/ccalc/examples")
        );
    }
}
