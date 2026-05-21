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

/// Combined plot style parsed from a MATLAB-style format string.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleSpec {
    /// Optional override color; `None` uses the series default.
    pub color: Option<StyleColor>,
    /// Optional marker symbol; `None` draws no marker.
    pub marker: Option<MarkerKind>,
    /// Line style; defaults to [`LinestyleKind::Solid`].
    pub linestyle: LinestyleKind,
}

impl Default for StyleSpec {
    fn default() -> Self {
        StyleSpec {
            color: None,
            marker: None,
            linestyle: LinestyleKind::Solid,
        }
    }
}

/// Returns `true` when every character in `s` is a valid style character.
pub fn looks_like_style_str(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| "rgbcmykw.-:osx+*d^".contains(c))
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
}
