use std::path::Path;

use serde::{Deserialize, Serialize};

use ccalc_engine::eval::Base;

// ---------------------------------------------------------------------------
// Default config file template — written when no config.toml exists
// ---------------------------------------------------------------------------

const DEFAULT_CONFIG: &str = r#"# ccalc configuration
# Edit this file and run 'config reload' in the REPL to apply changes.

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
            eprintln!("Warning: could not create config file '{}': {e}", path.display());
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
}
