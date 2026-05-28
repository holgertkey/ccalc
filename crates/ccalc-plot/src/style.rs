//! MATLAB-style plot style string parsing.
//!
//! A style string combines an optional color code, an optional marker code,
//! and an optional line-style code in any order:
//!
//! | Code | Meaning |
//! |------|---------|
//! | `r` `g` `b` `c` `m` `y` `k` `w` | Color |
//! | `.` `o` `x` `+` `*` `s` `d` `^` | Marker |
//! | `-` `--` `-.` `:` | Line style |

/// RGB color triple (red, green, blue).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleColor(pub u8, pub u8, pub u8);

/// Marker symbol kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerKind {
    /// Small dot (`.`).
    Dot,
    /// Circle (`o`).
    Circle,
    /// Cross / × (`x`).
    Cross,
    /// Plus / + (`+`).
    Plus,
    /// Asterisk (`*`).
    Star,
    /// Square (`s`).
    Square,
    /// Diamond (`d`).
    Diamond,
    /// Triangle / up-arrow (`^`).
    Triangle,
}

/// Line drawing style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinestyleKind {
    /// Continuous line (`-`).
    Solid,
    /// Dashed line (`--`).
    Dashed,
    /// Dotted line (`:`).
    Dotted,
    /// Dash-dot alternating line (`-.`).
    DashDot,
}

/// Coordinated colour preset for a figure.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Background fill colour.
    pub bg: StyleColor,
    /// Title and axis-label text colour.
    pub text: StyleColor,
    /// Axis line and tick colour.
    pub axis: StyleColor,
    /// Bold (major) grid line colour.
    pub grid_bold: StyleColor,
    /// Light (minor) grid line colour.
    pub grid_light: StyleColor,
}

impl Theme {
    /// Returns the built-in light theme (white background, black text).
    pub fn light() -> Self {
        Theme {
            bg: StyleColor(255, 255, 255),
            text: StyleColor(0, 0, 0),
            axis: StyleColor(0, 0, 0),
            grid_bold: StyleColor(180, 180, 180),
            grid_light: StyleColor(220, 220, 220),
        }
    }

    /// Returns the built-in dark theme (Catppuccin Mocha palette).
    pub fn dark() -> Self {
        Theme {
            bg: StyleColor(0x1E, 0x1E, 0x2E),
            text: StyleColor(0xCD, 0xD6, 0xF4),
            axis: StyleColor(0x6C, 0x70, 0x86),
            grid_bold: StyleColor(0x45, 0x47, 0x5A),
            grid_light: StyleColor(0x31, 0x32, 0x44),
        }
    }

    /// Looks up a theme by name (`"light"` or `"dark"`), case-insensitive.
    ///
    /// Returns `Err` for unrecognised names.
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name.to_ascii_lowercase().as_str() {
            "light" => Ok(Theme::light()),
            "dark" => Ok(Theme::dark()),
            other => Err(format!(
                "theme: unknown theme '{other}' — expected 'light' or 'dark'"
            )),
        }
    }
}

/// Axis display mode set via `axis(...)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisMode {
    /// Equal scaling: same data-units per pixel on both axes.
    Equal,
    /// Tight: no margin added around the data range.
    Tight,
    /// Hidden: axis lines and tick labels are not drawn.
    Off,
}

/// Active Y axis for new series in a dual-axis (`yyaxis`) figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum YAxis {
    /// Primary (left) Y axis — the default.
    #[default]
    Left,
    /// Secondary (right) Y axis.
    Right,
}

/// Combined plot style parsed from a MATLAB-style format string.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleSpec {
    /// Optional override color; `None` uses the series default.
    pub color: Option<StyleColor>,
    /// Optional marker symbol; `None` draws no marker.
    pub marker: Option<MarkerKind>,
    /// Line style; defaults to [`LinestyleKind::Solid`].
    pub linestyle: LinestyleKind,
    /// Stroke width in pixels; `None` falls back to the session or hardcoded default (1).
    pub line_width: Option<f32>,
    /// Marker radius in pixels; `None` falls back to the session or hardcoded default (3).
    pub marker_size: Option<u32>,
}

impl Default for StyleSpec {
    fn default() -> Self {
        StyleSpec {
            color: None,
            marker: None,
            linestyle: LinestyleKind::Solid,
            line_width: None,
            marker_size: None,
        }
    }
}

/// Tries to parse `token` as a color: single letter, full name, or `#RRGGBB` hex.
///
/// Returns `None` for unrecognised tokens.
pub fn parse_color_token(token: &str) -> Option<StyleColor> {
    match token.to_ascii_lowercase().as_str() {
        "r" | "red" => Some(StyleColor(255, 0, 0)),
        "g" | "green" => Some(StyleColor(0, 128, 0)),
        "b" | "blue" => Some(StyleColor(0, 0, 255)),
        "c" | "cyan" => Some(StyleColor(0, 255, 255)),
        "m" | "magenta" => Some(StyleColor(255, 0, 255)),
        "y" | "yellow" => Some(StyleColor(255, 255, 0)),
        "k" | "black" => Some(StyleColor(0, 0, 0)),
        "w" | "white" => Some(StyleColor(255, 255, 255)),
        "orange" => Some(StyleColor(255, 165, 0)),
        "purple" => Some(StyleColor(128, 0, 128)),
        "gray" | "grey" => Some(StyleColor(128, 128, 128)),
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).ok()?;
            let g = u8::from_str_radix(&s[3..5], 16).ok()?;
            let b = u8::from_str_radix(&s[5..7], 16).ok()?;
            Some(StyleColor(r, g, b))
        }
        _ => None,
    }
}

/// Returns `true` when `s` looks like a MATLAB-style format string.
///
/// Accepts the classic single-char set (`r`, `g`, `--`, `.`, …), full color
/// names (`red`, `orange`, …), and hex codes (`#RRGGBB`).
pub fn looks_like_style_str(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.starts_with('#') {
        return s.len() == 7;
    }
    if parse_color_token(s).is_some() {
        return true;
    }
    s.chars().all(|c| "rgbcmykw.-:osx+*d^".contains(c))
}

/// Parses a MATLAB-style format string into a [`StyleSpec`].
///
/// Returns an error for unrecognised characters. An empty string returns the
/// default spec (solid line, no color override, no marker).
///
/// # Examples
///
/// ```
/// use ccalc_plot::style::{parse_style_str, LinestyleKind, MarkerKind, StyleColor};
///
/// let spec = parse_style_str("r--").unwrap();
/// assert_eq!(spec.color, Some(StyleColor(255, 0, 0)));
/// assert_eq!(spec.linestyle, LinestyleKind::Dashed);
/// assert_eq!(spec.marker, None);
///
/// let spec2 = parse_style_str("b.").unwrap();
/// assert_eq!(spec2.color, Some(StyleColor(0, 0, 255)));
/// assert_eq!(spec2.marker, Some(MarkerKind::Dot));
/// assert_eq!(spec2.linestyle, LinestyleKind::Solid);
/// ```
pub fn parse_style_str(s: &str) -> Result<StyleSpec, String> {
    if s.is_empty() {
        return Ok(StyleSpec::default());
    }

    // Full color name or '#RRGGBB' hex — whole string is just a color.
    if let Some(sc) = parse_color_token(s) {
        return Ok(StyleSpec {
            color: Some(sc),
            ..StyleSpec::default()
        });
    }

    let mut spec = StyleSpec::default();
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Try 2-char linestyle patterns first (greedy).
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'-' {
            spec.linestyle = LinestyleKind::Dashed;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'.' {
            spec.linestyle = LinestyleKind::DashDot;
            i += 2;
            continue;
        }

        match bytes[i] {
            b'-' => spec.linestyle = LinestyleKind::Solid,
            b':' => spec.linestyle = LinestyleKind::Dotted,
            b'.' => spec.marker = Some(MarkerKind::Dot),
            b'o' => spec.marker = Some(MarkerKind::Circle),
            b'x' => spec.marker = Some(MarkerKind::Cross),
            b'+' => spec.marker = Some(MarkerKind::Plus),
            b'*' => spec.marker = Some(MarkerKind::Star),
            b's' => spec.marker = Some(MarkerKind::Square),
            b'd' => spec.marker = Some(MarkerKind::Diamond),
            b'^' => spec.marker = Some(MarkerKind::Triangle),
            b'r' => spec.color = Some(StyleColor(255, 0, 0)),
            b'g' => spec.color = Some(StyleColor(0, 128, 0)),
            b'b' => spec.color = Some(StyleColor(0, 0, 255)),
            b'c' => spec.color = Some(StyleColor(0, 255, 255)),
            b'm' => spec.color = Some(StyleColor(255, 0, 255)),
            b'y' => spec.color = Some(StyleColor(255, 255, 0)),
            b'k' => spec.color = Some(StyleColor(0, 0, 0)),
            b'w' => spec.color = Some(StyleColor(255, 255, 255)),
            other => {
                return Err(format!(
                    "plot: unknown style character '{}' in style string '{s}'",
                    other as char
                ));
            }
        }
        i += 1;
    }

    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_red_dashed() {
        let spec = parse_style_str("r--").unwrap();
        assert_eq!(spec.color, Some(StyleColor(255, 0, 0)));
        assert_eq!(spec.linestyle, LinestyleKind::Dashed);
        assert_eq!(spec.marker, None);
    }

    #[test]
    fn test_parse_blue_dot() {
        let spec = parse_style_str("b.").unwrap();
        assert_eq!(spec.color, Some(StyleColor(0, 0, 255)));
        assert_eq!(spec.marker, Some(MarkerKind::Dot));
        assert_eq!(spec.linestyle, LinestyleKind::Solid);
    }

    #[test]
    fn test_parse_green_solid() {
        let spec = parse_style_str("g-").unwrap();
        assert_eq!(spec.color, Some(StyleColor(0, 128, 0)));
        assert_eq!(spec.linestyle, LinestyleKind::Solid);
        assert_eq!(spec.marker, None);
    }

    #[test]
    fn test_parse_dashdot() {
        let spec = parse_style_str("-.").unwrap();
        assert_eq!(spec.linestyle, LinestyleKind::DashDot);
        assert_eq!(spec.marker, None);
    }

    #[test]
    fn test_parse_dot_then_solid() {
        // '.' is a marker; '-' that follows is a solid linestyle (not dashdot)
        let spec = parse_style_str(".-").unwrap();
        assert_eq!(spec.marker, Some(MarkerKind::Dot));
        assert_eq!(spec.linestyle, LinestyleKind::Solid);
    }

    #[test]
    fn test_parse_dotted_line() {
        let spec = parse_style_str(":").unwrap();
        assert_eq!(spec.linestyle, LinestyleKind::Dotted);
    }

    #[test]
    fn test_parse_empty_returns_default() {
        let spec = parse_style_str("").unwrap();
        assert_eq!(spec, StyleSpec::default());
    }

    #[test]
    fn test_parse_unknown_char_errors() {
        let result = parse_style_str("xyz");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown style character"));
    }

    #[test]
    fn test_looks_like_style_str_valid() {
        assert!(looks_like_style_str("r--"));
        assert!(looks_like_style_str("b."));
        assert!(looks_like_style_str("g-"));
        assert!(looks_like_style_str("ko"));
    }

    #[test]
    fn test_looks_like_style_str_invalid() {
        assert!(!looks_like_style_str(""));
        assert!(!looks_like_style_str("time"));
        assert!(!looks_like_style_str("file.svg"));
    }

    #[test]
    fn test_style_full_name_red() {
        let spec = parse_style_str("red").unwrap();
        assert_eq!(spec.color, Some(StyleColor(255, 0, 0)));
        assert_eq!(spec.marker, None);
        assert_eq!(spec.linestyle, LinestyleKind::Solid);
    }

    #[test]
    fn test_style_full_name_orange() {
        let spec = parse_style_str("orange").unwrap();
        assert_eq!(spec.color, Some(StyleColor(255, 165, 0)));
    }

    #[test]
    fn test_style_gray_grey_alias() {
        let spec_gray = parse_style_str("gray").unwrap();
        let spec_grey = parse_style_str("grey").unwrap();
        assert_eq!(spec_gray.color, spec_grey.color);
        assert_eq!(spec_gray.color, Some(StyleColor(128, 128, 128)));
    }

    #[test]
    fn test_style_hex_color() {
        let spec = parse_style_str("#1A2B3C").unwrap();
        assert_eq!(spec.color, Some(StyleColor(0x1A, 0x2B, 0x3C)));
    }

    #[test]
    fn test_style_hex_bad_format() {
        // Too short (6 chars instead of 7) — not a valid hex, falls through to
        // char-by-char where '#' is an unrecognised character.
        let result = parse_style_str("#1A2B3");
        assert!(result.is_err(), "short hex should error");
    }
}
