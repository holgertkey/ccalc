use crate::eval::{Expr, Op};

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
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
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_str.push(c);
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
            _ => return Err(format!("Unexpected character: '{c}'")),
        }
    }

    Ok(tokens)
}

/// Parses a full expression string into an AST.
pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut pos = 0;
    let expr = parse_expr(&tokens, &mut pos)?;
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
        Some('+' | '-' | '*' | '/')
    )
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

// term = factor (('*' | '/') factor)*
fn parse_term(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_factor(tokens, pos)?;

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Star => {
                *pos += 1;
                let right = parse_factor(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::Mul, Box::new(right));
            }
            Token::Slash => {
                *pos += 1;
                let right = parse_factor(tokens, pos)?;
                left = Expr::BinOp(Box::new(left), Op::Div, Box::new(right));
            }
            _ => break,
        }
    }

    Ok(left)
}

// factor = '-' factor | '(' expr ')' | number
fn parse_factor(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of expression".to_string());
    }

    match &tokens[*pos] {
        Token::Minus => {
            *pos += 1;
            let expr = parse_factor(tokens, pos)?;
            Ok(Expr::UnaryMinus(Box::new(expr)))
        }
        Token::Number(n) => {
            let n = *n;
            *pos += 1;
            Ok(Expr::Number(n))
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
        _ => Err("Expected number, '-', or '('".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::eval;

    fn calc(input: &str) -> f64 {
        eval(&parse(input).unwrap()).unwrap()
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
    fn test_is_partial() {
        assert!(is_partial("+ 2"));
        assert!(is_partial("- 3"));
        assert!(is_partial("* 100"));
        assert!(is_partial("/ 2"));
        assert!(!is_partial("1 + 1"));
        assert!(!is_partial("(3 + 3) * 2"));
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
}
