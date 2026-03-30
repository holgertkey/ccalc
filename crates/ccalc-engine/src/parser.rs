use crate::eval::{Expr, Op};

/// Top-level statement returned by [`parse`].
#[derive(Debug)]
pub enum Stmt {
    /// Variable assignment: `name = expr`
    Assign(String, Expr),
    /// Standalone expression — result goes into `ans`
    Expr(Expr),
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
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
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '%' => {
                // '%' starts a comment in Octave/MATLAB — stop tokenizing
                break;
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '0'..='9' | '.' => {
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
                                if d.is_ascii_digit() || d == '.' {
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
                    }
                } else {
                    let mut num_str = String::new();
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() || d == '.' {
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
        let expr = parse_expr(&tokens, &mut pos)?;
        if pos != tokens.len() {
            return Err("Unexpected token after expression".to_string());
        }
        return Ok(Stmt::Assign(name.to_string(), expr));
    }

    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut pos = 0;
    let expr = parse_expr(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err("Unexpected token after expression".to_string());
    }
    Ok(Stmt::Expr(expr))
}

/// Returns true if the input looks like a partial expression
/// (i.e. starts with an operator that needs a left-hand operand).
pub fn is_partial(input: &str) -> bool {
    matches!(
        input.trim_start().chars().next(),
        Some('+' | '-' | '*' | '/' | '^' | '%')
    )
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

// term = power (('*' | '/') power | '(' expr ')' )*
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

// power = unary ('^' power)?   -- right-associative
fn parse_power(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let base = parse_unary(tokens, pos)?;
    if *pos < tokens.len()
        && let Token::Caret = &tokens[*pos]
    {
        *pos += 1;
        let exp = parse_power(tokens, pos)?;
        return Ok(Expr::BinOp(Box::new(base), Op::Pow, Box::new(exp)));
    }
    Ok(base)
}

// unary = '-' unary | primary
fn parse_unary(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    if *pos < tokens.len()
        && let Token::Minus = &tokens[*pos]
    {
        *pos += 1;
        let expr = parse_unary(tokens, pos)?;
        return Ok(Expr::UnaryMinus(Box::new(expr)));
    }
    parse_primary(tokens, pos)
}

// primary = ident '(' expr ')' | ident '(' ')' | '(' expr ')' | number | ident
fn parse_primary(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of expression".to_string());
    }

    match &tokens[*pos] {
        Token::Number(n) => {
            let n = *n;
            *pos += 1;
            Ok(Expr::Number(n))
        }
        Token::Ident(name) => {
            let name = name.clone();
            *pos += 1;
            // Function call: ident '(' [expr] ')'
            if *pos < tokens.len()
                && let Token::LParen = &tokens[*pos]
            {
                *pos += 1;
                // Empty args: fn() uses ans
                let arg = if *pos < tokens.len() {
                    if let Token::RParen = &tokens[*pos] {
                        Box::new(Expr::Var("ans".to_string()))
                    } else {
                        Box::new(parse_expr(tokens, pos)?)
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
                        return Ok(Expr::Call(name, arg));
                    }
                    _ => return Err("Expected closing ')'".to_string()),
                }
            }
            // Built-in constants
            match name.as_str() {
                "pi" => Ok(Expr::Number(std::f64::consts::PI)),
                "e" => Ok(Expr::Number(std::f64::consts::E)),
                // All other identifiers → variable reference (resolved at eval time)
                _ => Ok(Expr::Var(name)),
            }
        }
        Token::LParen => {
            *pos += 1;
            let expr = parse_expr(tokens, pos)?;
            if *pos >= tokens.len() {
                return Err("Expected closing ')'".to_string());
            }
            match &tokens[*pos] {
                Token::RParen => {
                    *pos += 1;
                    Ok(expr)
                }
                _ => Err("Expected closing ')'".to_string()),
            }
        }
        _ => Err("Expected number, function, variable, '-', or '('".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Env;
    use crate::eval::eval;

    fn calc(input: &str) -> f64 {
        let env = Env::new();
        match parse(input).unwrap() {
            Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).unwrap(),
        }
    }

    fn calc_with_ans(input: &str, ans: f64) -> f64 {
        let mut env = Env::new();
        env.insert("ans".to_string(), ans);
        match parse(input).unwrap() {
            Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).unwrap(),
        }
    }

    fn calc_with_var(input: &str, name: &str, val: f64) -> f64 {
        let mut env = Env::new();
        env.insert(name.to_string(), val);
        match parse(input).unwrap() {
            Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).unwrap(),
        }
    }

    #[test]
    fn test_single_number() {
        assert_eq!(calc("42"), 42.0);
        assert_eq!(calc("3.14"), 3.14);
    }

    #[test]
    fn test_basic_operations() {
        assert_eq!(calc("1 + 1"), 2.0);
        assert_eq!(calc("10 - 4"), 6.0);
        assert_eq!(calc("3 * 7"), 21.0);
        assert_eq!(calc("10 / 4"), 2.5);
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(calc("2 + 3 * 4"), 14.0);
        assert_eq!(calc("10 - 2 * 3"), 4.0);
        assert_eq!(calc("8 / 2 + 1"), 5.0);
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(calc("(2 + 3) * 4"), 20.0);
        assert_eq!(calc("(3 + 3) * 2"), 12.0);
        assert_eq!(calc("(10 - 2) * (3 + 1)"), 32.0);
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(calc("-5"), -5.0);
        assert_eq!(calc("-5 + 3"), -2.0);
        assert_eq!(calc("-(3 + 2)"), -5.0);
    }

    #[test]
    fn test_power() {
        assert_eq!(calc("2 ^ 10"), 1024.0);
        assert_eq!(calc("3 ^ 3"), 27.0);
        assert_eq!(calc("4 ^ 0.5"), 2.0);
    }

    #[test]
    fn test_power_right_associative() {
        assert_eq!(calc("2 ^ 3 ^ 2"), 512.0);
    }

    #[test]
    fn test_power_precedence() {
        assert_eq!(calc("2 + 3 ^ 2"), 11.0);
        assert_eq!(calc("2 * 3 ^ 2"), 18.0);
    }

    #[test]
    fn test_constant_pi() {
        assert!((calc("pi") - std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    fn test_constant_e() {
        assert!((calc("e") - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn test_constant_in_expr() {
        assert!((calc("2 * pi") - 2.0 * std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    fn test_ans_variable() {
        assert_eq!(calc_with_ans("ans", 42.0), 42.0);
        assert_eq!(calc_with_ans("ans + 1", 10.0), 11.0);
        assert_eq!(calc_with_ans("ans * 2", 5.0), 10.0);
        assert_eq!(calc_with_ans("ans", 0.0), 0.0);
    }

    #[test]
    fn test_user_variable() {
        assert_eq!(calc_with_var("x + 1", "x", 5.0), 6.0);
        assert_eq!(calc_with_var("x * x", "x", 3.0), 9.0);
    }

    #[test]
    fn test_undefined_variable_is_error() {
        let env = Env::new();
        match parse("undefined_var").unwrap() {
            Stmt::Expr(expr) => assert!(eval(&expr, &env).is_err()),
            _ => panic!("expected Stmt::Expr"),
        }
    }

    #[test]
    fn test_assignment_parses() {
        match parse("x = 5").unwrap() {
            Stmt::Assign(name, expr) => {
                assert_eq!(name, "x");
                assert_eq!(eval(&expr, &Env::new()).unwrap(), 5.0);
            }
            _ => panic!("expected Stmt::Assign"),
        }
    }

    #[test]
    fn test_assignment_complex_expr() {
        match parse("result = 2 ^ 10 + 1").unwrap() {
            Stmt::Assign(name, expr) => {
                assert_eq!(name, "result");
                assert_eq!(eval(&expr, &Env::new()).unwrap(), 1025.0);
            }
            _ => panic!("expected Stmt::Assign"),
        }
    }

    #[test]
    fn test_fn_empty_args_uses_ans() {
        assert_eq!(calc_with_ans("sqrt()", 4.0), 2.0);
        assert_eq!(calc_with_ans("abs()", -7.0), 7.0);
        assert_eq!(calc_with_ans("floor()", 3.9), 3.0);
        assert_eq!(calc_with_ans("ceil()", 3.1), 4.0);
        assert_eq!(calc_with_ans("round()", 3.5), 4.0);
    }

    #[test]
    fn test_fn_ans_arg() {
        assert_eq!(calc_with_ans("sqrt(ans)", 9.0), 3.0);
        assert_eq!(calc_with_ans("abs(ans)", -5.0), 5.0);
    }

    #[test]
    fn test_fn_sqrt() {
        assert_eq!(calc("sqrt(144)"), 12.0);
        assert_eq!(calc("sqrt(4)"), 2.0);
    }

    #[test]
    fn test_fn_abs() {
        assert_eq!(calc("abs(-7)"), 7.0);
        assert_eq!(calc("abs(3)"), 3.0);
    }

    #[test]
    fn test_fn_floor() {
        assert_eq!(calc("floor(3.9)"), 3.0);
        assert_eq!(calc("floor(-1.1)"), -2.0);
    }

    #[test]
    fn test_fn_ceil() {
        assert_eq!(calc("ceil(3.1)"), 4.0);
        assert_eq!(calc("ceil(-1.9)"), -1.0);
    }

    #[test]
    fn test_fn_round() {
        assert_eq!(calc("round(3.4)"), 3.0);
        assert_eq!(calc("round(3.5)"), 4.0);
    }

    #[test]
    fn test_fn_log() {
        assert!((calc("log(1000)") - 3.0).abs() < 1e-10);
        assert_eq!(calc("log(1)"), 0.0);
    }

    #[test]
    fn test_fn_ln() {
        assert_eq!(calc("ln(1)"), 0.0);
        assert!((calc("ln(e)") - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_fn_exp() {
        assert_eq!(calc("exp(0)"), 1.0);
        assert!((calc("exp(1)") - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn test_fn_sin() {
        assert!((calc("sin(0)")).abs() < 1e-15);
        assert!((calc("sin(pi / 6)") - 0.5).abs() < 1e-15);
    }

    #[test]
    fn test_fn_cos() {
        assert!((calc("cos(0)") - 1.0).abs() < 1e-15);
        assert!((calc("cos(pi)") + 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_fn_tan() {
        assert!((calc("tan(0)")).abs() < 1e-15);
        assert!((calc("tan(pi / 4)") - 1.0).abs() < 1e-14);
    }

    #[test]
    fn test_fn_nested() {
        assert!((calc("sqrt(abs(-16))") - 4.0).abs() < 1e-15);
    }

    #[test]
    fn test_fn_in_expr() {
        assert_eq!(calc("sqrt(144) + 3"), 15.0);
    }

    #[test]
    fn test_hex_literal() {
        assert_eq!(calc("0xFF"), 255.0);
        assert_eq!(calc("0x10"), 16.0);
        assert_eq!(calc("0XFF"), 255.0);
    }

    #[test]
    fn test_bin_literal() {
        assert_eq!(calc("0b1010"), 10.0);
        assert_eq!(calc("0b1"), 1.0);
        assert_eq!(calc("0B1111"), 15.0);
    }

    #[test]
    fn test_oct_literal() {
        assert_eq!(calc("0o17"), 15.0);
        assert_eq!(calc("0o10"), 8.0);
        assert_eq!(calc("0O377"), 255.0);
    }

    #[test]
    fn test_mixed_base_expression() {
        assert_eq!(calc("0xFF + 0b1010"), 265.0);
        assert_eq!(calc("0x10 + 0o10 + 0b10"), 26.0);
    }

    #[test]
    fn test_hex_error_no_digits() {
        assert!(parse("0x").is_err());
        assert!(parse("0b").is_err());
        assert!(parse("0o").is_err());
    }

    #[test]
    fn test_decimal_zero_still_works() {
        assert_eq!(calc("0"), 0.0);
        assert_eq!(calc("0.5"), 0.5);
    }

    #[test]
    fn test_is_partial() {
        assert!(is_partial("+ 2"));
        assert!(is_partial("- 3"));
        assert!(is_partial("* 100"));
        assert!(is_partial("/ 2"));
        assert!(is_partial("^ 2"));
        assert!(is_partial("% 3"));
        assert!(!is_partial("1 + 1"));
        assert!(!is_partial("(3 + 3) * 2"));
        assert!(!is_partial("sqrt(4)"));
    }

    #[test]
    fn test_parse_error_empty() {
        assert!(parse("").is_err());
    }

    #[test]
    fn test_parse_error_unmatched_paren() {
        assert!(parse("(1 + 2").is_err());
    }

    #[test]
    fn test_parse_error_invalid_char() {
        assert!(parse("1 @ 2").is_err());
    }

    #[test]
    fn test_sci_notation_positive_exponent() {
        assert_eq!(calc("1e5"), 100000.0);
        assert_eq!(calc("1E5"), 100000.0);
        assert_eq!(calc("2.5e2"), 250.0);
        assert_eq!(calc("1e+5"), 100000.0);
    }

    #[test]
    fn test_sci_notation_negative_exponent() {
        assert!((calc("1e-5") - 1e-5).abs() < 1e-20);
        assert!((calc("1e-17") - 1e-17).abs() < 1e-32);
        assert!((calc("2.5e-3") - 0.0025).abs() < 1e-15);
    }

    #[test]
    fn test_sci_notation_in_expression() {
        assert!((calc("1e-5 * 100") - 1e-3).abs() < 1e-18);
        assert!((calc("1e3 + 1e2") - 1100.0).abs() < 1e-10);
        assert!((calc("1e-5 + 2e-5") - 3e-5).abs() < 1e-20);
    }

    #[test]
    fn test_sci_notation_zero() {
        assert_eq!(calc("0e5"), 0.0);
        assert_eq!(calc("0e-3"), 0.0);
    }

    #[test]
    fn test_constant_e_still_works() {
        assert!((calc("e") - std::f64::consts::E).abs() < 1e-15);
        assert!((calc("1 + e") - (1.0 + std::f64::consts::E)).abs() < 1e-15);
        assert!((calc("e ^ 2") - std::f64::consts::E.powi(2)).abs() < 1e-10);
    }

    #[test]
    fn test_eval_error_unknown_function() {
        let env = Env::new();
        match parse("foo(1)").unwrap() {
            Stmt::Expr(expr) => assert!(eval(&expr, &env).is_err()),
            _ => panic!("expected Stmt::Expr"),
        }
    }

    // --- Implicit multiplication ---

    #[test]
    fn test_implicit_mul_number_paren() {
        assert_eq!(calc("2(3 + 1)"), 8.0);
        assert_eq!(calc("3(2)"), 6.0);
        assert_eq!(calc("5(0)"), 0.0);
    }

    #[test]
    fn test_implicit_mul_paren_paren() {
        assert_eq!(calc("(2 + 1)(4 - 1)"), 9.0);
        assert_eq!(calc("(10)(10)"), 100.0);
    }

    #[test]
    fn test_implicit_mul_precedence_with_add() {
        assert_eq!(calc("2(3) + 1"), 7.0);
        assert_eq!(calc("1 + 2(3)"), 7.0);
    }

    #[test]
    fn test_implicit_mul_chained() {
        assert_eq!(calc("2(3)(4)"), 24.0);
    }
}
