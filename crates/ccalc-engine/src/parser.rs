use crate::eval::{Expr, Op};

/// Top-level statement returned by [`parse`] and [`parse_stmts`].
#[derive(Debug)]
pub enum Stmt {
    /// Variable assignment: `name = expr`
    Assign(String, Expr),
    /// Standalone expression — result goes into `ans`
    Expr(Expr),
    /// `if cond; body; elseif cond; ...; else; ...; end`
    If {
        cond: Expr,
        body: Vec<(Stmt, bool)>,
        elseif_branches: Vec<(Expr, Vec<(Stmt, bool)>)>,
        else_body: Option<Vec<(Stmt, bool)>>,
    },
    /// `for var = range_expr; body; end` — iterates over columns of the range matrix
    For {
        var: String,
        range_expr: Expr,
        body: Vec<(Stmt, bool)>,
    },
    /// `while cond; body; end`
    While { cond: Expr, body: Vec<(Stmt, bool)> },
    /// `break` — exits the innermost enclosing loop
    Break,
    /// `continue` — advances to next iteration of the innermost enclosing loop
    Continue,
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
    Tilde,    // ~ / ! (unary NOT)
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
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::StarEq);
                } else {
                    tokens.push(Token::Star);
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
                    return Err("Use '&&' for logical AND".to_string());
                }
            }
            '|' => {
                chars.next();
                if chars.peek().copied() == Some('|') {
                    chars.next();
                    tokens.push(Token::PipePipe);
                } else {
                    return Err("Use '||' for logical OR".to_string());
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
                }
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

/// Parses a full input string into a [`Stmt`].
///
/// Assignment (`name = expr`) is detected first. Everything else is treated as
/// an expression whose result will be stored in `ans`.
pub fn parse(input: &str) -> Result<Stmt, String> {
    if let Some((name, rhs)) = try_split_assignment(input) {
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

    let tokens = tokenize(input)?;
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
    let mut semis: Vec<usize> = Vec::new();
    let mut comment_at = input.len();
    let mut in_sq = false;
    let mut in_dq = false;
    let mut bracket_depth: i32 = 0;

    for (i, c) in input.char_indices() {
        match c {
            '\'' if !in_dq => {
                if in_sq {
                    in_sq = false;
                } else {
                    let before = input[..i].trim_end_matches([' ', '\t']);
                    let is_transpose = before.ends_with(|c: char| {
                        c.is_alphanumeric() || c == '_' || c == ')' || c == ']' || c == '\''
                    });
                    if !is_transpose {
                        in_sq = true;
                    }
                }
            }
            '"' if !in_sq => in_dq = !in_dq,
            '[' if !in_sq && !in_dq => bracket_depth += 1,
            ']' if !in_sq && !in_dq => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            }
            '%' | '#' if !in_sq && !in_dq && bracket_depth == 0 => {
                comment_at = i;
                break;
            }
            ';' if !in_sq && !in_dq && bracket_depth == 0 => semis.push(i),
            _ => {}
        }
    }

    let content = input[..comment_at].trim_end();
    if content.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut start = 0;
    for &sc in &semis {
        if sc >= content.len() {
            break;
        }
        let part = content[start..sc].trim();
        if !part.is_empty() {
            result.push((part, true));
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
        Some("if") | Some("for") | Some("while") => 1,
        Some("end") => -1,
        _ => 0,
    }
}

/// Parses a multi-line block string into a sequence of `(Stmt, silent)` pairs.
///
/// The input may contain multiple lines separated by `\n` or `\r\n`.
/// Block keywords (`if`/`for`/`while`/`end`/…) are handled recursively.
/// Each statement carries a `silent` flag (`true` when terminated by `;`).
pub fn parse_stmts(input: &str) -> Result<Vec<(Stmt, bool)>, String> {
    let lines: Vec<&str> = input.lines().collect();
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

            // ── unexpected terminators ───────────────────────────────────────
            Some(kw @ ("end" | "else" | "elseif")) => {
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
        "if" | "elseif" | "else" | "end" | "for" | "while" | "break" | "continue" => Some(word),
        _ => None,
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

// logical_and = comparison ('&&' comparison)*
fn parse_logical_and(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_comparison(tokens, pos)?;
    while matches!(tokens.get(*pos), Some(Token::AmpAmp)) {
        *pos += 1;
        let right = parse_comparison(tokens, pos)?;
        left = Expr::BinOp(Box::new(left), Op::And, Box::new(right));
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

// power = unary (('^' | '.^') power)?   -- right-associative
fn parse_power(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let base = parse_unary(tokens, pos)?;
    if *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Caret => {
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

// unary = '-' unary | '~' unary | primary
fn parse_unary(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    if *pos < tokens.len() {
        match &tokens[*pos] {
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
        Token::Ident(name) => {
            let name = name.clone();
            *pos += 1;
            // Function call: ident '(' [expr (',' expr)*] ')'
            if *pos < tokens.len()
                && let Token::LParen = &tokens[*pos]
            {
                *pos += 1;
                // Empty args fn() → pass ans as sole argument
                let args = if *pos < tokens.len() {
                    if let Token::RParen = &tokens[*pos] {
                        vec![Expr::Var("ans".to_string())]
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
                    "e" => Expr::Number(std::f64::consts::E),
                    "nan" => Expr::Number(f64::NAN),
                    "inf" => Expr::Number(f64::INFINITY),
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
        _ => {
            return Err(
                "Expected number, function, variable, string, '-', '[', or '('".to_string(),
            );
        }
    };

    // Postfix transpose: ' binds tighter than any binary operator
    while *pos < tokens.len() {
        if let Token::Apostrophe = &tokens[*pos] {
            *pos += 1;
            expr = Expr::Transpose(Box::new(expr));
        } else {
            break;
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
