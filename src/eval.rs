#[derive(Debug)]
pub enum Expr {
    Number(f64),
    UnaryMinus(Box<Expr>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Call(String, Box<Expr>),
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}

pub fn eval(expr: &Expr) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::UnaryMinus(e) => Ok(-eval(e)?),
        Expr::BinOp(left, op, right) => {
            let l = eval(left)?;
            let r = eval(right)?;
            match op {
                Op::Add => Ok(l + r),
                Op::Sub => Ok(l - r),
                Op::Mul => Ok(l * r),
                Op::Div => {
                    if r == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(l / r)
                    }
                }
                Op::Pow => Ok(l.powf(r)),
                Op::Mod => {
                    if r == 0.0 {
                        Err("Modulo by zero".to_string())
                    } else {
                        Ok(l % r)
                    }
                }
            }
        }
        Expr::Call(name, arg) => {
            let x = eval(arg)?;
            match name.as_str() {
                "sqrt"  => Ok(x.sqrt()),
                "abs"   => Ok(x.abs()),
                "floor" => Ok(x.floor()),
                "ceil"  => Ok(x.ceil()),
                "round" => Ok(x.round()),
                "log"   => Ok(x.log10()),
                "ln"    => Ok(x.ln()),
                "exp"   => Ok(x.exp()),
                "sin"   => Ok(x.sin()),
                "cos"   => Ok(x.cos()),
                "tan"   => Ok(x.tan()),
                _ => Err(format!("Unknown function: '{name}'")),
            }
        }
    }
}

/// Formats a number for display: integers without decimal point,
/// floats with up to 10 significant fractional digits, trailing zeros trimmed.
pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.10}", n);
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_number() {
        assert_eq!(eval(&Expr::Number(42.0)).unwrap(), 42.0);
    }

    #[test]
    fn test_eval_add() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(1.0)),
            Op::Add,
            Box::new(Expr::Number(2.0)),
        );
        assert_eq!(eval(&expr).unwrap(), 3.0);
    }

    #[test]
    fn test_eval_sub() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(10.0)),
            Op::Sub,
            Box::new(Expr::Number(4.0)),
        );
        assert_eq!(eval(&expr).unwrap(), 6.0);
    }

    #[test]
    fn test_eval_mul() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(3.0)),
            Op::Mul,
            Box::new(Expr::Number(7.0)),
        );
        assert_eq!(eval(&expr).unwrap(), 21.0);
    }

    #[test]
    fn test_eval_div() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(10.0)),
            Op::Div,
            Box::new(Expr::Number(4.0)),
        );
        assert_eq!(eval(&expr).unwrap(), 2.5);
    }

    #[test]
    fn test_eval_div_by_zero() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(1.0)),
            Op::Div,
            Box::new(Expr::Number(0.0)),
        );
        assert!(eval(&expr).is_err());
    }

    #[test]
    fn test_eval_unary_minus() {
        let expr = Expr::UnaryMinus(Box::new(Expr::Number(5.0)));
        assert_eq!(eval(&expr).unwrap(), -5.0);
    }

    #[test]
    fn test_eval_pow() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(2.0)),
            Op::Pow,
            Box::new(Expr::Number(10.0)),
        );
        assert_eq!(eval(&expr).unwrap(), 1024.0);
    }

    #[test]
    fn test_eval_mod() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(17.0)),
            Op::Mod,
            Box::new(Expr::Number(5.0)),
        );
        assert_eq!(eval(&expr).unwrap(), 2.0);
    }

    #[test]
    fn test_eval_mod_by_zero() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(5.0)),
            Op::Mod,
            Box::new(Expr::Number(0.0)),
        );
        assert!(eval(&expr).is_err());
    }

    #[test]
    fn test_eval_call_sqrt() {
        let expr = Expr::Call("sqrt".to_string(), Box::new(Expr::Number(144.0)));
        assert_eq!(eval(&expr).unwrap(), 12.0);
    }

    #[test]
    fn test_eval_call_abs() {
        let expr = Expr::Call("abs".to_string(), Box::new(Expr::Number(-7.0)));
        assert_eq!(eval(&expr).unwrap(), 7.0);
    }

    #[test]
    fn test_eval_call_floor() {
        let expr = Expr::Call("floor".to_string(), Box::new(Expr::Number(3.9)));
        assert_eq!(eval(&expr).unwrap(), 3.0);
    }

    #[test]
    fn test_eval_call_ceil() {
        let expr = Expr::Call("ceil".to_string(), Box::new(Expr::Number(3.1)));
        assert_eq!(eval(&expr).unwrap(), 4.0);
    }

    #[test]
    fn test_eval_call_round() {
        let expr = Expr::Call("round".to_string(), Box::new(Expr::Number(3.5)));
        assert_eq!(eval(&expr).unwrap(), 4.0);
    }

    #[test]
    fn test_eval_call_log() {
        let expr = Expr::Call("log".to_string(), Box::new(Expr::Number(1000.0)));
        assert!((eval(&expr).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_call_ln() {
        let expr = Expr::Call("ln".to_string(), Box::new(Expr::Number(1.0)));
        assert_eq!(eval(&expr).unwrap(), 0.0);
    }

    #[test]
    fn test_eval_call_exp() {
        let expr = Expr::Call("exp".to_string(), Box::new(Expr::Number(0.0)));
        assert_eq!(eval(&expr).unwrap(), 1.0);
    }

    #[test]
    fn test_eval_call_sin() {
        let expr = Expr::Call("sin".to_string(), Box::new(Expr::Number(0.0)));
        assert_eq!(eval(&expr).unwrap(), 0.0);
    }

    #[test]
    fn test_eval_call_cos() {
        let expr = Expr::Call("cos".to_string(), Box::new(Expr::Number(0.0)));
        assert_eq!(eval(&expr).unwrap(), 1.0);
    }

    #[test]
    fn test_eval_call_tan() {
        let expr = Expr::Call("tan".to_string(), Box::new(Expr::Number(0.0)));
        assert_eq!(eval(&expr).unwrap(), 0.0);
    }

    #[test]
    fn test_eval_call_unknown() {
        let expr = Expr::Call("foo".to_string(), Box::new(Expr::Number(1.0)));
        assert!(eval(&expr).is_err());
    }

    #[test]
    fn test_format_number_integer() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(-5.0), "-5");
        assert_eq!(format_number(400.0), "400");
    }

    #[test]
    fn test_format_number_float() {
        assert_eq!(format_number(2.5), "2.5");
        assert_eq!(format_number(3.14), "3.14");
        assert_eq!(format_number(0.1 + 0.2), "0.3");
    }
}
