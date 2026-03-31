use crate::env::Env;

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Var(String),
    UnaryMinus(Box<Expr>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Base {
    #[default]
    Dec,
    Hex,
    Bin,
    Oct,
}

pub fn eval(expr: &Expr, env: &Env) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Var(name) => env
            .get(name)
            .copied()
            .ok_or_else(|| format!("Undefined variable: '{name}'")),
        Expr::UnaryMinus(e) => Ok(-eval(e, env)?),
        Expr::BinOp(left, op, right) => {
            let l = eval(left, env)?;
            let r = eval(right, env)?;
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
            }
        }
        Expr::Call(name, args) => {
            let evaled: Result<Vec<f64>, String> = args.iter().map(|a| eval(a, env)).collect();
            call_builtin(name, &evaled?)
        }
    }
}

fn call_builtin(name: &str, args: &[f64]) -> Result<f64, String> {
    match (name, args) {
        // 1-argument functions
        ("sqrt",  [x]) => Ok(x.sqrt()),
        ("abs",   [x]) => Ok(x.abs()),
        ("floor", [x]) => Ok(x.floor()),
        ("ceil",  [x]) => Ok(x.ceil()),
        ("round", [x]) => Ok(x.round()),
        ("sign",  [x]) => Ok(x.signum()),
        // Note: log(x) = log10 (ccalc convention); Octave uses log(x) = ln.
        // Use ln(x) or log(x, base) for natural/arbitrary-base logarithm.
        ("log",   [x]) => Ok(x.log10()),
        ("ln",    [x]) => Ok(x.ln()),
        ("exp",   [x]) => Ok(x.exp()),
        ("sin",   [x]) => Ok(x.sin()),
        ("cos",   [x]) => Ok(x.cos()),
        ("tan",   [x]) => Ok(x.tan()),
        ("asin",  [x]) => Ok(x.asin()),
        ("acos",  [x]) => Ok(x.acos()),
        ("atan",  [x]) => Ok(x.atan()),
        // 2-argument functions
        ("atan2", [y, x])    => Ok(y.atan2(*x)),
        ("mod",   [a, b])    => Ok(a - b * (a / b).floor()),
        ("rem",   [a, b])    => Ok(a - b * (a / b).trunc()),
        ("max",   [a, b])    => Ok(a.max(*b)),
        ("min",   [a, b])    => Ok(a.min(*b)),
        ("hypot", [a, b])    => Ok(a.hypot(*b)),
        ("log",   [x, base]) => Ok(x.log(*base)),
        _ => Err(format!("Unknown function: '{name}'")),
    }
}

/// Formats a number for display: integers without decimal point,
/// floats with up to 10 significant fractional digits, trailing zeros trimmed.
/// Always decimal — used for expression re-display, not user-facing output.
pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else if n != 0.0 && (n.abs() >= 1e15 || n.abs() < 1e-9) {
        trim_sci(&format!("{:.15e}", n))
    } else {
        let s = format!("{:.10}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Formats a number for user-facing output using the given base and decimal precision.
pub fn format_value(n: f64, precision: usize, base: Base) -> String {
    match base {
        Base::Dec => format_decimal(n, precision),
        _ => format_non_dec(n, base),
    }
}

/// Formats a number in a non-decimal integer base (hex/bin/oct).
/// Rounds to the nearest integer before formatting.
pub fn format_non_dec(n: f64, base: Base) -> String {
    let i = n.round() as i64;
    let u = i.unsigned_abs();
    let sign = if i < 0 { "-" } else { "" };
    match base {
        Base::Hex => format!("{}0x{:X}", sign, u),
        Base::Bin => format!("{}0b{:b}", sign, u),
        Base::Oct => format!("{}0o{:o}", sign, u),
        Base::Dec => format_decimal(n, 10),
    }
}

fn format_decimal(n: f64, precision: usize) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else if n.abs() >= 1e15 || (n != 0.0 && n.abs() < 1e-9) {
        let s = format!("{:.prec$e}", n, prec = precision);
        trim_sci(&s)
    } else {
        let s = format!("{:.prec$}", n, prec = precision);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn trim_sci(s: &str) -> String {
    if let Some(e_pos) = s.find('e') {
        let mantissa = s[..e_pos].trim_end_matches('0').trim_end_matches('.');
        format!("{}{}", mantissa, &s[e_pos..])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_env() -> Env {
        Env::new()
    }

    fn env_with_ans(val: f64) -> Env {
        let mut env = Env::new();
        env.insert("ans".to_string(), val);
        env
    }

    #[test]
    fn test_eval_number() {
        assert_eq!(eval(&Expr::Number(42.0), &empty_env()).unwrap(), 42.0);
    }

    #[test]
    fn test_eval_var_found() {
        let mut env = Env::new();
        env.insert("x".to_string(), 7.0);
        assert_eq!(eval(&Expr::Var("x".to_string()), &env).unwrap(), 7.0);
    }

    #[test]
    fn test_eval_var_not_found() {
        assert!(eval(&Expr::Var("z".to_string()), &empty_env()).is_err());
    }

    #[test]
    fn test_eval_ans() {
        assert_eq!(
            eval(&Expr::Var("ans".to_string()), &env_with_ans(42.0)).unwrap(),
            42.0
        );
    }

    #[test]
    fn test_eval_add() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(1.0)),
            Op::Add,
            Box::new(Expr::Number(2.0)),
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 3.0);
    }

    #[test]
    fn test_eval_sub() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(10.0)),
            Op::Sub,
            Box::new(Expr::Number(4.0)),
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 6.0);
    }

    #[test]
    fn test_eval_mul() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(3.0)),
            Op::Mul,
            Box::new(Expr::Number(7.0)),
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 21.0);
    }

    #[test]
    fn test_eval_div() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(10.0)),
            Op::Div,
            Box::new(Expr::Number(4.0)),
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 2.5);
    }

    #[test]
    fn test_eval_div_by_zero() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(1.0)),
            Op::Div,
            Box::new(Expr::Number(0.0)),
        );
        assert!(eval(&expr, &empty_env()).is_err());
    }

    #[test]
    fn test_eval_unary_minus() {
        let expr = Expr::UnaryMinus(Box::new(Expr::Number(5.0)));
        assert_eq!(eval(&expr, &empty_env()).unwrap(), -5.0);
    }

    #[test]
    fn test_eval_pow() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(2.0)),
            Op::Pow,
            Box::new(Expr::Number(10.0)),
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 1024.0);
    }

    #[test]
    fn test_eval_call_sqrt() {
        let expr = Expr::Call("sqrt".to_string(), vec![Expr::Number(144.0)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 12.0);
    }

    #[test]
    fn test_eval_call_abs() {
        let expr = Expr::Call("abs".to_string(), vec![Expr::Number(-7.0)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 7.0);
    }

    #[test]
    fn test_eval_call_floor() {
        let expr = Expr::Call("floor".to_string(), vec![Expr::Number(3.9)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 3.0);
    }

    #[test]
    fn test_eval_call_ceil() {
        let expr = Expr::Call("ceil".to_string(), vec![Expr::Number(3.1)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 4.0);
    }

    #[test]
    fn test_eval_call_round() {
        let expr = Expr::Call("round".to_string(), vec![Expr::Number(3.5)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 4.0);
    }

    #[test]
    fn test_eval_call_log() {
        let expr = Expr::Call("log".to_string(), vec![Expr::Number(1000.0)]);
        assert!((eval(&expr, &empty_env()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_call_ln() {
        let expr = Expr::Call("ln".to_string(), vec![Expr::Number(1.0)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 0.0);
    }

    #[test]
    fn test_eval_call_exp() {
        let expr = Expr::Call("exp".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 1.0);
    }

    #[test]
    fn test_eval_call_sin() {
        let expr = Expr::Call("sin".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 0.0);
    }

    #[test]
    fn test_eval_call_cos() {
        let expr = Expr::Call("cos".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 1.0);
    }

    #[test]
    fn test_eval_call_tan() {
        let expr = Expr::Call("tan".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 0.0);
    }

    #[test]
    fn test_eval_call_unknown() {
        let expr = Expr::Call("foo".to_string(), vec![Expr::Number(1.0)]);
        assert!(eval(&expr, &empty_env()).is_err());
    }

    #[test]
    fn test_eval_call_atan2() {
        let expr = Expr::Call(
            "atan2".to_string(),
            vec![Expr::Number(1.0), Expr::Number(1.0)],
        );
        assert!((eval(&expr, &empty_env()).unwrap() - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn test_eval_call_mod() {
        let expr = Expr::Call(
            "mod".to_string(),
            vec![Expr::Number(10.0), Expr::Number(3.0)],
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 1.0);
    }

    #[test]
    fn test_eval_call_mod_negative() {
        // mod(-1, 3) = 2  (sign follows divisor, Octave convention)
        let expr = Expr::Call(
            "mod".to_string(),
            vec![Expr::Number(-1.0), Expr::Number(3.0)],
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 2.0);
    }

    #[test]
    fn test_eval_call_rem() {
        // rem(-1, 3) = -1  (sign follows dividend)
        let expr = Expr::Call(
            "rem".to_string(),
            vec![Expr::Number(-1.0), Expr::Number(3.0)],
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), -1.0);
    }

    #[test]
    fn test_eval_call_max() {
        let expr = Expr::Call(
            "max".to_string(),
            vec![Expr::Number(3.0), Expr::Number(7.0)],
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 7.0);
    }

    #[test]
    fn test_eval_call_min() {
        let expr = Expr::Call(
            "min".to_string(),
            vec![Expr::Number(3.0), Expr::Number(7.0)],
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 3.0);
    }

    #[test]
    fn test_eval_call_hypot() {
        let expr = Expr::Call(
            "hypot".to_string(),
            vec![Expr::Number(3.0), Expr::Number(4.0)],
        );
        assert_eq!(eval(&expr, &empty_env()).unwrap(), 5.0);
    }

    #[test]
    fn test_eval_call_log_two_arg() {
        // log(8, 2) = 3
        let expr = Expr::Call(
            "log".to_string(),
            vec![Expr::Number(8.0), Expr::Number(2.0)],
        );
        assert!((eval(&expr, &empty_env()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_call_asin_acos_atan() {
        let env = empty_env();
        let asin = Expr::Call("asin".to_string(), vec![Expr::Number(1.0)]);
        assert!((eval(&asin, &env).unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
        let acos = Expr::Call("acos".to_string(), vec![Expr::Number(1.0)]);
        assert!(eval(&acos, &env).unwrap().abs() < 1e-10);
        let atan = Expr::Call("atan".to_string(), vec![Expr::Number(1.0)]);
        assert!((eval(&atan, &env).unwrap() - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
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

    #[test]
    fn test_format_number_sci() {
        let s = format_number(1e-12);
        assert!(s.contains('e'), "expected sci notation, got: {s}");
        assert!((s.parse::<f64>().unwrap() - 1e-12).abs() < 1e-25);
        let s = format_number(1e20);
        assert!(s.contains('e'), "expected sci notation, got: {s}");
        assert!((s.parse::<f64>().unwrap() - 1e20).abs() < 1e10);
    }

    #[test]
    fn test_format_value_dec_integer() {
        assert_eq!(format_value(42.0, 10, Base::Dec), "42");
        assert_eq!(format_value(-5.0, 10, Base::Dec), "-5");
    }

    #[test]
    fn test_format_value_dec_float() {
        assert_eq!(format_value(3.14, 2, Base::Dec), "3.14");
        assert_eq!(format_value(1.0 / 3.0, 4, Base::Dec), "0.3333");
    }

    #[test]
    fn test_format_value_dec_sci_large() {
        let result = format_value(1e20, 2, Base::Dec);
        assert!(
            result.contains('e'),
            "expected scientific notation, got: {result}"
        );
    }

    #[test]
    fn test_format_value_dec_sci_small() {
        let result = format_value(1e-10, 4, Base::Dec);
        assert!(
            result.contains('e'),
            "expected scientific notation, got: {result}"
        );
    }

    #[test]
    fn test_format_value_hex() {
        assert_eq!(format_value(255.0, 10, Base::Hex), "0xFF");
        assert_eq!(format_value(256.0, 10, Base::Hex), "0x100");
        assert_eq!(format_value(0.0, 10, Base::Hex), "0x0");
    }

    #[test]
    fn test_format_value_bin() {
        assert_eq!(format_value(10.0, 10, Base::Bin), "0b1010");
        assert_eq!(format_value(1.0, 10, Base::Bin), "0b1");
    }

    #[test]
    fn test_format_value_oct() {
        assert_eq!(format_value(8.0, 10, Base::Oct), "0o10");
        assert_eq!(format_value(255.0, 10, Base::Oct), "0o377");
    }

    #[test]
    fn test_format_non_dec_negative() {
        assert_eq!(format_non_dec(-16.0, Base::Hex), "-0x10");
        assert_eq!(format_non_dec(-2.0, Base::Bin), "-0b10");
    }

    #[test]
    fn test_format_value_hex_rounds() {
        assert_eq!(format_value(255.6, 10, Base::Hex), "0x100");
    }
}
