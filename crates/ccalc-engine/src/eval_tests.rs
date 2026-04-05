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
