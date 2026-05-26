use std::collections::HashSet;

/// Control-flow keywords highlighted in keyword colour.
pub const KEYWORDS: &[&str] = &[
    "break",
    "case",
    "catch",
    "continue",
    "do",
    "else",
    "elseif",
    "end",
    "for",
    "function",
    "global",
    "if",
    "otherwise",
    "persistent",
    "return",
    "switch",
    "try",
    "until",
    "while",
];

/// Pre-resolved ANSI colour strings for each syntax category.
///
/// Each field stores the escape sequence directly (e.g. `"\x1b[33m"`) so that
/// `highlight_line` can concatenate strings without reformatting on every
/// keystroke.
pub struct ColorScheme {
    /// Colour for control-flow keywords (`if`, `for`, `while`, …).
    pub keywords: String,
    /// Colour for numeric literals.
    pub numbers: String,
    /// Colour for string literals (`'...'` and `"..."`).
    pub strings: String,
    /// Colour for comments (`%` and `#` to end of line).
    pub comments: String,
    /// Colour for built-in function names.
    pub builtins: String,
    /// Colour for unclosed strings.
    pub errors: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            keywords: resolve_color("yellow").unwrap_or_else(|| "\x1b[33m".to_string()),
            numbers: resolve_color("cyan").unwrap_or_else(|| "\x1b[36m".to_string()),
            strings: resolve_color("green").unwrap_or_else(|| "\x1b[32m".to_string()),
            comments: resolve_color("dark_gray").unwrap_or_else(|| "\x1b[90m".to_string()),
            builtins: resolve_color("bright_cyan").unwrap_or_else(|| "\x1b[96m".to_string()),
            errors: resolve_color("red").unwrap_or_else(|| "\x1b[31m".to_string()),
        }
    }
}

/// Translates a colour name or spec to an ANSI foreground escape sequence.
///
/// Supported formats:
/// - Named 4-bit: `"yellow"`, `"bright_cyan"`, `"dark_gray"`, …
/// - 8-bit palette: `"color256(N)"` where N = 0–255
/// - 24-bit truecolor: `"#RRGGBB"`
/// - Bold prefix: `"bold:yellow"`, `"bold:#FF0000"`, …
///
/// Returns `None` for unknown or malformed inputs.
pub fn resolve_color(s: &str) -> Option<String> {
    let (bold, rest) = match s.strip_prefix("bold:") {
        Some(r) => (true, r),
        None => (false, s),
    };

    let named_code: Option<&str> = match rest {
        "black" => Some("30"),
        "red" => Some("31"),
        "green" => Some("32"),
        "yellow" => Some("33"),
        "blue" => Some("34"),
        "magenta" => Some("35"),
        "cyan" => Some("36"),
        "white" => Some("37"),
        "dark_gray" | "bright_black" => Some("90"),
        "bright_red" => Some("91"),
        "bright_green" => Some("92"),
        "bright_yellow" => Some("93"),
        "bright_blue" => Some("94"),
        "bright_magenta" => Some("95"),
        "bright_cyan" => Some("96"),
        "bright_white" => Some("97"),
        _ => None,
    };

    if let Some(c) = named_code {
        return Some(if bold {
            format!("\x1b[1;{c}m")
        } else {
            format!("\x1b[{c}m")
        });
    }

    // color256(N)
    if let Some(Ok(n)) = rest
        .strip_prefix("color256(")
        .and_then(|s| s.strip_suffix(')'))
        .map(|s| s.parse::<u8>())
    {
        return Some(if bold {
            format!("\x1b[1;38;5;{n}m")
        } else {
            format!("\x1b[38;5;{n}m")
        });
    }

    // #RRGGBB
    if rest.starts_with('#') && rest.len() == 7 && rest[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        let rv = u8::from_str_radix(&rest[1..3], 16).ok()?;
        let gv = u8::from_str_radix(&rest[3..5], 16).ok()?;
        let bv = u8::from_str_radix(&rest[5..7], 16).ok()?;
        return Some(if bold {
            format!("\x1b[1;38;2;{rv};{gv};{bv}m")
        } else {
            format!("\x1b[38;2;{rv};{gv};{bv}m")
        });
    }

    None
}

/// Applies syntax highlighting to `line`, returning a new string with ANSI
/// escape codes injected.
///
/// - `env_keys` — variable names currently defined in the workspace.  An
///   identifier that shadows a keyword or built-in is coloured in the default
///   terminal colour instead.
/// - `builtin_keys` — names of all built-in functions (engine + plugins).
///
/// Colour categories: keywords (yellow), numbers (cyan), strings (green),
/// comments (dark gray), built-in functions (bright cyan), unclosed strings
/// (red).
pub fn highlight_line(
    line: &str,
    env_keys: &HashSet<String>,
    builtin_keys: &HashSet<String>,
    colors: &ColorScheme,
) -> String {
    if line.is_empty() {
        return String::new();
    }

    const RESET: &str = "\x1b[0m";
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();

    // Coloured spans: (start_char_idx, end_char_idx, ansi_code).
    // Text between spans is rendered in the default terminal colour (no code).
    let mut spans: Vec<(usize, usize, &str)> = Vec::new();

    // Tracks whether the previous meaningful token was a "value" (identifier,
    // number, ), ]) so a following `'` is treated as a transpose operator
    // rather than the start of a char-array literal.  Mirrors the tokenizer
    // rule from Phase 9.
    #[derive(Clone, Copy, PartialEq)]
    enum Prev {
        Value,
        Other,
    }
    let mut prev = Prev::Other;

    // Build keyword lookup once (small, static list).
    let keyword_set: HashSet<&str> = KEYWORDS.iter().copied().collect();

    let mut i = 0;
    while i < n {
        let c = chars[i];

        // ── Comment (% or #) ──────────────────────────────────────────────
        if c == '%' || c == '#' {
            spans.push((i, n, colors.comments.as_str()));
            break;
        }

        // ── Double-quoted string ──────────────────────────────────────────
        if c == '"' {
            let start = i;
            i += 1;
            let mut closed = false;
            while i < n {
                match chars[i] {
                    // Backslash escape — skip next char (handles \" etc.)
                    '\\' => {
                        i += (n - i).min(2);
                    }
                    '"' => {
                        i += 1;
                        closed = true;
                        break;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            let color = if closed {
                colors.strings.as_str()
            } else {
                colors.errors.as_str()
            };
            spans.push((start, i, color));
            prev = Prev::Value;
            continue;
        }

        // ── Single-quoted: transpose or char-array literal ────────────────
        if c == '\'' {
            if prev == Prev::Value {
                // Transpose operator — no special colour.
                i += 1;
                // After transpose the result is still a value (A'' is valid).
                prev = Prev::Value;
                continue;
            }
            // Char-array literal.
            let start = i;
            i += 1;
            let mut closed = false;
            while i < n {
                if chars[i] == '\'' {
                    if i + 1 < n && chars[i + 1] == '\'' {
                        i += 2; // '' is an escaped single quote inside the string
                    } else {
                        i += 1;
                        closed = true;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            let color = if closed {
                colors.strings.as_str()
            } else {
                colors.errors.as_str()
            };
            spans.push((start, i, color));
            prev = Prev::Value;
            continue;
        }

        // ── Number starting with a digit ──────────────────────────────────
        if c.is_ascii_digit() {
            let start = i;
            if c == '0' && i + 1 < n && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                // Hexadecimal literal 0x…
                i += 2;
                while i < n && chars[i].is_ascii_hexdigit() {
                    i += 1;
                }
            } else {
                // Integer part
                while i < n && chars[i].is_ascii_digit() {
                    i += 1;
                }
                // Optional decimal part.  A '.' followed by .*, ./, .^, or ..
                // belongs to a different token, so we do not absorb it.
                if i < n && chars[i] == '.' {
                    let absorb = !matches!(
                        chars.get(i + 1).copied(),
                        Some('*') | Some('/') | Some('^') | Some('.')
                    );
                    if absorb {
                        i += 1;
                        while i < n && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                }
                // Optional exponent: e/E [+/-] digit+
                if i < n && (chars[i] == 'e' || chars[i] == 'E') {
                    let save = i;
                    i += 1;
                    if i < n && (chars[i] == '+' || chars[i] == '-') {
                        i += 1;
                    }
                    if i < n && chars[i].is_ascii_digit() {
                        while i < n && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    } else {
                        i = save; // Not a valid exponent — backtrack.
                    }
                }
                // Optional imaginary suffix i/j (only when not followed by alnum/underscore).
                if i < n && (chars[i] == 'i' || chars[i] == 'j') {
                    let after = chars.get(i + 1).copied().unwrap_or(' ');
                    if !after.is_alphanumeric() && after != '_' {
                        i += 1;
                    }
                }
            }
            spans.push((start, i, colors.numbers.as_str()));
            prev = Prev::Value;
            continue;
        }

        // ── Number starting with '.' (.5, .314e1) ────────────────────────
        if c == '.'
            && chars
                .get(i + 1)
                .copied()
                .is_some_and(|ch| ch.is_ascii_digit())
        {
            let start = i;
            i += 1; // consume '.'
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
            // Optional exponent
            if i < n && (chars[i] == 'e' || chars[i] == 'E') {
                let save = i;
                i += 1;
                if i < n && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                if i < n && chars[i].is_ascii_digit() {
                    while i < n && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                } else {
                    i = save;
                }
            }
            spans.push((start, i, colors.numbers.as_str()));
            prev = Prev::Value;
            continue;
        }

        // ── Identifier: keyword, built-in, or user variable ───────────────
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let shadowed = env_keys.contains(&word);

            if keyword_set.contains(word.as_str()) && !shadowed {
                spans.push((start, i, colors.keywords.as_str()));
                // Keywords are not values for the purpose of ' disambiguation.
                prev = Prev::Other;
            } else if builtin_keys.contains(&word) && !shadowed {
                spans.push((start, i, colors.builtins.as_str()));
                prev = Prev::Value;
            } else {
                // User variable or unknown identifier — default terminal colour.
                prev = Prev::Value;
            }
            continue;
        }

        // ── Closing bracket/paren — next ' could be transpose ────────────
        if matches!(c, ')' | ']' | '}') {
            prev = Prev::Value;
            i += 1;
            continue;
        }

        // ── Opening bracket/paren ─────────────────────────────────────────
        if matches!(c, '(' | '[' | '{') {
            prev = Prev::Other;
            i += 1;
            continue;
        }

        // ── Whitespace — prev is preserved ───────────────────────────────
        if c == ' ' || c == '\t' {
            i += 1;
            continue;
        }

        // ── All other characters (operators, punctuation) ─────────────────
        prev = Prev::Other;
        i += 1;
    }

    // ── Render: interleave coloured spans with default-colour gaps ─────────
    let mut out = String::with_capacity(line.len() + spans.len() * 14);
    let mut pos = 0usize;

    for (start, end, color) in &spans {
        if pos < *start {
            out.extend(chars[pos..*start].iter().copied());
        }
        out.push_str(color);
        out.extend(chars[*start..*end].iter().copied());
        out.push_str(RESET);
        pos = *end;
    }
    if pos < n {
        out.extend(chars[pos..].iter().copied());
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env() -> HashSet<String> {
        HashSet::new()
    }

    fn no_builtins() -> HashSet<String> {
        HashSet::new()
    }

    fn with_builtins(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn with_env(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// Strips ANSI escape sequences from `s` (everything between ESC and `m`).
    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut in_esc = false;
        for c in s.chars() {
            if in_esc {
                if c == 'm' {
                    in_esc = false;
                }
            } else if c == '\x1b' {
                in_esc = true;
            } else {
                out.push(c);
            }
        }
        out
    }

    // ── resolve_color ─────────────────────────────────────────────────────

    #[test]
    fn resolve_named_color() {
        assert_eq!(resolve_color("yellow"), Some("\x1b[33m".to_string()));
        assert_eq!(resolve_color("cyan"), Some("\x1b[36m".to_string()));
        assert_eq!(resolve_color("bright_cyan"), Some("\x1b[96m".to_string()));
        assert_eq!(resolve_color("dark_gray"), Some("\x1b[90m".to_string()));
        assert_eq!(resolve_color("red"), Some("\x1b[31m".to_string()));
        assert_eq!(resolve_color("green"), Some("\x1b[32m".to_string()));
    }

    #[test]
    fn resolve_bold_prefix() {
        assert_eq!(resolve_color("bold:yellow"), Some("\x1b[1;33m".to_string()));
        assert_eq!(
            resolve_color("bold:bright_cyan"),
            Some("\x1b[1;96m".to_string())
        );
    }

    #[test]
    fn resolve_color256() {
        assert_eq!(
            resolve_color("color256(220)"),
            Some("\x1b[38;5;220m".to_string())
        );
        assert_eq!(
            resolve_color("color256(0)"),
            Some("\x1b[38;5;0m".to_string())
        );
        assert_eq!(
            resolve_color("color256(255)"),
            Some("\x1b[38;5;255m".to_string())
        );
    }

    #[test]
    fn resolve_rgb_color() {
        assert_eq!(
            resolve_color("#FF8800"),
            Some("\x1b[38;2;255;136;0m".to_string())
        );
        assert_eq!(
            resolve_color("#000000"),
            Some("\x1b[38;2;0;0;0m".to_string())
        );
    }

    #[test]
    fn resolve_unknown_returns_none() {
        assert_eq!(resolve_color("neon_purple"), None);
        assert_eq!(resolve_color(""), None);
        assert_eq!(resolve_color("#GGGGGG"), None);
        assert_eq!(resolve_color("color256(999)"), None);
    }

    // ── highlight_line ────────────────────────────────────────────────────

    #[test]
    fn empty_line_returns_empty() {
        let colors = ColorScheme::default();
        assert_eq!(highlight_line("", &no_env(), &no_builtins(), &colors), "");
    }

    #[test]
    fn plain_text_unchanged() {
        let colors = ColorScheme::default();
        let result = highlight_line("x + y", &no_env(), &no_builtins(), &colors);
        assert_eq!(strip_ansi(&result), "x + y");
        // No ANSI codes when there's nothing to highlight
        assert!(!result.contains('\x1b'));
    }

    #[test]
    fn keyword_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("if x > 0", &no_env(), &no_builtins(), &colors);
        assert!(
            result.contains(&colors.keywords),
            "'if' should be colored with keyword color"
        );
        assert_eq!(strip_ansi(&result), "if x > 0");
    }

    #[test]
    fn multiple_keywords() {
        let colors = ColorScheme::default();
        let result = highlight_line("for k = 1:10", &no_env(), &no_builtins(), &colors);
        assert!(result.contains(&colors.keywords));
        assert_eq!(strip_ansi(&result), "for k = 1:10");
    }

    #[test]
    fn builtin_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("sin(x)", &no_env(), &with_builtins(&["sin"]), &colors);
        assert!(
            result.contains(&colors.builtins),
            "'sin' should be colored as builtin"
        );
        assert!(!result.contains(&colors.keywords));
        assert_eq!(strip_ansi(&result), "sin(x)");
    }

    #[test]
    fn number_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("3.14 + 0", &no_env(), &no_builtins(), &colors);
        assert!(
            result.contains(&colors.numbers),
            "numeric literals should be colored"
        );
        assert_eq!(strip_ansi(&result), "3.14 + 0");
    }

    #[test]
    fn hex_number_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("0xFF + 1", &no_env(), &no_builtins(), &colors);
        assert!(result.contains(&colors.numbers));
        assert_eq!(strip_ansi(&result), "0xFF + 1");
    }

    #[test]
    fn imaginary_number_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("3 + 4i", &no_env(), &no_builtins(), &colors);
        assert!(result.contains(&colors.numbers));
        assert_eq!(strip_ansi(&result), "3 + 4i");
    }

    #[test]
    fn dot_number_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line(".5 + .3e2", &no_env(), &no_builtins(), &colors);
        assert!(result.contains(&colors.numbers));
        assert_eq!(strip_ansi(&result), ".5 + .3e2");
    }

    #[test]
    fn closed_single_string_green() {
        let colors = ColorScheme::default();
        let result = highlight_line("'hello'", &no_env(), &no_builtins(), &colors);
        assert!(
            result.contains(&colors.strings),
            "closed single-quoted string should be string color"
        );
        assert!(
            !result.contains(&colors.errors),
            "closed string should not be error color"
        );
        assert_eq!(strip_ansi(&result), "'hello'");
    }

    #[test]
    fn unclosed_single_string_red() {
        let colors = ColorScheme::default();
        let result = highlight_line("'hello", &no_env(), &no_builtins(), &colors);
        assert!(
            result.contains(&colors.errors),
            "unclosed single-quoted string should be error color"
        );
        assert_eq!(strip_ansi(&result), "'hello");
    }

    #[test]
    fn closed_double_string_green() {
        let colors = ColorScheme::default();
        let result = highlight_line("\"hello\"", &no_env(), &no_builtins(), &colors);
        assert!(result.contains(&colors.strings));
        assert!(!result.contains(&colors.errors));
        assert_eq!(strip_ansi(&result), "\"hello\"");
    }

    #[test]
    fn unclosed_double_string_red() {
        let colors = ColorScheme::default();
        let result = highlight_line("\"hello", &no_env(), &no_builtins(), &colors);
        assert!(result.contains(&colors.errors));
        assert_eq!(strip_ansi(&result), "\"hello");
    }

    #[test]
    fn comment_percent_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("x + 1 % a comment", &no_env(), &no_builtins(), &colors);
        assert!(
            result.contains(&colors.comments),
            "% comment should be colored"
        );
        assert_eq!(strip_ansi(&result), "x + 1 % a comment");
    }

    #[test]
    fn comment_hash_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("x # comment", &no_env(), &no_builtins(), &colors);
        assert!(
            result.contains(&colors.comments),
            "# comment should be colored"
        );
    }

    #[test]
    fn transpose_not_string() {
        let colors = ColorScheme::default();
        let result = highlight_line("A'", &no_env(), &no_builtins(), &colors);
        // ' after identifier is transpose — should not be colored as string or error
        assert!(
            !result.contains(&colors.strings),
            "transpose ' should not be string color"
        );
        assert!(
            !result.contains(&colors.errors),
            "transpose ' should not be error color"
        );
    }

    #[test]
    fn double_transpose_not_string() {
        let colors = ColorScheme::default();
        let result = highlight_line("A''", &no_env(), &no_builtins(), &colors);
        assert!(!result.contains(&colors.strings));
        assert!(!result.contains(&colors.errors));
    }

    #[test]
    fn keyword_shadowed_by_env_not_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line("end", &with_env(&["end"]), &no_builtins(), &colors);
        assert!(
            !result.contains(&colors.keywords),
            "'end' in env should not be keyword color"
        );
    }

    #[test]
    fn builtin_shadowed_by_env_not_colored() {
        let colors = ColorScheme::default();
        let result = highlight_line(
            "sin",
            &with_env(&["sin"]),
            &with_builtins(&["sin"]),
            &colors,
        );
        assert!(
            !result.contains(&colors.builtins),
            "'sin' in env should not be builtin color"
        );
    }

    #[test]
    fn percent_inside_string_not_comment() {
        let colors = ColorScheme::default();
        let result = highlight_line("'hello % world'", &no_env(), &no_builtins(), &colors);
        // The % is inside the string, not a comment
        assert!(result.contains(&colors.strings));
        assert!(!result.contains(&colors.comments));
        assert_eq!(strip_ansi(&result), "'hello % world'");
    }

    #[test]
    fn string_after_operator_is_string() {
        let colors = ColorScheme::default();
        // After '=', the ' starts a char array
        let result = highlight_line("x = 'hello'", &no_env(), &no_builtins(), &colors);
        assert!(result.contains(&colors.strings));
        assert_eq!(strip_ansi(&result), "x = 'hello'");
    }

    #[test]
    fn text_preserved_after_stripping_ansi() {
        let colors = ColorScheme::default();
        let inputs = [
            "if x > 0",
            "sin(3.14)",
            "'hello world'",
            "A' + B",
            "% pure comment",
            "x = 0xFF",
            "function y = f(x)",
        ];
        for input in &inputs {
            let result = highlight_line(input, &no_env(), &no_builtins(), &colors);
            assert_eq!(
                strip_ansi(&result),
                *input,
                "text changed for input: {input}"
            );
        }
    }
}
