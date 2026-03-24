use crate::eval::{Expr, Op};

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
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
                tokens.push(Token::Percent);
                chars.next();
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
                    chars.next(); // consume '0'
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
                            // Decimal number starting with '0' (e.g. 0.5 or just 0)
                            let mut num_str = String::from("0");
                            while let Some(&d) = chars.peek() {
                                if d.is_ascii_digit() || d == '.' {
                                    num_str.push(d);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            let n: f64 = num_str
                                .parse()
                                .map_err(|_| format!("Invalid number: '{num_str}'"))?;
                            tokens.push(Token::Number(n));
                        }
                    }
                } else {
                    // Decimal number not starting with '0'
                    let mut num_str = String::new();
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() || d == '.' {
                            num_str.push(d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
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

/// Parses a full expression string into an AST.
/// `accumulator` is the current REPL value, used by `acc` and empty-arg function calls.
pub fn parse(input: &str, accumulator: f64) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut pos = 0;
    let expr = parse_expr(&tokens, &mut pos, accumulator)?;
    if pos != tokens.len() {
        return Err("Unexpected token after expression".to_string());
    }
    Ok(expr)
}

/// Returns true if the input looks like a partial expression
/// (i.e. starts with an operator that needs a left-hand operand).
pub fn is_partial(input: &str) -> bool {
    matches!(
        input.trim_start().chars().next(),
        Some('+' | '-' | '*' | '/' | '^' | '%')
    )
}

// expr = term (('+' | '-') term)*
fn parse_expr(tokens: &[Token], pos: &mut usize, acc: f64) -> Result<Expr, String> {
    let mut left = parse_term(tokens, pos, acc)?;

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Plus => {
                *pos += 1;
                let right = parse_term(tokens, pos, acc)?;
                left = Expr::BinOp(Box::new(left), Op::Add, Box::new(right));
            }
            Token::Minus => {
                *pos += 1;
                let right = parse_term(tokens, pos, acc)?;
                left = Expr::BinOp(Box::new(left), Op::Sub, Box::new(right));
            }
            _ => break,
        }
    }

    Ok(left)
}

// term = power (('*' | '/' | '%') power)*
fn parse_term(tokens: &[Token], pos: &mut usize, acc: f64) -> Result<Expr, String> {
    let mut left = parse_power(tokens, pos, acc)?;

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Star => {
                *pos += 1;
                let right = parse_power(tokens, pos, acc)?;
                left = Expr::BinOp(Box::new(left), Op::Mul, Box::new(right));
            }
            Token::Slash => {
                *pos += 1;
                let right = parse_power(tokens, pos, acc)?;
                left = Expr::BinOp(Box::new(left), Op::Div, Box::new(right));
            }
            Token::Percent => {
                *pos += 1;
                let right = parse_power(tokens, pos, acc)?;
                left = Expr::BinOp(Box::new(left), Op::Mod, Box::new(right));
            }
            _ => break,
        }
    }

    Ok(left)
}

// power = unary ('^' power)?   -- right-associative
fn parse_power(tokens: &[Token], pos: &mut usize, acc: f64) -> Result<Expr, String> {
    let base = parse_unary(tokens, pos, acc)?;
    if *pos < tokens.len() {
        if let Token::Caret = &tokens[*pos] {
            *pos += 1;
            let exp = parse_power(tokens, pos, acc)?;
            return Ok(Expr::BinOp(Box::new(base), Op::Pow, Box::new(exp)));
        }
    }
    Ok(base)
}

// unary = '-' unary | primary
fn parse_unary(tokens: &[Token], pos: &mut usize, acc: f64) -> Result<Expr, String> {
    if *pos < tokens.len() {
        if let Token::Minus = &tokens[*pos] {
            *pos += 1;
            let expr = parse_unary(tokens, pos, acc)?;
            return Ok(Expr::UnaryMinus(Box::new(expr)));
        }
    }
    parse_primary(tokens, pos, acc)
}

// primary = ident '(' expr ')' | ident '(' ')' | '(' expr ')' | number | ident
fn parse_primary(tokens: &[Token], pos: &mut usize, acc: f64) -> Result<Expr, String> {
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
            if *pos < tokens.len() {
                if let Token::LParen = &tokens[*pos] {
                    *pos += 1;
                    // Empty args: fn() uses the accumulator
                    let arg = if *pos < tokens.len() {
                        if let Token::RParen = &tokens[*pos] {
                            Box::new(Expr::Number(acc))
                        } else {
                            Box::new(parse_expr(tokens, pos, acc)?)
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
            }
            // Constants and accumulator alias
            match name.as_str() {
                "pi"  => Ok(Expr::Number(std::f64::consts::PI)),
                "e"   => Ok(Expr::Number(std::f64::consts::E)),
                "acc" => Ok(Expr::Number(acc)),
                _     => Err(format!("Unknown identifier: '{name}'")),
            }
        }
        Token::LParen => {
            *pos += 1;
            let expr = parse_expr(tokens, pos, acc)?;
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
        _ => Err("Expected number, function, constant, '-', or '('".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::eval;

    fn calc(input: &str) -> f64 {
        eval(&parse(input, 0.0).unwrap()).unwrap()
    }

    fn calc_with(input: &str, acc: f64) -> f64 {
        eval(&parse(input, acc).unwrap()).unwrap()
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
        // 2^3^2 = 2^(3^2) = 2^9 = 512
        assert_eq!(calc("2 ^ 3 ^ 2"), 512.0);
    }

    #[test]
    fn test_power_precedence() {
        assert_eq!(calc("2 + 3 ^ 2"), 11.0);
        assert_eq!(calc("2 * 3 ^ 2"), 18.0);
    }

    #[test]
    fn test_modulo() {
        assert_eq!(calc("17 % 5"), 2.0);
        assert_eq!(calc("10 % 3"), 1.0);
        assert_eq!(calc("6 % 2"), 0.0);
    }

    #[test]
    fn test_modulo_precedence() {
        assert_eq!(calc("10 + 17 % 5"), 12.0);
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
    fn test_acc() {
        assert_eq!(calc_with("acc", 42.0), 42.0);
        assert_eq!(calc_with("acc + 1", 10.0), 11.0);
        assert_eq!(calc_with("acc * 2", 5.0), 10.0);
        assert_eq!(calc_with("acc", 0.0), 0.0);
    }

    #[test]
    fn test_fn_empty_args_uses_accumulator() {
        assert_eq!(calc_with("sqrt()", 4.0), 2.0);
        assert_eq!(calc_with("abs()", -7.0), 7.0);
        assert_eq!(calc_with("floor()", 3.9), 3.0);
        assert_eq!(calc_with("ceil()", 3.1), 4.0);
        assert_eq!(calc_with("round()", 3.5), 4.0);
    }

    #[test]
    fn test_fn_acc_arg() {
        assert_eq!(calc_with("sqrt(acc)", 9.0), 3.0);
        assert_eq!(calc_with("abs(acc)", -5.0), 5.0);
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
        assert!(parse("0x", 0.0).is_err());
        assert!(parse("0b", 0.0).is_err());
        assert!(parse("0o", 0.0).is_err());
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
        assert!(parse("", 0.0).is_err());
    }

    #[test]
    fn test_parse_error_unmatched_paren() {
        assert!(parse("(1 + 2", 0.0).is_err());
    }

    #[test]
    fn test_parse_error_invalid_char() {
        assert!(parse("1 @ 2", 0.0).is_err());
    }

    #[test]
    fn test_parse_error_unknown_ident() {
        assert!(parse("foo", 0.0).is_err());
    }

    #[test]
    fn test_eval_error_unknown_function() {
        // parse succeeds — unknown function is caught at eval time
        let ast = parse("foo(1)", 0.0).unwrap();
        assert!(eval(&ast).is_err());
    }
}
