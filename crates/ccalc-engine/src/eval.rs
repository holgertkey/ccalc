use ndarray::Array2;

use crate::env::{Env, Value};

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Var(String),
    UnaryMinus(Box<Expr>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Call(String, Vec<Expr>),
    Matrix(Vec<Vec<Expr>>),
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

pub fn eval(expr: &Expr, env: &Env) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Scalar(*n)),
        Expr::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined variable: '{name}'")),
        Expr::UnaryMinus(e) => match eval(e, env)? {
            Value::Scalar(n) => Ok(Value::Scalar(-n)),
            Value::Matrix(m) => Ok(Value::Matrix(m.mapv(|x| -x))),
        },
        Expr::BinOp(left, op, right) => {
            let l = eval(left, env)?;
            let r = eval(right, env)?;
            eval_binop(l, op, r)
        }
        Expr::Call(name, args) => {
            let evaled: Result<Vec<Value>, String> = args.iter().map(|a| eval(a, env)).collect();
            call_builtin(name, &evaled?)
        }
        Expr::Matrix(rows) => {
            if rows.is_empty() {
                // Empty matrix: 0x0
                let m = Array2::<f64>::zeros((0, 0));
                return Ok(Value::Matrix(m));
            }
            let nrows = rows.len();
            let ncols = rows[0].len();
            // Validate all rows have the same number of columns
            for (i, row) in rows.iter().enumerate() {
                if row.len() != ncols {
                    return Err(format!(
                        "Matrix row {} has {} elements, expected {}",
                        i,
                        row.len(),
                        ncols
                    ));
                }
            }
            let mut flat = Vec::with_capacity(nrows * ncols);
            for row in rows {
                for elem_expr in row {
                    match eval(elem_expr, env)? {
                        Value::Scalar(n) => flat.push(n),
                        Value::Matrix(_) => {
                            return Err(
                                "Matrix elements must be scalars (nested matrices not supported)"
                                    .to_string(),
                            );
                        }
                    }
                }
            }
            let m = Array2::from_shape_vec((nrows, ncols), flat)
                .map_err(|e| format!("Matrix shape error: {e}"))?;
            Ok(Value::Matrix(m))
        }
    }
}

fn eval_binop(l: Value, op: &Op, r: Value) -> Result<Value, String> {
    match (l, r) {
        (Value::Scalar(lv), Value::Scalar(rv)) => {
            let result = match op {
                Op::Add => lv + rv,
                Op::Sub => lv - rv,
                Op::Mul => lv * rv,
                Op::Div => {
                    if rv == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    lv / rv
                }
                Op::Pow => lv.powf(rv),
            };
            Ok(Value::Scalar(result))
        }
        (Value::Matrix(lm), Value::Matrix(rm)) => {
            if lm.shape() != rm.shape() {
                return Err(format!(
                    "Matrix size mismatch: {}x{} vs {}x{}",
                    lm.nrows(),
                    lm.ncols(),
                    rm.nrows(),
                    rm.ncols()
                ));
            }
            match op {
                Op::Add => Ok(Value::Matrix(&lm + &rm)),
                Op::Sub => Ok(Value::Matrix(&lm - &rm)),
                Op::Mul => Err(
                    "Matrix-matrix multiplication requires Phase 4".to_string(),
                ),
                Op::Div => Err("Matrix-matrix division is not supported".to_string()),
                Op::Pow => Err("Matrix-matrix power is not supported".to_string()),
            }
        }
        (Value::Scalar(s), Value::Matrix(m)) => match op {
            Op::Add => Ok(Value::Matrix(s + &m)),
            Op::Sub => Ok(Value::Matrix(m.mapv(|x| s - x))),
            Op::Mul => Ok(Value::Matrix(s * &m)),
            Op::Div => Err("Scalar / Matrix is not supported".to_string()),
            Op::Pow => Ok(Value::Matrix(m.mapv(|x| s.powf(x)))),
        },
        (Value::Matrix(m), Value::Scalar(s)) => match op {
            Op::Add => Ok(Value::Matrix(&m + s)),
            Op::Sub => Ok(Value::Matrix(&m - s)),
            Op::Mul => Ok(Value::Matrix(&m * s)),
            Op::Div => Ok(Value::Matrix(m.mapv(|x| x / s))),
            Op::Pow => Ok(Value::Matrix(m.mapv(|x| x.powf(s)))),
        },
    }
}

fn scalar_arg(v: &Value, fname: &str, pos: usize) -> Result<f64, String> {
    match v {
        Value::Scalar(n) => Ok(*n),
        Value::Matrix(_) => Err(format!(
            "Function '{fname}' argument {pos} must be a scalar, got a matrix"
        )),
    }
}

fn call_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    let result = match (name, args.len()) {
        // 1-argument functions
        ("sqrt",  1) => scalar_arg(&args[0], name, 1)?.sqrt(),
        ("abs",   1) => scalar_arg(&args[0], name, 1)?.abs(),
        ("floor", 1) => scalar_arg(&args[0], name, 1)?.floor(),
        ("ceil",  1) => scalar_arg(&args[0], name, 1)?.ceil(),
        ("round", 1) => scalar_arg(&args[0], name, 1)?.round(),
        ("sign",  1) => scalar_arg(&args[0], name, 1)?.signum(),
        // Note: log(x) = log10; use ln(x) for natural logarithm.
        ("log",   1) => scalar_arg(&args[0], name, 1)?.log10(),
        ("ln",    1) => scalar_arg(&args[0], name, 1)?.ln(),
        ("exp",   1) => scalar_arg(&args[0], name, 1)?.exp(),
        ("sin",   1) => scalar_arg(&args[0], name, 1)?.sin(),
        ("cos",   1) => scalar_arg(&args[0], name, 1)?.cos(),
        ("tan",   1) => scalar_arg(&args[0], name, 1)?.tan(),
        ("asin",  1) => scalar_arg(&args[0], name, 1)?.asin(),
        ("acos",  1) => scalar_arg(&args[0], name, 1)?.acos(),
        ("atan",  1) => scalar_arg(&args[0], name, 1)?.atan(),
        // 2-argument functions
        ("atan2", 2) => scalar_arg(&args[0], name, 1)?.atan2(scalar_arg(&args[1], name, 2)?),
        ("mod",   2) => {
            let a = scalar_arg(&args[0], name, 1)?;
            let b = scalar_arg(&args[1], name, 2)?;
            a - b * (a / b).floor()
        }
        ("rem",   2) => {
            let a = scalar_arg(&args[0], name, 1)?;
            let b = scalar_arg(&args[1], name, 2)?;
            a - b * (a / b).trunc()
        }
        ("max",   2) => scalar_arg(&args[0], name, 1)?.max(scalar_arg(&args[1], name, 2)?),
        ("min",   2) => scalar_arg(&args[0], name, 1)?.min(scalar_arg(&args[1], name, 2)?),
        ("hypot", 2) => scalar_arg(&args[0], name, 1)?.hypot(scalar_arg(&args[1], name, 2)?),
        ("log",   2) => scalar_arg(&args[0], name, 1)?.log(scalar_arg(&args[1], name, 2)?),
        _ => return Err(format!("Unknown function: '{name}'")),
    };
    Ok(Value::Scalar(result))
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

/// Formats a scalar `f64` for user-facing output using the given base and decimal precision.
/// Replaces the old `format_value(f64, ...)` signature for scalar use sites.
pub fn format_scalar(n: f64, precision: usize, base: Base) -> String {
    match base {
        Base::Dec => format_decimal(n, precision),
        _ => format_non_dec(n, base),
    }
}

/// Formats a `Value` compactly: scalars as a number string, matrices as `[NxM double]`.
pub fn format_value(v: &Value, precision: usize, base: Base) -> String {
    match v {
        Value::Scalar(n) => format_scalar(*n, precision, base),
        Value::Matrix(m) => format!("[{}x{} double]", m.nrows(), m.ncols()),
    }
}

/// Returns `None` for scalars; `Some(full_string)` for matrices.
/// The full string is the MATLAB-style column-aligned matrix display.
pub fn format_value_full(v: &Value, precision: usize) -> Option<String> {
    match v {
        Value::Scalar(_) => None,
        Value::Matrix(m) => Some(format_matrix(m, precision)),
    }
}

/// Formats a matrix with right-aligned columns, 3-space indent, 3 spaces between columns.
fn format_matrix(m: &Array2<f64>, precision: usize) -> String {
    if m.nrows() == 0 || m.ncols() == 0 {
        return "   []".to_string();
    }
    let ncols = m.ncols();
    // Format all cells
    let cells: Vec<Vec<String>> = m
        .rows()
        .into_iter()
        .map(|row| {
            row.iter()
                .map(|&x| format_decimal(x, precision))
                .collect()
        })
        .collect();
    // Compute column widths
    let col_widths: Vec<usize> = (0..ncols)
        .map(|c| cells.iter().map(|row| row[c].len()).max().unwrap_or(0))
        .collect();
    let mut lines = Vec::new();
    for row in &cells {
        let mut line = String::from("   "); // 3-space indent
        for (c, cell) in row.iter().enumerate() {
            if c > 0 {
                line.push_str("   "); // 3 spaces between columns
            }
            // Right-align
            let pad = col_widths[c].saturating_sub(cell.len());
            for _ in 0..pad {
                line.push(' ');
            }
            line.push_str(cell);
        }
        lines.push(line);
    }
    lines.join("\n")
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
        env.insert("ans".to_string(), Value::Scalar(val));
        env
    }

    // Helper to evaluate and extract scalar — panics if result is a matrix.
    fn eval_s(expr: &Expr, env: &Env) -> f64 {
        match eval(expr, env).unwrap() {
            Value::Scalar(n) => n,
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn test_eval_number() {
        assert_eq!(eval_s(&Expr::Number(42.0), &empty_env()), 42.0);
    }

    #[test]
    fn test_eval_var_found() {
        let mut env = Env::new();
        env.insert("x".to_string(), Value::Scalar(7.0));
        assert_eq!(eval_s(&Expr::Var("x".to_string()), &env), 7.0);
    }

    #[test]
    fn test_eval_var_not_found() {
        assert!(eval(&Expr::Var("z".to_string()), &empty_env()).is_err());
    }

    #[test]
    fn test_eval_ans() {
        assert_eq!(
            eval_s(&Expr::Var("ans".to_string()), &env_with_ans(42.0)),
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
        assert_eq!(eval_s(&expr, &empty_env()), 3.0);
    }

    #[test]
    fn test_eval_sub() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(10.0)),
            Op::Sub,
            Box::new(Expr::Number(4.0)),
        );
        assert_eq!(eval_s(&expr, &empty_env()), 6.0);
    }

    #[test]
    fn test_eval_mul() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(3.0)),
            Op::Mul,
            Box::new(Expr::Number(7.0)),
        );
        assert_eq!(eval_s(&expr, &empty_env()), 21.0);
    }

    #[test]
    fn test_eval_div() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(10.0)),
            Op::Div,
            Box::new(Expr::Number(4.0)),
        );
        assert_eq!(eval_s(&expr, &empty_env()), 2.5);
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
        assert_eq!(eval_s(&expr, &empty_env()), -5.0);
    }

    #[test]
    fn test_eval_pow() {
        let expr = Expr::BinOp(
            Box::new(Expr::Number(2.0)),
            Op::Pow,
            Box::new(Expr::Number(10.0)),
        );
        assert_eq!(eval_s(&expr, &empty_env()), 1024.0);
    }

    #[test]
    fn test_eval_call_sqrt() {
        let expr = Expr::Call("sqrt".to_string(), vec![Expr::Number(144.0)]);
        assert_eq!(eval_s(&expr, &empty_env()), 12.0);
    }

    #[test]
    fn test_eval_call_abs() {
        let expr = Expr::Call("abs".to_string(), vec![Expr::Number(-7.0)]);
        assert_eq!(eval_s(&expr, &empty_env()), 7.0);
    }

    #[test]
    fn test_eval_call_floor() {
        let expr = Expr::Call("floor".to_string(), vec![Expr::Number(3.9)]);
        assert_eq!(eval_s(&expr, &empty_env()), 3.0);
    }

    #[test]
    fn test_eval_call_ceil() {
        let expr = Expr::Call("ceil".to_string(), vec![Expr::Number(3.1)]);
        assert_eq!(eval_s(&expr, &empty_env()), 4.0);
    }

    #[test]
    fn test_eval_call_round() {
        let expr = Expr::Call("round".to_string(), vec![Expr::Number(3.5)]);
        assert_eq!(eval_s(&expr, &empty_env()), 4.0);
    }

    #[test]
    fn test_eval_call_log() {
        let expr = Expr::Call("log".to_string(), vec![Expr::Number(1000.0)]);
        assert!((eval_s(&expr, &empty_env()) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_call_ln() {
        let expr = Expr::Call("ln".to_string(), vec![Expr::Number(1.0)]);
        assert_eq!(eval_s(&expr, &empty_env()), 0.0);
    }

    #[test]
    fn test_eval_call_exp() {
        let expr = Expr::Call("exp".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval_s(&expr, &empty_env()), 1.0);
    }

    #[test]
    fn test_eval_call_sin() {
        let expr = Expr::Call("sin".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval_s(&expr, &empty_env()), 0.0);
    }

    #[test]
    fn test_eval_call_cos() {
        let expr = Expr::Call("cos".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval_s(&expr, &empty_env()), 1.0);
    }

    #[test]
    fn test_eval_call_tan() {
        let expr = Expr::Call("tan".to_string(), vec![Expr::Number(0.0)]);
        assert_eq!(eval_s(&expr, &empty_env()), 0.0);
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
        assert!((eval_s(&expr, &empty_env()) - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn test_eval_call_mod() {
        let expr = Expr::Call(
            "mod".to_string(),
            vec![Expr::Number(10.0), Expr::Number(3.0)],
        );
        assert_eq!(eval_s(&expr, &empty_env()), 1.0);
    }

    #[test]
    fn test_eval_call_mod_negative() {
        // mod(-1, 3) = 2  (sign follows divisor, Octave convention)
        let expr = Expr::Call(
            "mod".to_string(),
            vec![Expr::Number(-1.0), Expr::Number(3.0)],
        );
        assert_eq!(eval_s(&expr, &empty_env()), 2.0);
    }

    #[test]
    fn test_eval_call_rem() {
        // rem(-1, 3) = -1  (sign follows dividend)
        let expr = Expr::Call(
            "rem".to_string(),
            vec![Expr::Number(-1.0), Expr::Number(3.0)],
        );
        assert_eq!(eval_s(&expr, &empty_env()), -1.0);
    }

    #[test]
    fn test_eval_call_max() {
        let expr = Expr::Call(
            "max".to_string(),
            vec![Expr::Number(3.0), Expr::Number(7.0)],
        );
        assert_eq!(eval_s(&expr, &empty_env()), 7.0);
    }

    #[test]
    fn test_eval_call_min() {
        let expr = Expr::Call(
            "min".to_string(),
            vec![Expr::Number(3.0), Expr::Number(7.0)],
        );
        assert_eq!(eval_s(&expr, &empty_env()), 3.0);
    }

    #[test]
    fn test_eval_call_hypot() {
        let expr = Expr::Call(
            "hypot".to_string(),
            vec![Expr::Number(3.0), Expr::Number(4.0)],
        );
        assert_eq!(eval_s(&expr, &empty_env()), 5.0);
    }

    #[test]
    fn test_eval_call_log_two_arg() {
        // log(8, 2) = 3
        let expr = Expr::Call(
            "log".to_string(),
            vec![Expr::Number(8.0), Expr::Number(2.0)],
        );
        assert!((eval_s(&expr, &empty_env()) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_call_asin_acos_atan() {
        let env = empty_env();
        let asin = Expr::Call("asin".to_string(), vec![Expr::Number(1.0)]);
        assert!((eval_s(&asin, &env) - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
        let acos = Expr::Call("acos".to_string(), vec![Expr::Number(1.0)]);
        assert!(eval_s(&acos, &env).abs() < 1e-10);
        let atan = Expr::Call("atan".to_string(), vec![Expr::Number(1.0)]);
        assert!((eval_s(&atan, &env) - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
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
        assert_eq!(format_scalar(42.0, 10, Base::Dec), "42");
        assert_eq!(format_scalar(-5.0, 10, Base::Dec), "-5");
    }

    #[test]
    fn test_format_value_dec_float() {
        assert_eq!(format_scalar(3.14, 2, Base::Dec), "3.14");
        assert_eq!(format_scalar(1.0 / 3.0, 4, Base::Dec), "0.3333");
    }

    #[test]
    fn test_format_value_dec_sci_large() {
        let result = format_scalar(1e20, 2, Base::Dec);
        assert!(
            result.contains('e'),
            "expected scientific notation, got: {result}"
        );
    }

    #[test]
    fn test_format_value_dec_sci_small() {
        let result = format_scalar(1e-10, 4, Base::Dec);
        assert!(
            result.contains('e'),
            "expected scientific notation, got: {result}"
        );
    }

    #[test]
    fn test_format_value_hex() {
        assert_eq!(format_scalar(255.0, 10, Base::Hex), "0xFF");
        assert_eq!(format_scalar(256.0, 10, Base::Hex), "0x100");
        assert_eq!(format_scalar(0.0, 10, Base::Hex), "0x0");
    }

    #[test]
    fn test_format_value_bin() {
        assert_eq!(format_scalar(10.0, 10, Base::Bin), "0b1010");
        assert_eq!(format_scalar(1.0, 10, Base::Bin), "0b1");
    }

    #[test]
    fn test_format_value_oct() {
        assert_eq!(format_scalar(8.0, 10, Base::Oct), "0o10");
        assert_eq!(format_scalar(255.0, 10, Base::Oct), "0o377");
    }

    #[test]
    fn test_format_non_dec_negative() {
        assert_eq!(format_non_dec(-16.0, Base::Hex), "-0x10");
        assert_eq!(format_non_dec(-2.0, Base::Bin), "-0b10");
    }

    #[test]
    fn test_format_value_hex_rounds() {
        assert_eq!(format_scalar(255.6, 10, Base::Hex), "0x100");
    }

    // --- Matrix tests ---

    #[test]
    fn test_eval_matrix_row_vector() {
        // [1 2 3] — row vector
        let expr = Expr::Matrix(vec![vec![Expr::Number(1.0), Expr::Number(2.0), Expr::Number(3.0)]]);
        let env = empty_env();
        match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[1, 3]);
                assert_eq!(m[[0, 0]], 1.0);
                assert_eq!(m[[0, 1]], 2.0);
                assert_eq!(m[[0, 2]], 3.0);
            }
            _ => panic!("expected matrix"),
        }
    }

    #[test]
    fn test_eval_matrix_col_vector() {
        // [1; 2; 3] — column vector
        let expr = Expr::Matrix(vec![
            vec![Expr::Number(1.0)],
            vec![Expr::Number(2.0)],
            vec![Expr::Number(3.0)],
        ]);
        let env = empty_env();
        match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[3, 1]);
                assert_eq!(m[[0, 0]], 1.0);
                assert_eq!(m[[1, 0]], 2.0);
                assert_eq!(m[[2, 0]], 3.0);
            }
            _ => panic!("expected matrix"),
        }
    }

    #[test]
    fn test_eval_matrix_2x2() {
        // [1 2; 3 4]
        let expr = Expr::Matrix(vec![
            vec![Expr::Number(1.0), Expr::Number(2.0)],
            vec![Expr::Number(3.0), Expr::Number(4.0)],
        ]);
        let env = empty_env();
        match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[2, 2]);
                assert_eq!(m[[0, 0]], 1.0);
                assert_eq!(m[[0, 1]], 2.0);
                assert_eq!(m[[1, 0]], 3.0);
                assert_eq!(m[[1, 1]], 4.0);
            }
            _ => panic!("expected matrix"),
        }
    }

    #[test]
    fn test_eval_matrix_add() {
        use ndarray::array;
        let a = Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]);
        let b = Value::Matrix(array![[5.0, 6.0], [7.0, 8.0]]);
        let mut env = empty_env();
        env.insert("a".to_string(), a);
        env.insert("b".to_string(), b);
        let expr = Expr::BinOp(
            Box::new(Expr::Var("a".to_string())),
            Op::Add,
            Box::new(Expr::Var("b".to_string())),
        );
        match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m[[0, 0]], 6.0);
                assert_eq!(m[[0, 1]], 8.0);
                assert_eq!(m[[1, 0]], 10.0);
                assert_eq!(m[[1, 1]], 12.0);
            }
            _ => panic!("expected matrix"),
        }
    }

    #[test]
    fn test_eval_matrix_scalar_mul() {
        use ndarray::array;
        let a = Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]);
        let mut env = empty_env();
        env.insert("a".to_string(), a);
        let expr = Expr::BinOp(
            Box::new(Expr::Number(2.0)),
            Op::Mul,
            Box::new(Expr::Var("a".to_string())),
        );
        match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m[[0, 0]], 2.0);
                assert_eq!(m[[0, 1]], 4.0);
                assert_eq!(m[[1, 0]], 6.0);
                assert_eq!(m[[1, 1]], 8.0);
            }
            _ => panic!("expected matrix"),
        }
    }
}
