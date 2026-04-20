use crate::eval::{Expr, Op, expr_to_string};

/// Top-level statement returned by [`parse`] and [`parse_stmts`].
#[derive(Debug)]
pub enum Stmt {
    /// Variable assignment: `name = expr`
    Assign(String, Expr),
    /// Standalone expression — result goes into `ans`
    Expr(Expr),
    /// `if cond; body; elseif cond; ...; else; ...; end`
    If {
        /// The condition expression evaluated to decide which branch to take.
        cond: Expr,
        /// Statements to execute when `cond` is truthy.
        body: Vec<(Stmt, bool)>,
        /// Zero or more `elseif (cond) body` branches, in source order.
        elseif_branches: Vec<(Expr, Vec<(Stmt, bool)>)>,
        /// Statements to execute when no condition matched, or `None` if there is no `else`.
        else_body: Option<Vec<(Stmt, bool)>>,
    },
    /// `for var = range_expr; body; end` — iterates over columns of the range matrix
    For {
        /// The loop variable assigned on each iteration.
        var: String,
        /// Expression that produces the matrix whose columns are iterated.
        range_expr: Expr,
        /// Loop body statements.
        body: Vec<(Stmt, bool)>,
    },
    /// `while cond; body; end`
    While {
        /// Loop condition — re-evaluated before each iteration.
        cond: Expr,
        /// Loop body statements.
        body: Vec<(Stmt, bool)>,
    },
    /// `break` — exits the innermost enclosing loop
    Break,
    /// `continue` — advances to next iteration of the innermost enclosing loop
    Continue,
    /// `switch expr; case val; body; ...; otherwise; body; end`
    ///
    /// Each case carries a list of match expressions (single value today; cell-array
    /// multi-value is deferred to Phase 11.5b) and a statement body.
    /// `otherwise` is optional.
    #[allow(clippy::type_complexity)]
    Switch {
        /// The expression whose value is matched against each `case`.
        expr: Expr,
        /// Each case is a list of match patterns and the body to run on a match.
        cases: Vec<(Vec<Expr>, Vec<(Stmt, bool)>)>,
        /// Fallback body executed when no `case` matches, or `None` if there is no `otherwise`.
        otherwise_body: Option<Vec<(Stmt, bool)>>,
    },
    /// `do; body; until (cond)` — Octave-specific post-test loop.
    ///
    /// The body always executes at least once. `break` and `continue` work as in `while`.
    DoUntil {
        /// Loop body — always executed at least once before the condition is checked.
        body: Vec<(Stmt, bool)>,
        /// Condition tested after each iteration; loop exits when it becomes truthy.
        cond: Expr,
    },
    /// `function [outputs] = name(params) body end` — named user function definition.
    ///
    /// The body is stored as raw source text and re-parsed on each call by `exec.rs`.
    /// Named functions execute in an isolated scope (only params and built-in constants visible).
    FunctionDef {
        /// The function name (e.g. `"fib"` in `function y = fib(n)`).
        name: String,
        /// Output variable names in declaration order.
        outputs: Vec<String>,
        /// Parameter names in declaration order.
        params: Vec<String>,
        /// Raw source text of the function body, stored verbatim for re-parsing on each call.
        body_source: String,
    },
    /// `return` — exits the current function immediately.
    ///
    /// Inside a named function, `return` causes the function to return its current output
    /// variable values. At the top level it is treated as an error by `exec.rs`.
    Return,
    /// `[a, b] = f()` — multi-output assignment.
    ///
    /// Produced when the LHS is a bracket list of identifiers.
    /// The RHS must evaluate to a `Value::Tuple`; extra values are discarded,
    /// missing values produce an error.
    MultiAssign {
        /// The list of output variable names on the LHS (e.g. `["a", "b"]` in `[a, b] = f()`).
        targets: Vec<String>,
        /// The RHS expression — must evaluate to [`Value::Tuple`](crate::env::Value::Tuple).
        expr: Expr,
    },
    /// `try; body; catch [e]; catch_body; end` — protected block.
    ///
    /// If `catch_var` is `Some(name)`, the catch variable is bound to a struct
    /// with field `message` containing the error string.
    TryCatch {
        /// Statements in the protected `try` block.
        try_body: Vec<(Stmt, bool)>,
        /// Optional name of the catch variable bound to a struct with a `message` field.
        catch_var: Option<String>,
        /// Statements executed when an error is caught.
        catch_body: Vec<(Stmt, bool)>,
    },
    /// `c{i} = v` — cell element assignment.
    ///
    /// Updates element `i` (1-based) of the cell array named `name`.
    CellSet(String, Expr, Expr),
    /// `s.x = v` / `s.a.b = v` — struct field assignment.
    ///
    /// `FieldSet(base_var, field_path, rhs)`:
    /// - `base_var`: the top-level variable name (e.g. `"s"`).
    /// - `field_path`: one or more field names leading to the target (e.g. `["x"]` or `["a", "b"]`).
    /// - `rhs`: the value to store.
    ///
    /// At runtime the base variable is loaded (or a fresh empty struct is created),
    /// the path is walked (creating intermediate structs on demand), and the final
    /// field is set.
    FieldSet(String, Vec<String>, Expr),
    /// `s(i).field = v` / `s(i).a.b = v` — struct array element field assignment.
    ///
    /// `StructArrayFieldSet(base_var, idx_expr, field_path, rhs)`:
    /// - `base_var`: the top-level variable name (e.g. `"s"`).
    /// - `idx_expr`: the 1-based integer index expression (e.g. `1` or `i+1`).
    /// - `field_path`: one or more field names (e.g. `["x"]` or `["a", "b"]`).
    /// - `rhs`: the value to store.
    ///
    /// At runtime the struct array is loaded (or created), grown if necessary,
    /// and the field of element `idx` is set.
    StructArrayFieldSet(String, Expr, Vec<String>, Expr),
    /// `v(i) = x`, `A(i,j) = x`, `v(1:3) = [1 2 3]` — indexed assignment.
    ///
    /// Modifies one or more elements of an existing matrix (or creates/grows it).
    /// Index expressions follow the same syntax as the read-path (Phase 6):
    /// `:`, scalars, ranges, and logical masks (Phase 15d).
    /// A scalar RHS is broadcast to all selected positions.
    IndexSet {
        /// The variable name being modified.
        name: String,
        /// Index expressions (1 = linear, 2 = row/col).
        indices: Vec<Expr>,
        /// The value to write.
        value: Expr,
    },
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Ident(String),
    Str(String),       // 'text' char array literal
    StringObj(String), // "text" string object literal
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    DotStar,
    DotSlash,
    DotCaret,
    Apostrophe,
    LParen,
    RParen,
    Comma,
    LBracket,
    RBracket,
    Semicolon,
    Colon,
    // --- Compound assignment ---
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PlusPlus,   // ++
    MinusMinus, // --
    // --- Comparison ---
    EqEq,  // ==
    NotEq, // ~=
    Lt,    // <
    Gt,    // >
    LtEq,  // <=
    GtEq,  // >=
    // --- Logical ---
    AmpAmp,   // &&
    PipePipe, // ||
    Amp,      // & (element-wise AND)
    Pipe,     // | (element-wise OR)
    Tilde,    // ~ / ! (unary NOT)
    At,       // @ (lambda / function handle prefix)
    LBrace,   // {
    RBrace,   // }
    // --- Additional operators ---
    StarStar,      // ** (alias for ^)
    DotApostrophe, // .' (non-conjugate transpose)
    Backslash,     // \ (left division / linear solve)
    // --- Struct field access ---
    Dot, // '.' followed by an ASCII letter (field access)
}

fn parse_integer_literal(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    radix: u32,
    prefix: &str,
) -> Result<f64, String> {
    let mut digit_str = String::new();
    while let Some(&d) = chars.peek() {
        let valid = match radix {
            16 => d.is_ascii_hexdigit(),
            2 => d == '0' || d == '1',
            8 => ('0'..='7').contains(&d),
            _ => false,
        };
        if valid {
            digit_str.push(d);
            chars.next();
        } else {
            break;
        }
    }
    if digit_str.is_empty() {
        return Err(format!("Expected digits after '{prefix}'"));
    }
    i64::from_str_radix(&digit_str, radix)
        .map(|i| i as f64)
        .map_err(|_| format!("Invalid {prefix} literal: '{prefix}{digit_str}'"))
}

/// If the next chars look like a sci exponent (`e+5`, `E-3`, `e10`), consume and append them.
/// Uses a cloned iterator for lookahead — only advances the real iterator on a confirmed match.
fn try_consume_sci_exponent(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    num_str: &mut String,
) {
    if !matches!(chars.peek(), Some('e') | Some('E')) {
        return;
    }
    let mut lookahead = chars.clone();
    let e_char = lookahead.next().unwrap();
    match lookahead.peek().copied() {
        Some('+') | Some('-') => {
            let sign = lookahead.next().unwrap();
            if lookahead.peek().is_some_and(|d| d.is_ascii_digit()) {
                chars.next();
                chars.next();
                num_str.push(e_char);
                num_str.push(sign);
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num_str.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        }
        Some(d) if d.is_ascii_digit() => {
            chars.next();
            num_str.push(e_char);
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
        }
        _ => {}
    }
}

/// After parsing a decimal number, if the next char is `i` or `j` and NOT
/// followed by another identifier character, consume it and emit `* i` so
/// that `4i` → `4 * i` = `Complex(0, 4)`.
#[inline]
fn push_imag_suffix(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, tokens: &mut Vec<Token>) {
    if matches!(chars.peek(), Some('i') | Some('j')) {
        let mut la = chars.clone();
        la.next();
        if !la.peek().is_some_and(|c| c.is_alphanumeric() || *c == '_') {
            chars.next(); // consume i/j
            tokens.push(Token::Star);
            tokens.push(Token::Ident("i".to_string()));
        }
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '+' => {
                chars.next();
                match chars.peek() {
                    Some('=') => {
                        chars.next();
                        tokens.push(Token::PlusEq);
                    }
                    Some('+') => {
                        chars.next();
                        tokens.push(Token::PlusPlus);
                    }
                    _ => tokens.push(Token::Plus),
                }
            }
            '-' => {
                chars.next();
                match chars.peek() {
                    Some('=') => {
                        chars.next();
                        tokens.push(Token::MinusEq);
                    }
                    Some('-') => {
                        chars.next();
                        tokens.push(Token::MinusMinus);
                    }
                    _ => tokens.push(Token::Minus),
                }
            }
            '*' => {
                chars.next();
                match chars.peek() {
                    Some('=') => {
                        chars.next();
                        tokens.push(Token::StarEq);
                    }
                    Some('*') => {
                        chars.next();
                        tokens.push(Token::StarStar);
                    }
                    _ => tokens.push(Token::Star),
                }
            }
            '/' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::SlashEq);
                } else {
                    tokens.push(Token::Slash);
                }
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '\'' => {
                // Determine whether this is a transpose operator or a char array literal.
                // Transpose if preceded by a value-producing token (number, ident, ')', ']', or a previous apostrophe).
                let is_transpose = matches!(
                    tokens.last(),
                    Some(
                        Token::Number(_)
                            | Token::Ident(_)
                            | Token::RParen
                            | Token::RBracket
                            | Token::Apostrophe
                            | Token::Str(_)
                    )
                );
                chars.next(); // consume the opening '
                if is_transpose {
                    tokens.push(Token::Apostrophe);
                } else {
                    // Parse char array literal; '' inside is an escaped single quote.
                    let mut content = String::new();
                    loop {
                        match chars.next() {
                            None => return Err("Unterminated string literal".to_string()),
                            Some('\'') => {
                                // Check for escaped '' (two single quotes in a row)
                                if chars.peek().copied() == Some('\'') {
                                    chars.next();
                                    content.push('\'');
                                } else {
                                    break;
                                }
                            }
                            Some(c) => content.push(c),
                        }
                    }
                    tokens.push(Token::Str(content));
                }
            }
            '"' => {
                chars.next(); // consume the opening "
                let mut content = String::new();
                loop {
                    match chars.next() {
                        None => return Err("Unterminated string literal".to_string()),
                        Some('"') => {
                            // Check for escaped "" (two double quotes in a row)
                            if chars.peek().copied() == Some('"') {
                                chars.next();
                                content.push('"');
                            } else {
                                break;
                            }
                        }
                        Some('\\') => match chars.next() {
                            Some('n') => content.push('\n'),
                            Some('t') => content.push('\t'),
                            Some('\\') => content.push('\\'),
                            Some('\'') => content.push('\''),
                            Some('"') => content.push('"'),
                            Some(other) => {
                                content.push('\\');
                                content.push(other);
                            }
                            None => return Err("Unterminated string literal".to_string()),
                        },
                        Some(c) => content.push(c),
                    }
                }
                tokens.push(Token::StringObj(content));
            }
            '.' => {
                chars.next();
                match chars.peek().copied() {
                    Some('.') => {
                        // Could be '...' (line continuation)
                        chars.next(); // consume second '.'
                        if chars.peek() == Some(&'.') {
                            chars.next(); // consume third '.'
                            // Line continuation: treat rest of input as a comment
                            while chars.next().is_some() {}
                        } else {
                            return Err("Unexpected '..'".to_string());
                        }
                    }
                    Some('\'') => {
                        chars.next();
                        tokens.push(Token::DotApostrophe);
                    }
                    Some('*') => {
                        chars.next();
                        tokens.push(Token::DotStar);
                    }
                    Some('/') => {
                        chars.next();
                        tokens.push(Token::DotSlash);
                    }
                    Some('^') => {
                        chars.next();
                        tokens.push(Token::DotCaret);
                    }
                    Some(d) if d.is_ascii_digit() => {
                        let mut num_str = String::from(".");
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_digit() {
                                num_str.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        try_consume_sci_exponent(&mut chars, &mut num_str);
                        let n: f64 = num_str
                            .parse()
                            .map_err(|_| format!("Invalid number: '{num_str}'"))?;
                        tokens.push(Token::Number(n));
                    }
                    // Field access: '.' followed by an identifier letter/underscore.
                    // Don't consume the letter — it will be tokenized as Ident on the next pass.
                    Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                        tokens.push(Token::Dot);
                    }
                    _ => return Err("Unexpected '.'".to_string()),
                }
            }
            '%' | '#' => {
                // '%' / '#' start a comment — stop tokenizing
                break;
            }
            '!' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::NotEq);
                } else {
                    tokens.push(Token::Tilde);
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RBracket);
                chars.next();
            }
            '{' => {
                tokens.push(Token::LBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RBrace);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '=' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::EqEq);
                } else {
                    return Err("Unexpected '=': use '==' for comparison".to_string());
                }
            }
            '~' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::NotEq);
                } else {
                    tokens.push(Token::Tilde);
                }
            }
            '<' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::LtEq);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '>' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::GtEq);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            '&' => {
                chars.next();
                if chars.peek().copied() == Some('&') {
                    chars.next();
                    tokens.push(Token::AmpAmp);
                } else {
                    tokens.push(Token::Amp);
                }
            }
            '|' => {
                chars.next();
                if chars.peek().copied() == Some('|') {
                    chars.next();
                    tokens.push(Token::PipePipe);
                } else {
                    tokens.push(Token::Pipe);
                }
            }
            '0'..='9' => {
                if c == '0' {
                    chars.next();
                    match chars.peek().copied() {
                        Some('x') | Some('X') => {
                            chars.next();
                            let n = parse_integer_literal(&mut chars, 16, "0x")?;
                            tokens.push(Token::Number(n));
                        }
                        Some('b') | Some('B') => {
                            chars.next();
                            let n = parse_integer_literal(&mut chars, 2, "0b")?;
                            tokens.push(Token::Number(n));
                        }
                        Some('o') | Some('O') => {
                            chars.next();
                            let n = parse_integer_literal(&mut chars, 8, "0o")?;
                            tokens.push(Token::Number(n));
                        }
                        _ => {
                            let mut num_str = String::from("0");
                            while let Some(&d) = chars.peek() {
                                if d.is_ascii_digit() {
                                    num_str.push(d);
                                    chars.next();
                                } else if d == '.' {
                                    // Don't eat '.' if followed by *, /, ^ (element-wise ops)
                                    let mut la = chars.clone();
                                    la.next();
                                    if matches!(la.peek(), Some('*') | Some('/') | Some('^')) {
                                        break;
                                    }
                                    num_str.push('.');
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            try_consume_sci_exponent(&mut chars, &mut num_str);
                            let n: f64 = num_str
                                .parse()
                                .map_err(|_| format!("Invalid number: '{num_str}'"))?;
                            tokens.push(Token::Number(n));
                            push_imag_suffix(&mut chars, &mut tokens);
                        }
                    }
                } else {
                    let mut num_str = String::new();
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            num_str.push(d);
                            chars.next();
                        } else if d == '.' {
                            // Don't eat '.' if followed by *, /, ^ (element-wise ops)
                            let mut la = chars.clone();
                            la.next();
                            if matches!(la.peek(), Some('*') | Some('/') | Some('^')) {
                                break;
                            }
                            num_str.push('.');
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    try_consume_sci_exponent(&mut chars, &mut num_str);
                    let n: f64 = num_str
                        .parse()
                        .map_err(|_| format!("Invalid number: '{num_str}'"))?;
                    tokens.push(Token::Number(n));
                    push_imag_suffix(&mut chars, &mut tokens);
                }
            }
            '@' => {
                tokens.push(Token::At);
                chars.next();
            }
            '\\' => {
                tokens.push(Token::Backslash);
                chars.next();
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            _ => return Err(format!("Unexpected character: '{c}'")),
        }
    }

    Ok(tokens)
}

/// Detects `base(idx_expr).f1.f2...fn = rhs` at the string level (before tokenising).
///
/// Returns `Some((base, idx_str, fields, rhs))` on a match, `None` otherwise.
fn try_split_struct_array_field_assign(input: &str) -> Option<(String, &str, Vec<String>, &str)> {
    let trimmed = input.trim();
    let bytes = trimmed.as_bytes();
    let mut i = 0;

    // Parse leading identifier
    if i >= bytes.len() || !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
        return None;
    }
    let base_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    let base_var = trimmed[base_start..i].to_string();

    // Expect '('
    if i >= bytes.len() || bytes[i] != b'(' {
        return None;
    }
    i += 1;

    // Scan for the matching ')' (tracking nested parens/brackets/braces)
    let idx_start = i;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    if depth != 0 {
        return None;
    }
    let idx_str = &trimmed[idx_start..i - 1]; // exclude the closing ')'

    // Expect '.field' (at least one field access)
    if i >= bytes.len() || bytes[i] != b'.' {
        return None;
    }
    let mut fields = Vec::new();
    while i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        if i >= bytes.len() || !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
            return None;
        }
        let field_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        fields.push(trimmed[field_start..i].to_string());
    }
    if fields.is_empty() {
        return None;
    }

    // Skip optional whitespace then expect bare `=` (not `==`)
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'=' {
        return None;
    }
    i += 1;
    if i < bytes.len() && bytes[i] == b'=' {
        return None; // '==' comparison
    }

    let rhs = trimmed[i..].trim();
    if rhs.is_empty() {
        return None;
    }
    Some((base_var, idx_str, fields, rhs))
}

/// Detects `base.f1.f2...fn = rhs` at the string level (before tokenising).
///
/// Returns `Some((base, fields, rhs))` on a match, `None` otherwise.
/// The detection is done at string level to avoid tokenising twice.
fn try_split_field_assign(input: &str) -> Option<(String, Vec<String>, &str)> {
    let trimmed = input.trim();
    let bytes = trimmed.as_bytes();
    let mut i = 0;

    // Parse leading identifier (base variable name)
    if i >= bytes.len() || !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
        return None;
    }
    let base_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    let base_var = trimmed[base_start..i].to_string();

    // Parse one or more `.field` segments
    let mut fields = Vec::new();
    while i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        if i >= bytes.len() || !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
            return None;
        }
        let field_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        fields.push(trimmed[field_start..i].to_string());
    }
    if fields.is_empty() {
        return None;
    }

    // Skip optional whitespace then expect a bare `=` (not `==`)
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'=' {
        return None;
    }
    i += 1;
    if i < bytes.len() && bytes[i] == b'=' {
        return None; // '==' comparison
    }

    let rhs = trimmed[i..].trim();
    if rhs.is_empty() {
        return None;
    }
    Some((base_var, fields, rhs))
}

/// Parses a full input string into a [`Stmt`].
///
/// Assignment (`name = expr`) is detected first. Everything else is treated as
/// an expression whose result will be stored in `ans`.
pub fn parse(input: &str) -> Result<Stmt, String> {
    let trimmed = input.trim();

    // 'return' statement
    if trimmed == "return" {
        return Ok(Stmt::Return);
    }

    // Struct array element field assignment: name(idx).field[.field]* = rhs
    if let Some((base_var, idx_str, fields, rhs)) = try_split_struct_array_field_assign(trimmed) {
        let idx_tokens = tokenize(idx_str)?;
        if idx_tokens.is_empty() {
            return Err("Expected index expression inside '()'".to_string());
        }
        let mut idx_pos = 0;
        let idx_expr = parse_logical_or(&idx_tokens, &mut idx_pos)?;
        if idx_pos != idx_tokens.len() {
            return Err("Unexpected token in struct array index expression".to_string());
        }
        let rhs_tokens = tokenize(rhs)?;
        if rhs_tokens.is_empty() {
            return Err("Expected expression after '='".to_string());
        }
        let mut rhs_pos = 0;
        let rhs_expr = parse_logical_or(&rhs_tokens, &mut rhs_pos)?;
        if rhs_pos != rhs_tokens.len() {
            return Err("Unexpected token after expression".to_string());
        }
        return Ok(Stmt::StructArrayFieldSet(
            base_var, idx_expr, fields, rhs_expr,
        ));
    }

    // Indexed assignment: name(args) = rhs  (Phase 15)
    if let Some((name, idx_str, rhs)) = try_split_index_assign(trimmed) {
        let idx_tokens = tokenize(idx_str)?;
        let indices = parse_index_args(&idx_tokens)?;
        if indices.len() > 2 {
            return Err("Indexed assignment supports at most 2 indices".to_string());
        }
        let rhs_tokens = tokenize(rhs)?;
        if rhs_tokens.is_empty() {
            return Err("Expected expression after '='".to_string());
        }
        let mut rhs_pos = 0;
        let value = parse_logical_or(&rhs_tokens, &mut rhs_pos)?;
        if rhs_pos != rhs_tokens.len() {
            return Err("Unexpected token after expression".to_string());
        }
        return Ok(Stmt::IndexSet {
            name,
            indices,
            value,
        });
    }

    // Struct field assignment: name.field[.field]* = rhs
    if let Some((base_var, fields, rhs)) = try_split_field_assign(trimmed) {
        let tokens = tokenize(rhs)?;
        if tokens.is_empty() {
            return Err("Expected expression after '='".to_string());
        }
        let mut pos = 0;
        let rhs_expr = parse_logical_or(&tokens, &mut pos)?;
        if pos != tokens.len() {
            return Err("Unexpected token after expression".to_string());
        }
        return Ok(Stmt::FieldSet(base_var, fields, rhs_expr));
    }

    // Cell element assignment: name{expr} = rhs
    if let Some((name, idx_str, rhs)) = try_split_cell_assign(trimmed) {
        let idx_tokens = tokenize(idx_str)?;
        if idx_tokens.is_empty() {
            return Err("Expected index expression inside '{}'".to_string());
        }
        let mut idx_pos = 0;
        let idx_expr = parse_logical_or(&idx_tokens, &mut idx_pos)?;
        if idx_pos != idx_tokens.len() {
            return Err("Unexpected token in cell index expression".to_string());
        }
        let rhs_tokens = tokenize(rhs)?;
        if rhs_tokens.is_empty() {
            return Err("Expected expression after '='".to_string());
        }
        let mut rhs_pos = 0;
        let rhs_expr = parse_logical_or(&rhs_tokens, &mut rhs_pos)?;
        if rhs_pos != rhs_tokens.len() {
            return Err("Unexpected token after expression".to_string());
        }
        return Ok(Stmt::CellSet(name.to_string(), idx_expr, rhs_expr));
    }

    // Multi-assign: [a, b] = expr
    if let Some((targets, rhs)) = try_split_multi_assign(trimmed) {
        let tokens = tokenize(rhs)?;
        if tokens.is_empty() {
            return Err("Expected expression after '='".to_string());
        }
        let mut pos = 0;
        let expr = parse_logical_or(&tokens, &mut pos)?;
        if pos != tokens.len() {
            return Err("Unexpected token after expression".to_string());
        }
        return Ok(Stmt::MultiAssign { targets, expr });
    }

    if let Some((name, rhs)) = try_split_assignment(trimmed) {
        let tokens = tokenize(rhs)?;
        if tokens.is_empty() {
            return Err("Expected expression after '='".to_string());
        }
        let mut pos = 0;
        let expr = parse_logical_or(&tokens, &mut pos)?;
        if pos != tokens.len() {
            return Err("Unexpected token after expression".to_string());
        }
        return Ok(Stmt::Assign(name.to_string(), expr));
    }

    let tokens = tokenize(trimmed)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }

    // Check for compound assignment: x += expr, x -= expr, x *= expr, x /= expr,
    // x++, x--, ++x, --x (all desugar to simple Stmt::Assign at parse time).
    if let Some(stmt) = try_parse_compound(&tokens)? {
        return Ok(stmt);
    }

    let mut pos = 0;
    let expr = parse_logical_or(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err("Unexpected token after expression".to_string());
    }
    Ok(Stmt::Expr(expr))
}

/// Tries to parse a compound assignment or increment/decrement statement from an already-
/// tokenised token list. Returns `Ok(Some(stmt))` on a match, `Ok(None)` otherwise.
///
/// Supported forms (all desugar to `Stmt::Assign` — no new AST nodes required):
/// - `x op= rhs`  →  `x = x op rhs`   (`op` ∈ {+, −, ×, ÷})
/// - `x++`        →  `x = x + 1`
/// - `x--`        →  `x = x - 1`
/// - `++x`        →  `x = x + 1`   (prefix)
/// - `--x`        →  `x = x - 1`   (prefix)
///
/// **Limitation**: `++`/`--` are statement-level only. Using them inside a larger
/// expression (e.g. `b = a - b--`) is not supported.
fn try_parse_compound(tokens: &[Token]) -> Result<Option<Stmt>, String> {
    // Prefix ++x / --x
    if tokens.len() == 2
        && let Token::Ident(name) = &tokens[1]
    {
        let op = match &tokens[0] {
            Token::PlusPlus => Some(Op::Add),
            Token::MinusMinus => Some(Op::Sub),
            _ => None,
        };
        if let Some(op) = op {
            let expr = Expr::BinOp(
                Box::new(Expr::Var(name.clone())),
                op,
                Box::new(Expr::Number(1.0)),
            );
            return Ok(Some(Stmt::Assign(name.clone(), expr)));
        }
    }

    // All remaining forms start with an identifier
    let name = match tokens.first() {
        Some(Token::Ident(n)) => n.clone(),
        _ => return Ok(None),
    };

    if tokens.len() < 2 {
        return Ok(None);
    }

    match &tokens[1] {
        // Suffix x++ / x--
        Token::PlusPlus | Token::MinusMinus if tokens.len() == 2 => {
            let op = if matches!(&tokens[1], Token::PlusPlus) {
                Op::Add
            } else {
                Op::Sub
            };
            let expr = Expr::BinOp(
                Box::new(Expr::Var(name.clone())),
                op,
                Box::new(Expr::Number(1.0)),
            );
            Ok(Some(Stmt::Assign(name, expr)))
        }

        // x op= rhs
        Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq => {
            let op = match &tokens[1] {
                Token::PlusEq => Op::Add,
                Token::MinusEq => Op::Sub,
                Token::StarEq => Op::Mul,
                Token::SlashEq => Op::Div,
                _ => unreachable!(),
            };
            let rhs_tokens = &tokens[2..];
            if rhs_tokens.is_empty() {
                let op_str = match op {
                    Op::Add => "+=",
                    Op::Sub => "-=",
                    Op::Mul => "*=",
                    Op::Div => "/=",
                    _ => "op=",
                };
                return Err(format!("Expected expression after '{op_str}'"));
            }
            let mut pos = 0;
            let rhs = parse_logical_or(rhs_tokens, &mut pos)?;
            if pos != rhs_tokens.len() {
                return Err("Unexpected token after expression".to_string());
            }
            let expr = Expr::BinOp(Box::new(Expr::Var(name.clone())), op, Box::new(rhs));
            Ok(Some(Stmt::Assign(name, expr)))
        }

        _ => Ok(None),
    }
}

/// Returns true if the input looks like a partial expression
/// (i.e. starts with an operator that needs a left-hand operand).
pub fn is_partial(input: &str) -> bool {
    let mut chars = input.trim_start().chars();
    match chars.next() {
        // '++' and '--' are prefix increment/decrement, not binary operators
        Some('+') => !matches!(chars.next(), Some('+')),
        Some('-') => !matches!(chars.next(), Some('-')),
        Some('*' | '/' | '^' | '<' | '>') => true,
        // '.*', './', '.^' are element-wise binary operators
        Some('.') => matches!(chars.next(), Some('*' | '/' | '^')),
        // '==' comparison; '~=' not-equal
        Some('=') => chars.next() == Some('='),
        Some('~') => chars.next() == Some('='),
        // '&&', '||' short-circuit logical
        Some('&') => chars.next() == Some('&'),
        Some('|') => chars.next() == Some('|'),
        _ => false,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Multi-line block parsing (Phase 11a)
// ──────────────────────────────────────────────────────────────────────────────

/// Splits a raw input line into `(statement_str, silent)` pairs.
///
/// - Strips inline `%` comments (outside string literals).
/// - Splits on `;` outside string literals and outside `[...]` brackets.
/// - `silent = true` when the statement was terminated by `;`.
pub fn split_stmts(input: &str) -> Vec<(&str, bool)> {
    // (position, is_silent): ';' → silent=true, ',' → silent=false
    let mut separators: Vec<(usize, bool)> = Vec::new();
    let mut comment_at = input.len();
    let mut in_sq = false;
    let mut in_dq = false;
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;

    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut ci = 0;
    while ci < chars.len() {
        let (i, c) = chars[ci];
        let at_depth0 =
            !in_sq && !in_dq && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
        match c {
            '\'' if !in_dq => {
                if in_sq {
                    // Check for '' (escaped single quote) — stay inside the string.
                    let next = chars.get(ci + 1).map(|&(_, c)| c);
                    if next == Some('\'') {
                        ci += 1; // skip the second '
                    } else {
                        in_sq = false;
                    }
                } else {
                    let before = input[..i].trim_end_matches([' ', '\t']);
                    let is_transpose = before.ends_with(|c: char| {
                        c.is_alphanumeric()
                            || c == '_'
                            || c == ')'
                            || c == ']'
                            || c == '\''
                            || c == '.'
                    });
                    if !is_transpose {
                        in_sq = true;
                    }
                }
            }
            '"' if !in_sq => in_dq = !in_dq,
            '(' if !in_sq && !in_dq => paren_depth += 1,
            ')' if !in_sq && !in_dq && paren_depth > 0 => {
                paren_depth -= 1;
            }
            '[' if !in_sq && !in_dq => bracket_depth += 1,
            ']' if !in_sq && !in_dq && bracket_depth > 0 => {
                bracket_depth -= 1;
            }
            '{' if !in_sq && !in_dq => brace_depth += 1,
            '}' if !in_sq && !in_dq && brace_depth > 0 => {
                brace_depth -= 1;
            }
            '%' | '#' if at_depth0 => {
                comment_at = i;
                break;
            }
            ';' if at_depth0 => separators.push((i, true)),
            ',' if at_depth0 => separators.push((i, false)),
            _ => {}
        }
        ci += 1;
    }

    let content = input[..comment_at].trim_end();
    if content.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut start = 0;
    for &(sc, silent) in &separators {
        if sc >= content.len() {
            break;
        }
        let part = content[start..sc].trim();
        if !part.is_empty() {
            result.push((part, silent));
        }
        start = sc + 1;
    }
    if start <= content.len() {
        let last = content[start..].trim();
        if !last.is_empty() {
            result.push((last, false));
        }
    }
    result
}

/// Returns the net block-depth change for a single (comment-stripped, trimmed) line.
///
/// Used by the REPL to decide whether to buffer more lines before executing.
/// `if`/`for`/`while` → +1; `end` → -1; all other lines → 0.
pub fn block_depth_delta(line: &str) -> i32 {
    let stripped = strip_line_comment(line).trim();
    match leading_keyword(stripped) {
        Some("if") | Some("for") | Some("while") | Some("switch") | Some("do")
        | Some("function") | Some("try") => 1,
        Some("end") | Some("until") => -1,
        _ => 0,
    }
}

/// Returns `true` when `line` is a self-contained single-line block, e.g.
/// `if cond; body; end`.  These lines start with a block-opening keyword but
/// also close themselves with `end` / `until` in the same line, so they do
/// not need multi-line buffering.
pub fn is_single_line_block(line: &str) -> bool {
    let stripped = strip_line_comment(line).trim();
    if !matches!(
        leading_keyword(stripped),
        Some("if" | "for" | "while" | "switch" | "do")
    ) {
        return false;
    }
    let parts = split_block_line(stripped);
    matches!(
        parts.last().map(|s| leading_keyword(s.trim())),
        Some(Some("end" | "until"))
    )
}

/// Joins lines ending with `...` (line continuation) into a single logical line.
///
/// `...` at the end of a line (after stripping trailing comments) causes the next
/// line to be treated as a continuation. The `...` and newline are replaced by a space.
fn join_line_continuations(input: &str) -> String {
    let mut result = String::new();
    let mut pending = String::new();

    for line in input.lines() {
        let stripped = strip_line_comment(line);
        let trimmed = stripped.trim_end();
        if let Some(before_dots) = trimmed.strip_suffix("...") {
            // Append everything before `...` (and a space) to pending
            pending.push_str(before_dots);
            pending.push(' ');
        } else if pending.is_empty() {
            result.push_str(line);
            result.push('\n');
        } else {
            // Continuation: join pending with this line
            pending.push_str(line.trim_start());
            result.push_str(&pending);
            result.push('\n');
            pending.clear();
        }
    }
    // Any remaining pending (file ends with `...`)
    if !pending.is_empty() {
        result.push_str(pending.trim_end());
    }
    result
}

/// Splits a single-line block (e.g. `if x > 0; y = 1; end`) into individual
/// statement strings, splitting on `;` at depth 0 (outside strings/brackets/parens).
fn split_block_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_sq = false;
    let mut in_dq = false;
    let mut paren: i32 = 0;
    let mut bracket: i32 = 0;
    let mut brace: i32 = 0;

    for c in line.chars() {
        let at_depth0 = !in_sq && !in_dq && paren == 0 && bracket == 0 && brace == 0;
        match c {
            '\'' if !in_dq => {
                in_sq = !in_sq;
                current.push(c);
            }
            '"' if !in_sq => {
                in_dq = !in_dq;
                current.push(c);
            }
            '(' if !in_sq && !in_dq => {
                paren += 1;
                current.push(c);
            }
            ')' if !in_sq && !in_dq => {
                if paren > 0 {
                    paren -= 1;
                }
                current.push(c);
            }
            '[' if !in_sq && !in_dq => {
                bracket += 1;
                current.push(c);
            }
            ']' if !in_sq && !in_dq => {
                if bracket > 0 {
                    bracket -= 1;
                }
                current.push(c);
            }
            '{' if !in_sq && !in_dq => {
                brace += 1;
                current.push(c);
            }
            '}' if !in_sq && !in_dq => {
                if brace > 0 {
                    brace -= 1;
                }
                current.push(c);
            }
            ';' if at_depth0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    parts.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let last = current.trim().to_string();
    if !last.is_empty() {
        parts.push(last);
    }
    parts
}

/// Parses a multi-line block string into a sequence of `(Stmt, silent)` pairs.
///
/// The input may contain multiple lines separated by `\n` or `\r\n`.
/// Block keywords (`if`/`for`/`while`/`end`/…) are handled recursively.
/// Each statement carries a `silent` flag (`true` when terminated by `;`).
pub fn parse_stmts(input: &str) -> Result<Vec<(Stmt, bool)>, String> {
    let joined = join_line_continuations(input);
    let lines: Vec<&str> = joined.lines().collect();
    let mut pos = 0;
    parse_stmts_from_lines(&lines, &mut pos, &[])
}

/// Recursive block parser. Reads statements from `lines[*pos..]`, stopping when
/// a keyword found in `stop_at` is encountered (without consuming that line).
fn parse_stmts_from_lines(
    lines: &[&str],
    pos: &mut usize,
    stop_at: &[&str],
) -> Result<Vec<(Stmt, bool)>, String> {
    let mut stmts = Vec::new();

    while *pos < lines.len() {
        let raw = lines[*pos];
        let line = strip_line_comment(raw).trim();

        if line.is_empty() {
            *pos += 1;
            continue;
        }

        // Stop at a terminator keyword — caller is responsible for consuming it.
        if let Some(kw) = leading_keyword(line)
            && stop_at.contains(&kw)
        {
            return Ok(stmts);
        }

        // Single-line block: `if cond; body; end` on one line.
        // Detect: line starts with a block opener AND the last semicolon-split part is 'end'.
        // Expand to virtual multi-line and re-parse.
        if matches!(
            leading_keyword(line),
            Some("if" | "for" | "while" | "switch" | "do")
        ) {
            let virtual_parts = split_block_line(line);
            let last_kw = virtual_parts
                .last()
                .map(|s| leading_keyword(s.trim()))
                .unwrap_or(None);
            if matches!(last_kw, Some("end") | Some("until")) {
                let virtual_refs: Vec<&str> = virtual_parts.iter().map(|s| s.as_str()).collect();
                let mut vpos = 0;
                let inner = parse_stmts_from_lines(&virtual_refs, &mut vpos, stop_at)?;
                stmts.extend(inner);
                *pos += 1;
                continue;
            }
        }

        match leading_keyword(line) {
            // ── if / elseif / else / end ────────────────────────────────────
            Some("if") => {
                let cond_str = line["if".len()..].trim();
                if cond_str.is_empty() {
                    return Err("Expected condition after 'if'".to_string());
                }
                let cond = parse_condition(cond_str)?;
                *pos += 1;

                let body = parse_stmts_from_lines(lines, pos, &["elseif", "else", "end"])?;

                let mut elseif_branches = Vec::new();
                loop {
                    if *pos >= lines.len() {
                        return Err(
                            "Unexpected end of input inside 'if': expected 'end'".to_string()
                        );
                    }
                    let kw_line = strip_line_comment(lines[*pos]).trim();
                    if leading_keyword(kw_line) == Some("elseif") {
                        let ei_str = kw_line["elseif".len()..].trim();
                        if ei_str.is_empty() {
                            return Err("Expected condition after 'elseif'".to_string());
                        }
                        let ei_cond = parse_condition(ei_str)?;
                        *pos += 1;
                        let ei_body =
                            parse_stmts_from_lines(lines, pos, &["elseif", "else", "end"])?;
                        elseif_branches.push((ei_cond, ei_body));
                    } else {
                        break;
                    }
                }

                let else_body = if *pos < lines.len()
                    && leading_keyword(strip_line_comment(lines[*pos]).trim()) == Some("else")
                {
                    *pos += 1; // consume "else"
                    Some(parse_stmts_from_lines(lines, pos, &["end"])?)
                } else {
                    None
                };

                expect_end(lines, pos, "if")?;

                stmts.push((
                    Stmt::If {
                        cond,
                        body,
                        elseif_branches,
                        else_body,
                    },
                    false,
                ));
            }

            // ── for ─────────────────────────────────────────────────────────
            Some("for") => {
                let rest = line["for".len()..].trim();
                if rest.is_empty() {
                    return Err("Expected 'var = expr' after 'for'".to_string());
                }
                let (var, range_expr) = parse_for_header(rest)?;
                *pos += 1;
                let body = parse_stmts_from_lines(lines, pos, &["end"])?;
                expect_end(lines, pos, "for")?;
                stmts.push((
                    Stmt::For {
                        var,
                        range_expr,
                        body,
                    },
                    false,
                ));
            }

            // ── while ────────────────────────────────────────────────────────
            Some("while") => {
                let cond_str = line["while".len()..].trim();
                if cond_str.is_empty() {
                    return Err("Expected condition after 'while'".to_string());
                }
                let cond = parse_condition(cond_str)?;
                *pos += 1;
                let body = parse_stmts_from_lines(lines, pos, &["end"])?;
                expect_end(lines, pos, "while")?;
                stmts.push((Stmt::While { cond, body }, false));
            }

            // ── break / continue ─────────────────────────────────────────────
            Some("break") => {
                stmts.push((Stmt::Break, false));
                *pos += 1;
            }
            Some("continue") => {
                stmts.push((Stmt::Continue, false));
                *pos += 1;
            }

            // ── switch / case / otherwise / end ──────────────────────────────
            Some("switch") => {
                let expr_str = line["switch".len()..].trim();
                if expr_str.is_empty() {
                    return Err("Expected expression after 'switch'".to_string());
                }
                let expr = parse_condition(expr_str)?;
                *pos += 1;

                #[allow(clippy::type_complexity)]
                let mut cases: Vec<(Vec<Expr>, Vec<(Stmt, bool)>)> = Vec::new();
                let mut otherwise_body: Option<Vec<(Stmt, bool)>> = None;

                loop {
                    if *pos >= lines.len() {
                        return Err(
                            "Unexpected end of input inside 'switch': expected 'end'".to_string()
                        );
                    }
                    let kw_line = strip_line_comment(lines[*pos]).trim();
                    match leading_keyword(kw_line) {
                        Some("case") => {
                            let case_str = kw_line["case".len()..].trim();
                            if case_str.is_empty() {
                                return Err("Expected value after 'case'".to_string());
                            }
                            let case_expr = parse_condition(case_str)?;
                            *pos += 1;
                            let case_body =
                                parse_stmts_from_lines(lines, pos, &["case", "otherwise", "end"])?;
                            cases.push((vec![case_expr], case_body));
                        }
                        Some("otherwise") => {
                            *pos += 1;
                            let ob = parse_stmts_from_lines(lines, pos, &["end"])?;
                            otherwise_body = Some(ob);
                            break;
                        }
                        Some("end") => break,
                        _ => {
                            return Err(format!(
                                "Expected 'case', 'otherwise', or 'end' in switch block, found: '{kw_line}'"
                            ));
                        }
                    }
                }

                expect_end(lines, pos, "switch")?;
                stmts.push((
                    Stmt::Switch {
                        expr,
                        cases,
                        otherwise_body,
                    },
                    false,
                ));
            }

            // ── do...until ───────────────────────────────────────────────────
            Some("do") => {
                *pos += 1;
                let body = parse_stmts_from_lines(lines, pos, &["until"])?;
                if *pos >= lines.len() {
                    return Err("Unexpected end of input inside 'do': expected 'until'".to_string());
                }
                let until_line = strip_line_comment(lines[*pos]).trim();
                if leading_keyword(until_line) != Some("until") {
                    return Err(format!("Expected 'until', found: '{until_line}'"));
                }
                let cond_str = until_line["until".len()..].trim();
                if cond_str.is_empty() {
                    return Err("Expected condition after 'until'".to_string());
                }
                let cond = parse_condition(cond_str)?;
                *pos += 1;
                stmts.push((Stmt::DoUntil { body, cond }, false));
            }

            // ── function definition ──────────────────────────────────────────
            Some("function") => {
                let header = line["function".len()..].trim();
                if header.is_empty() {
                    return Err("Expected function header after 'function'".to_string());
                }
                let (name, outputs, params) = parse_function_header(header)?;
                *pos += 1;
                // Collect raw body lines until the matching 'end', tracking nested block depth.
                let body_start = *pos;
                let mut depth: i32 = 1;
                while *pos < lines.len() && depth > 0 {
                    let l = strip_line_comment(lines[*pos]).trim();
                    depth += block_depth_delta(l);
                    if depth == 0 {
                        break;
                    }
                    *pos += 1;
                }
                if depth != 0 {
                    return Err(format!(
                        "Unexpected end of input: expected 'end' to close 'function {name}'"
                    ));
                }
                let body_source = lines[body_start..*pos].join("\n");
                *pos += 1; // consume 'end'
                stmts.push((
                    Stmt::FunctionDef {
                        name,
                        outputs,
                        params,
                        body_source,
                    },
                    false,
                ));
            }

            // ── return ───────────────────────────────────────────────────────
            Some("return") => {
                stmts.push((Stmt::Return, false));
                *pos += 1;
            }

            // ── try / catch / end ────────────────────────────────────────────
            Some("try") => {
                *pos += 1;
                let try_body = parse_stmts_from_lines(lines, pos, &["catch", "end"])?;

                if *pos >= lines.len() {
                    return Err(
                        "Unexpected end of input inside 'try': expected 'catch' or 'end'"
                            .to_string(),
                    );
                }
                let kw_line = strip_line_comment(lines[*pos]).trim();
                let (catch_var, catch_body) = if leading_keyword(kw_line) == Some("catch") {
                    let catch_rest = kw_line["catch".len()..].trim();
                    let catch_var = if catch_rest.is_empty() {
                        None
                    } else if is_valid_ident(catch_rest) {
                        Some(catch_rest.to_string())
                    } else {
                        return Err(format!(
                            "Expected identifier after 'catch', got '{catch_rest}'"
                        ));
                    };
                    *pos += 1;
                    let catch_body = parse_stmts_from_lines(lines, pos, &["end"])?;
                    (catch_var, catch_body)
                } else {
                    // 'end' closes the try block with no catch body
                    (None, vec![])
                };

                expect_end(lines, pos, "try")?;
                stmts.push((
                    Stmt::TryCatch {
                        try_body,
                        catch_var,
                        catch_body,
                    },
                    false,
                ));
            }

            // ── unexpected terminators ───────────────────────────────────────
            Some(kw @ ("end" | "else" | "elseif" | "case" | "otherwise" | "until" | "catch")) => {
                return Err(format!("Unexpected '{kw}' without matching block opener"));
            }

            // ── regular statement(s) — may contain ';' ──────────────────────
            _ => {
                for (stmt_str, silent) in split_stmts(raw) {
                    stmts.push((parse(stmt_str)?, silent));
                }
                *pos += 1;
            }
        }
    }

    Ok(stmts)
}

/// Expects `lines[*pos]` to contain `end`, consumes it, or returns an error.
fn expect_end(lines: &[&str], pos: &mut usize, opener: &str) -> Result<(), String> {
    if *pos >= lines.len() {
        return Err(format!(
            "Unexpected end of input: expected 'end' to close '{opener}'"
        ));
    }
    let kw_line = strip_line_comment(lines[*pos]).trim();
    if leading_keyword(kw_line) != Some("end") {
        return Err(format!(
            "Expected 'end' to close '{opener}', found '{kw_line}'"
        ));
    }
    *pos += 1;
    Ok(())
}

/// Strips a trailing `%` comment from a line, respecting single- and double-quoted strings.
fn strip_line_comment(line: &str) -> &str {
    let mut in_sq = false;
    let mut in_dq = false;
    for (i, c) in line.char_indices() {
        match c {
            '\'' if !in_dq => in_sq = !in_sq,
            '"' if !in_sq => in_dq = !in_dq,
            '%' | '#' if !in_sq && !in_dq => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Returns the leading keyword of a trimmed line if it is a recognised block keyword.
///
/// Uses word-boundary detection so `if_flag` → `None` but `if x > 0` → `Some("if")`.
fn leading_keyword(line: &str) -> Option<&str> {
    let end = line
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(line.len());
    let word = &line[..end];
    match word {
        "if" | "elseif" | "else" | "end" | "for" | "while" | "break" | "continue" | "switch"
        | "case" | "otherwise" | "do" | "until" | "function" | "return" | "try" | "catch" => {
            Some(word)
        }
        _ => None,
    }
}

/// Parses the function header text (everything after `function` keyword).
///
/// Handles three forms:
/// - `name(params)` — no outputs
/// - `y = name(params)` — single output
/// - `[y1, y2] = name(params)` — multiple outputs
fn parse_function_header(header: &str) -> Result<(String, Vec<String>, Vec<String>), String> {
    // Detect output list if there is an `=` (that is not `==`)
    if let Some(eq_pos) = header.find('=')
        && !header[eq_pos + 1..].starts_with('=')
    {
        let lhs = header[..eq_pos].trim();
        let rhs = header[eq_pos + 1..].trim();
        let outputs = parse_output_list(lhs)?;
        let (name, params) = parse_func_name_params(rhs)?;
        return Ok((name, outputs, params));
    }
    // No outputs: just name(params)
    let (name, params) = parse_func_name_params(header.trim())?;
    Ok((name, vec![], params))
}

/// Parses an output variable list: `y`, `[y1, y2]`.
fn parse_output_list(lhs: &str) -> Result<Vec<String>, String> {
    let lhs = lhs.trim();
    if lhs.starts_with('[') && lhs.ends_with(']') {
        let inner = &lhs[1..lhs.len() - 1];
        inner
            .split(',')
            .map(|s| {
                let s = s.trim();
                if is_valid_ident(s) {
                    Ok(s.to_string())
                } else {
                    Err(format!("Invalid output variable name: '{s}'"))
                }
            })
            .collect()
    } else if is_valid_ident(lhs) {
        Ok(vec![lhs.to_string()])
    } else {
        Err(format!("Invalid function output list: '{lhs}'"))
    }
}

/// Parses `name(p1, p2)` or `name` — returns `(name, params)`.
fn parse_func_name_params(s: &str) -> Result<(String, Vec<String>), String> {
    let s = s.trim();
    if let Some(paren_pos) = s.find('(') {
        let name = s[..paren_pos].trim();
        if !is_valid_ident(name) {
            return Err(format!("Invalid function name: '{name}'"));
        }
        let rest = s[paren_pos + 1..].trim();
        if !rest.ends_with(')') {
            return Err(format!("Expected ')' in function header: '{s}'"));
        }
        let params_str = rest[..rest.len() - 1].trim();
        let params = if params_str.is_empty() {
            vec![]
        } else {
            params_str
                .split(',')
                .map(|p| {
                    let p = p.trim();
                    if is_valid_ident(p) {
                        Ok(p.to_string())
                    } else {
                        Err(format!("Invalid parameter name: '{p}'"))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok((name.to_string(), params))
    } else {
        if !is_valid_ident(s) {
            return Err(format!("Invalid function name: '{s}'"));
        }
        Ok((s.to_string(), vec![]))
    }
}

/// Parses `cond_str` (the text after `if`/`elseif`/`while`) as an expression.
fn parse_condition(cond_str: &str) -> Result<Expr, String> {
    match parse(cond_str)? {
        Stmt::Expr(e) => Ok(e),
        Stmt::Assign(_, _) => Err("Expected condition expression, found assignment".to_string()),
        _ => Err("Expected condition expression".to_string()),
    }
}

/// Parses the `for` header `var = range_expr`.
fn parse_for_header(rest: &str) -> Result<(String, Expr), String> {
    match parse(rest)? {
        Stmt::Assign(var, expr) => Ok((var, expr)),
        _ => Err(format!(
            "Expected 'variable = expression' in 'for' header, found: '{rest}'"
        )),
    }
}

// ──────────────────────────────────────────────────────────────────────────────

/// If `input` matches `"[a, b] = rhs"` (not `==`), returns the target names and rhs string.
/// All targets must be valid identifiers or `~` (discard placeholder).
fn try_split_multi_assign(input: &str) -> Option<(Vec<String>, &str)> {
    let trimmed = input.trim();
    if !trimmed.starts_with('[') {
        return None;
    }
    let close = trimmed.find(']')?;
    let rest = trimmed[close + 1..].trim();
    if !rest.starts_with('=') || rest.starts_with("==") {
        return None;
    }
    let rhs = rest[1..].trim();
    let inner = trimmed[1..close].trim();
    if inner.is_empty() {
        return None;
    }
    let targets: Vec<String> = inner.split(',').map(|s| s.trim().to_string()).collect();
    for t in &targets {
        if t != "~" && !is_valid_ident(t) {
            return None;
        }
    }
    Some((targets, rhs))
}

/// If `input` matches `"name{idx} = rhs"` (not `==`), returns `Some((name, idx, rhs))`.
/// The name must be a valid identifier; otherwise returns `None`.
fn try_split_cell_assign(input: &str) -> Option<(&str, &str, &str)> {
    let trimmed = input.trim();
    // Find an identifier followed immediately by '{'
    let brace_pos = trimmed.find('{')?;
    let name = trimmed[..brace_pos].trim();
    if !is_valid_ident(name) {
        return None;
    }
    // Find the matching '}'
    let after_open = &trimmed[brace_pos + 1..];
    let close_pos = after_open.find('}')?;
    let idx_str = after_open[..close_pos].trim();
    // After '}' must be '=' (not '==')
    let after_close = after_open[close_pos + 1..].trim();
    if !after_close.starts_with('=') || after_close.starts_with("==") {
        return None;
    }
    let rhs = after_close[1..].trim();
    Some((name, idx_str, rhs))
}

/// If `input` matches `"name(args) = rhs"` (not `==`), returns `Some((name, args_str, rhs))`.
///
/// The name must be a valid identifier and no `.field` may follow the closing `)` (those
/// patterns are handled by `try_split_struct_array_field_assign`).
fn try_split_index_assign(input: &str) -> Option<(String, &str, &str)> {
    let trimmed = input.trim();
    let bytes = trimmed.as_bytes();
    let mut i = 0;

    // Parse leading identifier
    if i >= bytes.len() || !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
        return None;
    }
    let name_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    let name = trimmed[name_start..i].to_string();

    // Skip optional whitespace then expect '('
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'(' {
        return None;
    }
    i += 1;

    // Scan for the matching ')' (tracking nested parens/brackets/braces)
    let idx_start = i;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    if depth != 0 {
        return None;
    }
    let idx_str = trimmed[idx_start..i - 1].trim();

    // After ')' must not be '.' (that is struct-array-field-assign, handled earlier)
    let rest = trimmed[i..].trim_start();
    if rest.starts_with('.') {
        return None;
    }
    // After ')' must be bare '=' (not '==')
    if !rest.starts_with('=') || rest.starts_with("==") {
        return None;
    }
    let rhs = rest[1..].trim();
    if rhs.is_empty() {
        return None;
    }
    Some((name, idx_str, rhs))
}

/// Parses a comma-separated list of index arguments (`:` allowed) from a token slice.
fn parse_index_args(tokens: &[Token]) -> Result<Vec<Expr>, String> {
    if tokens.is_empty() {
        return Err("Expected index expression inside '()'".to_string());
    }
    let mut pos = 0;
    let mut args = Vec::new();
    loop {
        args.push(parse_call_arg(tokens, &mut pos)?);
        match tokens.get(pos) {
            Some(Token::Comma) => {
                pos += 1;
            }
            None => break,
            Some(_) => return Err("Unexpected token in index expression".to_string()),
        }
    }
    Ok(args)
}

/// If `input` matches `"name = rhs"` (not `==`), returns `Some((name, rhs))`.
/// The name must be a valid identifier; otherwise returns `None`.
fn try_split_assignment(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim();
    let eq_pos = trimmed.find('=')?;
    // Reject `==`
    if trimmed[eq_pos + 1..].starts_with('=') {
        return None;
    }
    let lhs = trimmed[..eq_pos].trim();
    let rhs = trimmed[eq_pos + 1..].trim();
    if is_valid_ident(lhs) {
        Some((lhs, rhs))
    } else {
        None
    }
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => chars.all(|c| c.is_alphanumeric() || c == '_'),
        _ => false,
    }
}

// call_arg = ':' | logical_or_expr
// Used when parsing function call / index arguments.
// A bare ':' at the start of an argument position becomes Expr::Colon (all-elements index).
fn parse_call_arg(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    if matches!(tokens.get(*pos), Some(Token::Colon)) {
        *pos += 1;
        return Ok(Expr::Colon);
    }
    parse_logical_or(tokens, pos)
}

// logical_or = logical_and ('||' logical_and)*
fn parse_logical_or(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_logical_and(tokens, pos)?;
    while matches!(tokens.get(*pos), Some(Token::PipePipe)) {
        *pos += 1;
        let right = parse_logical_and(tokens, pos)?;
        left = Expr::BinOp(Box::new(left), Op::Or, Box::new(right));
    }
    Ok(left)
}

// logical_and = elem_or ('&&' elem_or)*
fn parse_logical_and(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_elem_or(tokens, pos)?;
    while matches!(tokens.get(*pos), Some(Token::AmpAmp)) {
        *pos += 1;
        let right = parse_elem_or(tokens, pos)?;
        left = Expr::BinOp(Box::new(left), Op::And, Box::new(right));
    }
    Ok(left)
}

// elem_or = elem_and ('|' elem_and)*  -- element-wise OR, lower precedence than '&'
fn parse_elem_or(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_elem_and(tokens, pos)?;
    while matches!(tokens.get(*pos), Some(Token::Pipe)) {
        *pos += 1;
        let right = parse_elem_and(tokens, pos)?;
        left = Expr::BinOp(Box::new(left), Op::ElemOr, Box::new(right));
    }
    Ok(left)
}

// elem_and = comparison ('&' comparison)*  -- element-wise AND
fn parse_elem_and(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_comparison(tokens, pos)?;
    while matches!(tokens.get(*pos), Some(Token::Amp)) {
        *pos += 1;
        let right = parse_comparison(tokens, pos)?;
        left = Expr::BinOp(Box::new(left), Op::ElemAnd, Box::new(right));
    }
    Ok(left)
}

// comparison = range_expr (('==' | '~=' | '<' | '>' | '<=' | '>=') range_expr)?
// Comparison operators are non-associative (no chaining: `a < b < c` is an error).
fn parse_comparison(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let left = parse_range(tokens, pos)?;
    let op = match tokens.get(*pos) {
        Some(Token::EqEq) => Op::Eq,
        Some(Token::NotEq) => Op::NotEq,
        Some(Token::Lt) => Op::Lt,
        Some(Token::Gt) => Op::Gt,
        Some(Token::LtEq) => Op::LtEq,
        Some(Token::GtEq) => Op::GtEq,
        _ => return Ok(left),
    };
    *pos += 1;
    let right = parse_range(tokens, pos)?;
    Ok(Expr::BinOp(Box::new(left), op, Box::new(right)))
}

// range_expr = expr (':' expr (':' expr)?)?
// Range has lower precedence than arithmetic: `1+1:5` = `2:5`.
// Two-colon form: `a:step:b`; one-colon form: `a:b` (step defaults to 1).
fn parse_range(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let start = parse_expr(tokens, pos)?;
    if !matches!(tokens.get(*pos), Some(Token::Colon)) {
        return Ok(start);
    }
    *pos += 1;
    let second = parse_expr(tokens, pos)?;
    if !matches!(tokens.get(*pos), Some(Token::Colon)) {
        // a:b form — start:stop with implicit step 1
        return Ok(Expr::Range(Box::new(start), None, Box::new(second)));
    }
    *pos += 1;
    let third = parse_expr(tokens, pos)?;
    // a:step:b form
    Ok(Expr::Range(
        Box::new(start),
        Some(Box::new(second)),
        Box::new(third),
    ))
}

// expr = term (('+' | '-') term)*
fn parse_expr(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_term(tokens, pos)?;

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Plus => {
                *pos += 1;
                let right = parse_term(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::Add, Box::new(right));
            }
            Token::Minus => {
                *pos += 1;
                let right = parse_term(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::Sub, Box::new(right));
            }
            _ => break,
        }
    }

    Ok(left)
}

// term = power (('*' | '/' | '.*' | './') power | '(' expr ')' )*
// '(' without an operator triggers implicit multiplication.
fn parse_term(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_power(tokens, pos)?;

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Star => {
                *pos += 1;
                let right = parse_power(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::Mul, Box::new(right));
            }
            Token::Slash => {
                *pos += 1;
                let right = parse_power(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::Div, Box::new(right));
            }
            Token::DotStar => {
                *pos += 1;
                let right = parse_power(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::ElemMul, Box::new(right));
            }
            Token::DotSlash => {
                *pos += 1;
                let right = parse_power(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::ElemDiv, Box::new(right));
            }
            Token::Backslash => {
                *pos += 1;
                let right = parse_power(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::LDiv, Box::new(right));
            }
            Token::LParen => {
                // Implicit multiplication: expr(...)
                let right = parse_power(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::Mul, Box::new(right));
            }
            _ => break,
        }
    }

    Ok(left)
}

// power = unary (('^' | '.^' | '**') power)?   -- right-associative
// '**' is an Octave alias for '^'.
fn parse_power(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let base = parse_unary(tokens, pos)?;
    if *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Caret | Token::StarStar => {
                *pos += 1;
                let exp = parse_power(tokens, pos)?;
                return Ok(Expr::BinOp(Box::new(base), Op::Pow, Box::new(exp)));
            }
            Token::DotCaret => {
                *pos += 1;
                let exp = parse_power(tokens, pos)?;
                return Ok(Expr::BinOp(Box::new(base), Op::ElemPow, Box::new(exp)));
            }
            _ => {}
        }
    }
    Ok(base)
}

// unary = '+' unary | '-' unary | '~' unary | primary
// Unary '+' is a no-op: `+x` = `x`, `+[1 2 3]` = `[1 2 3]`.
fn parse_unary(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    if *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Plus => {
                *pos += 1;
                return parse_unary(tokens, pos); // noop
            }
            Token::Minus => {
                *pos += 1;
                let expr = parse_unary(tokens, pos)?;
                return Ok(Expr::UnaryMinus(Box::new(expr)));
            }
            Token::Tilde => {
                *pos += 1;
                let expr = parse_unary(tokens, pos)?;
                return Ok(Expr::UnaryNot(Box::new(expr)));
            }
            _ => {}
        }
    }
    parse_primary(tokens, pos)
}

// primary = ident '(' expr ')' | ident '(' ')' | '(' expr ')' | '[' matrix ']' | number | ident
// followed by optional postfix transpose: expr '\''*
fn parse_primary(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of expression".to_string());
    }

    let mut expr = match &tokens[*pos] {
        Token::Number(n) => {
            let n = *n;
            *pos += 1;
            Expr::Number(n)
        }
        Token::LBrace => {
            *pos += 1;
            // Cell literal: { expr, expr, ... }
            let mut elems = Vec::new();
            loop {
                match tokens.get(*pos) {
                    None => return Err("Expected '}'".to_string()),
                    Some(Token::RBrace) => {
                        *pos += 1;
                        break;
                    }
                    Some(Token::Comma) => {
                        *pos += 1;
                    }
                    _ => {
                        elems.push(parse_logical_or(tokens, pos)?);
                    }
                }
            }
            Expr::CellLiteral(elems)
        }
        Token::Ident(name) => {
            let name = name.clone();
            *pos += 1;
            // Cell brace-indexing: ident '{' expr '}'
            if *pos < tokens.len()
                && let Token::LBrace = &tokens[*pos]
            {
                *pos += 1;
                let idx = parse_logical_or(tokens, pos)?;
                if *pos >= tokens.len() {
                    return Err("Expected '}'".to_string());
                }
                match &tokens[*pos] {
                    Token::RBrace => {
                        *pos += 1;
                        Expr::CellIndex(Box::new(Expr::Var(name)), Box::new(idx))
                    }
                    _ => return Err("Expected '}'".to_string()),
                }
            // Function call: ident '(' [expr (',' expr)*] ')'
            } else if *pos < tokens.len()
                && let Token::LParen = &tokens[*pos]
            {
                *pos += 1;
                let args = if *pos < tokens.len() {
                    if let Token::RParen = &tokens[*pos] {
                        // Empty call: no arguments. Builtins and lambdas inject `ans` at eval
                        // time; user functions receive truly empty arg lists (varargin = {}).
                        vec![]
                    } else {
                        let mut list = vec![parse_call_arg(tokens, pos)?];
                        while *pos < tokens.len() {
                            if let Token::Comma = &tokens[*pos] {
                                *pos += 1;
                                list.push(parse_call_arg(tokens, pos)?);
                            } else {
                                break;
                            }
                        }
                        list
                    }
                } else {
                    return Err("Expected closing ')'".to_string());
                };
                if *pos >= tokens.len() {
                    return Err("Expected closing ')'".to_string());
                }
                match &tokens[*pos] {
                    Token::RParen => {
                        *pos += 1;
                        Expr::Call(name, args)
                    }
                    _ => return Err("Expected closing ')'".to_string()),
                }
            } else {
                // Built-in constants
                match name.as_str() {
                    "pi" => Expr::Number(std::f64::consts::PI),
                    // 'e' is a variable-shadowing constant: env lookup first, fallback to Euler's number.
                    "e" => Expr::Var("e".to_string()),
                    "nan" | "NaN" => Expr::Number(f64::NAN),
                    "inf" | "Inf" => Expr::Number(f64::INFINITY),
                    // All other identifiers → variable reference (resolved at eval time)
                    _ => Expr::Var(name),
                }
            }
        }
        Token::LParen => {
            *pos += 1;
            let inner = parse_logical_or(tokens, pos)?;
            if *pos >= tokens.len() {
                return Err("Expected closing ')'".to_string());
            }
            match &tokens[*pos] {
                Token::RParen => {
                    *pos += 1;
                    inner
                }
                _ => return Err("Expected closing ')'".to_string()),
            }
        }
        Token::LBracket => {
            *pos += 1;
            parse_matrix(tokens, pos)?
        }
        Token::Str(s) => {
            let s = s.clone();
            *pos += 1;
            Expr::StrLiteral(s)
        }
        Token::StringObj(s) => {
            let s = s.clone();
            *pos += 1;
            Expr::StringObjLiteral(s)
        }
        Token::At => {
            *pos += 1;
            // @funcname — function handle (wraps named function as a lambda)
            if let Some(Token::Ident(name)) = tokens.get(*pos) {
                let name = name.clone();
                *pos += 1;
                return Ok(Expr::FuncHandle(name));
            }
            // @(params) body — anonymous function (lambda)
            if !matches!(tokens.get(*pos), Some(Token::LParen)) {
                return Err("Expected '(' or identifier after '@'".to_string());
            }
            *pos += 1;
            let mut params = Vec::new();
            loop {
                match tokens.get(*pos) {
                    Some(Token::RParen) => {
                        *pos += 1;
                        break;
                    }
                    Some(Token::Ident(name)) => {
                        params.push(name.clone());
                        *pos += 1;
                        if matches!(tokens.get(*pos), Some(Token::Comma)) {
                            *pos += 1;
                        }
                    }
                    None => return Err("Expected ')' in lambda parameter list".to_string()),
                    _ => return Err("Expected parameter name in lambda".to_string()),
                }
            }
            let body = parse_logical_or(tokens, pos)?;
            let source = format!("@({}) {}", params.join(", "), expr_to_string(&body));
            Expr::Lambda {
                params,
                body: Box::new(body),
                source,
            }
        }
        _ => {
            return Err(
                "Expected number, function, variable, string, '-', '[', '@', or '('".to_string(),
            );
        }
    };

    // Postfix operators: field access (`.field`), transpose (`'`), plain-transpose (`.'`)
    // All bind tighter than any binary operator.
    loop {
        match tokens.get(*pos) {
            Some(Token::Dot) => {
                *pos += 1;
                match tokens.get(*pos) {
                    Some(Token::Ident(field)) => {
                        let field = field.clone();
                        *pos += 1;
                        expr = Expr::FieldGet(Box::new(expr), field);
                    }
                    _ => return Err("Expected field name after '.'".to_string()),
                }
            }
            Some(Token::Apostrophe) => {
                *pos += 1;
                expr = Expr::Transpose(Box::new(expr));
            }
            Some(Token::DotApostrophe) => {
                *pos += 1;
                expr = Expr::PlainTranspose(Box::new(expr));
            }
            _ => break,
        }
    }

    Ok(expr)
}

/// Parses the contents of a matrix literal after the opening `[` has been consumed.
fn parse_matrix(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    // Handle empty matrix []
    if matches!(tokens.get(*pos), Some(Token::RBracket)) {
        *pos += 1;
        return Ok(Expr::Matrix(vec![]));
    }
    let mut rows: Vec<Vec<Expr>> = Vec::new();
    let mut current_row: Vec<Expr> = Vec::new();
    loop {
        match tokens.get(*pos) {
            None => return Err("Expected ']'".to_string()),
            Some(Token::RBracket) => {
                *pos += 1;
                if !current_row.is_empty() {
                    rows.push(current_row);
                }
                break;
            }
            Some(Token::Semicolon) => {
                *pos += 1;
                if !current_row.is_empty() {
                    rows.push(std::mem::take(&mut current_row));
                }
            }
            Some(Token::Comma) => {
                *pos += 1;
            }
            _ => {
                current_row.push(parse_logical_or(tokens, pos)?);
            }
        }
    }
    Ok(Expr::Matrix(rows))
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
