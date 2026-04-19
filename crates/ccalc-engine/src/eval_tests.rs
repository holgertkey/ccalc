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
            assert!(parts.len() >= 2, "should include root and at least one subdir");
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
