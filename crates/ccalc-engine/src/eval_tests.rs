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
    // log is natural log (MATLAB-compatible)
    let expr = Expr::Call("log".to_string(), vec![Expr::Number(std::f64::consts::E)]);
    assert!((eval_s(&expr, &empty_env()) - 1.0).abs() < 1e-10);
}

#[test]
fn test_eval_call_log10() {
    let expr = Expr::Call("log10".to_string(), vec![Expr::Number(1000.0)]);
    assert!((eval_s(&expr, &empty_env()) - 3.0).abs() < 1e-10);
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

#[allow(clippy::approx_constant)]
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
    assert_eq!(
        format_scalar(42.0, Base::Dec, &FormatMode::Custom(10)),
        "42"
    );
    assert_eq!(
        format_scalar(-5.0, Base::Dec, &FormatMode::Custom(10)),
        "-5"
    );
}

#[allow(clippy::approx_constant)]
#[test]
fn test_format_value_dec_float() {
    assert_eq!(
        format_scalar(3.14, Base::Dec, &FormatMode::Custom(2)),
        "3.14"
    );
    assert_eq!(
        format_scalar(1.0 / 3.0, Base::Dec, &FormatMode::Custom(4)),
        "0.3333"
    );
}

#[test]
fn test_format_value_dec_sci_large() {
    let result = format_scalar(1e20, Base::Dec, &FormatMode::Custom(2));
    assert!(
        result.contains('e'),
        "expected scientific notation, got: {result}"
    );
}

#[test]
fn test_format_value_dec_sci_small() {
    let result = format_scalar(1e-10, Base::Dec, &FormatMode::Custom(4));
    assert!(
        result.contains('e'),
        "expected scientific notation, got: {result}"
    );
}

#[test]
fn test_format_value_hex() {
    assert_eq!(
        format_scalar(255.0, Base::Hex, &FormatMode::Custom(10)),
        "0xFF"
    );
    assert_eq!(
        format_scalar(256.0, Base::Hex, &FormatMode::Custom(10)),
        "0x100"
    );
    assert_eq!(
        format_scalar(0.0, Base::Hex, &FormatMode::Custom(10)),
        "0x0"
    );
}

#[test]
fn test_format_value_bin() {
    assert_eq!(
        format_scalar(10.0, Base::Bin, &FormatMode::Custom(10)),
        "0b1010"
    );
    assert_eq!(
        format_scalar(1.0, Base::Bin, &FormatMode::Custom(10)),
        "0b1"
    );
}

#[test]
fn test_format_value_oct() {
    assert_eq!(
        format_scalar(8.0, Base::Oct, &FormatMode::Custom(10)),
        "0o10"
    );
    assert_eq!(
        format_scalar(255.0, Base::Oct, &FormatMode::Custom(10)),
        "0o377"
    );
}

#[test]
fn test_format_non_dec_negative() {
    assert_eq!(format_non_dec(-16.0, Base::Hex), "-0x10");
    assert_eq!(format_non_dec(-2.0, Base::Bin), "-0b10");
}

#[test]
fn test_format_value_hex_rounds() {
    assert_eq!(
        format_scalar(255.6, Base::Hex, &FormatMode::Custom(10)),
        "0x100"
    );
}

// --- FormatMode tests ---

#[test]
fn test_format_short() {
    let m = &FormatMode::Short;
    assert_eq!(format_scalar(std::f64::consts::PI, Base::Dec, m), "3.1416");
    assert_eq!(format_scalar(1.0 / 3.0, Base::Dec, m), "0.33333");
    assert_eq!(format_scalar(42.0, Base::Dec, m), "42");
    assert_eq!(format_scalar(0.001, Base::Dec, m), "0.001");
    assert_eq!(format_scalar(0.0001, Base::Dec, m), "1e-04");
    assert_eq!(format_scalar(1234567.89, Base::Dec, m), "1.2346e+06");
}

#[test]
fn test_format_long() {
    let m = &FormatMode::Long;
    assert_eq!(
        format_scalar(std::f64::consts::PI, Base::Dec, m),
        "3.14159265358979"
    );
    assert_eq!(format_scalar(42.0, Base::Dec, m), "42");
}

#[test]
fn test_format_shorte() {
    let m = &FormatMode::ShortE;
    assert_eq!(
        format_scalar(std::f64::consts::PI, Base::Dec, m),
        "3.1416e+00"
    );
    assert_eq!(format_scalar(1234.5, Base::Dec, m), "1.2345e+03");
}

#[test]
fn test_format_bank() {
    let m = &FormatMode::Bank;
    assert_eq!(format_scalar(1.0 / 3.0, Base::Dec, m), "0.33");
    assert_eq!(format_scalar(199.999, Base::Dec, m), "200.00");
    assert_eq!(format_scalar(3.0, Base::Dec, m), "3.00");
}

#[test]
fn test_format_rat() {
    let m = &FormatMode::Rat;
    assert_eq!(format_scalar(std::f64::consts::PI, Base::Dec, m), "355/113");
    assert_eq!(format_scalar(1.0 / 3.0, Base::Dec, m), "1/3");
    assert_eq!(format_scalar(42.0, Base::Dec, m), "42");
    assert_eq!(format_scalar(0.125, Base::Dec, m), "1/8");
}

#[test]
fn test_format_hex_ieee754() {
    let m = &FormatMode::Hex;
    // IEEE 754 bit pattern for 1.0
    assert_eq!(format_scalar(1.0, Base::Dec, m), "3FF0000000000000");
    // FormatMode::Hex overrides Base
    assert_eq!(format_scalar(1.0, Base::Hex, m), "3FF0000000000000");
}

#[test]
fn test_format_plus() {
    let m = &FormatMode::Plus;
    assert_eq!(format_scalar(3.0, Base::Dec, m), "+");
    assert_eq!(format_scalar(-2.0, Base::Dec, m), "-");
    assert_eq!(format_scalar(0.0, Base::Dec, m), " ");
}

#[test]
fn test_format_custom() {
    assert_eq!(
        format_scalar(1.0 / 3.0, Base::Dec, &FormatMode::Custom(4)),
        "0.3333"
    );
    assert_eq!(
        format_scalar(1.0 / 3.0, Base::Dec, &FormatMode::Custom(2)),
        "0.33"
    );
}

#[test]
fn test_format_nan_inf() {
    let m = &FormatMode::Short;
    assert_eq!(format_scalar(f64::NAN, Base::Dec, m), "NaN");
    assert_eq!(format_scalar(f64::INFINITY, Base::Dec, m), "Inf");
    assert_eq!(format_scalar(f64::NEG_INFINITY, Base::Dec, m), "-Inf");
}

// --- Matrix tests ---

#[test]
fn test_eval_matrix_row_vector() {
    // [1 2 3] — row vector
    let expr = Expr::Matrix(vec![vec![
        Expr::Number(1.0),
        Expr::Number(2.0),
        Expr::Number(3.0),
    ]]);
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

// --- Phase 4: matrix operations ---

#[test]
fn test_eval_matrix_mul() {
    use ndarray::array;
    // [1 2; 3 4] * [1; 1] = [3; 7]
    let a = Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]);
    let b = Value::Matrix(array![[1.0], [1.0]]);
    let mut env = empty_env();
    env.insert("a".to_string(), a);
    env.insert("b".to_string(), b);
    let expr = Expr::BinOp(
        Box::new(Expr::Var("a".to_string())),
        Op::Mul,
        Box::new(Expr::Var("b".to_string())),
    );
    match eval(&expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[2, 1]);
            assert_eq!(m[[0, 0]], 3.0);
            assert_eq!(m[[1, 0]], 7.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_matrix_mul_inner_mismatch() {
    use ndarray::array;
    let a = Value::Matrix(array![[1.0, 2.0]]);
    let b = Value::Matrix(array![[1.0, 2.0]]);
    assert!(eval_binop(a, &Op::Mul, b).is_err());
}

#[test]
fn test_eval_matrix_elem_mul() {
    use ndarray::array;
    let a = Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]);
    let b = Value::Matrix(array![[2.0, 3.0], [4.0, 5.0]]);
    match eval_binop(a, &Op::ElemMul, b).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m[[0, 0]], 2.0);
            assert_eq!(m[[0, 1]], 6.0);
            assert_eq!(m[[1, 0]], 12.0);
            assert_eq!(m[[1, 1]], 20.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_matrix_elem_div() {
    use ndarray::array;
    let a = Value::Matrix(array![[6.0, 8.0]]);
    let b = Value::Matrix(array![[2.0, 4.0]]);
    match eval_binop(a, &Op::ElemDiv, b).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m[[0, 0]], 3.0);
            assert_eq!(m[[0, 1]], 2.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_matrix_elem_pow() {
    use ndarray::array;
    let a = Value::Matrix(array![[2.0, 3.0]]);
    let b = Value::Matrix(array![[3.0, 2.0]]);
    match eval_binop(a, &Op::ElemPow, b).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m[[0, 0]], 8.0);
            assert_eq!(m[[0, 1]], 9.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_transpose_matrix() {
    use ndarray::array;
    let a = Value::Matrix(array![[1.0, 2.0, 3.0]]);
    let mut env = empty_env();
    env.insert("a".to_string(), a);
    let expr = Expr::Transpose(Box::new(Expr::Var("a".to_string())));
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
fn test_eval_transpose_scalar() {
    let expr = Expr::Transpose(Box::new(Expr::Number(5.0)));
    match eval(&expr, &empty_env()).unwrap() {
        Value::Scalar(n) => assert_eq!(n, 5.0),
        _ => panic!("expected scalar"),
    }
}

#[test]
fn test_eval_zeros() {
    let expr = Expr::Call(
        "zeros".to_string(),
        vec![Expr::Number(2.0), Expr::Number(3.0)],
    );
    match eval(&expr, &empty_env()).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[2, 3]);
            assert!(m.iter().all(|&x| x == 0.0));
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_ones() {
    let expr = Expr::Call(
        "ones".to_string(),
        vec![Expr::Number(2.0), Expr::Number(2.0)],
    );
    match eval(&expr, &empty_env()).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[2, 2]);
            assert!(m.iter().all(|&x| x == 1.0));
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_eye() {
    let expr = Expr::Call("eye".to_string(), vec![Expr::Number(3.0)]);
    match eval(&expr, &empty_env()).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[3, 3]);
            assert_eq!(m[[0, 0]], 1.0);
            assert_eq!(m[[1, 1]], 1.0);
            assert_eq!(m[[2, 2]], 1.0);
            assert_eq!(m[[0, 1]], 0.0);
            assert_eq!(m[[1, 0]], 0.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_size() {
    use ndarray::array;
    let mut env = empty_env();
    env.insert(
        "a".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
    );
    let expr = Expr::Call("size".to_string(), vec![Expr::Var("a".to_string())]);
    match eval(&expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[1, 2]);
            assert_eq!(m[[0, 0]], 2.0);
            assert_eq!(m[[0, 1]], 3.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_length_numel() {
    use ndarray::array;
    let mut env = empty_env();
    env.insert(
        "a".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
    );
    let len = Expr::Call("length".to_string(), vec![Expr::Var("a".to_string())]);
    let num = Expr::Call("numel".to_string(), vec![Expr::Var("a".to_string())]);
    assert_eq!(eval_s(&len, &env), 3.0);
    assert_eq!(eval_s(&num, &env), 6.0);
}

#[test]
fn test_eval_trace() {
    use ndarray::array;
    let mut env = empty_env();
    env.insert(
        "a".to_string(),
        Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]),
    );
    let expr = Expr::Call("trace".to_string(), vec![Expr::Var("a".to_string())]);
    assert_eq!(eval_s(&expr, &env), 5.0);
}

#[test]
fn test_eval_det_2x2() {
    use ndarray::array;
    let mut env = empty_env();
    env.insert(
        "a".to_string(),
        Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]),
    );
    let expr = Expr::Call("det".to_string(), vec![Expr::Var("a".to_string())]);
    assert!((eval_s(&expr, &env) - (-2.0)).abs() < 1e-10);
}

#[test]
fn test_eval_det_singular() {
    use ndarray::array;
    let mut env = empty_env();
    env.insert(
        "a".to_string(),
        Value::Matrix(array![[1.0, 2.0], [2.0, 4.0]]),
    );
    let expr = Expr::Call("det".to_string(), vec![Expr::Var("a".to_string())]);
    assert_eq!(eval_s(&expr, &env), 0.0);
}

#[test]
fn test_eval_inv_2x2() {
    use ndarray::array;
    let mut env = empty_env();
    env.insert(
        "a".to_string(),
        Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]),
    );
    let expr = Expr::Call("inv".to_string(), vec![Expr::Var("a".to_string())]);
    match eval(&expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert!((m[[0, 0]] - (-2.0)).abs() < 1e-10);
            assert!((m[[0, 1]] - 1.0).abs() < 1e-10);
            assert!((m[[1, 0]] - 1.5).abs() < 1e-10);
            assert!((m[[1, 1]] - (-0.5)).abs() < 1e-10);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_eval_inv_singular() {
    use ndarray::array;
    let mut env = empty_env();
    env.insert(
        "a".to_string(),
        Value::Matrix(array![[1.0, 2.0], [2.0, 4.0]]),
    );
    let expr = Expr::Call("inv".to_string(), vec![Expr::Var("a".to_string())]);
    assert!(eval(&expr, &env).is_err());
}

// ---------------------------------------------------------------------------
// Phase 8 — Complex numbers
// ---------------------------------------------------------------------------

fn env_with_ij() -> Env {
    let mut env = Env::new();
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    env.insert("j".to_string(), Value::Complex(0.0, 1.0));
    env.insert("ans".to_string(), Value::Scalar(0.0));
    env
}

fn eval_parse(input: &str, env: &Env) -> Result<Value, String> {
    use crate::parser::{Stmt, parse};
    let stmt = parse(input)?;
    let expr = match stmt {
        Stmt::Expr(e) | Stmt::Assign(_, e) => e,
        _ => return Err("Block statements not valid in expression context".to_string()),
    };
    eval(&expr, env)
}

#[test]
fn test_complex_imaginary_unit() {
    let env = env_with_ij();
    assert_eq!(env.get("i"), Some(&Value::Complex(0.0, 1.0)));
    assert_eq!(env.get("j"), Some(&Value::Complex(0.0, 1.0)));
}

#[test]
fn test_complex_literal_4i() {
    // 4*i = 0 + 4i
    let env = env_with_ij();
    let result = eval_parse("4*i", &env).unwrap();
    assert_eq!(result, Value::Complex(0.0, 4.0));
}

#[test]
fn test_complex_literal_3_plus_4i() {
    // 3 + 4*i
    let env = env_with_ij();
    let result = eval_parse("3 + 4*i", &env).unwrap();
    assert_eq!(result, Value::Complex(3.0, 4.0));
}

#[test]
fn test_complex_add() {
    let mut env = empty_env();
    env.insert("z1".to_string(), Value::Complex(1.0, 2.0));
    env.insert("z2".to_string(), Value::Complex(3.0, 4.0));
    let result = eval_parse("z1 + z2", &env).unwrap();
    assert_eq!(result, Value::Complex(4.0, 6.0));
}

#[test]
fn test_complex_sub() {
    let mut env = empty_env();
    env.insert("z1".to_string(), Value::Complex(5.0, 6.0));
    env.insert("z2".to_string(), Value::Complex(1.0, 2.0));
    let result = eval_parse("z1 - z2", &env).unwrap();
    assert_eq!(result, Value::Complex(4.0, 4.0));
}

#[test]
fn test_complex_mul() {
    // (1+2i)(3+4i) = (3-8) + (4+6)i = -5 + 10i
    let mut env = empty_env();
    env.insert("z1".to_string(), Value::Complex(1.0, 2.0));
    env.insert("z2".to_string(), Value::Complex(3.0, 4.0));
    let result = eval_parse("z1 * z2", &env).unwrap();
    assert_eq!(result, Value::Complex(-5.0, 10.0));
}

#[test]
fn test_complex_mul_gives_real() {
    // (1+i)(1-i) = 1 - i + i - i² = 2 + 0i → should collapse to Scalar
    let mut env = empty_env();
    env.insert("z1".to_string(), Value::Complex(1.0, 1.0));
    env.insert("z2".to_string(), Value::Complex(1.0, -1.0));
    let result = eval_parse("z1 * z2", &env).unwrap();
    assert_eq!(result, Value::Scalar(2.0));
}

#[test]
fn test_complex_div() {
    // (1+2i)/(1+i) = ((1+2)(1+1) + (2-1)i) ... let's compute:
    // (1+2i)/(1+i) = ((1*1+2*1) + (2*1-1*1)i) / (1+1) = (3+1i)/2 = 1.5 + 0.5i
    let mut env = empty_env();
    env.insert("z1".to_string(), Value::Complex(1.0, 2.0));
    env.insert("z2".to_string(), Value::Complex(1.0, 1.0));
    let result = eval_parse("z1 / z2", &env).unwrap();
    assert_eq!(result, Value::Complex(1.5, 0.5));
}

#[test]
fn test_complex_plus_scalar() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(1.0, 2.0));
    env.insert("x".to_string(), Value::Scalar(3.0));
    let result = eval_parse("z + x", &env).unwrap();
    assert_eq!(result, Value::Complex(4.0, 2.0));
}

#[test]
fn test_scalar_plus_complex() {
    let mut env = empty_env();
    env.insert("x".to_string(), Value::Scalar(5.0));
    env.insert("z".to_string(), Value::Complex(1.0, 2.0));
    let result = eval_parse("x + z", &env).unwrap();
    assert_eq!(result, Value::Complex(6.0, 2.0));
}

#[test]
fn test_complex_unary_minus() {
    let expr = Expr::UnaryMinus(Box::new(Expr::Var("z".to_string())));
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(3.0, -4.0));
    assert_eq!(eval(&expr, &env).unwrap(), Value::Complex(-3.0, 4.0));
}

#[test]
fn test_complex_unary_not() {
    let expr = Expr::UnaryNot(Box::new(Expr::Var("z".to_string())));
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(3.0, 4.0));
    assert_eq!(eval(&expr, &env).unwrap(), Value::Scalar(0.0));
    env.insert("z".to_string(), Value::Complex(0.0, 0.0));
    assert_eq!(eval(&expr, &env).unwrap(), Value::Scalar(1.0));
}

#[test]
fn test_complex_transpose_is_conjugate() {
    let expr = Expr::Transpose(Box::new(Expr::Var("z".to_string())));
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(3.0, 4.0));
    assert_eq!(eval(&expr, &env).unwrap(), Value::Complex(3.0, -4.0));
}

#[test]
fn test_complex_eq() {
    let mut env = empty_env();
    env.insert("z1".to_string(), Value::Complex(1.0, 2.0));
    env.insert("z2".to_string(), Value::Complex(1.0, 2.0));
    env.insert("z3".to_string(), Value::Complex(1.0, 3.0));
    let eq = eval_parse("z1 == z2", &env).unwrap();
    assert_eq!(eq, Value::Scalar(1.0));
    let ne = eval_parse("z1 == z3", &env).unwrap();
    assert_eq!(ne, Value::Scalar(0.0));
}

#[test]
fn test_complex_ordering_error() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(1.0, 2.0));
    assert!(eval_parse("z > 0", &env).is_err());
    assert!(eval_parse("z < 0", &env).is_err());
}

#[test]
fn test_complex_pow_squared() {
    // (1+i)^2 = 1 + 2i + i² = 1 + 2i - 1 = 2i
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(1.0, 1.0));
    let result = eval_parse("z^2", &env).unwrap();
    // result should be approximately 0 + 2i
    match result {
        Value::Complex(re, im) => {
            assert!((re).abs() < 1e-10, "re = {re}");
            assert!((im - 2.0).abs() < 1e-10, "im = {im}");
        }
        Value::Scalar(n) if n.abs() < 1e-10 => {} // collapsed real near 0
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_builtin_real_imag() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(3.0, 4.0));
    let re = eval_parse("real(z)", &env).unwrap();
    let im = eval_parse("imag(z)", &env).unwrap();
    assert_eq!(re, Value::Scalar(3.0));
    assert_eq!(im, Value::Scalar(4.0));
    // imag of a real scalar = 0
    env.insert("x".to_string(), Value::Scalar(5.0));
    let im2 = eval_parse("imag(x)", &env).unwrap();
    assert_eq!(im2, Value::Scalar(0.0));
}

#[test]
fn test_builtin_abs_complex() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(3.0, 4.0));
    let result = eval_parse("abs(z)", &env).unwrap();
    assert_eq!(result, Value::Scalar(5.0));
}

#[test]
fn test_builtin_angle() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(0.0, 1.0)); // i → angle = π/2
    let result = eval_parse("angle(z)", &env).unwrap();
    match result {
        Value::Scalar(n) => assert!((n - std::f64::consts::FRAC_PI_2).abs() < 1e-10),
        other => panic!("{:?}", other),
    }
}

#[test]
fn test_builtin_conj() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(3.0, 4.0));
    let result = eval_parse("conj(z)", &env).unwrap();
    assert_eq!(result, Value::Complex(3.0, -4.0));
}

#[test]
fn test_builtin_complex_construct() {
    let env = empty_env();
    let result = eval_parse("complex(3, 4)", &env).unwrap();
    assert_eq!(result, Value::Complex(3.0, 4.0));
    // im = 0 → Scalar
    let result2 = eval_parse("complex(5, 0)", &env).unwrap();
    assert_eq!(result2, Value::Scalar(5.0));
}

#[test]
fn test_builtin_isreal() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(1.0, 2.0));
    env.insert("x".to_string(), Value::Scalar(3.0));
    assert_eq!(eval_parse("isreal(z)", &env).unwrap(), Value::Scalar(0.0));
    assert_eq!(eval_parse("isreal(x)", &env).unwrap(), Value::Scalar(1.0));
}

#[test]
fn test_format_complex_display() {
    let m = &FormatMode::Custom(10);
    assert_eq!(format_complex(3.0, 4.0, m), "3 + 4i");
    assert_eq!(format_complex(3.0, -4.0, m), "3 - 4i");
    assert_eq!(format_complex(0.0, 1.0, m), "i");
    assert_eq!(format_complex(0.0, -1.0, m), "-i");
    assert_eq!(format_complex(0.0, 2.0, m), "2i");
    assert_eq!(format_complex(3.0, 0.0, m), "3");
    assert_eq!(format_complex(1.0, 1.0, m), "1 + i");
    assert_eq!(format_complex(1.0, -1.0, m), "1 - i");
}

#[test]
fn test_complex_matrix_literal_error() {
    let env = env_with_ij();
    // [1+2i] should error since complex in matrix is not supported
    let result = eval_parse("[1+2*i, 3]", &env);
    assert!(result.is_err());
}

#[test]
fn test_scalar_arg_accepts_complex_with_zero_im() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(4.0, 0.0));
    // sqrt(complex(4, 0)) should work since im == 0
    let result = eval_parse("sqrt(z)", &env).unwrap();
    assert_eq!(result, Value::Scalar(2.0));
}

// --- Element-wise math functions on vectors/matrices ---

#[test]
fn test_sin_vector() {
    let env = empty_env();
    let result = eval_parse("sin([0, pi/2, pi])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 3));
    assert!((m[[0, 0]] - 0.0).abs() < 1e-15);
    assert!((m[[0, 1]] - 1.0).abs() < 1e-15);
    assert!((m[[0, 2]] - 0.0).abs() < 1e-14);
}

#[test]
fn test_cos_vector() {
    let env = empty_env();
    let result = eval_parse("cos([0, pi/2, pi])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 3));
    assert!((m[[0, 0]] - 1.0).abs() < 1e-15);
    assert!((m[[0, 2]] - (-1.0)).abs() < 1e-15);
}

#[test]
fn test_exp_vector() {
    let env = empty_env();
    let result = eval_parse("exp([0, 1, 2])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 3));
    assert!((m[[0, 0]] - 1.0).abs() < 1e-15);
    assert!((m[[0, 1]] - std::f64::consts::E).abs() < 1e-14);
}

#[test]
fn test_sqrt_vector() {
    let env = empty_env();
    let result = eval_parse("sqrt([1, 4, 9, 16])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 4));
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[0, 1]], 2.0);
    assert_eq!(m[[0, 2]], 3.0);
    assert_eq!(m[[0, 3]], 4.0);
}

#[test]
fn test_floor_ceil_matrix() {
    let env = empty_env();
    let r = eval_parse("floor([1.2, 2.7; -0.5, 3.9])", &env).unwrap();
    let Value::Matrix(m) = r else {
        panic!("expected matrix")
    };
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[0, 1]], 2.0);
    assert_eq!(m[[1, 0]], -1.0);
    assert_eq!(m[[1, 1]], 3.0);
}

#[test]
fn test_complex_pow_zero_base() {
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(0.0, 0.0));
    // 0^2 = 0
    let result = eval_parse("z^2", &env).unwrap();
    assert_eq!(result, Value::Scalar(0.0));
}

#[test]
fn test_complex_inv_builtin() {
    // inv(2+0i) = 0.5
    let mut env = empty_env();
    env.insert("z".to_string(), Value::Complex(2.0, 0.0));
    let result = eval_parse("inv(z)", &env).unwrap();
    assert_eq!(result, Value::Scalar(0.5));
}

// ---------------------------------------------------------------------------
// Phase 9 — String tests
// ---------------------------------------------------------------------------

#[test]
fn test_str_literal_basic() {
    let env = empty_env();
    let expr = Expr::StrLiteral("hello".to_string());
    assert_eq!(eval(&expr, &env), Ok(Value::Str("hello".to_string())));
}

#[test]
fn test_string_obj_literal_basic() {
    let env = empty_env();
    let expr = Expr::StringObjLiteral("world".to_string());
    assert_eq!(eval(&expr, &env), Ok(Value::StringObj("world".to_string())));
}

#[test]
fn test_str_arithmetic_single_char() {
    // 'a' + 0 = 97
    let env = empty_env();
    let expr = Expr::BinOp(
        Box::new(Expr::StrLiteral("a".to_string())),
        Op::Add,
        Box::new(Expr::Number(0.0)),
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(97.0)));
}

#[test]
fn test_str_arithmetic_multi_char() {
    // 'ab' + 0 = [97, 98] as a row vector
    use ndarray::array;
    let env = empty_env();
    let expr = Expr::BinOp(
        Box::new(Expr::StrLiteral("ab".to_string())),
        Op::Add,
        Box::new(Expr::Number(0.0)),
    );
    match eval(&expr, &env).unwrap() {
        Value::Matrix(m) => assert_eq!(m, array![[97.0, 98.0]]),
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_string_obj_concat() {
    let env = empty_env();
    let expr = Expr::BinOp(
        Box::new(Expr::StringObjLiteral("hello".to_string())),
        Op::Add,
        Box::new(Expr::StringObjLiteral(" world".to_string())),
    );
    assert_eq!(
        eval(&expr, &env),
        Ok(Value::StringObj("hello world".to_string()))
    );
}

#[test]
fn test_string_obj_eq() {
    let env = empty_env();
    let expr = Expr::BinOp(
        Box::new(Expr::StringObjLiteral("abc".to_string())),
        Op::Eq,
        Box::new(Expr::StringObjLiteral("abc".to_string())),
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

#[test]
fn test_num2str() {
    let env = empty_env();
    let expr = Expr::Call("num2str".to_string(), vec![Expr::Number(42.0)]);
    assert_eq!(eval(&expr, &env), Ok(Value::Str("42".to_string())));
}

#[allow(clippy::approx_constant)]
#[test]
fn test_str2double_valid() {
    let env = empty_env();
    let expr = Expr::Call(
        "str2double".to_string(),
        vec![Expr::StrLiteral("3.14".to_string())],
    );
    match eval(&expr, &env).unwrap() {
        Value::Scalar(n) => assert!((n - 3.14).abs() < 1e-10),
        _ => panic!("expected scalar"),
    }
}

#[test]
fn test_str2double_invalid() {
    let env = empty_env();
    let expr = Expr::Call(
        "str2double".to_string(),
        vec![Expr::StrLiteral("abc".to_string())],
    );
    match eval(&expr, &env).unwrap() {
        Value::Scalar(n) => assert!(n.is_nan()),
        _ => panic!("expected scalar"),
    }
}

#[test]
fn test_strcat_char_arrays() {
    let env = empty_env();
    let expr = Expr::Call(
        "strcat".to_string(),
        vec![
            Expr::StrLiteral("hello".to_string()),
            Expr::StrLiteral(" world".to_string()),
        ],
    );
    // strcat trims trailing whitespace from char array args per MATLAB behavior
    assert_eq!(eval(&expr, &env), Ok(Value::Str("hello world".to_string())));
}

#[test]
fn test_strcmp_equal() {
    let env = empty_env();
    let expr = Expr::Call(
        "strcmp".to_string(),
        vec![
            Expr::StrLiteral("abc".to_string()),
            Expr::StrLiteral("abc".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

#[test]
fn test_strcmp_not_equal() {
    let env = empty_env();
    let expr = Expr::Call(
        "strcmp".to_string(),
        vec![
            Expr::StrLiteral("abc".to_string()),
            Expr::StrLiteral("def".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));
}

#[test]
fn test_length_of_str() {
    let env = empty_env();
    let expr = Expr::Call(
        "length".to_string(),
        vec![Expr::StrLiteral("hello".to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(5.0)));
}

#[test]
fn test_ischar_true() {
    let env = empty_env();
    let expr = Expr::Call(
        "ischar".to_string(),
        vec![Expr::StrLiteral("hi".to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

#[test]
fn test_ischar_false_for_number() {
    let env = empty_env();
    let expr = Expr::Call("ischar".to_string(), vec![Expr::Number(5.0)]);
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));
}

#[test]
fn test_isstring_true() {
    let env = empty_env();
    let expr = Expr::Call(
        "isstring".to_string(),
        vec![Expr::StringObjLiteral("hi".to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

#[test]
fn test_lower_upper() {
    let env = empty_env();
    let lower_expr = Expr::Call(
        "lower".to_string(),
        vec![Expr::StrLiteral("HELLO".to_string())],
    );
    assert_eq!(eval(&lower_expr, &env), Ok(Value::Str("hello".to_string())));

    let upper_expr = Expr::Call(
        "upper".to_string(),
        vec![Expr::StrLiteral("hello".to_string())],
    );
    assert_eq!(eval(&upper_expr, &env), Ok(Value::Str("HELLO".to_string())));
}

#[test]
fn test_strtrim() {
    let env = empty_env();
    let expr = Expr::Call(
        "strtrim".to_string(),
        vec![Expr::StrLiteral("  hello  ".to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Str("hello".to_string())));
}

#[test]
fn test_strrep() {
    let env = empty_env();
    let expr = Expr::Call(
        "strrep".to_string(),
        vec![
            Expr::StrLiteral("hello world".to_string()),
            Expr::StrLiteral("world".to_string()),
            Expr::StrLiteral("Rust".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Str("hello Rust".to_string())));
}

#[test]
fn test_strcmpi_case_insensitive() {
    let env = empty_env();
    let expr = Expr::Call(
        "strcmpi".to_string(),
        vec![
            Expr::StrLiteral("Hello".to_string()),
            Expr::StrLiteral("hello".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

// ---------------------------------------------------------------------------
// Phase 10 — format_printf tests
// ---------------------------------------------------------------------------

#[test]
fn test_printf_no_args() {
    assert_eq!(format_printf("hello", &[]).unwrap(), "hello");
}

#[test]
fn test_printf_escape_sequences() {
    assert_eq!(format_printf("a\\nb\\tc", &[]).unwrap(), "a\nb\tc");
    assert_eq!(format_printf("back\\\\slash", &[]).unwrap(), "back\\slash");
}

#[test]
fn test_printf_percent_literal() {
    assert_eq!(format_printf("100%%", &[]).unwrap(), "100%");
}

#[test]
fn test_printf_d() {
    let args = vec![Value::Scalar(42.0)];
    assert_eq!(format_printf("%d", &args).unwrap(), "42");
}

#[test]
fn test_printf_d_negative() {
    let args = vec![Value::Scalar(-7.0)];
    assert_eq!(format_printf("%d", &args).unwrap(), "-7");
}

#[test]
fn test_printf_d_float_truncated() {
    let args = vec![Value::Scalar(3.9)];
    assert_eq!(format_printf("%d", &args).unwrap(), "3");
}

#[allow(clippy::approx_constant)]
#[test]
fn test_printf_f_default() {
    let args = vec![Value::Scalar(3.14159)];
    let s = format_printf("%f", &args).unwrap();
    assert_eq!(s, "3.141590");
}

#[allow(clippy::approx_constant)]
#[test]
fn test_printf_f_precision() {
    let args = vec![Value::Scalar(3.14159)];
    assert_eq!(format_printf("%.2f", &args).unwrap(), "3.14");
}

#[test]
fn test_printf_f_precision_zero() {
    let args = vec![Value::Scalar(3.7)];
    assert_eq!(format_printf("%.0f", &args).unwrap(), "4");
}

#[test]
fn test_printf_e() {
    let args = vec![Value::Scalar(12345.6789)];
    let s = format_printf("%e", &args).unwrap();
    assert_eq!(s, "1.234568e+04");
}

#[test]
fn test_printf_e_precision() {
    let args = vec![Value::Scalar(1.0)];
    assert_eq!(format_printf("%.2e", &args).unwrap(), "1.00e+00");
}

#[allow(clippy::approx_constant)]
#[test]
fn test_printf_g_small() {
    let args = vec![Value::Scalar(3.14)];
    let s = format_printf("%g", &args).unwrap();
    assert_eq!(s, "3.14");
}

#[test]
fn test_printf_g_large() {
    let args = vec![Value::Scalar(1234567.0)];
    let s = format_printf("%g", &args).unwrap();
    // exponent >= 6 (default prec) → scientific
    assert!(s.contains('e'), "expected scientific notation, got {s}");
}

#[test]
fn test_printf_g_trailing_zeros_removed() {
    let args = vec![Value::Scalar(1.0)];
    assert_eq!(format_printf("%g", &args).unwrap(), "1");
}

#[test]
fn test_printf_s_char_array() {
    let args = vec![Value::Str("hello".to_string())];
    assert_eq!(format_printf("%s", &args).unwrap(), "hello");
}

#[test]
fn test_printf_s_string_obj() {
    let args = vec![Value::StringObj("world".to_string())];
    assert_eq!(format_printf("%s", &args).unwrap(), "world");
}

#[test]
fn test_printf_width_right_align() {
    let args = vec![Value::Scalar(42.0)];
    assert_eq!(format_printf("%6d", &args).unwrap(), "    42");
}

#[test]
fn test_printf_width_left_align() {
    let args = vec![Value::Scalar(42.0)];
    assert_eq!(format_printf("%-6d", &args).unwrap(), "42    ");
}

#[test]
fn test_printf_zero_pad() {
    let args = vec![Value::Scalar(42.0)];
    assert_eq!(format_printf("%06d", &args).unwrap(), "000042");
}

#[test]
fn test_printf_force_sign() {
    let args = vec![Value::Scalar(42.0)];
    assert_eq!(format_printf("%+d", &args).unwrap(), "+42");
}

#[test]
fn test_printf_multiple_args() {
    let args = vec![Value::Scalar(3.0), Value::Scalar(4.0)];
    assert_eq!(format_printf("%d + %d", &args).unwrap(), "3 + 4");
}

#[test]
fn test_printf_repeat_format_octave() {
    // More args than specifiers → format repeats
    let args = vec![Value::Scalar(1.0), Value::Scalar(2.0), Value::Scalar(3.0)];
    assert_eq!(format_printf("%d ", &args).unwrap(), "1 2 3 ");
}

#[test]
fn test_printf_more_specifiers_than_args() {
    // More specifiers than args → extra silently ignored
    let args = vec![Value::Scalar(1.0)];
    assert_eq!(format_printf("%d %d", &args).unwrap(), "1 ");
}

#[test]
fn test_printf_mixed_types() {
    let args = vec![
        Value::Str("pi".to_string()),
        Value::Scalar(std::f64::consts::PI),
    ];
    let s = format_printf("%s = %.4f", &args).unwrap();
    assert_eq!(s, "pi = 3.1416");
}

#[test]
fn test_printf_s_precision_truncate() {
    let args = vec![Value::Str("hello".to_string())];
    assert_eq!(format_printf("%.3s", &args).unwrap(), "hel");
}

#[test]
fn test_sprintf_via_eval() {
    let env = empty_env();
    let expr = Expr::Call(
        "sprintf".to_string(),
        vec![Expr::StrLiteral("x = %d".to_string()), Expr::Number(5.0)],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Str("x = 5".to_string())));
}

#[test]
fn test_sprintf_no_args_escape() {
    let env = empty_env();
    let expr = Expr::Call(
        "sprintf".to_string(),
        vec![Expr::StrLiteral("a\\nb".to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Str("a\nb".to_string())));
}

#[test]
fn test_fprintf_returns_void() {
    let env = empty_env();
    let expr = Expr::Call(
        "fprintf".to_string(),
        vec![Expr::StrLiteral("".to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Void));
}

// --- Phase 10.5a: fopen / fclose / fgetl / fgets ---

#[test]
fn test_fopen_write_and_fclose() {
    use crate::io::IoContext;
    let env = empty_env();
    let mut io = IoContext::new();
    let tmp = std::env::temp_dir().join("ccalc_test_fopen_write.txt");
    let path = tmp.to_string_lossy().to_string();

    // fopen returns fd >= 3
    let open_expr = Expr::Call(
        "fopen".to_string(),
        vec![
            Expr::StrLiteral(path.clone()),
            Expr::StrLiteral("w".to_string()),
        ],
    );
    let fd_val = eval_with_io(&open_expr, &env, &mut io).unwrap();
    let fd = match fd_val {
        Value::Scalar(n) => n,
        _ => panic!("expected scalar fd"),
    };
    assert!(fd >= 3.0, "expected fd >= 3, got {fd}");

    // fclose returns 0
    let close_expr = Expr::Call("fclose".to_string(), vec![Expr::Number(fd)]);
    let result = eval_with_io(&close_expr, &env, &mut io).unwrap();
    assert_eq!(result, Value::Scalar(0.0));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_fopen_nonexistent_returns_minus_one() {
    use crate::io::IoContext;
    let env = empty_env();
    let mut io = IoContext::new();

    let expr = Expr::Call(
        "fopen".to_string(),
        vec![
            Expr::StrLiteral("/nonexistent/path/file.txt".to_string()),
            Expr::StrLiteral("r".to_string()),
        ],
    );
    let result = eval_with_io(&expr, &env, &mut io).unwrap();
    assert_eq!(result, Value::Scalar(-1.0));
}

#[test]
fn test_fgetl_reads_lines() {
    use crate::io::IoContext;
    let env = empty_env();
    let mut io = IoContext::new();
    let tmp = std::env::temp_dir().join("ccalc_test_fgetl.txt");
    std::fs::write(&tmp, "hello\nworld\n").unwrap();

    let path = tmp.to_string_lossy().to_string();
    let fd = io.fopen(&path, "r");
    assert!(fd >= 3);

    let expr_fgetl = |fd: i32| Expr::Call("fgetl".to_string(), vec![Expr::Number(fd as f64)]);

    let line1 = eval_with_io(&expr_fgetl(fd), &env, &mut io).unwrap();
    assert_eq!(line1, Value::Str("hello".to_string()));

    let line2 = eval_with_io(&expr_fgetl(fd), &env, &mut io).unwrap();
    assert_eq!(line2, Value::Str("world".to_string()));

    // EOF returns -1
    let eof = eval_with_io(&expr_fgetl(fd), &env, &mut io).unwrap();
    assert_eq!(eof, Value::Scalar(-1.0));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_fgets_keeps_newline() {
    use crate::io::IoContext;
    let env = empty_env();
    let mut io = IoContext::new();
    let tmp = std::env::temp_dir().join("ccalc_test_fgets.txt");
    std::fs::write(&tmp, "hello\n").unwrap();

    let path = tmp.to_string_lossy().to_string();
    let fd = io.fopen(&path, "r");

    let expr = Expr::Call("fgets".to_string(), vec![Expr::Number(fd as f64)]);
    let result = eval_with_io(&expr, &env, &mut io).unwrap();
    assert_eq!(result, Value::Str("hello\n".to_string()));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_fclose_all() {
    use crate::io::IoContext;
    let env = empty_env();
    let mut io = IoContext::new();
    let tmp1 = std::env::temp_dir().join("ccalc_test_fclose_all_1.txt");
    let tmp2 = std::env::temp_dir().join("ccalc_test_fclose_all_2.txt");
    io.fopen(&tmp1.to_string_lossy(), "w");
    io.fopen(&tmp2.to_string_lossy(), "w");

    let expr = Expr::Call(
        "fclose".to_string(),
        vec![Expr::StrLiteral("all".to_string())],
    );
    let result = eval_with_io(&expr, &env, &mut io).unwrap();
    assert_eq!(result, Value::Scalar(0.0));

    let _ = std::fs::remove_file(&tmp1);
    let _ = std::fs::remove_file(&tmp2);
}

#[test]
fn test_fprintf_to_file() {
    use crate::io::IoContext;
    let env = empty_env();
    let mut io = IoContext::new();
    let tmp = std::env::temp_dir().join("ccalc_test_fprintf_file.txt");
    let path = tmp.to_string_lossy().to_string();

    let fd = io.fopen(&path, "w") as f64;

    let expr = Expr::Call(
        "fprintf".to_string(),
        vec![
            Expr::Number(fd),
            Expr::StrLiteral("value = %d\n".to_string()),
            Expr::Number(42.0),
        ],
    );
    let result = eval_with_io(&expr, &env, &mut io).unwrap();
    assert_eq!(result, Value::Void);

    io.fclose(fd as i32);
    let content = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(content, "value = 42\n");

    let _ = std::fs::remove_file(&tmp);
}

// --- Phase 10.5b: dlmread / dlmwrite ---

#[test]
fn test_dlmwrite_and_dlmread_comma() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_dlm_comma.csv");
    let path = tmp.to_string_lossy().to_string();

    // Write 2×3 matrix
    let write_expr = Expr::Call(
        "dlmwrite".to_string(),
        vec![
            Expr::StrLiteral(path.clone()),
            Expr::Matrix(vec![
                vec![Expr::Number(1.0), Expr::Number(2.0), Expr::Number(3.0)],
                vec![Expr::Number(4.0), Expr::Number(5.0), Expr::Number(6.0)],
            ]),
        ],
    );
    assert_eq!(eval(&write_expr, &env), Ok(Value::Void));

    let content = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(content, "1,2,3\n4,5,6\n");

    // Read back
    let read_expr = Expr::Call("dlmread".to_string(), vec![Expr::StrLiteral(path.clone())]);
    match eval(&read_expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[2, 3]);
            assert_eq!(m[[0, 0]], 1.0);
            assert_eq!(m[[1, 2]], 6.0);
        }
        other => panic!("expected matrix, got {other:?}"),
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_dlmwrite_tab_delimiter() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_dlm_tab.tsv");
    let path = tmp.to_string_lossy().to_string();

    let write_expr = Expr::Call(
        "dlmwrite".to_string(),
        vec![
            Expr::StrLiteral(path.clone()),
            Expr::Matrix(vec![vec![Expr::Number(10.0), Expr::Number(20.0)]]),
            Expr::StrLiteral(r"\t".to_string()),
        ],
    );
    assert_eq!(eval(&write_expr, &env), Ok(Value::Void));

    let content = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(content, "10\t20\n");

    // Read back with explicit tab delimiter
    let read_expr = Expr::Call(
        "dlmread".to_string(),
        vec![
            Expr::StrLiteral(path.clone()),
            Expr::StrLiteral(r"\t".to_string()),
        ],
    );
    match eval(&read_expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[1, 2]);
            assert_eq!(m[[0, 0]], 10.0);
            assert_eq!(m[[0, 1]], 20.0);
        }
        other => panic!("expected matrix, got {other:?}"),
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_dlmread_whitespace_auto() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_dlm_ws.txt");
    std::fs::write(&tmp, "1 2 3\n4 5 6\n").unwrap();

    let read_expr = Expr::Call(
        "dlmread".to_string(),
        vec![Expr::StrLiteral(tmp.to_string_lossy().to_string())],
    );
    match eval(&read_expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.shape(), &[2, 3]);
            assert_eq!(m[[0, 2]], 3.0);
            assert_eq!(m[[1, 0]], 4.0);
        }
        other => panic!("expected matrix, got {other:?}"),
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_dlmread_empty_file() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_dlm_empty.csv");
    std::fs::write(&tmp, "").unwrap();

    let read_expr = Expr::Call(
        "dlmread".to_string(),
        vec![Expr::StrLiteral(tmp.to_string_lossy().to_string())],
    );
    match eval(&read_expr, &env).unwrap() {
        Value::Matrix(m) => assert_eq!(m.shape(), &[0, 0]),
        other => panic!("expected empty matrix, got {other:?}"),
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_dlmread_non_numeric_error() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_dlm_bad.csv");
    std::fs::write(&tmp, "1,2,three\n").unwrap();

    let read_expr = Expr::Call(
        "dlmread".to_string(),
        vec![Expr::StrLiteral(tmp.to_string_lossy().to_string())],
    );
    let result = eval(&read_expr, &env);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("non-numeric"));

    let _ = std::fs::remove_file(&tmp);
}

#[allow(clippy::approx_constant)]
#[test]
fn test_dlmwrite_scalar() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_dlm_scalar.csv");
    let path = tmp.to_string_lossy().to_string();

    let write_expr = Expr::Call(
        "dlmwrite".to_string(),
        vec![Expr::StrLiteral(path.clone()), Expr::Number(3.14)],
    );
    assert_eq!(eval(&write_expr, &env), Ok(Value::Void));

    let content = std::fs::read_to_string(&tmp).unwrap();
    assert!(content.starts_with("3.14"));

    let _ = std::fs::remove_file(&tmp);
}

// --- Phase 10.5c: isfile / isfolder / exist / pwd ---

#[test]
fn test_isfile_existing_file() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_isfile.txt");
    std::fs::write(&tmp, "").unwrap();

    let expr = Expr::Call(
        "isfile".to_string(),
        vec![Expr::StrLiteral(tmp.to_string_lossy().to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_isfile_nonexistent() {
    let env = empty_env();
    let expr = Expr::Call(
        "isfile".to_string(),
        vec![Expr::StrLiteral("/no/such/file.txt".to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));
}

#[test]
fn test_isfile_directory_is_false() {
    let env = empty_env();
    let dir = std::env::temp_dir();
    let expr = Expr::Call(
        "isfile".to_string(),
        vec![Expr::StrLiteral(dir.to_string_lossy().to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));
}

#[test]
fn test_isfolder_existing_dir() {
    let env = empty_env();
    let dir = std::env::temp_dir();
    let expr = Expr::Call(
        "isfolder".to_string(),
        vec![Expr::StrLiteral(dir.to_string_lossy().to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

#[test]
fn test_isfolder_file_is_false() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_isfolder.txt");
    std::fs::write(&tmp, "").unwrap();

    let expr = Expr::Call(
        "isfolder".to_string(),
        vec![Expr::StrLiteral(tmp.to_string_lossy().to_string())],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_pwd_returns_string() {
    let env = empty_env();
    let expr = Expr::Call("pwd".to_string(), vec![]);
    let result = eval(&expr, &env).unwrap();
    match result {
        Value::Str(s) => assert!(!s.is_empty()),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn test_exist_var_found() {
    let mut env = empty_env();
    env.insert("myvar".to_string(), Value::Scalar(42.0));

    let expr = Expr::Call(
        "exist".to_string(),
        vec![
            Expr::StrLiteral("myvar".to_string()),
            Expr::StrLiteral("var".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

#[test]
fn test_exist_var_not_found() {
    let env = empty_env();
    let expr = Expr::Call(
        "exist".to_string(),
        vec![
            Expr::StrLiteral("novar".to_string()),
            Expr::StrLiteral("var".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));
}

#[test]
fn test_exist_file_found() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_test_exist_file.txt");
    std::fs::write(&tmp, "").unwrap();

    let expr = Expr::Call(
        "exist".to_string(),
        vec![
            Expr::StrLiteral(tmp.to_string_lossy().to_string()),
            Expr::StrLiteral("file".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(2.0)));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_exist_file_not_found() {
    let env = empty_env();
    let expr = Expr::Call(
        "exist".to_string(),
        vec![
            Expr::StrLiteral("/no/such/file.txt".to_string()),
            Expr::StrLiteral("file".to_string()),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));
}

#[test]
fn test_exist_one_arg_checks_var_then_file() {
    let mut env = empty_env();
    env.insert("x".to_string(), Value::Scalar(1.0));

    // variable found → 1
    let expr = Expr::Call("exist".to_string(), vec![Expr::StrLiteral("x".to_string())]);
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));

    // not in env, not a file → 0
    let expr2 = Expr::Call(
        "exist".to_string(),
        vec![Expr::StrLiteral("novar".to_string())],
    );
    assert_eq!(eval(&expr2, &env), Ok(Value::Scalar(0.0)));
}

// --- genpath ---

#[test]
fn test_genpath_includes_root() {
    let env = empty_env();
    let tmp = std::env::temp_dir();
    let expr = Expr::Call(
        "genpath".to_string(),
        vec![Expr::StrLiteral(tmp.to_string_lossy().to_string())],
    );
    let result = eval(&expr, &env).unwrap();
    match result {
        Value::Str(s) => {
            let sep = if cfg!(windows) { ';' } else { ':' };
            let parts: Vec<&str> = s.split(sep).collect();
            assert!(
                parts[0] == tmp.to_string_lossy().as_ref(),
                "root dir must be first entry"
            );
        }
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn test_genpath_includes_subdirs() {
    let env = empty_env();
    let tmp = std::env::temp_dir().join("ccalc_genpath_test");
    let sub = tmp.join("sub");
    std::fs::create_dir_all(&sub).unwrap();

    let expr = Expr::Call(
        "genpath".to_string(),
        vec![Expr::StrLiteral(tmp.to_string_lossy().to_string())],
    );
    let result = eval(&expr, &env).unwrap();

    let _ = std::fs::remove_dir_all(&tmp);

    match result {
        Value::Str(s) => {
            let sep = if cfg!(windows) { ';' } else { ':' };
            let parts: Vec<&str> = s.split(sep).collect();
            assert!(
                parts.len() >= 2,
                "should include root and at least one subdir"
            );
            assert!(parts.iter().any(|p| p.ends_with("sub")));
        }
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn test_genpath_nonexistent_returns_empty() {
    let env = empty_env();
    let expr = Expr::Call(
        "genpath".to_string(),
        vec![Expr::StrLiteral("/does/not/exist/ccalc_xyz".to_string())],
    );
    let result = eval(&expr, &env).unwrap();
    assert_eq!(result, Value::Str(String::new()));
}

#[test]
fn test_log_is_natural_log() {
    let env = empty_env();
    // log(e) must be 1.0 (natural log, MATLAB-compatible)
    let result = eval(
        &Expr::Call("log".to_string(), vec![Expr::Number(std::f64::consts::E)]),
        &env,
    )
    .unwrap();
    assert_eq!(result, Value::Scalar(1.0));
}

#[test]
fn test_log10() {
    let env = empty_env();
    let result = eval(
        &Expr::Call("log10".to_string(), vec![Expr::Number(100.0)]),
        &env,
    )
    .unwrap();
    assert_eq!(result, Value::Scalar(2.0));
}

#[test]
fn test_log2() {
    let env = empty_env();
    let result = eval(
        &Expr::Call("log2".to_string(), vec![Expr::Number(8.0)]),
        &env,
    )
    .unwrap();
    assert_eq!(result, Value::Scalar(3.0));
}

#[test]
fn test_inf_capital() {
    let env = empty_env();
    let result = eval_parse("Inf", &env).unwrap();
    assert!(matches!(result, Value::Scalar(v) if v.is_infinite() && v > 0.0));
}

#[test]
fn test_nan_capital() {
    let env = empty_env();
    let result = eval_parse("NaN", &env).unwrap();
    assert!(matches!(result, Value::Scalar(v) if v.is_nan()));
}

// --- diag ---

#[test]
fn test_diag_row_vector_to_matrix() {
    let env = empty_env();
    let result = eval_parse("diag([1 2 3])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (3, 3));
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[1, 1]], 2.0);
    assert_eq!(m[[2, 2]], 3.0);
    assert_eq!(m[[0, 1]], 0.0);
    assert_eq!(m[[1, 0]], 0.0);
}

#[test]
fn test_diag_col_vector_to_matrix() {
    let env = empty_env();
    let result = eval_parse("diag([4; 5; 6])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (3, 3));
    assert_eq!(m[[0, 0]], 4.0);
    assert_eq!(m[[1, 1]], 5.0);
    assert_eq!(m[[2, 2]], 6.0);
}

#[test]
fn test_diag_square_matrix_extract() {
    let env = empty_env();
    let result = eval_parse("diag([1 2 3; 4 5 6; 7 8 9])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (3, 1));
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[1, 0]], 5.0);
    assert_eq!(m[[2, 0]], 9.0);
}

#[test]
fn test_diag_nonsquare_matrix_extract() {
    // 2×4 matrix: min(2,4) = 2 diagonal elements
    let env = empty_env();
    let result = eval_parse("diag([1 2 3 4; 5 6 7 8])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (2, 1));
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[1, 0]], 6.0);
}

#[test]
fn test_diag_scalar() {
    let env = empty_env();
    let result = eval_parse("diag([7])", &env).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 1));
    assert_eq!(m[[0, 0]], 7.0);
}

#[test]
fn test_diag_roundtrip() {
    // diag(diag(v)) should reconstruct the diagonal matrix
    let env = empty_env();
    let d = eval_parse("diag([1 2 3])", &env).unwrap();
    let Value::Matrix(dm) = &d else {
        panic!("expected matrix")
    };
    assert_eq!(dm[[0, 0]], 1.0);
    assert_eq!(dm[[1, 1]], 2.0);
    assert_eq!(dm[[2, 2]], 3.0);
    // Now extract diagonal back
    let mut env2 = env.clone();
    env2.insert("D".to_string(), d);
    let result = eval_parse("diag(D)", &env2).unwrap();
    let Value::Matrix(v) = result else {
        panic!("expected matrix")
    };
    assert_eq!(v.dim(), (3, 1));
    assert_eq!(v[[0, 0]], 1.0);
    assert_eq!(v[[1, 0]], 2.0);
    assert_eq!(v[[2, 0]], 3.0);
}

// --- Matrix literal concatenation (Phase 15 extension) ---

#[test]
fn test_matrix_horzcat_col_vector() {
    // [A, b] where A is 2×2 and b is 2×1 → augmented matrix 2×3
    let env = empty_env();
    let result = eval_parse("[1 2; 3 4]", &env).unwrap();
    let mut env2 = env.clone();
    env2.insert("A".to_string(), result);
    let b = eval_parse("[5; 6]", &env).unwrap();
    env2.insert("b".to_string(), b);
    let aug = eval_parse("[A, b]", &env2).unwrap();
    let Value::Matrix(m) = aug else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (2, 3));
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[0, 2]], 5.0);
    assert_eq!(m[[1, 2]], 6.0);
}

#[test]
fn test_matrix_horzcat_two_matrices() {
    // [A, B] where both are 2×2 → 2×4
    let env = empty_env();
    let a = eval_parse("[1 2; 3 4]", &env).unwrap();
    let b = eval_parse("[5 6; 7 8]", &env).unwrap();
    let mut env2 = env.clone();
    env2.insert("A".to_string(), a);
    env2.insert("B".to_string(), b);
    let result = eval_parse("[A, B]", &env2).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (2, 4));
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[0, 2]], 5.0);
    assert_eq!(m[[1, 3]], 8.0);
}

#[test]
fn test_matrix_vertcat_two_matrices() {
    // [A; B] where both are 2×2 → 4×2
    let env = empty_env();
    let a = eval_parse("[1 2; 3 4]", &env).unwrap();
    let b = eval_parse("[5 6; 7 8]", &env).unwrap();
    let mut env2 = env.clone();
    env2.insert("A".to_string(), a);
    env2.insert("B".to_string(), b);
    let result = eval_parse("[A; B]", &env2).unwrap();
    let Value::Matrix(m) = result else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (4, 2));
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[2, 0]], 5.0);
    assert_eq!(m[[3, 1]], 8.0);
}

#[test]
fn test_matrix_horzcat_height_mismatch_error() {
    // [A, b] where A is 2×2 and b is 3×1 → error
    let env = empty_env();
    let a = eval_parse("[1 2; 3 4]", &env).unwrap();
    let b = eval_parse("[5; 6; 7]", &env).unwrap();
    let mut env2 = env.clone();
    env2.insert("A".to_string(), a);
    env2.insert("b".to_string(), b);
    assert!(eval_parse("[A, b]", &env2).is_err());
}

#[test]
fn test_matrix_vertcat_width_mismatch_error() {
    // [A; B] where A is 2×2 and B is 2×3 → error
    let env = empty_env();
    let a = eval_parse("[1 2; 3 4]", &env).unwrap();
    let b = eval_parse("[5 6 7; 8 9 10]", &env).unwrap();
    let mut env2 = env.clone();
    env2.insert("A".to_string(), a);
    env2.insert("B".to_string(), b);
    assert!(eval_parse("[A; B]", &env2).is_err());
}

// ── Phase 17a — Random number generation ─────────────────────────────────────

#[test]
fn test_rand_scalar_in_range() {
    // rand() returns a scalar in [0, 1)
    let env = empty_env();
    let v = eval_parse("rand()", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((0.0..1.0).contains(&x));
}

#[test]
fn test_rand_square_matrix() {
    let env = empty_env();
    let v = eval_parse("rand(3)", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (3, 3));
    for &x in m.iter() {
        assert!((0.0..1.0).contains(&x));
    }
}

#[test]
fn test_rand_rect_matrix() {
    let env = empty_env();
    let v = eval_parse("rand(2, 5)", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (2, 5));
    for &x in m.iter() {
        assert!((0.0..1.0).contains(&x));
    }
}

#[test]
fn test_randn_scalar() {
    // randn() returns a scalar (no range guarantee — just check it's finite)
    let env = empty_env();
    let v = eval_parse("randn()", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!(x.is_finite());
}

#[test]
fn test_randn_square_matrix() {
    let env = empty_env();
    let v = eval_parse("randn(4)", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (4, 4));
    for &x in m.iter() {
        assert!(x.is_finite());
    }
}

#[test]
fn test_randn_rect_matrix() {
    let env = empty_env();
    let v = eval_parse("randn(2, 6)", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (2, 6));
}

#[test]
fn test_randi_scalar_max() {
    // randi(10) → integer in [1, 10]
    let env = empty_env();
    let v = eval_parse("randi(10)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((1.0..=10.0).contains(&x) && x.fract() == 0.0);
}

#[test]
fn test_randi_square_matrix() {
    let env = empty_env();
    let v = eval_parse("randi(6, 3)", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (3, 3));
    for &x in m.iter() {
        assert!((1.0..=6.0).contains(&x) && x.fract() == 0.0);
    }
}

#[test]
fn test_randi_rect_matrix() {
    let env = empty_env();
    let v = eval_parse("randi(100, 2, 4)", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (2, 4));
    for &x in m.iter() {
        assert!((1.0..=100.0).contains(&x) && x.fract() == 0.0);
    }
}

#[test]
fn test_rng_seed_reproducible() {
    // Two identically-seeded runs must produce the same sequence.
    let env = empty_env();
    eval_parse("rng(42)", &env).unwrap();
    let v1 = eval_parse("rand()", &env).unwrap();

    eval_parse("rng(42)", &env).unwrap();
    let v2 = eval_parse("rand()", &env).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn test_rng_shuffle_returns_void() {
    let env = empty_env();
    let r = eval_parse("rng('shuffle')", &env).unwrap();
    assert_eq!(r, Value::Void);
}

// ── Phase 17b — Descriptive statistics ───────────────────────────────────────

// --- std ---

#[test]
fn test_std_vector_sample() {
    // [0 2 4]: mean=2, ss=8, var(n-1)=4, std=2.0
    let env = empty_env();
    let v = eval_parse("std([0 2 4])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 2.0).abs() < 1e-10);
}

#[test]
fn test_std_vector_population() {
    // [2 4 4 4 5 5 7 9]: population std = sqrt(32/8) = 2.0
    let env = empty_env();
    let v = eval_parse("std([2 4 4 4 5 5 7 9], 1)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 2.0).abs() < 1e-10);
}

#[test]
fn test_std_scalar_is_zero() {
    let env = empty_env();
    let v = eval_parse("std(5)", &env).unwrap();
    assert_eq!(v, Value::Scalar(0.0));
}

#[test]
fn test_std_matrix_columnwise() {
    // [1 10; 2 20; 3 30] — col1: std([1,2,3])=1; col2: std([10,20,30])=10
    let env = empty_env();
    let v = eval_parse("std([1 10; 2 20; 3 30])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 2));
    assert!((m[[0, 0]] - 1.0).abs() < 1e-10);
    assert!((m[[0, 1]] - 10.0).abs() < 1e-10);
}

// --- var ---

#[test]
fn test_var_vector_sample() {
    // [0 2 4]: mean=2, ss=8, var(n-1)=8/2=4.0
    let env = empty_env();
    let v = eval_parse("var([0 2 4])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 4.0).abs() < 1e-10);
}

#[test]
fn test_var_vector_population() {
    // [2 4 4 4 5 5 7 9]: population var = 32/8 = 4.0
    let env = empty_env();
    let v = eval_parse("var([2 4 4 4 5 5 7 9], 1)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 4.0).abs() < 1e-10);
}

#[test]
fn test_var_two_elements() {
    // var([1 3]) = ((1-2)^2 + (3-2)^2) / 1 = 2
    let env = empty_env();
    let v = eval_parse("var([1 3])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 2.0).abs() < 1e-10);
}

// --- cov ---

#[test]
fn test_cov_vector_equals_var() {
    // cov of a vector = var with n-1
    let env = empty_env();
    let v = eval_parse("cov([1 2 3 4 5])", &env).unwrap();
    let var_v = eval_parse("var([1 2 3 4 5])", &env).unwrap();
    assert_eq!(v, var_v);
}

#[test]
fn test_cov_matrix_shape() {
    // 4 observations × 3 variables → 3×3 covariance matrix
    let env = empty_env();
    let v = eval_parse("cov([1 2 3; 4 5 6; 7 8 9; 10 11 12])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (3, 3));
}

#[test]
fn test_cov_matrix_symmetric() {
    let env = empty_env();
    let v = eval_parse("cov([1 4; 2 5; 3 6; 4 7])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert!((m[[0, 1]] - m[[1, 0]]).abs() < 1e-12);
}

// --- median ---

#[test]
fn test_median_odd_length() {
    let env = empty_env();
    let v = eval_parse("median([3 1 4 1 5 9 2 6 5])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 4.0).abs() < 1e-10);
}

#[test]
fn test_median_even_length() {
    let env = empty_env();
    let v = eval_parse("median([1 2 3 4])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 2.5).abs() < 1e-10);
}

#[test]
fn test_median_scalar() {
    let env = empty_env();
    let v = eval_parse("median(7)", &env).unwrap();
    assert_eq!(v, Value::Scalar(7.0));
}

#[test]
fn test_median_matrix_columnwise() {
    // [1 10; 3 30; 2 20] → sorted col1 [1,2,3] median=2; col2 median=20
    let env = empty_env();
    let v = eval_parse("median([1 10; 3 30; 2 20])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 2));
    assert!((m[[0, 0]] - 2.0).abs() < 1e-10);
    assert!((m[[0, 1]] - 20.0).abs() < 1e-10);
}

// --- mode ---

#[test]
fn test_mode_single_mode() {
    let env = empty_env();
    let v = eval_parse("mode([1 2 2 3 3 3 4])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 3.0).abs() < 1e-10);
}

#[test]
fn test_mode_tie_smallest_wins() {
    // Tie between 1 and 2 — smallest (1) should win
    let env = empty_env();
    let v = eval_parse("mode([1 1 2 2 3])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 1.0).abs() < 1e-10);
}

#[test]
fn test_mode_scalar() {
    let env = empty_env();
    let v = eval_parse("mode(5)", &env).unwrap();
    assert_eq!(v, Value::Scalar(5.0));
}

// --- histc ---

#[test]
fn test_histc_basic() {
    // Values [1 2 3 4 5], edges [1 3 5]
    // bin 0: 1 <= x < 3 → 1, 2 → count 2
    // bin 1: 3 <= x < 5 → 3, 4 → count 2
    // bin 2: x == 5 exactly → 5 → count 1
    let env = empty_env();
    let v = eval_parse("histc([1 2 3 4 5], [1 3 5])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 3));
    assert_eq!(m[[0, 0]], 2.0);
    assert_eq!(m[[0, 1]], 2.0);
    assert_eq!(m[[0, 2]], 1.0);
}

#[test]
fn test_histc_out_of_range_not_counted() {
    // Values [0 1 2 3 10], edges [1 2 3] — 0 and 10 out of range
    let env = empty_env();
    let v = eval_parse("histc([0 1 2 3 10], [1 2 3])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[0, 1]], 1.0);
    assert_eq!(m[[0, 2]], 1.0);
}

// --- hist (returns Void) ---

#[test]
fn test_hist_returns_void() {
    let env = empty_env();
    let v = eval_parse("hist([1 2 3 4 5])", &env).unwrap();
    assert_eq!(v, Value::Void);
}

#[test]
fn test_hist_custom_bins_returns_void() {
    let env = empty_env();
    let v = eval_parse("hist([1 2 3 4 5], 5)", &env).unwrap();
    assert_eq!(v, Value::Void);
}

// ── Phase 17c — Percentiles and distributions ─────────────────────────────────

// --- prctile ---

#[test]
fn test_prctile_median() {
    // prctile([1 2 3 4 5], 50) = 3 (median)
    let env = empty_env();
    let v = eval_parse("prctile([1 2 3 4 5], 50)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 3.0).abs() < 1e-10);
}

#[test]
fn test_prctile_0th_percentile() {
    let env = empty_env();
    let v = eval_parse("prctile([1 2 3 4 5], 0)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 1.0).abs() < 1e-10);
}

#[test]
fn test_prctile_100th_percentile() {
    let env = empty_env();
    let v = eval_parse("prctile([1 2 3 4 5], 100)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 5.0).abs() < 1e-10);
}

#[test]
fn test_prctile_interpolation() {
    // prctile([1 3], 50) = (1+3)/2 = 2.0
    let env = empty_env();
    let v = eval_parse("prctile([1 3], 50)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 2.0).abs() < 1e-10);
}

#[test]
fn test_prctile_vector_of_percentiles() {
    // prctile([1 2 3 4 5], [0 50 100]) = [1 3 5]
    let env = empty_env();
    let v = eval_parse("prctile([1 2 3 4 5], [0 50 100])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 3));
    assert!((m[[0, 0]] - 1.0).abs() < 1e-10);
    assert!((m[[0, 1]] - 3.0).abs() < 1e-10);
    assert!((m[[0, 2]] - 5.0).abs() < 1e-10);
}

#[test]
fn test_prctile_matrix_columnwise() {
    // prctile([1 10; 2 20; 3 30], 50) → [2 20]
    let env = empty_env();
    let v = eval_parse("prctile([1 10; 2 20; 3 30], 50)", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (1, 2));
    assert!((m[[0, 0]] - 2.0).abs() < 1e-10);
    assert!((m[[0, 1]] - 20.0).abs() < 1e-10);
}

// --- iqr ---

#[test]
fn test_iqr_basic() {
    // iqr([1 2 3 4 5]) = prctile(75) - prctile(25) = 4 - 2 = 2
    let env = empty_env();
    let v = eval_parse("iqr([1 2 3 4 5])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 2.0).abs() < 1e-10);
}

#[test]
fn test_iqr_scalar() {
    // iqr of a single value is 0
    let env = empty_env();
    let v = eval_parse("iqr(5)", &env).unwrap();
    assert_eq!(v, Value::Scalar(0.0));
}

// --- zscore ---

#[test]
fn test_zscore_basic() {
    // zscore([2 4 6]): mean=4, std=2; z = [-1 0 1]
    let env = empty_env();
    let v = eval_parse("zscore([2 4 6])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.len(), 3);
    let vals: Vec<f64> = m.iter().copied().collect();
    assert!((vals[0] - (-1.0)).abs() < 1e-10);
    assert!((vals[1] - 0.0).abs() < 1e-10);
    assert!((vals[2] - 1.0).abs() < 1e-10);
}

#[test]
fn test_zscore_scalar_is_zero() {
    let env = empty_env();
    let v = eval_parse("zscore(42)", &env).unwrap();
    assert_eq!(v, Value::Scalar(0.0));
}

#[test]
fn test_zscore_constant_vector() {
    // All same values → std=0 → zscore all zeros
    let env = empty_env();
    let v = eval_parse("zscore([3 3 3])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    for &x in m.iter() {
        assert_eq!(x, 0.0);
    }
}

#[test]
fn test_zscore_preserves_shape() {
    let env = empty_env();
    let v = eval_parse("zscore([1; 2; 3])", &env).unwrap();
    let Value::Matrix(m) = v else {
        panic!("expected matrix")
    };
    assert_eq!(m.dim(), (3, 1));
}

// ── Phase 17d — Special functions ────────────────────────────────────────────

// --- erf / erfc ---

#[test]
fn test_erf_zero() {
    let env = empty_env();
    let v = eval_parse("erf(0)", &env).unwrap();
    assert_eq!(v, Value::Scalar(0.0));
}

#[test]
fn test_erf_large_positive() {
    // erf(∞) → 1.0 (erf(10) is very close to 1)
    let env = empty_env();
    let v = eval_parse("erf(10)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 1.0).abs() < 1e-10);
}

#[test]
fn test_erfc_zero() {
    // erfc(0) = 1 - erf(0) = 1
    let env = empty_env();
    let v = eval_parse("erfc(0)", &env).unwrap();
    assert_eq!(v, Value::Scalar(1.0));
}

#[test]
fn test_erf_erfc_sum() {
    // erf(x) + erfc(x) = 1 for any x
    let env = empty_env();
    let erf_v = eval_parse("erf(1.5)", &env).unwrap();
    let erfc_v = eval_parse("erfc(1.5)", &env).unwrap();
    let Value::Scalar(e) = erf_v else { panic!() };
    let Value::Scalar(ec) = erfc_v else { panic!() };
    assert!((e + ec - 1.0).abs() < 1e-14);
}

// --- normcdf ---

#[test]
fn test_normcdf_at_zero() {
    // normcdf(0) = 0.5
    let env = empty_env();
    let v = eval_parse("normcdf(0)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 0.5).abs() < 1e-10);
}

#[test]
fn test_normcdf_symmetry() {
    // normcdf(-x) = 1 - normcdf(x)
    let env = empty_env();
    let v1 = eval_parse("normcdf(1.5)", &env).unwrap();
    let v2 = eval_parse("normcdf(-1.5)", &env).unwrap();
    let Value::Scalar(x1) = v1 else { panic!() };
    let Value::Scalar(x2) = v2 else { panic!() };
    assert!((x1 + x2 - 1.0).abs() < 1e-14);
}

#[test]
fn test_normcdf_general() {
    // normcdf(2, 2, 1) = normcdf(0) = 0.5
    let env = empty_env();
    let v = eval_parse("normcdf(2, 2, 1)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 0.5).abs() < 1e-10);
}

// --- normpdf ---

#[test]
fn test_normpdf_at_zero() {
    // normpdf(0) = 1/sqrt(2*pi) ≈ 0.39894
    let env = empty_env();
    let v = eval_parse("normpdf(0)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    let expected = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
    assert!((x - expected).abs() < 1e-10);
}

#[test]
fn test_normpdf_symmetry() {
    // normpdf(x) = normpdf(-x)
    let env = empty_env();
    let v1 = eval_parse("normpdf(1.5)", &env).unwrap();
    let v2 = eval_parse("normpdf(-1.5)", &env).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn test_normpdf_general() {
    // normpdf(mu, mu, s) = normpdf(0) / s = peak of the distribution
    let env = empty_env();
    let v = eval_parse("normpdf(3, 3, 2)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    // normpdf(x, mu, s) at x=mu: exp(0) / (s * sqrt(2π)) = 1 / (s * sqrt(2π))
    let expected = 1.0 / (2.0 * (2.0 * std::f64::consts::PI).sqrt());
    assert!((x - expected).abs() < 1e-10);
}

// ── Phase 17e — Skewness and kurtosis ────────────────────────────────────────

// --- skewness ---

#[test]
fn test_skewness_symmetric_is_zero() {
    // [1 2 3 4 5]: symmetric → skewness = 0 exactly
    let env = empty_env();
    let v = eval_parse("skewness([1 2 3 4 5])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!(x.abs() < 1e-12);
}

#[test]
fn test_skewness_right_skewed_positive() {
    // [1 1 2 3 10]: outlier at 10 makes distribution right-skewed
    let env = empty_env();
    let v = eval_parse("skewness([1 1 2 3 10])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!(x > 0.0);
}

#[test]
fn test_skewness_scalar_is_zero() {
    let env = empty_env();
    let v = eval_parse("skewness(5)", &env).unwrap();
    assert_eq!(v, Value::Scalar(0.0));
}

#[test]
fn test_skewness_constant_vector_is_zero() {
    let env = empty_env();
    let v = eval_parse("skewness([3 3 3 3])", &env).unwrap();
    assert_eq!(v, Value::Scalar(0.0));
}

// --- kurtosis ---

#[test]
fn test_kurtosis_uniform_vector() {
    // [1 2 3 4 5]: mu=3, m2=2, m4=6.8 → kurtosis = 6.8/4 = 1.7
    let env = empty_env();
    let v = eval_parse("kurtosis([1 2 3 4 5])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!((x - 1.7).abs() < 1e-10);
}

#[test]
fn test_kurtosis_scalar_is_nan() {
    let env = empty_env();
    let v = eval_parse("kurtosis(5)", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!(x.is_nan());
}

#[test]
fn test_kurtosis_constant_vector_is_nan() {
    let env = empty_env();
    let v = eval_parse("kurtosis([4 4 4 4])", &env).unwrap();
    let Value::Scalar(x) = v else {
        panic!("expected scalar")
    };
    assert!(x.is_nan());
}

// ── Phase 18 — Advanced linear algebra ──────────────────────────────────────

fn run_linalg(src: &str) -> crate::env::Env {
    use crate::eval::{Base, FormatMode};
    use crate::io::IoContext;
    use crate::parser::parse_stmts;
    crate::exec::init();
    let stmts = parse_stmts(src).expect("parse_stmts failed");
    let mut env = crate::env::Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    crate::exec::exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .expect("exec_stmts failed");
    env
}

#[allow(dead_code)]
fn mat_approx_zero(env: &crate::env::Env, name: &str, tol: f64) {
    match env.get(name) {
        Some(Value::Matrix(m)) => {
            let max = m.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
            assert!(max < tol, "{name}: max element {max} >= tol {tol}");
        }
        Some(Value::Scalar(x)) => assert!(x.abs() < tol, "{name} = {x} >= tol {tol}"),
        v => panic!("expected numeric for '{name}', got {v:?}"),
    }
}

fn scalar_near(env: &crate::env::Env, name: &str, expected: f64, tol: f64) {
    match env.get(name) {
        Some(Value::Scalar(x)) => {
            assert!(
                (x - expected).abs() < tol,
                "{name}: got {x}, expected {expected}"
            )
        }
        v => panic!("expected scalar for '{name}', got {v:?}"),
    }
}

#[test]
fn test_lu_pa_equals_lu() {
    let env = run_linalg("A = [4 3; 6 3]; [L, U, P] = lu(A); err = norm(P*A - L*U);");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_lu_l_lower_triangular() {
    let env = run_linalg("A = [4 3; 6 3]; [L, U, P] = lu(A); off = L(1,2);");
    scalar_near(&env, "off", 0.0, 1e-14);
}

#[test]
fn test_qr_a_equals_qr() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; [Q, R] = qr(A); err = norm(A - Q*R);");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_qr_q_orthogonal() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; [Q, R] = qr(A); err = norm(Q'*Q - eye(3));");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_chol_rtr_equals_a() {
    let env = run_linalg("A = [4 2; 2 3]; R = chol(A); err = norm(R'*R - A);");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_chol_not_posdef_errors() {
    use crate::eval::Base;
    use crate::eval::FormatMode;
    use crate::io::IoContext;
    use crate::parser::parse_stmts;
    crate::exec::init();
    let stmts = parse_stmts("R = chol([-1 0; 0 1])").unwrap();
    let mut env = crate::env::Env::new();
    let mut io = IoContext::new();
    let res = crate::exec::exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    );
    assert!(
        res.is_err(),
        "expected error for non-positive-definite matrix"
    );
}

#[test]
fn test_svd_singular_values_sorted_descending() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; s = svd(A); ok = (s(1) >= s(2));");
    scalar_near(&env, "ok", 1.0, 1e-14);
}

#[test]
fn test_svd_full_reconstruction() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; [U, S, V] = svd(A); err = norm(A - U*S*V');");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_svd_u_orthogonal() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; [U, S, V] = svd(A); err = norm(U'*U - eye(3));");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_svd_v_orthogonal() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; [U, S, V] = svd(A); err = norm(V'*V - eye(2));");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_eig_symmetric_eigenvalues() {
    let env = run_linalg("A = [2 1; 1 2]; d = eig(A);");
    match env.get("d") {
        Some(Value::Matrix(m)) => {
            let mut vals: Vec<f64> = m.iter().copied().collect();
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert!((vals[0] - 1.0).abs() < 1e-10, "smallest eig: {}", vals[0]);
            assert!((vals[1] - 3.0).abs() < 1e-10, "largest eig: {}", vals[1]);
        }
        v => panic!("expected matrix, got {v:?}"),
    }
}

#[test]
fn test_eig_multi_output() {
    let env = run_linalg("A = [2 1; 1 2]; [V, D] = eig(A); err = norm(A*V - V*D);");
    scalar_near(&env, "err", 0.0, 1e-10);
}

#[test]
fn test_rank_full_rank() {
    let env = run_linalg("r = rank([1 2; 3 4]);");
    scalar_near(&env, "r", 2.0, 1e-14);
}

#[test]
fn test_rank_rank_deficient() {
    let env = run_linalg("r = rank([1 2; 2 4]);");
    scalar_near(&env, "r", 1.0, 1e-14);
}

#[test]
fn test_rank_zero_matrix() {
    let env = run_linalg("r = rank(zeros(3,3));");
    scalar_near(&env, "r", 0.0, 1e-14);
}

#[test]
fn test_null_space() {
    let env = run_linalg("A = [1 2; 2 4]; N = null(A); err = norm(A*N);");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_orth_columns_in_column_space() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; Q = orth(A); err = norm(Q'*Q - eye(2));");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_cond_identity() {
    let env = run_linalg("c = cond(eye(3));");
    scalar_near(&env, "c", 1.0, 1e-12);
}

#[test]
fn test_cond_singular_is_inf() {
    let env = run_linalg("c = cond([1 2; 2 4]);");
    match env.get("c") {
        Some(Value::Scalar(x)) => assert!(x.is_infinite(), "expected Inf, got {x}"),
        v => panic!("expected scalar, got {v:?}"),
    }
}

#[test]
fn test_pinv_pseudoinverse() {
    let env = run_linalg("A = [1 2; 3 4; 5 6]; B = pinv(A); err = norm(A*B*A - A);");
    scalar_near(&env, "err", 0.0, 1e-12);
}

#[test]
fn test_norm_matrix_2norm() {
    let env = run_linalg("A = [3 0; 0 1]; n = norm(A);");
    scalar_near(&env, "n", 3.0, 1e-12);
}

#[test]
fn test_norm_matrix_frobenius() {
    let env = run_linalg("A = [3 4; 0 0]; n = norm(A, 'fro');");
    scalar_near(&env, "n", 5.0, 1e-12);
}

#[test]
fn test_norm_matrix_1norm() {
    let env = run_linalg("A = [1 2; 3 4]; n = norm(A, 1);");
    scalar_near(&env, "n", 6.0, 1e-12);
}

#[test]
fn test_norm_matrix_inf() {
    let env = run_linalg("A = [1 2; 3 4]; n = norm(A, inf);");
    scalar_near(&env, "n", 7.0, 1e-12);
}

#[test]
fn test_svd_econ() {
    let env =
        run_linalg("A = [1 2; 3 4; 5 6]; [U, S, V] = svd(A, 'econ'); err = norm(A - U*S*V');");
    scalar_near(&env, "err", 0.0, 1e-12);
}

// ── Bug regression tests ─────────────────────────────────────────────────────

fn run_script_src(src: &str) -> crate::env::Env {
    use crate::eval::{Base, FormatMode};
    use crate::io::IoContext;
    use crate::parser::parse_stmts;
    crate::exec::init();
    let stmts = parse_stmts(src).expect("parse_stmts failed");
    let mut env = crate::env::Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    crate::exec::exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .expect("exec_stmts failed");
    env
}

// Bug 1: run() inside a script used to return from exec_stmts immediately,
// so all statements after the first run() call were silently skipped.
#[test]
fn test_run_does_not_abort_outer_script() {
    use std::io::Write;

    let dir = std::env::temp_dir();
    let helper_path = dir.join("ccalc_test_run_helper.calc");
    {
        let mut f = std::fs::File::create(&helper_path).expect("create helper");
        f.write_all(b"helper_ran = 1;\n").expect("write");
    }
    let path_str = helper_path.to_str().unwrap().replace('\\', "/");

    let script = format!("before = 10;\nrun('{path_str}');\nafter = 20;\n");
    let env = run_script_src(&script);

    let _ = std::fs::remove_file(&helper_path);

    assert_eq!(
        env.get("before"),
        Some(&Value::Scalar(10.0)),
        "before should be set"
    );
    assert_eq!(
        env.get("helper_ran"),
        Some(&Value::Scalar(1.0)),
        "helper script should have executed"
    );
    assert_eq!(
        env.get("after"),
        Some(&Value::Scalar(20.0)),
        "statements after run() must not be skipped"
    );
}

// Bug 2: a mixed script+function file (functions defined at the top, script
// body below) was wrongly detected as a pure function file, so the script body
// never executed and produced no output / set no variables.
#[test]
fn test_mixed_function_script_file_runs_body() {
    use std::io::Write;

    let dir = std::env::temp_dir();
    let file_path = dir.join("ccalc_test_mixed_fn_script.calc");
    {
        let mut f = std::fs::File::create(&file_path).expect("create file");
        f.write_all(b"function y = double_it(x)\n  y = x * 2;\nend\n\nresult = double_it(21);\n")
            .expect("write");
    }
    let path_str = file_path.to_str().unwrap().replace('\\', "/");

    let script = format!("run('{path_str}');\n");
    let env = run_script_src(&script);

    let _ = std::fs::remove_file(&file_path);

    assert_eq!(
        env.get("result"),
        Some(&Value::Scalar(42.0)),
        "script body must execute even when the file starts with function defs"
    );
}

// ── Phase 19d: assert built-ins ──────────────────────────────────────────────

#[test]
fn test_assert_true_condition() {
    let env = empty_env();
    assert_eq!(eval_parse("assert(1)", &env).unwrap(), Value::Void);
}

#[test]
fn test_assert_false_condition() {
    let env = empty_env();
    assert!(eval_parse("assert(0)", &env).is_err());
}

#[test]
fn test_assert_nonzero_is_true() {
    let env = empty_env();
    assert_eq!(eval_parse("assert(42)", &env).unwrap(), Value::Void);
}

#[test]
fn test_assert_nan_is_false() {
    let env = empty_env();
    assert!(eval_parse("assert(nan)", &env).is_err());
}

#[test]
fn test_assert_equal_scalars_pass() {
    let env = empty_env();
    assert_eq!(eval_parse("assert(3, 3)", &env).unwrap(), Value::Void);
}

#[test]
fn test_assert_equal_scalars_fail() {
    let env = empty_env();
    assert!(eval_parse("assert(3, 4)", &env).is_err());
}

#[test]
fn test_assert_tol_pass() {
    let env = empty_env();
    assert_eq!(eval_parse("assert(1, 2, 1.5)", &env).unwrap(), Value::Void);
}

#[test]
fn test_assert_tol_fail() {
    let env = empty_env();
    assert!(eval_parse("assert(1, 2, 0.5)", &env).is_err());
}

#[test]
fn test_assert_exact_tol_zero() {
    let env = empty_env();
    assert_eq!(eval_parse("assert(5, 5, 0)", &env).unwrap(), Value::Void);
}

// ── Phase 19c: "did you mean?" suggestions ───────────────────────────────────

#[test]
fn test_suggest_similar_undefined_var() {
    let mut env = empty_env();
    env.insert("length".to_string(), Value::Scalar(1.0));
    let err = eval(&Expr::Var("lnegth".to_string()), &env).unwrap_err();
    assert!(
        err.contains("did you mean"),
        "expected suggestion in: {err}"
    );
    assert!(err.contains("length"), "expected 'length' in: {err}");
}

#[test]
fn test_suggest_similar_unknown_fn() {
    let env = empty_env();
    let err = eval_parse("sqrtt(4)", &env).unwrap_err();
    assert!(
        err.contains("did you mean"),
        "expected suggestion in: {err}"
    );
    assert!(err.contains("sqrt"), "expected 'sqrt' in: {err}");
}

#[test]
fn test_no_suggestion_for_totally_different_name() {
    let env = empty_env();
    let err = eval_parse("zzzzzzz(4)", &env).unwrap_err();
    assert!(
        !err.contains("did you mean"),
        "unexpected suggestion in: {err}"
    );
}

// ── Phase 19a: builtin_names list ────────────────────────────────────────────

#[test]
fn test_builtin_names_contains_expected() {
    let names = builtin_names();
    assert!(names.contains(&"sqrt"), "sqrt missing from builtin_names");
    assert!(
        names.contains(&"assert"),
        "assert missing from builtin_names"
    );
    assert!(names.contains(&"zeros"), "zeros missing from builtin_names");
    assert!(
        names.contains(&"fprintf"),
        "fprintf missing from builtin_names"
    );
}

#[test]
fn test_builtin_names_no_duplicates() {
    let names = builtin_names();
    let mut seen = std::collections::HashSet::new();
    for n in names {
        assert!(seen.insert(n), "duplicate builtin name: {n}");
    }
}

// --- Phase 20a: JSON ---

#[test]
fn test_json_builtins_in_names() {
    let names = builtin_names();
    assert!(
        names.contains(&"jsondecode"),
        "jsondecode missing from builtin_names"
    );
    assert!(
        names.contains(&"jsonencode"),
        "jsonencode missing from builtin_names"
    );
}

#[cfg(feature = "json")]
mod json_tests {
    use super::*;

    fn decode(s: &str) -> Value {
        let expr = Expr::Call(
            "jsondecode".to_string(),
            vec![Expr::StrLiteral(s.to_string())],
        );
        eval(&expr, &empty_env()).expect("jsondecode failed")
    }

    fn encode(v: Value) -> String {
        let mut env = empty_env();
        env.insert("_x".to_string(), v);
        let expr = Expr::Call("jsonencode".to_string(), vec![Expr::Var("_x".to_string())]);
        match eval(&expr, &env).expect("jsonencode failed") {
            Value::Str(s) => s,
            other => panic!("expected Str, got {other:?}"),
        }
    }

    #[test]
    fn decode_scalar_number() {
        assert_eq!(decode("42"), Value::Scalar(42.0));
    }

    #[test]
    fn decode_null_is_nan() {
        match decode("null") {
            Value::Scalar(x) => assert!(x.is_nan()),
            other => panic!("expected Scalar(NaN), got {other:?}"),
        }
    }

    #[test]
    fn decode_bool_true() {
        assert_eq!(decode("true"), Value::Scalar(1.0));
    }

    #[test]
    fn decode_bool_false() {
        assert_eq!(decode("false"), Value::Scalar(0.0));
    }

    #[test]
    fn decode_string() {
        assert_eq!(decode(r#""hello""#), Value::Str("hello".to_string()));
    }

    #[test]
    fn decode_numeric_array() {
        use ndarray::array;
        match decode("[1, 2, 3]") {
            Value::Matrix(m) => assert_eq!(m, array![[1.0, 2.0, 3.0]]),
            other => panic!("expected Matrix, got {other:?}"),
        }
    }

    #[test]
    fn decode_empty_array() {
        use ndarray::Array2;
        match decode("[]") {
            Value::Matrix(m) => assert_eq!(m, Array2::<f64>::zeros((1, 0))),
            other => panic!("expected empty Matrix, got {other:?}"),
        }
    }

    #[test]
    fn decode_mixed_array_is_cell() {
        match decode(r#"[1, "two", 3]"#) {
            Value::Cell(cells) => {
                assert_eq!(cells.len(), 3);
                assert_eq!(cells[0], Value::Scalar(1.0));
                assert_eq!(cells[1], Value::Str("two".to_string()));
                assert_eq!(cells[2], Value::Scalar(3.0));
            }
            other => panic!("expected Cell, got {other:?}"),
        }
    }

    #[test]
    fn decode_object_is_struct() {
        match decode(r#"{"x": 1, "y": 2}"#) {
            Value::Struct(fields) => {
                assert_eq!(fields.get("x"), Some(&Value::Scalar(1.0)));
                assert_eq!(fields.get("y"), Some(&Value::Scalar(2.0)));
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn decode_nested_struct() {
        match decode(r#"{"a": {"b": 42}}"#) {
            Value::Struct(outer) => match outer.get("a") {
                Some(Value::Struct(inner)) => {
                    assert_eq!(inner.get("b"), Some(&Value::Scalar(42.0)));
                }
                other => panic!("expected inner Struct, got {other:?}"),
            },
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn decode_invalid_json_errors() {
        let expr = Expr::Call(
            "jsondecode".to_string(),
            vec![Expr::StrLiteral("{bad json".to_string())],
        );
        assert!(eval(&expr, &empty_env()).is_err());
    }

    #[test]
    fn decode_non_string_arg_errors() {
        let expr = Expr::Call("jsondecode".to_string(), vec![Expr::Number(42.0)]);
        assert!(eval(&expr, &empty_env()).is_err());
    }

    #[test]
    fn encode_scalar() {
        assert_eq!(encode(Value::Scalar(3.14)), "3.14");
    }

    #[test]
    fn encode_scalar_integer() {
        assert_eq!(encode(Value::Scalar(5.0)), "5.0");
    }

    #[test]
    fn encode_nan_is_null() {
        assert_eq!(encode(Value::Scalar(f64::NAN)), "null");
    }

    #[test]
    fn encode_inf_errors() {
        let mut env = empty_env();
        env.insert("_x".to_string(), Value::Scalar(f64::INFINITY));
        let expr = Expr::Call("jsonencode".to_string(), vec![Expr::Var("_x".to_string())]);
        assert!(eval(&expr, &env).is_err());
    }

    #[test]
    fn encode_string() {
        assert_eq!(encode(Value::Str("hello".to_string())), r#""hello""#);
    }

    #[test]
    fn encode_row_vector() {
        use ndarray::array;
        let m = Value::Matrix(array![[1.0, 2.0, 3.0]]);
        assert_eq!(encode(m), "[1.0,2.0,3.0]");
    }

    #[test]
    fn encode_matrix_2d() {
        use ndarray::array;
        let m = Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(encode(m), "[[1.0,2.0],[3.0,4.0]]");
    }

    #[test]
    fn encode_struct() {
        use indexmap::IndexMap;
        let mut fields = IndexMap::new();
        fields.insert("x".to_string(), Value::Scalar(1.0));
        let result = encode(Value::Struct(fields));
        assert_eq!(result, r#"{"x":1.0}"#);
    }

    #[test]
    fn encode_cell() {
        let cells = vec![Value::Scalar(1.0), Value::Str("a".to_string())];
        let result = encode(Value::Cell(cells));
        assert_eq!(result, r#"[1.0,"a"]"#);
    }

    #[test]
    fn roundtrip_object() {
        let json = r#"{"name":"Alice","score":99}"#;
        let decoded = decode(json);
        let reencoded = {
            let mut env = empty_env();
            env.insert("_x".to_string(), decoded);
            let expr = Expr::Call("jsonencode".to_string(), vec![Expr::Var("_x".to_string())]);
            match eval(&expr, &env).unwrap() {
                Value::Str(s) => s,
                other => panic!("{other:?}"),
            }
        };
        // Re-decode and compare field by field
        let redecoded = decode(&reencoded);
        match redecoded {
            Value::Struct(fields) => {
                assert_eq!(fields.get("name"), Some(&Value::Str("Alice".to_string())));
                assert_eq!(fields.get("score"), Some(&Value::Scalar(99.0)));
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }
}

mod csv_tests {
    use super::*;
    use indexmap::IndexMap;
    use ndarray::Array2;

    fn tmp_csv(tag: &str, content: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("ccalc_csv_{}_{}.csv", std::process::id(), tag));
        std::fs::write(&path, content).unwrap();
        path
    }

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("ccalc_csv_{}_{}.csv", std::process::id(), tag));
        path
    }

    fn call_rm(path: &str) -> Value {
        let expr = Expr::Call(
            "readmatrix".to_string(),
            vec![Expr::StrLiteral(path.to_string())],
        );
        eval(&expr, &empty_env()).expect("readmatrix failed")
    }

    fn call_rt(path: &str) -> Value {
        let expr = Expr::Call(
            "readtable".to_string(),
            vec![Expr::StrLiteral(path.to_string())],
        );
        eval(&expr, &empty_env()).expect("readtable failed")
    }

    fn call_wt(tbl: Value, path: &str) {
        let mut env = empty_env();
        env.insert("_t".to_string(), tbl);
        let expr = Expr::Call(
            "writetable".to_string(),
            vec![
                Expr::Var("_t".to_string()),
                Expr::StrLiteral(path.to_string()),
            ],
        );
        eval(&expr, &env).expect("writetable failed");
    }

    // ----- readmatrix -----

    #[test]
    fn readmatrix_basic_numeric() {
        let p = tmp_csv("rm_basic", "1,2,3\n4,5,6\n");
        let ps = p.to_str().unwrap();
        match call_rm(ps) {
            Value::Matrix(m) => {
                assert_eq!((m.nrows(), m.ncols()), (2, 3));
                assert_eq!(m[[0, 0]], 1.0);
                assert_eq!(m[[1, 2]], 6.0);
            }
            v => panic!("expected Matrix, got {v:?}"),
        }
    }

    #[test]
    fn readmatrix_header_skipped() {
        let p = tmp_csv("rm_hdr", "x,y,z\n1,2,3\n4,5,6\n");
        let ps = p.to_str().unwrap();
        match call_rm(ps) {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 2);
                assert_eq!(m[[0, 0]], 1.0);
            }
            v => panic!("expected Matrix, got {v:?}"),
        }
    }

    #[test]
    fn readmatrix_numeric_first_row_not_header() {
        let p = tmp_csv("rm_numfirst", "1,2,3\n4,5,6\n");
        let ps = p.to_str().unwrap();
        match call_rm(ps) {
            Value::Matrix(m) => assert_eq!(m.nrows(), 2),
            v => panic!("expected Matrix, got {v:?}"),
        }
    }

    #[test]
    fn readmatrix_explicit_tab_delim() {
        let p = tmp_csv("rm_tab", "1\t2\t3\n4\t5\t6\n");
        let ps = p.to_str().unwrap();
        let expr = Expr::Call(
            "readmatrix".to_string(),
            vec![
                Expr::StrLiteral(ps.to_string()),
                Expr::StrLiteral("Delimiter".to_string()),
                Expr::StrLiteral(r"\t".to_string()),
            ],
        );
        match eval(&expr, &empty_env()).unwrap() {
            Value::Matrix(m) => {
                assert_eq!((m.nrows(), m.ncols()), (2, 3));
                assert_eq!(m[[1, 1]], 5.0);
            }
            v => panic!("expected Matrix, got {v:?}"),
        }
    }

    #[test]
    fn readmatrix_empty_cell_becomes_nan() {
        let p = tmp_csv("rm_nan", "1,,3\n");
        let ps = p.to_str().unwrap();
        match call_rm(ps) {
            Value::Matrix(m) => {
                assert_eq!(m[[0, 0]], 1.0);
                assert!(m[[0, 1]].is_nan());
                assert_eq!(m[[0, 2]], 3.0);
            }
            v => panic!("expected Matrix, got {v:?}"),
        }
    }

    #[test]
    fn readmatrix_empty_file() {
        let p = tmp_csv("rm_empty", "");
        let ps = p.to_str().unwrap();
        match call_rm(ps) {
            Value::Matrix(m) => assert_eq!((m.nrows(), m.ncols()), (0, 0)),
            v => panic!("expected Matrix, got {v:?}"),
        }
    }

    // ----- readtable -----

    #[test]
    fn readtable_numeric_columns() {
        let p = tmp_csv("rt_num", "x,y\n1,2\n3,4\n");
        let ps = p.to_str().unwrap();
        match call_rt(ps) {
            Value::Struct(fields) => {
                assert!(fields.contains_key("x") && fields.contains_key("y"));
                match &fields["x"] {
                    Value::Matrix(m) => {
                        assert_eq!((m.nrows(), m.ncols()), (2, 1));
                        assert_eq!(m[[0, 0]], 1.0);
                        assert_eq!(m[[1, 0]], 3.0);
                    }
                    v => panic!("expected Matrix column, got {v:?}"),
                }
            }
            v => panic!("expected Struct, got {v:?}"),
        }
    }

    #[test]
    fn readtable_mixed_columns() {
        let p = tmp_csv("rt_mixed", "name,score\nAlice,95\nBob,87\n");
        let ps = p.to_str().unwrap();
        match call_rt(ps) {
            Value::Struct(fields) => {
                match &fields["name"] {
                    Value::Cell(c) => {
                        assert_eq!(c[0], Value::Str("Alice".to_string()));
                        assert_eq!(c[1], Value::Str("Bob".to_string()));
                    }
                    v => panic!("expected Cell column, got {v:?}"),
                }
                match &fields["score"] {
                    Value::Matrix(m) => {
                        assert_eq!(m[[0, 0]], 95.0);
                        assert_eq!(m[[1, 0]], 87.0);
                    }
                    v => panic!("expected Matrix column, got {v:?}"),
                }
            }
            v => panic!("expected Struct, got {v:?}"),
        }
    }

    #[test]
    fn readtable_header_only() {
        let p = tmp_csv("rt_hdronly", "x,y\n");
        let ps = p.to_str().unwrap();
        match call_rt(ps) {
            Value::Struct(fields) => {
                assert!(fields.contains_key("x") && fields.contains_key("y"));
                match &fields["x"] {
                    Value::Matrix(m) => assert_eq!(m.nrows(), 0),
                    v => panic!("expected empty Matrix column, got {v:?}"),
                }
            }
            v => panic!("expected Struct, got {v:?}"),
        }
    }

    #[test]
    fn readtable_quoted_field_with_comma() {
        let p = tmp_csv("rt_quoted", "name,value\n\"Smith, John\",100\n");
        let ps = p.to_str().unwrap();
        match call_rt(ps) {
            Value::Struct(fields) => match &fields["name"] {
                Value::Cell(c) => {
                    assert_eq!(c[0], Value::Str("Smith, John".to_string()));
                }
                v => panic!("expected Cell, got {v:?}"),
            },
            v => panic!("expected Struct, got {v:?}"),
        }
    }

    #[test]
    fn readtable_empty_file() {
        let p = tmp_csv("rt_empty", "");
        let ps = p.to_str().unwrap();
        match call_rt(ps) {
            Value::Struct(fields) => assert!(fields.is_empty()),
            v => panic!("expected empty Struct, got {v:?}"),
        }
    }

    // ----- writetable -----

    #[test]
    fn writetable_basic() {
        let mut fields = IndexMap::new();
        fields.insert(
            "x".to_string(),
            Value::Matrix(Array2::from_shape_vec((2, 1), vec![1.0, 2.0]).unwrap()),
        );
        fields.insert(
            "y".to_string(),
            Value::Matrix(Array2::from_shape_vec((2, 1), vec![3.0, 4.0]).unwrap()),
        );
        let path = tmp_path("wt_basic");
        let ps = path.to_str().unwrap();
        call_wt(Value::Struct(fields), ps);
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "x,y");
        assert_eq!(lines[1], "1,3");
        assert_eq!(lines[2], "2,4");
    }

    #[test]
    fn writetable_quoting() {
        let mut fields = IndexMap::new();
        fields.insert(
            "text".to_string(),
            Value::Cell(vec![
                Value::Str("hello, world".to_string()),
                Value::Str("plain".to_string()),
            ]),
        );
        let path = tmp_path("wt_quote");
        let ps = path.to_str().unwrap();
        call_wt(Value::Struct(fields), ps);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"hello, world\""));
        assert!(content.contains("plain"));
    }

    #[test]
    fn writetable_readtable_roundtrip() {
        let p_in = tmp_csv("rt_rtrip_in", "name,score\nAlice,95\nBob,87\n");
        let tbl = call_rt(p_in.to_str().unwrap());
        let p_out = tmp_path("rt_rtrip_out");
        call_wt(tbl, p_out.to_str().unwrap());
        let content = std::fs::read_to_string(&p_out).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "name,score");
        assert!(lines[1..].contains(&"Alice,95") || lines[1..].contains(&"Alice,95.0"));
        assert!(lines[1..].contains(&"Bob,87") || lines[1..].contains(&"Bob,87.0"));
    }

    #[test]
    fn writetable_wrong_type_errors() {
        let mut fields = IndexMap::new();
        fields.insert("a".to_string(), Value::Scalar(1.0));
        fields.insert("b".to_string(), Value::Matrix(Array2::<f64>::zeros((2, 2))));
        let path = tmp_path("wt_err");
        let ps = path.to_str().unwrap();
        let mut env = empty_env();
        env.insert("_t".to_string(), Value::Struct(fields));
        let expr = Expr::Call(
            "writetable".to_string(),
            vec![
                Expr::Var("_t".to_string()),
                Expr::StrLiteral(ps.to_string()),
            ],
        );
        assert!(eval(&expr, &env).is_err());
    }
}

#[cfg(feature = "mat")]
mod mat_tests {
    use super::*;
    use crate::mat::mat_load;

    fn tmp_mat_path(tag: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("ccalc_mat_{}_{}.mat", std::process::id(), tag));
        path
    }

    #[test]
    fn test_mat_load_nonexistent() {
        let result = mat_load("/nonexistent/does_not_exist.mat");
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert!(e.contains("load:"), "error should mention 'load:': {e}");
    }

    #[test]
    fn test_load_builtin_not_mat_extension() {
        let expr = Expr::Call(
            "load".to_string(),
            vec![Expr::StrLiteral("data.toml".to_string())],
        );
        let env = empty_env();
        let result = eval(&expr, &env);
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert!(e.contains("non-.mat"), "error: {e}");
    }

    #[test]
    fn test_load_builtin_bad_arg() {
        let expr = Expr::Call("load".to_string(), vec![Expr::Number(42.0)]);
        let env = empty_env();
        let result = eval(&expr, &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_in_builtin_names() {
        assert!(
            builtin_names().contains(&"load"),
            "load missing from builtin_names"
        );
    }

    #[test]
    fn test_mat_load_error_prefix() {
        let path = tmp_mat_path("nonexist");
        let result = mat_load(path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("load:"));
    }

    #[test]
    fn test_mat_roundtrip_scalar() {
        use matrw::{matfile, matvar, save_matfile_v7};
        let path = tmp_mat_path("scalar");
        let ps = path.to_str().unwrap();
        let mat = matfile!(x: matvar!(3.14),);
        save_matfile_v7(ps, mat, false).expect("write .mat");
        let result = mat_load(ps).unwrap();
        std::fs::remove_file(&path).ok();
        match result {
            Value::Struct(fields) => {
                assert_eq!(fields.get("x"), Some(&Value::Scalar(3.14)));
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn test_mat_roundtrip_vector() {
        use matrw::{matfile, matvar, save_matfile_v7};
        let path = tmp_mat_path("vector");
        let ps = path.to_str().unwrap();
        let mat = matfile!(v: matvar!([1.0, 2.0, 3.0]),);
        save_matfile_v7(ps, mat, false).expect("write .mat");
        let result = mat_load(ps).unwrap();
        std::fs::remove_file(&path).ok();
        match result {
            Value::Struct(fields) => match fields.get("v").unwrap() {
                Value::Matrix(m) => {
                    assert_eq!(m.nrows(), 1);
                    assert_eq!(m.ncols(), 3);
                    assert_eq!(m[[0, 0]], 1.0);
                    assert_eq!(m[[0, 2]], 3.0);
                }
                other => panic!("expected Matrix, got {other:?}"),
            },
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn test_mat_roundtrip_matrix() {
        use matrw::{matfile, matvar, save_matfile_v7};
        let path = tmp_mat_path("matrix");
        let ps = path.to_str().unwrap();
        // 2×3: rows [[1,2,3],[4,5,6]]
        let mat = matfile!(A: matvar!([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),);
        save_matfile_v7(ps, mat, false).expect("write .mat");
        let result = mat_load(ps).unwrap();
        std::fs::remove_file(&path).ok();
        match result {
            Value::Struct(fields) => match fields.get("A").unwrap() {
                Value::Matrix(m) => {
                    assert_eq!((m.nrows(), m.ncols()), (2, 3));
                    assert_eq!(m[[0, 0]], 1.0);
                    assert_eq!(m[[0, 2]], 3.0);
                    assert_eq!(m[[1, 0]], 4.0);
                    assert_eq!(m[[1, 2]], 6.0);
                }
                other => panic!("expected Matrix, got {other:?}"),
            },
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn test_mat_roundtrip_string() {
        use matrw::{matfile, matvar, save_matfile_v7};
        let path = tmp_mat_path("string");
        let ps = path.to_str().unwrap();
        let mat = matfile!(label: matvar!("hello"),);
        save_matfile_v7(ps, mat, false).expect("write .mat");
        let result = mat_load(ps).unwrap();
        std::fs::remove_file(&path).ok();
        match result {
            Value::Struct(fields) => match fields.get("label").unwrap() {
                Value::Str(s) => assert_eq!(s, "hello"),
                other => panic!("expected Str, got {other:?}"),
            },
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn test_mat_roundtrip_struct() {
        use matrw::{matfile, matvar, save_matfile_v7};
        let path = tmp_mat_path("struct");
        let ps = path.to_str().unwrap();
        let mat = matfile!(s: matvar!({ x: 1.0, y: 2.0 }),);
        save_matfile_v7(ps, mat, false).expect("write .mat");
        let result = mat_load(ps).unwrap();
        std::fs::remove_file(&path).ok();
        match result {
            Value::Struct(outer) => match outer.get("s").unwrap() {
                Value::Struct(inner) => {
                    assert_eq!(inner.get("x"), Some(&Value::Scalar(1.0)));
                    assert_eq!(inner.get("y"), Some(&Value::Scalar(2.0)));
                }
                other => panic!("expected inner Struct, got {other:?}"),
            },
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    /// Run once to regenerate examples/mat/fixtures/sample.mat.
    /// Invoke with: cargo test --features mat create_example_fixture -- --ignored
    #[test]
    #[ignore]
    fn create_example_fixture() {
        use matrw::{matfile, matvar, save_matfile_v7};
        // Resolve path relative to workspace root (two levels up from this crate).
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let dir = workspace.join("examples/mat/fixtures");
        std::fs::create_dir_all(&dir).unwrap();
        let mat = matfile!(
            score:    matvar!(92.5),
            readings: matvar!([23.1, 21.8, 24.3, 22.7, 25.0, 23.6]),
            A:        matvar!([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
            label:    matvar!("experiment-1"),
            sensor:   matvar!({ id: 1.0, gain: 0.5 }),
        );
        let out = dir.join("sample.mat");
        save_matfile_v7(out.to_str().unwrap(), mat, false).expect("write sample.mat");
        println!("Created {}", out.display());
    }
}

// ---------------------------------------------------------------------------
// Phase 21a — String predicates and joining
// ---------------------------------------------------------------------------

fn call1(fname: &str, a: Value) -> Result<Value, String> {
    let mut env = empty_env();
    env.insert("_a".to_string(), a);
    eval(
        &Expr::Call(fname.to_string(), vec![Expr::Var("_a".to_string())]),
        &env,
    )
}

fn call2(fname: &str, a: Value, b: Value) -> Result<Value, String> {
    let mut env = empty_env();
    env.insert("_a".to_string(), a);
    env.insert("_b".to_string(), b);
    eval(
        &Expr::Call(
            fname.to_string(),
            vec![Expr::Var("_a".to_string()), Expr::Var("_b".to_string())],
        ),
        &env,
    )
}

#[cfg(feature = "regex")]
fn call3(fname: &str, a: Value, b: Value, c: Value) -> Result<Value, String> {
    let mut env = empty_env();
    env.insert("_a".to_string(), a);
    env.insert("_b".to_string(), b);
    env.insert("_c".to_string(), c);
    eval(
        &Expr::Call(
            fname.to_string(),
            vec![
                Expr::Var("_a".to_string()),
                Expr::Var("_b".to_string()),
                Expr::Var("_c".to_string()),
            ],
        ),
        &env,
    )
}

#[test]
fn test_contains_found() {
    assert_eq!(
        call2(
            "contains",
            Value::Str("hello world".into()),
            Value::Str("world".into())
        ),
        Ok(Value::Scalar(1.0))
    );
}

#[test]
fn test_contains_not_found() {
    assert_eq!(
        call2(
            "contains",
            Value::Str("hello".into()),
            Value::Str("xyz".into())
        ),
        Ok(Value::Scalar(0.0))
    );
}

#[test]
fn test_contains_ignore_case() {
    let env = empty_env();
    let expr = Expr::Call(
        "contains".to_string(),
        vec![
            Expr::StrLiteral("Hello".to_string()),
            Expr::StrLiteral("hello".to_string()),
            Expr::StrLiteral("IgnoreCase".to_string()),
            Expr::Number(1.0),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(1.0)));
}

#[test]
fn test_contains_ignore_case_false() {
    let env = empty_env();
    let expr = Expr::Call(
        "contains".to_string(),
        vec![
            Expr::StrLiteral("Hello".to_string()),
            Expr::StrLiteral("hello".to_string()),
            Expr::StrLiteral("IgnoreCase".to_string()),
            Expr::Number(0.0),
        ],
    );
    assert_eq!(eval(&expr, &env), Ok(Value::Scalar(0.0)));
}

#[test]
fn test_starts_with_true() {
    assert_eq!(
        call2(
            "startsWith",
            Value::Str("hello".into()),
            Value::Str("he".into())
        ),
        Ok(Value::Scalar(1.0))
    );
}

#[test]
fn test_starts_with_false() {
    assert_eq!(
        call2(
            "startsWith",
            Value::Str("hello".into()),
            Value::Str("lo".into())
        ),
        Ok(Value::Scalar(0.0))
    );
}

#[test]
fn test_ends_with_true() {
    assert_eq!(
        call2(
            "endsWith",
            Value::Str("hello".into()),
            Value::Str("lo".into())
        ),
        Ok(Value::Scalar(1.0))
    );
}

#[test]
fn test_ends_with_false() {
    assert_eq!(
        call2(
            "endsWith",
            Value::Str("hello".into()),
            Value::Str("he".into())
        ),
        Ok(Value::Scalar(0.0))
    );
}

#[test]
fn test_strjoin_with_delimiter() {
    let cell = Value::Cell(vec![
        Value::Str("a".into()),
        Value::Str("b".into()),
        Value::Str("c".into()),
    ]);
    assert_eq!(
        call2("strjoin", cell, Value::Str(",".into())),
        Ok(Value::Str("a,b,c".into()))
    );
}

#[test]
fn test_strjoin_default_space() {
    let cell = Value::Cell(vec![Value::Str("x".into()), Value::Str("y".into())]);
    assert_eq!(call1("strjoin", cell), Ok(Value::Str("x y".into())));
}

#[test]
fn test_strjoin_single_element() {
    let cell = Value::Cell(vec![Value::Str("only".into())]);
    assert_eq!(call1("strjoin", cell), Ok(Value::Str("only".into())));
}

#[test]
fn test_strjoin_empty_cell() {
    let cell = Value::Cell(vec![]);
    assert_eq!(call1("strjoin", cell), Ok(Value::Str("".into())));
}

#[test]
fn test_strjoin_non_string_cell_errors() {
    let cell = Value::Cell(vec![Value::Str("ok".into()), Value::Scalar(1.0)]);
    assert!(call1("strjoin", cell).is_err());
}

#[test]
fn test_strjoin_non_cell_errors() {
    assert!(call1("strjoin", Value::Str("not a cell".into())).is_err());
}

#[test]
fn test_contains_in_builtin_names() {
    assert!(
        builtin_names().contains(&"contains"),
        "contains missing from builtin_names"
    );
}

#[test]
fn test_strjoin_in_builtin_names() {
    assert!(
        builtin_names().contains(&"strjoin"),
        "strjoin missing from builtin_names"
    );
}

#[test]
fn test_starts_ends_with_in_builtin_names() {
    assert!(builtin_names().contains(&"startsWith"));
    assert!(builtin_names().contains(&"endsWith"));
}

#[test]
fn test_regexp_in_builtin_names() {
    assert!(builtin_names().contains(&"regexp"));
    assert!(builtin_names().contains(&"regexpi"));
    assert!(builtin_names().contains(&"regexprep"));
}

// ---------------------------------------------------------------------------
// Phase 21b — Regular expressions (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "regex")]
mod regex_tests {
    use super::*;

    fn regexp(s: &str, pat: &str) -> Value {
        call2("regexp", Value::Str(s.into()), Value::Str(pat.into())).expect("regexp failed")
    }

    fn regexp_match(s: &str, pat: &str) -> Value {
        call3(
            "regexp",
            Value::Str(s.into()),
            Value::Str(pat.into()),
            Value::Str("match".into()),
        )
        .expect("regexp match failed")
    }

    fn regexpi(s: &str, pat: &str) -> Value {
        call2("regexpi", Value::Str(s.into()), Value::Str(pat.into())).expect("regexpi failed")
    }

    fn regexprep(s: &str, pat: &str, rep: &str) -> Value {
        call3(
            "regexprep",
            Value::Str(s.into()),
            Value::Str(pat.into()),
            Value::Str(rep.into()),
        )
        .expect("regexprep failed")
    }

    #[test]
    fn regexp_returns_start_index() {
        // 'abc 123 def' — digits start at char 5 (1-based)
        assert_eq!(regexp("abc 123 def", r"\d+"), Value::Scalar(5.0));
    }

    #[test]
    fn regexp_no_match_returns_empty_matrix() {
        use ndarray::Array2;
        assert_eq!(
            regexp("hello", r"\d+"),
            Value::Matrix(Array2::zeros((0, 0)))
        );
    }

    #[test]
    fn regexp_match_returns_cell_of_strings() {
        match regexp_match("abc 123 def 456", r"\d+") {
            Value::Cell(v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], Value::Str("123".into()));
                assert_eq!(v[1], Value::Str("456".into()));
            }
            other => panic!("expected Cell, got {other:?}"),
        }
    }

    #[test]
    fn regexp_match_no_matches_returns_empty_cell() {
        match regexp_match("hello world", r"\d+") {
            Value::Cell(v) => assert!(v.is_empty()),
            other => panic!("expected empty Cell, got {other:?}"),
        }
    }

    #[test]
    fn regexpi_case_insensitive() {
        assert_eq!(regexpi("Hello World", "hello"), Value::Scalar(1.0));
    }

    #[test]
    fn regexprep_basic() {
        assert_eq!(
            regexprep("foo  bar", r"\s+", "_"),
            Value::Str("foo_bar".into())
        );
    }

    #[test]
    fn regexprep_date_slash() {
        assert_eq!(
            regexprep("2024-01-15", "-", "/"),
            Value::Str("2024/01/15".into())
        );
    }

    #[test]
    fn regexprep_literal_dollar_sign() {
        // replacement is literal: '$1' must not be expanded as a capture group
        assert_eq!(regexprep("a", "a", "$1"), Value::Str("$1".into()));
    }

    #[test]
    fn regexp_invalid_pattern_errors() {
        let result = call2(
            "regexp",
            Value::Str("x".into()),
            Value::Str("[invalid".into()),
        );
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("invalid pattern"), "unexpected error: {msg}");
    }

    #[test]
    fn regexp_unknown_option_errors() {
        let result = call3(
            "regexp",
            Value::Str("x".into()),
            Value::Str(r"\w".into()),
            Value::Str("tokens".into()),
        );
        assert!(result.is_err());
    }
}

// ── Phase 22 — Datetime & Duration tests ─────────────────────────────────────

#[cfg(test)]
mod datetime_tests {
    use super::*;
    use crate::datetime::{civil_to_timestamp, format_datetime, format_duration};

    fn dt(y: i64, mo: u32, d: u32, h: u32, mi: u32, s: f64) -> Value {
        Value::DateTime(civil_to_timestamp(y, mo, d, h, mi, s))
    }

    fn call1(fname: &str, a: Value) -> Result<Value, String> {
        let mut env = Env::new();
        env.insert("_a".to_string(), a);
        eval(
            &Expr::Call(fname.to_string(), vec![Expr::Var("_a".to_string())]),
            &env,
        )
    }

    fn call2(fname: &str, a: Value, b: Value) -> Result<Value, String> {
        let mut env = Env::new();
        env.insert("_a".to_string(), a);
        env.insert("_b".to_string(), b);
        eval(
            &Expr::Call(
                fname.to_string(),
                vec![Expr::Var("_a".to_string()), Expr::Var("_b".to_string())],
            ),
            &env,
        )
    }

    fn call3(fname: &str, a: Value, b: Value, c: Value) -> Result<Value, String> {
        let mut env = Env::new();
        env.insert("_a".to_string(), a);
        env.insert("_b".to_string(), b);
        env.insert("_c".to_string(), c);
        eval(
            &Expr::Call(
                fname.to_string(),
                vec![
                    Expr::Var("_a".to_string()),
                    Expr::Var("_b".to_string()),
                    Expr::Var("_c".to_string()),
                ],
            ),
            &env,
        )
    }

    fn scalar(v: &Value) -> f64 {
        match v {
            Value::Scalar(n) => *n,
            other => panic!("expected Scalar, got {other:?}"),
        }
    }

    fn dur(v: &Value) -> f64 {
        match v {
            Value::Duration(s) => *s,
            other => panic!("expected Duration, got {other:?}"),
        }
    }

    fn ts(v: &Value) -> f64 {
        match v {
            Value::DateTime(t) => *t,
            other => panic!("expected DateTime, got {other:?}"),
        }
    }

    // ── Constructors ──────────────────────────────────────────────────────────

    #[test]
    fn datetime_iso_string() {
        let v = call1("datetime", Value::Str("2024-01-15".into())).unwrap();
        let expected = civil_to_timestamp(2024, 1, 15, 0, 0, 0.0);
        assert!((ts(&v) - expected).abs() < 1e-9);
    }

    #[test]
    fn datetime_iso_with_time() {
        let v = call1("datetime", Value::Str("2024-01-15 09:30:00".into())).unwrap();
        let expected = civil_to_timestamp(2024, 1, 15, 9, 30, 0.0);
        assert!((ts(&v) - expected).abs() < 1e-9);
    }

    #[test]
    fn datetime_three_args() {
        let v = call3(
            "datetime",
            Value::Scalar(2024.0),
            Value::Scalar(6.0),
            Value::Scalar(1.0),
        )
        .unwrap();
        let expected = civil_to_timestamp(2024, 6, 1, 0, 0, 0.0);
        assert!((ts(&v) - expected).abs() < 1e-9);
    }

    #[test]
    fn datetime_six_args() {
        let mut env = Env::new();
        for (k, v) in [
            ("_y", 2024.0),
            ("_mo", 3.0),
            ("_d", 10.0),
            ("_h", 14.0),
            ("_mi", 30.0),
            ("_s", 0.0),
        ] {
            env.insert(k.to_string(), Value::Scalar(v));
        }
        let args: Vec<Expr> = ["_y", "_mo", "_d", "_h", "_mi", "_s"]
            .iter()
            .map(|k| Expr::Var(k.to_string()))
            .collect();
        let v = eval(&Expr::Call("datetime".to_string(), args), &env).unwrap();
        let expected = civil_to_timestamp(2024, 3, 10, 14, 30, 0.0);
        assert!((ts(&v) - expected).abs() < 1e-9);
    }

    #[test]
    fn datetime_posixtime_convert() {
        let ts_val = 1_700_000_000.0_f64;
        let v = call3(
            "datetime",
            Value::Scalar(ts_val),
            Value::Str("ConvertFrom".into()),
            Value::Str("posixtime".into()),
        )
        .unwrap();
        assert!((ts(&v) - ts_val).abs() < 1e-9);
    }

    #[test]
    fn nat_constant() {
        let env = Env::new();
        let v = eval(&Expr::NaT, &env).unwrap();
        match v {
            Value::DateTime(t) => assert!(t.is_nan()),
            _ => panic!("expected DateTime(NaN)"),
        }
    }

    // ── Duration constructors ─────────────────────────────────────────────────

    #[test]
    fn duration_hms() {
        let v = call3(
            "duration",
            Value::Scalar(1.0),
            Value::Scalar(30.0),
            Value::Scalar(0.0),
        )
        .unwrap();
        assert!((dur(&v) - 5400.0).abs() < 1e-9);
    }

    #[test]
    fn hours_constructor() {
        let v = call1("hours", Value::Scalar(2.0)).unwrap();
        assert!((dur(&v) - 7200.0).abs() < 1e-9);
    }

    #[test]
    fn minutes_constructor() {
        let v = call1("minutes", Value::Scalar(90.0)).unwrap();
        assert!((dur(&v) - 5400.0).abs() < 1e-9);
    }

    #[test]
    fn seconds_constructor() {
        let v = call1("seconds", Value::Scalar(45.0)).unwrap();
        assert!((dur(&v) - 45.0).abs() < 1e-9);
    }

    #[test]
    fn days_constructor() {
        let v = call1("days", Value::Scalar(2.0)).unwrap();
        assert!((dur(&v) - 172800.0).abs() < 1e-9);
    }

    #[test]
    fn milliseconds_constructor() {
        let v = call1("milliseconds", Value::Scalar(500.0)).unwrap();
        assert!((dur(&v) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn years_constructor() {
        let v = call1("years", Value::Scalar(1.0)).unwrap();
        assert!((dur(&v) - 365.2425 * 86400.0).abs() < 1e-9);
    }

    // ── Duration extractors (Duration → Scalar) ───────────────────────────────

    #[test]
    fn hours_extractor() {
        let d = Value::Duration(7200.0);
        let v = call1("hours", d).unwrap();
        assert!((scalar(&v) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn minutes_extractor() {
        let d = Value::Duration(5400.0);
        let v = call1("minutes", d).unwrap();
        assert!((scalar(&v) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn seconds_extractor() {
        let d = Value::Duration(45.0);
        let v = call1("seconds", d).unwrap();
        assert!((scalar(&v) - 45.0).abs() < 1e-9);
    }

    #[test]
    fn days_extractor() {
        let d = Value::Duration(172800.0);
        let v = call1("days", d).unwrap();
        assert!((scalar(&v) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn milliseconds_extractor() {
        let d = Value::Duration(0.5);
        let v = call1("milliseconds", d).unwrap();
        assert!((scalar(&v) - 500.0).abs() < 1e-9);
    }

    // ── Component extractors ──────────────────────────────────────────────────

    #[test]
    fn year_extractor() {
        let v = call1("year", dt(2024, 3, 10, 0, 0, 0.0)).unwrap();
        assert_eq!(scalar(&v) as i64, 2024);
    }

    #[test]
    fn month_extractor() {
        let v = call1("month", dt(2024, 3, 10, 0, 0, 0.0)).unwrap();
        assert_eq!(scalar(&v) as u32, 3);
    }

    #[test]
    fn day_extractor() {
        let v = call1("day", dt(2024, 3, 10, 0, 0, 0.0)).unwrap();
        assert_eq!(scalar(&v) as u32, 10);
    }

    #[test]
    fn hour_extractor() {
        let v = call1("hour", dt(2024, 3, 10, 14, 30, 0.0)).unwrap();
        assert_eq!(scalar(&v) as u32, 14);
    }

    #[test]
    fn minute_extractor() {
        let v = call1("minute", dt(2024, 3, 10, 14, 30, 0.0)).unwrap();
        assert_eq!(scalar(&v) as u32, 30);
    }

    #[test]
    fn second_extractor() {
        let v = call1("second", dt(2024, 3, 10, 14, 30, 45.0)).unwrap();
        assert!((scalar(&v) - 45.0).abs() < 1e-6);
    }

    // ── Predicates ────────────────────────────────────────────────────────────

    #[test]
    fn isdatetime_true() {
        let v = call1("isdatetime", dt(2024, 1, 1, 0, 0, 0.0)).unwrap();
        assert_eq!(scalar(&v), 1.0);
    }

    #[test]
    fn isdatetime_false_for_scalar() {
        let v = call1("isdatetime", Value::Scalar(42.0)).unwrap();
        assert_eq!(scalar(&v), 0.0);
    }

    #[test]
    fn isduration_true() {
        let v = call1("isduration", Value::Duration(3600.0)).unwrap();
        assert_eq!(scalar(&v), 1.0);
    }

    #[test]
    fn isduration_false_for_scalar() {
        let v = call1("isduration", Value::Scalar(42.0)).unwrap();
        assert_eq!(scalar(&v), 0.0);
    }

    #[test]
    fn isnat_true() {
        let v = call1("isnat", Value::DateTime(f64::NAN)).unwrap();
        assert_eq!(scalar(&v), 1.0);
    }

    #[test]
    fn isnat_false() {
        let v = call1("isnat", dt(2024, 1, 1, 0, 0, 0.0)).unwrap();
        assert_eq!(scalar(&v), 0.0);
    }

    // ── Arithmetic ────────────────────────────────────────────────────────────

    #[test]
    fn datetime_plus_duration() {
        let t = dt(2024, 1, 1, 0, 0, 0.0);
        let d = Value::Duration(3600.0);
        let env = {
            let mut e = Env::new();
            e.insert("t".to_string(), t);
            e.insert("d".to_string(), d);
            e
        };
        let result = eval(
            &Expr::BinOp(
                Box::new(Expr::Var("t".to_string())),
                Op::Add,
                Box::new(Expr::Var("d".to_string())),
            ),
            &env,
        )
        .unwrap();
        let expected = civil_to_timestamp(2024, 1, 1, 1, 0, 0.0);
        assert!((ts(&result) - expected).abs() < 1e-9);
    }

    #[test]
    fn datetime_minus_duration() {
        let t = dt(2024, 1, 2, 0, 0, 0.0);
        let d = Value::Duration(86400.0);
        let env = {
            let mut e = Env::new();
            e.insert("t".to_string(), t);
            e.insert("d".to_string(), d);
            e
        };
        let result = eval(
            &Expr::BinOp(
                Box::new(Expr::Var("t".to_string())),
                Op::Sub,
                Box::new(Expr::Var("d".to_string())),
            ),
            &env,
        )
        .unwrap();
        let expected = civil_to_timestamp(2024, 1, 1, 0, 0, 0.0);
        assert!((ts(&result) - expected).abs() < 1e-9);
    }

    #[test]
    fn datetime_minus_datetime() {
        let t1 = dt(2024, 1, 2, 0, 0, 0.0);
        let t2 = dt(2024, 1, 1, 0, 0, 0.0);
        let env = {
            let mut e = Env::new();
            e.insert("t1".to_string(), t1);
            e.insert("t2".to_string(), t2);
            e
        };
        let result = eval(
            &Expr::BinOp(
                Box::new(Expr::Var("t1".to_string())),
                Op::Sub,
                Box::new(Expr::Var("t2".to_string())),
            ),
            &env,
        )
        .unwrap();
        assert!((dur(&result) - 86400.0).abs() < 1e-9);
    }

    #[test]
    fn duration_plus_duration() {
        let d1 = Value::Duration(3600.0);
        let d2 = Value::Duration(1800.0);
        let env = {
            let mut e = Env::new();
            e.insert("d1".to_string(), d1);
            e.insert("d2".to_string(), d2);
            e
        };
        let result = eval(
            &Expr::BinOp(
                Box::new(Expr::Var("d1".to_string())),
                Op::Add,
                Box::new(Expr::Var("d2".to_string())),
            ),
            &env,
        )
        .unwrap();
        assert!((dur(&result) - 5400.0).abs() < 1e-9);
    }

    #[test]
    fn duration_times_scalar() {
        let d = Value::Duration(3600.0);
        let env = {
            let mut e = Env::new();
            e.insert("d".to_string(), d);
            e
        };
        let result = eval(
            &Expr::BinOp(
                Box::new(Expr::Var("d".to_string())),
                Op::Mul,
                Box::new(Expr::Number(2.0)),
            ),
            &env,
        )
        .unwrap();
        assert!((dur(&result) - 7200.0).abs() < 1e-9);
    }

    // ── Formatting ────────────────────────────────────────────────────────────

    #[test]
    fn format_datetime_known() {
        let ts_val = civil_to_timestamp(2024, 1, 15, 9, 30, 0.0);
        assert_eq!(format_datetime(ts_val), "2024-01-15 09:30:00");
    }

    #[test]
    fn format_datetime_nat() {
        assert_eq!(format_datetime(f64::NAN), "NaT");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(3600.0), "01:00:00");
    }

    #[test]
    fn format_duration_days() {
        assert_eq!(format_duration(86400.0 + 7200.0), "1d 02:00:00");
    }

    #[test]
    fn format_duration_subsecond() {
        assert_eq!(format_duration(0.5), "00:00:00.500");
    }

    #[test]
    fn datestr_default_format() {
        let v = call1("datestr", dt(2024, 1, 15, 9, 30, 0.0)).unwrap();
        match v {
            Value::Str(s) => assert_eq!(s, "15-Jan-2024 09:30:00"),
            _ => panic!(),
        }
    }

    #[test]
    fn datestr_custom_format() {
        let v = call2(
            "datestr",
            dt(2024, 6, 1, 0, 0, 0.0),
            Value::Str("yyyy/MM/dd".into()),
        )
        .unwrap();
        match v {
            Value::Str(s) => assert_eq!(s, "2024/06/01"),
            _ => panic!(),
        }
    }

    // ── datevec ───────────────────────────────────────────────────────────────

    #[test]
    fn datevec_returns_row_vector() {
        let v = call1("datevec", dt(2024, 3, 10, 14, 30, 0.0)).unwrap();
        match &v {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[1, 6]);
                assert_eq!(m[[0, 0]], 2024.0);
                assert_eq!(m[[0, 1]], 3.0);
                assert_eq!(m[[0, 2]], 10.0);
                assert_eq!(m[[0, 3]], 14.0);
                assert_eq!(m[[0, 4]], 30.0);
                assert_eq!(m[[0, 5]], 0.0);
            }
            _ => panic!("expected Matrix"),
        }
    }

    // ── datenum / posixtime ───────────────────────────────────────────────────

    #[test]
    fn datenum_epoch() {
        let v = call1("datenum", dt(1970, 1, 1, 0, 0, 0.0)).unwrap();
        assert!((scalar(&v) - 719529.0).abs() < 1e-9);
    }

    #[test]
    fn datenum_three_args() {
        let v = call3(
            "datenum",
            Value::Scalar(1970.0),
            Value::Scalar(1.0),
            Value::Scalar(1.0),
        )
        .unwrap();
        assert!((scalar(&v) - 719529.0).abs() < 1e-9);
    }

    #[test]
    fn posixtime_roundtrip() {
        let ts_val = civil_to_timestamp(2024, 6, 1, 12, 0, 0.0);
        let v = call1("posixtime", Value::DateTime(ts_val)).unwrap();
        assert!((scalar(&v) - ts_val).abs() < 1e-9);
    }

    // ── Array operations ──────────────────────────────────────────────────────

    #[test]
    fn diff_datetime_array() {
        let v1 = civil_to_timestamp(2024, 1, 1, 0, 0, 0.0);
        let v2 = civil_to_timestamp(2024, 1, 2, 0, 0, 0.0);
        let v3 = civil_to_timestamp(2024, 1, 3, 0, 0, 0.0);
        let arr = Value::DateTimeArray(vec![v1, v2, v3]);
        let result = call1("diff", arr).unwrap();
        match result {
            Value::DurationArray(diffs) => {
                assert_eq!(diffs.len(), 2);
                assert!((diffs[0] - 86400.0).abs() < 1e-9);
                assert!((diffs[1] - 86400.0).abs() < 1e-9);
            }
            _ => panic!("expected DurationArray"),
        }
    }

    #[test]
    fn diff_duration_array() {
        let arr = Value::DurationArray(vec![3600.0, 7200.0, 10800.0]);
        let result = call1("diff", arr).unwrap();
        match result {
            Value::DurationArray(diffs) => {
                assert_eq!(diffs.len(), 2);
                assert!((diffs[0] - 3600.0).abs() < 1e-9);
            }
            _ => panic!("expected DurationArray"),
        }
    }

    #[test]
    fn year_extractor_on_array() {
        let t1 = civil_to_timestamp(2023, 6, 1, 0, 0, 0.0);
        let t2 = civil_to_timestamp(2024, 6, 1, 0, 0, 0.0);
        let arr = Value::DateTimeArray(vec![t1, t2]);
        let result = call1("year", arr).unwrap();
        match result {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[2, 1]);
                assert_eq!(m[[0, 0]], 2023.0);
                assert_eq!(m[[1, 0]], 2024.0);
            }
            _ => panic!("expected Matrix"),
        }
    }

    // ── Fix: isnat on non-datetime returns 0 ─────────────────────────────────

    #[test]
    fn isnat_on_scalar_returns_zero() {
        assert_eq!(
            call1("isnat", Value::Scalar(42.0)).unwrap(),
            Value::Scalar(0.0)
        );
        assert_eq!(
            call1("isnat", Value::Scalar(0.0)).unwrap(),
            Value::Scalar(0.0)
        );
    }

    #[test]
    fn isnat_on_duration_returns_zero() {
        assert_eq!(
            call1("isnat", Value::Duration(3600.0)).unwrap(),
            Value::Scalar(0.0)
        );
    }

    // ── Fix: fprintf %s accepts DateTime and Duration ─────────────────────────

    fn eval_str(src: &str) -> Result<Value, String> {
        let env = Env::new();
        eval_parse(src, &env)
    }

    #[test]
    fn sprintf_datetime_as_string() {
        let result = eval_str("sprintf('%s', datetime(2024, 6, 1))").unwrap();
        assert_eq!(result, Value::Str("2024-06-01 00:00:00".to_string()));
    }

    #[test]
    fn sprintf_duration_as_string() {
        let result = eval_str("sprintf('%s', hours(2))").unwrap();
        assert_eq!(result, Value::Str("02:00:00".to_string()));
    }

    #[test]
    fn sprintf_nat_as_string() {
        let result = eval_str("sprintf('%s', NaT)").unwrap();
        assert_eq!(result, Value::Str("NaT".to_string()));
    }

    // ── Fix: [datetime(...); datetime(...)] matrix literals ───────────────────

    #[test]
    fn matrix_literal_datetime_column() {
        let t1 = civil_to_timestamp(2024, 1, 1, 0, 0, 0.0);
        let t2 = civil_to_timestamp(2024, 1, 2, 0, 0, 0.0);
        let t3 = civil_to_timestamp(2024, 1, 3, 0, 0, 0.0);
        let result =
            eval_str("[datetime(2024,1,1); datetime(2024,1,2); datetime(2024,1,3)]").unwrap();
        match result {
            Value::DateTimeArray(v) => {
                assert_eq!(v.len(), 3);
                assert!((v[0] - t1).abs() < 1e-9);
                assert!((v[1] - t2).abs() < 1e-9);
                assert!((v[2] - t3).abs() < 1e-9);
            }
            _ => panic!("expected DateTimeArray, got {result:?}"),
        }
    }

    #[test]
    fn matrix_literal_datetime_row() {
        let result = eval_str("[datetime(2024,1,1), datetime(2024,1,2)]").unwrap();
        match result {
            Value::DateTimeArray(v) => assert_eq!(v.len(), 2),
            _ => panic!("expected DateTimeArray"),
        }
    }

    #[test]
    fn matrix_literal_single_datetime() {
        let result = eval_str("[datetime(2024,6,1)]").unwrap();
        match result {
            Value::DateTimeArray(v) => assert_eq!(v.len(), 1),
            _ => panic!("expected DateTimeArray"),
        }
    }

    #[test]
    fn matrix_literal_duration_column() {
        let result = eval_str("[hours(1); hours(2); hours(3)]").unwrap();
        match result {
            Value::DurationArray(v) => {
                assert_eq!(v.len(), 3);
                assert!((v[0] - 3600.0).abs() < 1e-9);
                assert!((v[1] - 7200.0).abs() < 1e-9);
                assert!((v[2] - 10800.0).abs() < 1e-9);
            }
            _ => panic!("expected DurationArray"),
        }
    }

    #[test]
    fn matrix_literal_duration_row() {
        let result = eval_str("[minutes(30), minutes(60)]").unwrap();
        match result {
            Value::DurationArray(v) => {
                assert_eq!(v.len(), 2);
                assert!((v[0] - 1800.0).abs() < 1e-9);
                assert!((v[1] - 3600.0).abs() < 1e-9);
            }
            _ => panic!("expected DurationArray"),
        }
    }

    #[test]
    fn matrix_literal_datetime_concat_array() {
        // [DateTimeArray; DateTime] should flatten
        let result =
            eval_str("[datetime(2024,1,1); datetime(2024,1,2); datetime(2024,1,3)]").unwrap();
        match result {
            Value::DateTimeArray(v) => assert_eq!(v.len(), 3),
            _ => panic!("expected DateTimeArray"),
        }
    }

    #[test]
    fn matrix_literal_mixed_type_error() {
        let result = eval_str("[datetime(2024,1,1); hours(1)]");
        assert!(
            result.is_err(),
            "expected error for mixed datetime/duration"
        );
    }

    #[test]
    fn matrix_literal_datetime_diff_roundtrip() {
        // Build a DateTimeArray via literal, then diff it
        let result =
            eval_str("diff([datetime(2024,1,1); datetime(2024,1,2); datetime(2024,1,3)])").unwrap();
        match result {
            Value::DurationArray(v) => {
                assert_eq!(v.len(), 2);
                assert!((v[0] - 86400.0).abs() < 1e-9);
                assert!((v[1] - 86400.0).abs() < 1e-9);
            }
            _ => panic!("expected DurationArray"),
        }
    }
}

// ── Phase 23 — Matrix utilities and set operations ────────────────────────────
mod phase23_tests {
    use super::*;
    use crate::env::Env;

    fn ep(src: &str) -> Result<Value, String> {
        let env = Env::new();
        eval_parse(src, &env)
    }

    fn mat(rows: usize, cols: usize, data: Vec<f64>) -> Value {
        Value::Matrix(ndarray::Array2::from_shape_vec((rows, cols), data).unwrap())
    }

    // ── 23a — triu / tril / repmat / kron ────────────────────────────────────

    #[test]
    fn triu_no_offset() {
        // triu([1 2 3; 4 5 6; 7 8 9]) → [1 2 3; 0 5 6; 0 0 9]
        let result = ep("triu([1 2 3; 4 5 6; 7 8 9])").unwrap();
        assert_eq!(
            result,
            mat(3, 3, vec![1.0, 2.0, 3.0, 0.0, 5.0, 6.0, 0.0, 0.0, 9.0])
        );
    }

    #[test]
    fn triu_positive_offset() {
        // triu(A, 1) — above main diagonal
        let result = ep("triu([1 2 3; 4 5 6; 7 8 9], 1)").unwrap();
        assert_eq!(
            result,
            mat(3, 3, vec![0.0, 2.0, 3.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0])
        );
    }

    #[test]
    fn triu_negative_offset() {
        // triu(A, -1) — includes one sub-diagonal
        let result = ep("triu([1 2 3; 4 5 6; 7 8 9], -1)").unwrap();
        assert_eq!(
            result,
            mat(3, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 0.0, 8.0, 9.0])
        );
    }

    #[test]
    fn tril_no_offset() {
        // tril([1 2 3; 4 5 6; 7 8 9]) → [1 0 0; 4 5 0; 7 8 9]
        let result = ep("tril([1 2 3; 4 5 6; 7 8 9])").unwrap();
        assert_eq!(
            result,
            mat(3, 3, vec![1.0, 0.0, 0.0, 4.0, 5.0, 0.0, 7.0, 8.0, 9.0])
        );
    }

    #[test]
    fn tril_negative_offset() {
        // tril(A, -1) — below main diagonal
        let result = ep("tril([1 2 3; 4 5 6; 7 8 9], -1)").unwrap();
        assert_eq!(
            result,
            mat(3, 3, vec![0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 7.0, 8.0, 0.0])
        );
    }

    #[test]
    fn tril_positive_offset() {
        // tril(A, 1) — includes one super-diagonal
        let result = ep("tril([1 2 3; 4 5 6; 7 8 9], 1)").unwrap();
        assert_eq!(
            result,
            mat(3, 3, vec![1.0, 2.0, 0.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])
        );
    }

    #[test]
    fn repmat_2x3() {
        // repmat([1 2; 3 4], 2, 3) → 4×6
        let result = ep("repmat([1 2; 3 4], 2, 3)").unwrap();
        #[rustfmt::skip]
        let expected = mat(4, 6, vec![
            1.0, 2.0, 1.0, 2.0, 1.0, 2.0,
            3.0, 4.0, 3.0, 4.0, 3.0, 4.0,
            1.0, 2.0, 1.0, 2.0, 1.0, 2.0,
            3.0, 4.0, 3.0, 4.0, 3.0, 4.0,
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn repmat_1x1() {
        let result = ep("repmat([5 6], 1, 1)").unwrap();
        assert_eq!(result, mat(1, 2, vec![5.0, 6.0]));
    }

    #[test]
    fn kron_identity_scaling() {
        // kron([1 0; 0 1], [1 2; 3 4]) → 4×4 block-diagonal
        let result = ep("kron([1 0; 0 1], [1 2; 3 4])").unwrap();
        #[rustfmt::skip]
        let expected = mat(4, 4, vec![
            1.0, 2.0, 0.0, 0.0,
            3.0, 4.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 2.0,
            0.0, 0.0, 3.0, 4.0,
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn kron_simple() {
        // kron([1 2], [3; 4]) → [3; 4; 6; 8]... actually [3 6; 4 8] — 2×2
        let result = ep("kron([1 2], [3; 4])").unwrap();
        assert_eq!(result, mat(2, 2, vec![3.0, 6.0, 4.0, 8.0]));
    }

    // ── 23b — cross / dot ─────────────────────────────────────────────────────

    #[test]
    fn cross_unit_vectors() {
        // cross([1 0 0], [0 1 0]) → [0 0 1]
        let result = ep("cross([1 0 0], [0 1 0])").unwrap();
        assert_eq!(result, mat(1, 3, vec![0.0, 0.0, 1.0]));
    }

    #[test]
    fn cross_general() {
        // cross([1 2 3], [4 5 6]) → [-3 6 -3]
        let result = ep("cross([1 2 3], [4 5 6])").unwrap();
        assert_eq!(result, mat(1, 3, vec![-3.0, 6.0, -3.0]));
    }

    #[test]
    fn cross_column_orientation() {
        // Result orientation matches first argument (column vector in → column vector out)
        let result = ep("cross([1; 0; 0], [0; 1; 0])").unwrap();
        assert_eq!(result, mat(3, 1, vec![0.0, 0.0, 1.0]));
    }

    #[test]
    fn cross_wrong_length_error() {
        assert!(ep("cross([1 2], [3 4])").is_err());
    }

    #[test]
    fn dot_basic() {
        // dot([1 2 3], [4 5 6]) → 32
        let result = ep("dot([1 2 3], [4 5 6])").unwrap();
        assert_eq!(result, Value::Scalar(32.0));
    }

    #[test]
    fn dot_length_mismatch_error() {
        assert!(ep("dot([1 2], [3 4 5])").is_err());
    }

    // ── 23c — intersect / union / setdiff / ismember ──────────────────────────

    #[test]
    fn intersect_basic() {
        let result = ep("intersect([1 3 5 7], [3 5 9])").unwrap();
        assert_eq!(result, mat(1, 2, vec![3.0, 5.0]));
    }

    #[test]
    fn intersect_empty() {
        let result = ep("intersect([1 2], [3 4])").unwrap();
        match result {
            Value::Matrix(m) => assert_eq!(m.ncols(), 0),
            _ => panic!("expected empty matrix"),
        }
    }

    #[test]
    fn intersect_nan_excluded() {
        // NaN ≠ NaN, so NaN is never a member
        let result = ep("intersect([1 nan 3], [nan 1 2])").unwrap();
        assert_eq!(result, mat(1, 1, vec![1.0]));
    }

    #[test]
    fn union_basic() {
        let result = ep("union([1 3 5], [3 5 7])").unwrap();
        assert_eq!(result, mat(1, 4, vec![1.0, 3.0, 5.0, 7.0]));
    }

    #[test]
    fn union_with_duplicates() {
        let result = ep("union([1 2 2 3], [2 3 4])").unwrap();
        assert_eq!(result, mat(1, 4, vec![1.0, 2.0, 3.0, 4.0]));
    }

    #[test]
    fn setdiff_basic() {
        let result = ep("setdiff([1 2 3 4 5], [2 4])").unwrap();
        assert_eq!(result, mat(1, 3, vec![1.0, 3.0, 5.0]));
    }

    #[test]
    fn setdiff_empty_result() {
        let result = ep("setdiff([1 2], [1 2 3])").unwrap();
        match result {
            Value::Matrix(m) => assert_eq!(m.ncols(), 0),
            _ => panic!("expected empty matrix"),
        }
    }

    #[test]
    fn ismember_scalar_found() {
        let result = ep("ismember(3, [1 2 3 4])").unwrap();
        assert_eq!(result, Value::Scalar(1.0));
    }

    #[test]
    fn ismember_scalar_not_found() {
        let result = ep("ismember(5, [1 2 3 4])").unwrap();
        assert_eq!(result, Value::Scalar(0.0));
    }

    #[test]
    fn ismember_vector() {
        let result = ep("ismember([1 6 3], [1 2 3 4])").unwrap();
        assert_eq!(result, mat(1, 3, vec![1.0, 0.0, 1.0]));
    }

    #[test]
    fn ismember_nan_is_false() {
        // NaN ≠ NaN in IEEE semantics
        let result = ep("ismember(nan, [nan])").unwrap();
        assert_eq!(result, Value::Scalar(0.0));
    }

    // ── 23d — sub2ind / ind2sub / repelem ─────────────────────────────────────

    #[test]
    fn sub2ind_scalar() {
        // sub2ind([3 4], 2, 3) → 8
        let result = ep("sub2ind([3 4], 2, 3)").unwrap();
        assert_eq!(result, Value::Scalar(8.0));
    }

    #[test]
    fn sub2ind_vector() {
        // sub2ind([3 4], [1 2], [1 3]) → [1 7]... wait:
        // (1-1)*3+1=1, (3-1)*3+2=8 — let's verify formula: (c-1)*rows + r
        // r=[1,2], c=[1,3]: (1-1)*3+1=1, (3-1)*3+2=8
        let result = ep("sub2ind([3 4], [1 2], [1 3])").unwrap();
        assert_eq!(result, mat(1, 2, vec![1.0, 8.0]));
    }

    #[test]
    fn ind2sub_scalar() {
        // ind2sub([3 4], 8) → r=2, c=3
        let result = ep("ind2sub([3 4], 8)").unwrap();
        match result {
            Value::Tuple(v) => {
                assert_eq!(v[0], Value::Scalar(2.0));
                assert_eq!(v[1], Value::Scalar(3.0));
            }
            _ => panic!("expected Tuple"),
        }
    }

    #[test]
    fn ind2sub_vector() {
        // ind2sub([3 4], [1 7]) → r=[1 1], c=[1 3]
        // idx=1: (1-1)%3+1=1, (1-1)/3+1=1
        // idx=7: (7-1)%3+1=1, (7-1)/3+1=3
        let result = ep("ind2sub([3 4], [1 7])").unwrap();
        match result {
            Value::Tuple(v) => {
                assert_eq!(v[0], mat(1, 2, vec![1.0, 1.0]));
                assert_eq!(v[1], mat(1, 2, vec![1.0, 3.0]));
            }
            _ => panic!("expected Tuple"),
        }
    }

    #[test]
    fn repelem_scalar_n() {
        // repelem([1 2 3], 3) → [1 1 1 2 2 2 3 3 3]
        let result = ep("repelem([1 2 3], 3)").unwrap();
        assert_eq!(
            result,
            mat(1, 9, vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0])
        );
    }

    #[test]
    fn repelem_vector_n() {
        // repelem([1 2 3], [2 1 3]) → [1 1 2 3 3 3]
        let result = ep("repelem([1 2 3], [2 1 3])").unwrap();
        assert_eq!(result, mat(1, 6, vec![1.0, 1.0, 2.0, 3.0, 3.0, 3.0]));
    }

    #[test]
    fn repelem_2d() {
        // repelem([1 2; 3 4], 2, 3) — each element repeated 2 rows × 3 cols
        let result = ep("repelem([1 2; 3 4], 2, 3)").unwrap();
        #[rustfmt::skip]
        let expected = mat(4, 6, vec![
            1.0, 1.0, 1.0, 2.0, 2.0, 2.0,
            1.0, 1.0, 1.0, 2.0, 2.0, 2.0,
            3.0, 3.0, 3.0, 4.0, 4.0, 4.0,
            3.0, 3.0, 3.0, 4.0, 4.0, 4.0,
        ]);
        assert_eq!(result, expected);
    }
}

// ============================================================================
// Phase 24 — Polynomial operations and interpolation
// ============================================================================

mod phase24_tests {
    use super::*;
    use crate::env::Env;

    fn ep(src: &str) -> Result<Value, String> {
        let env = Env::new();
        eval_parse(src, &env)
    }

    fn mat(rows: usize, cols: usize, data: Vec<f64>) -> Value {
        Value::Matrix(ndarray::Array2::from_shape_vec((rows, cols), data).unwrap())
    }

    /// Evaluate `src` with a pre-seeded variable `"p"` holding a 1×N polynomial row vector.
    /// Avoids the parser ambiguity where `[a -b c]` is parsed as `[a-b, c]` (binary minus).
    fn ep_p(src: &str, coeffs: &[f64]) -> Result<Value, String> {
        let mut env = Env::new();
        let n = coeffs.len();
        env.insert(
            "p".to_string(),
            Value::Matrix(ndarray::Array2::from_shape_vec((1, n), coeffs.to_vec()).unwrap()),
        );
        eval_parse(src, &env)
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    // ── polyval ──────────────────────────────────────────────────────────────

    #[test]
    fn polyval_root1() {
        // p(x) = x^2 - 3x + 2 = [1, -3, 2]; p(1) = 0
        assert_eq!(
            ep_p("polyval(p, 1)", &[1.0, -3.0, 2.0]).unwrap(),
            Value::Scalar(0.0)
        );
    }

    #[test]
    fn polyval_root2() {
        // p(2) = 4 - 6 + 2 = 0
        assert_eq!(
            ep_p("polyval(p, 2)", &[1.0, -3.0, 2.0]).unwrap(),
            Value::Scalar(0.0)
        );
    }

    #[test]
    fn polyval_non_root() {
        // p(3) = 9 - 9 + 2 = 2
        assert_eq!(
            ep_p("polyval(p, 3)", &[1.0, -3.0, 2.0]).unwrap(),
            Value::Scalar(2.0)
        );
    }

    #[test]
    fn polyval_vector_x() {
        // polyval([1 -3 2], [1 2 3]) → [0 0 2]
        assert_eq!(
            ep_p("polyval(p, [1 2 3])", &[1.0, -3.0, 2.0]).unwrap(),
            mat(1, 3, vec![0.0, 0.0, 2.0])
        );
    }

    #[test]
    fn polyval_constant_poly() {
        // polyval([5], 99) → 5
        assert_eq!(ep("polyval([5], 99)").unwrap(), Value::Scalar(5.0));
    }

    // ── polyfit ──────────────────────────────────────────────────────────────

    #[test]
    fn polyfit_quadratic() {
        // Fit x^2 + 1 through x=0:4, y=[1 2 5 10 17]
        let r = ep("polyfit([0 1 2 3 4], [1 2 5 10 17], 2)").unwrap();
        match r {
            Value::Matrix(m) => {
                assert!(approx(m[[0, 0]], 1.0), "a={}", m[[0, 0]]);
                assert!(approx(m[[0, 1]], 0.0), "b={}", m[[0, 1]]);
                assert!(approx(m[[0, 2]], 1.0), "c={}", m[[0, 2]]);
            }
            _ => panic!("expected Matrix"),
        }
    }

    #[test]
    fn polyfit_linear() {
        // Fit y = 2x + 1
        let r = ep("polyfit([0 1 2 3], [1 3 5 7], 1)").unwrap();
        match r {
            Value::Matrix(m) => {
                assert!(approx(m[[0, 0]], 2.0), "slope={}", m[[0, 0]]);
                assert!(approx(m[[0, 1]], 1.0), "intercept={}", m[[0, 1]]);
            }
            _ => panic!("expected Matrix"),
        }
    }

    #[test]
    fn polyfit_returns_row_vector() {
        let r = ep("polyfit([0 1 2], [1 2 5], 2)").unwrap();
        match r {
            Value::Matrix(m) => assert_eq!(m.nrows(), 1),
            _ => panic!("expected Matrix"),
        }
    }

    // ── roots ────────────────────────────────────────────────────────────────

    #[test]
    fn roots_two_real() {
        // roots([1 -3 2]) → [2; 1]  (coefficients pre-seeded to avoid parser ambiguity)
        let r = ep_p("roots(p)", &[1.0, -3.0, 2.0]).unwrap();
        match r {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[2, 1]);
                let mut vals = [m[[0, 0]], m[[1, 0]]];
                vals.sort_by(|a, b| b.partial_cmp(a).unwrap());
                assert!(approx(vals[0], 2.0), "root0={}", vals[0]);
                assert!(approx(vals[1], 1.0), "root1={}", vals[1]);
            }
            _ => panic!("expected real Matrix"),
        }
    }

    #[test]
    fn roots_complex_pair() {
        // roots([1 0 1]) → ±i (Cell of Complex)
        let r = ep("roots([1 0 1])").unwrap();
        match r {
            Value::Cell(vals) => {
                assert_eq!(vals.len(), 2);
                let mut ims: Vec<f64> = vals
                    .iter()
                    .map(|v| match v {
                        Value::Complex(_, im) => *im,
                        _ => panic!("expected Complex, got {v:?}"),
                    })
                    .collect();
                ims.sort_by(|a, b| b.partial_cmp(a).unwrap());
                assert!(approx(ims[0], 1.0), "im0={}", ims[0]);
                assert!(approx(ims[1], -1.0), "im1={}", ims[1]);
            }
            _ => panic!("expected Cell for complex roots"),
        }
    }

    #[test]
    fn roots_degree1() {
        // roots of [2, -6] → x = 3  (pre-seeded to avoid parser ambiguity)
        let r = ep_p("roots(p)", &[2.0, -6.0]).unwrap();
        match r {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[1, 1]);
                assert!(approx(m[[0, 0]], 3.0), "root={}", m[[0, 0]]);
            }
            _ => panic!("expected Matrix"),
        }
    }

    #[test]
    fn roots_constant_empty() {
        // roots([5]) → empty 0×1 column
        let r = ep("roots([5])").unwrap();
        match r {
            Value::Matrix(m) => assert_eq!(m.shape(), &[0, 1]),
            _ => panic!("expected empty Matrix"),
        }
    }

    // ── poly ─────────────────────────────────────────────────────────────────

    #[test]
    fn poly_from_roots_3() {
        // poly([1 2 3]) → [1 -6 11 -6]
        let r = ep("poly([1 2 3])").unwrap();
        match r {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 1);
                assert_eq!(m.ncols(), 4);
                assert!(approx(m[[0, 0]], 1.0));
                assert!(approx(m[[0, 1]], -6.0));
                assert!(approx(m[[0, 2]], 11.0));
                assert!(approx(m[[0, 3]], -6.0));
            }
            _ => panic!("expected Matrix"),
        }
    }

    #[test]
    fn poly_from_roots_2() {
        // poly([1 2]) → [1 -3 2]
        let r = ep("poly([1 2])").unwrap();
        match r {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 1);
                assert_eq!(m.ncols(), 3);
                assert!(approx(m[[0, 0]], 1.0));
                assert!(approx(m[[0, 1]], -3.0));
                assert!(approx(m[[0, 2]], 2.0));
            }
            _ => panic!("expected Matrix"),
        }
    }

    #[test]
    fn poly_char_poly_2x2() {
        // poly([1 2; 0 3]) char poly: (λ-1)(λ-3) = λ^2 - 4λ + 3
        let r = ep("poly([1 2; 0 3])").unwrap();
        match r {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 1);
                assert_eq!(m.ncols(), 3);
                assert!(approx(m[[0, 0]], 1.0));
                assert!(approx(m[[0, 1]], -4.0));
                assert!(approx(m[[0, 2]], 3.0));
            }
            _ => panic!("expected Matrix"),
        }
    }

    #[test]
    fn poly_scalar_root() {
        // poly(3.0) → [1 -3]
        let r = ep("poly(3)").unwrap();
        assert_eq!(r, mat(1, 2, vec![1.0, -3.0]));
    }

    // ── conv ─────────────────────────────────────────────────────────────────

    #[test]
    fn conv_basic() {
        // conv([1 2 3], [1 1]) → [1 3 5 3]
        assert_eq!(
            ep("conv([1 2 3], [1 1])").unwrap(),
            mat(1, 4, vec![1.0, 3.0, 5.0, 3.0])
        );
    }

    #[test]
    fn conv_poly_mul() {
        // conv([1 2], [3 4]) = [3 10 8]  (no negative elements needed)
        assert_eq!(
            ep("conv([1 2], [3 4])").unwrap(),
            mat(1, 3, vec![3.0, 10.0, 8.0])
        );
    }

    #[test]
    fn conv_single() {
        // conv([2], [3]) → [6]
        assert_eq!(ep("conv([2], [3])").unwrap(), mat(1, 1, vec![6.0]));
    }

    #[test]
    fn conv_length() {
        // length(conv(a,b)) == len(a)+len(b)-1
        let r = ep("conv([1 2 3 4], [5 6 7])").unwrap();
        match r {
            Value::Matrix(m) => assert_eq!(m.ncols(), 6),
            _ => panic!("expected Matrix"),
        }
    }

    // ── deconv ───────────────────────────────────────────────────────────────

    #[test]
    fn deconv_exact_division() {
        // deconv([1 3 5 3], [1 1]) → q=[1 2 3], r=[0 0 0 0]
        let r = ep("deconv([1 3 5 3], [1 1])").unwrap();
        match r {
            Value::Tuple(v) => {
                assert_eq!(v[0], mat(1, 3, vec![1.0, 2.0, 3.0]));
                assert_eq!(v[1], mat(1, 4, vec![0.0, 0.0, 0.0, 0.0]));
            }
            _ => panic!("expected Tuple"),
        }
    }

    #[test]
    fn deconv_with_remainder() {
        // deconv([1 2 3], [1 1]) → q=[1 1], r=[0 0 2]
        let r = ep("deconv([1 2 3], [1 1])").unwrap();
        match r {
            Value::Tuple(v) => {
                assert_eq!(v[0], mat(1, 2, vec![1.0, 1.0]));
                match &v[1] {
                    Value::Matrix(m) => {
                        assert!(approx(m[[0, 0]], 0.0));
                        assert!(approx(m[[0, 1]], 0.0));
                        assert!(approx(m[[0, 2]], 2.0));
                    }
                    _ => panic!("expected Matrix remainder"),
                }
            }
            _ => panic!("expected Tuple"),
        }
    }

    #[test]
    fn deconv_invariant() {
        // conv(q, b) + r == c (verified numerically)
        // c=[1 5 8 4], b=[1 2], q=[1 3 2], r=[0 0 0 0]
        let r = ep("deconv([1 5 8 4], [1 2])").unwrap();
        match r {
            Value::Tuple(v) => {
                assert_eq!(v[0], mat(1, 3, vec![1.0, 3.0, 2.0]));
                assert_eq!(v[1], mat(1, 4, vec![0.0, 0.0, 0.0, 0.0]));
            }
            _ => panic!("expected Tuple"),
        }
    }

    // ── interp1 ──────────────────────────────────────────────────────────────

    #[test]
    fn interp1_linear_scalar() {
        // interp1([0 1 2 3], [0 1 4 9], 1.5) → 2.5
        let r = ep("interp1([0 1 2 3], [0 1 4 9], 1.5)").unwrap();
        match r {
            Value::Scalar(v) => assert!(approx(v, 2.5), "got {v}"),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn interp1_linear_vector_xi() {
        // interp1([0 1 2 3], [0 1 4 9], [0.5 1.5 2.5]) → [0.5 2.5 6.5]
        let r = ep("interp1([0 1 2 3], [0 1 4 9], [0.5 1.5 2.5])").unwrap();
        match r {
            Value::Matrix(m) => {
                assert!(approx(m[[0, 0]], 0.5));
                assert!(approx(m[[0, 1]], 2.5));
                assert!(approx(m[[0, 2]], 6.5));
            }
            _ => panic!("expected Matrix"),
        }
    }

    #[test]
    fn interp1_at_knot() {
        // At an exact knot, must return exact y value
        let r = ep("interp1([0 1 2 3], [0 1 4 9], 3)").unwrap();
        match r {
            Value::Scalar(v) => assert!(approx(v, 9.0), "got {v}"),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn interp1_nearest() {
        // Tie (1.5 equidistant from 1 and 2): goes left
        let r = ep("interp1([0 1 2 3], [0 1 4 9], 1.5, 'nearest')").unwrap();
        match r {
            Value::Scalar(v) => assert!(approx(v, 1.0), "got {v}"),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn interp1_previous() {
        // Floor to left knot
        let r = ep("interp1([0 1 2 3], [0 1 4 9], 1.5, 'previous')").unwrap();
        match r {
            Value::Scalar(v) => assert!(approx(v, 1.0), "got {v}"),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn interp1_next() {
        // Ceil to right knot
        let r = ep("interp1([0 1 2 3], [0 1 4 9], 1.5, 'next')").unwrap();
        match r {
            Value::Scalar(v) => assert!(approx(v, 4.0), "got {v}"),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn interp1_extrapolation_nan() {
        // Out of range → NaN
        let r = ep("interp1([0 1 2], [0 1 4], -0.5)").unwrap();
        match r {
            Value::Scalar(v) => assert!(v.is_nan()),
            _ => panic!("expected scalar NaN"),
        }
    }

    #[test]
    fn interp1_previous_at_last_knot() {
        // 'previous' at x[end] should return y[end]
        let r = ep("interp1([0 1 2 3], [0 1 4 9], 3, 'previous')").unwrap();
        match r {
            Value::Scalar(v) => assert!(approx(v, 9.0), "got {v}"),
            _ => panic!("expected scalar"),
        }
    }

    // ── builtin_names ────────────────────────────────────────────────────────

    #[test]
    fn phase24_names_registered() {
        let names = builtin_names();
        for name in &[
            "polyval", "polyfit", "roots", "poly", "conv", "deconv", "interp1",
        ] {
            assert!(names.contains(name), "{name} missing from builtin_names");
        }
    }
}

// ============================================================================
// Phase 25 — Dynamic evaluation and timing
// ============================================================================

mod phase25_tests {
    use super::*;

    fn run_code(src: &str) -> crate::env::Env {
        use crate::eval::{Base, FormatMode};
        use crate::io::IoContext;
        use crate::parser::parse_stmts;
        crate::exec::init();
        let stmts = parse_stmts(src).expect("parse failed");
        let mut env = crate::env::Env::new();
        env.insert("ans".to_string(), Value::Scalar(0.0));
        let mut io = IoContext::new();
        crate::exec::exec_stmts(
            &stmts,
            &mut env,
            &mut io,
            &FormatMode::Short,
            Base::Dec,
            true,
        )
        .expect("exec failed");
        env
    }

    fn scalar_val(env: &crate::env::Env, name: &str) -> f64 {
        match env.get(name) {
            Some(Value::Scalar(x)) => *x,
            other => panic!("{name} not a scalar: {other:?}"),
        }
    }

    // ── 25a — eval (statement context — env mutations persist) ───────────────

    #[test]
    fn eval_basic_assignment() {
        let env = run_code("eval('x = sqrt(2)')");
        let x = scalar_val(&env, "x");
        assert!((x - 2.0_f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn eval_defines_variable_in_scope() {
        let env = run_code("eval('y = 42')");
        assert_eq!(scalar_val(&env, "y"), 42.0);
    }

    #[test]
    fn eval_dynamic_naming_in_loop() {
        // sprintf avoids the apostrophe/transpose ambiguity in string building
        let env = run_code("for k = 1:3\n  eval(sprintf('v%d = k*k', k))\nend");
        assert_eq!(scalar_val(&env, "v1"), 1.0);
        assert_eq!(scalar_val(&env, "v2"), 4.0);
        assert_eq!(scalar_val(&env, "v3"), 9.0);
    }

    #[test]
    fn eval_catch_triggered_on_error() {
        let env = run_code("eval('error(''intentional'')', 'caught = 1')");
        assert_eq!(scalar_val(&env, "caught"), 1.0);
    }

    #[test]
    fn eval_catch_not_triggered_on_success() {
        let env = run_code("eval('ok = 1', 'caught = 1')");
        assert_eq!(scalar_val(&env, "ok"), 1.0);
        assert!(env.get("caught").is_none());
    }

    #[test]
    fn eval_nested_eval() {
        // eval inside eval — depth guard allows it up to 64 levels
        let env = run_code("eval('eval(\"inner = 7\")')");
        assert_eq!(scalar_val(&env, "inner"), 7.0);
    }

    // ── 25a — eval (expression context — env mutations do not persist) ────────

    #[test]
    fn eval_expression_context_returns_ans() {
        // y = eval('2+2') captures the ans from the inner evaluation
        let env = run_code("y = eval('2 + 2')");
        assert_eq!(scalar_val(&env, "y"), 4.0);
    }

    // ── 25b — tic / toc ──────────────────────────────────────────────────────

    #[test]
    fn tic_toc_nonnegative() {
        let env = run_code("tic; t = toc");
        assert!(scalar_val(&env, "t") >= 0.0);
    }

    #[test]
    fn toc_multiple_calls_after_single_tic() {
        // Both calls should return non-negative and t2 >= t1
        let env = run_code("tic; t1 = toc; t2 = toc");
        let t1 = scalar_val(&env, "t1");
        let t2 = scalar_val(&env, "t2");
        assert!(t1 >= 0.0, "t1 negative: {t1}");
        assert!(t2 >= t1, "t2 < t1: {t2} < {t1}");
    }

    #[test]
    fn tic_inside_loop() {
        let env = run_code("tic()\nfor i = 1:100\n  x = i * i\nend\nt = toc()");
        assert!(scalar_val(&env, "t") >= 0.0);
    }

    // ── builtin_names ────────────────────────────────────────────────────────

    #[test]
    fn phase25_names_registered() {
        let names = builtin_names();
        for name in &["eval", "tic", "toc"] {
            assert!(names.contains(name), "{name} missing from builtin_names");
        }
    }
}
