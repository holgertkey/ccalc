use super::*;
use crate::env::{Env, Value};
use crate::eval::eval;

fn eval_s(expr: &Expr, env: &Env) -> f64 {
    match eval(expr, env).unwrap() {
        Value::Scalar(n) => n,
        _ => panic!("expected scalar"),
    }
}

fn calc(input: &str) -> f64 {
    let env = Env::new();
    match parse(input).unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval_s(&expr, &env),
    }
}

fn calc_with_ans(input: &str, ans: f64) -> f64 {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(ans));
    match parse(input).unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval_s(&expr, &env),
    }
}

fn calc_with_var(input: &str, name: &str, val: f64) -> f64 {
    let mut env = Env::new();
    env.insert(name.to_string(), Value::Scalar(val));
    match parse(input).unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval_s(&expr, &env),
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
            assert_eq!(eval_s(&expr, &Env::new()), 5.0);
        }
        _ => panic!("expected Stmt::Assign"),
    }
}

#[test]
fn test_assignment_complex_expr() {
    match parse("result = 2 ^ 10 + 1").unwrap() {
        Stmt::Assign(name, expr) => {
            assert_eq!(name, "result");
            assert_eq!(eval_s(&expr, &Env::new()), 1025.0);
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

// --- Multi-argument functions ---

#[test]
fn test_fn_atan2() {
    assert!((calc("atan2(1, 1)") - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    assert!((calc("atan2(1, 0)") - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
}

#[test]
fn test_fn_mod() {
    assert_eq!(calc("mod(10, 3)"), 1.0);
    assert_eq!(calc("mod(-1, 3)"), 2.0);
}

#[test]
fn test_fn_rem() {
    assert_eq!(calc("rem(10, 3)"), 1.0);
    assert_eq!(calc("rem(-1, 3)"), -1.0);
}

#[test]
fn test_fn_max_min() {
    assert_eq!(calc("max(3, 7)"), 7.0);
    assert_eq!(calc("min(3, 7)"), 3.0);
}

#[test]
fn test_fn_hypot() {
    assert_eq!(calc("hypot(3, 4)"), 5.0);
}

#[test]
fn test_fn_log_two_arg() {
    assert!((calc("log(8, 2)") - 3.0).abs() < 1e-10);
    assert!((calc("log(100, 10)") - 2.0).abs() < 1e-10);
}

#[test]
fn test_fn_asin_acos_atan() {
    assert!((calc("asin(1)") - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    assert!(calc("acos(1)").abs() < 1e-10);
    assert!((calc("atan(1)") - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
}

#[test]
fn test_fn_two_arg_with_exprs() {
    // Arguments can be arbitrary expressions
    assert_eq!(calc("max(1 + 1, 3)"), 3.0);
    assert!((calc("hypot(2 + 1, 2 ^ 2)") - 5.0).abs() < 1e-10);
}

#[test]
fn test_fn_empty_arg_still_uses_ans() {
    assert_eq!(calc_with_ans("sqrt()", 4.0), 2.0);
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

// --- Matrix literal tests ---

#[test]
fn test_matrix_empty() {
    match parse("[]").unwrap() {
        Stmt::Expr(Expr::Matrix(rows)) => assert!(rows.is_empty()),
        _ => panic!("expected empty matrix"),
    }
}

#[test]
fn test_matrix_row_vector_commas() {
    // [1, 2, 3]
    match parse("[1, 2, 3]").unwrap() {
        Stmt::Expr(Expr::Matrix(rows)) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 3);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_matrix_row_vector_spaces() {
    // [1 2 3] — space-separated
    match parse("[1 2 3]").unwrap() {
        Stmt::Expr(Expr::Matrix(rows)) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 3);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_matrix_col_vector() {
    // [1; 2; 3]
    match parse("[1; 2; 3]").unwrap() {
        Stmt::Expr(Expr::Matrix(rows)) => {
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0].len(), 1);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_matrix_2x2() {
    // [1 2; 3 4]
    match parse("[1 2; 3 4]").unwrap() {
        Stmt::Expr(Expr::Matrix(rows)) => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[1].len(), 2);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_matrix_assign() {
    match parse("A = [1 2; 3 4]").unwrap() {
        Stmt::Assign(name, Expr::Matrix(rows)) => {
            assert_eq!(name, "A");
            assert_eq!(rows.len(), 2);
        }
        _ => panic!("expected assign matrix"),
    }
}

#[test]
fn test_matrix_with_expressions() {
    // [1+1, 2*3]
    match parse("[1+1, 2*3]").unwrap() {
        Stmt::Expr(Expr::Matrix(rows)) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 2);
            let env = Env::new();
            assert_eq!(eval_s(&rows[0][0], &env), 2.0);
            assert_eq!(eval_s(&rows[0][1], &env), 6.0);
        }
        _ => panic!("expected matrix"),
    }
}

// --- Phase 4: element-wise operators and transpose ---

#[test]
fn test_elem_mul_scalars() {
    assert_eq!(calc("3 .* 4"), 12.0);
}

#[test]
fn test_elem_div_scalars() {
    assert_eq!(calc("8 ./ 2"), 4.0);
}

#[test]
fn test_elem_pow_scalars() {
    assert_eq!(calc("2 .^ 10"), 1024.0);
}

#[test]
fn test_elem_pow_right_associative() {
    // 2 .^ 3 .^ 2 == 2 .^ (3 .^ 2) == 2 .^ 9 == 512
    assert_eq!(calc("2 .^ 3 .^ 2"), 512.0);
}

#[test]
fn test_elem_operators_precedence() {
    // 2 + 3 .* 4 == 2 + 12 == 14 (.* same level as *)
    assert_eq!(calc("2 + 3 .* 4"), 14.0);
    assert_eq!(calc("2 .* 3 + 4"), 10.0);
}

#[test]
fn test_number_dot_elem_op() {
    // 3.*4 — '.' should NOT be absorbed into the number 3.
    assert_eq!(calc("3.*4"), 12.0);
    assert_eq!(calc("3./2"), 1.5);
    assert_eq!(calc("2.^3"), 8.0);
}

#[test]
fn test_transpose_parse() {
    // A' should produce Transpose(Var("A"))
    match parse("A'").unwrap() {
        Stmt::Expr(Expr::Transpose(inner)) => {
            assert!(matches!(*inner, Expr::Var(ref n) if n == "A"));
        }
        _ => panic!("expected Transpose"),
    }
}

#[test]
fn test_transpose_double() {
    // A'' should produce Transpose(Transpose(Var("A")))
    match parse("A''").unwrap() {
        Stmt::Expr(Expr::Transpose(inner)) => {
            assert!(matches!(*inner, Expr::Transpose(_)));
        }
        _ => panic!("expected double transpose"),
    }
}

#[test]
fn test_transpose_eval() {
    // [1 2 3]' → column vector, eval to 3x1
    let env = Env::new();
    match parse("[1 2 3]'").unwrap() {
        Stmt::Expr(expr) => match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[3, 1]);
                assert_eq!(m[[0, 0]], 1.0);
                assert_eq!(m[[1, 0]], 2.0);
                assert_eq!(m[[2, 0]], 3.0);
            }
            _ => panic!("expected matrix"),
        },
        _ => panic!("expected expr"),
    }
}

#[test]
fn test_transpose_matrix_mul() {
    // v' * v where v = [1;2;3] → scalar 14
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0], [2.0], [3.0]]));
    match parse("v' * v").unwrap() {
        Stmt::Expr(expr) => match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[1, 1]);
                assert_eq!(m[[0, 0]], 14.0);
            }
            _ => panic!("expected matrix"),
        },
        _ => panic!("expected expr"),
    }
}

#[test]
fn test_is_partial_elem_ops() {
    assert!(is_partial(".* 2"));
    assert!(is_partial("./ 2"));
    assert!(is_partial(".^ 2"));
    assert!(!is_partial(".5"));
    assert!(!is_partial(". "));
}

// --- Phase 5: Range operator ---

fn calc_vec(input: &str) -> Vec<f64> {
    let env = Env::new();
    match parse(input).unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => match eval(&expr, &env).unwrap() {
            Value::Matrix(m) => m.iter().copied().collect(),
            _ => panic!("expected matrix"),
        },
    }
}

#[test]
fn test_range_simple() {
    assert_eq!(calc_vec("1:5"), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn test_range_single_element() {
    assert_eq!(calc_vec("3:3"), vec![3.0]);
}

#[test]
fn test_range_with_step() {
    assert_eq!(calc_vec("1:2:9"), vec![1.0, 3.0, 5.0, 7.0, 9.0]);
}

#[test]
fn test_range_float_step() {
    let v = calc_vec("0:0.5:2");
    assert_eq!(v.len(), 5);
    assert!((v[0] - 0.0).abs() < 1e-10);
    assert!((v[1] - 0.5).abs() < 1e-10);
    assert!((v[2] - 1.0).abs() < 1e-10);
    assert!((v[3] - 1.5).abs() < 1e-10);
    assert!((v[4] - 2.0).abs() < 1e-10);
}

#[test]
fn test_range_negative_step() {
    assert_eq!(calc_vec("5:-1:1"), vec![5.0, 4.0, 3.0, 2.0, 1.0]);
}

#[test]
fn test_range_empty_wrong_direction() {
    assert_eq!(calc_vec("5:1"), vec![]);
}

#[test]
fn test_range_arithmetic_in_bounds() {
    // 1+1:2+2 = 2:4
    assert_eq!(calc_vec("1+1:2+2"), vec![2.0, 3.0, 4.0]);
}

#[test]
fn test_range_inside_brackets() {
    // [1:4] == [1 2 3 4]
    assert_eq!(calc_vec("[1:4]"), vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_range_inside_brackets_with_extras() {
    // [0, 1:3, 10] == [0 1 2 3 10]
    assert_eq!(calc_vec("[0, 1:3, 10]"), vec![0.0, 1.0, 2.0, 3.0, 10.0]);
}

#[test]
fn test_range_step_inside_brackets() {
    // [1:2:7] == [1 3 5 7]
    assert_eq!(calc_vec("[1:2:7]"), vec![1.0, 3.0, 5.0, 7.0]);
}

#[test]
fn test_range_zero_step_is_error() {
    let env = Env::new();
    match parse("1:0:5").unwrap() {
        Stmt::Expr(expr) => assert!(eval(&expr, &env).is_err()),
        _ => panic!("expected expr"),
    }
}

#[test]
fn test_range_assign() {
    let env = Env::new();
    match parse("v = 1:3").unwrap() {
        Stmt::Assign(name, expr) => {
            assert_eq!(name, "v");
            match eval(&expr, &env).unwrap() {
                Value::Matrix(m) => {
                    assert_eq!(m.shape(), &[1, 3]);
                    assert_eq!(m[[0, 0]], 1.0);
                    assert_eq!(m[[0, 2]], 3.0);
                }
                _ => panic!("expected matrix"),
            }
        }
        _ => panic!("expected assign"),
    }
}

#[test]
fn test_linspace_basic() {
    let v = calc_vec("linspace(0, 1, 3)");
    assert_eq!(v.len(), 3);
    assert!((v[0] - 0.0).abs() < 1e-10);
    assert!((v[1] - 0.5).abs() < 1e-10);
    assert!((v[2] - 1.0).abs() < 1e-10);
}

#[test]
fn test_linspace_integers() {
    assert_eq!(calc_vec("linspace(1, 5, 5)"), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn test_linspace_single() {
    // linspace(a, b, 1) returns [b] (MATLAB convention)
    assert_eq!(calc_vec("linspace(3, 7, 1)"), vec![7.0]);
}

#[test]
fn test_linspace_empty() {
    assert_eq!(calc_vec("linspace(0, 1, 0)"), vec![]);
}

// --- Phase 6: Indexing ---

fn index_env() -> Env {
    use ndarray::array;
    let mut env = Env::new();
    // v = [10 20 30 40 50]  (1×5 row vector)
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[10.0, 20.0, 30.0, 40.0, 50.0]]),
    );
    // A = [1 2 3; 4 5 6; 7 8 9]  (3×3)
    env.insert(
        "A".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]),
    );
    // x = 42  (scalar in env — can be indexed)
    env.insert("x".to_string(), Value::Scalar(42.0));
    env
}

fn eval_with(input: &str, env: &Env) -> Value {
    match parse(input).unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, env).unwrap(),
    }
}

fn try_eval_with(input: &str, env: &Env) -> Result<Value, String> {
    match parse(input)? {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, env),
    }
}

fn scalar_with(input: &str, env: &Env) -> f64 {
    match eval_with(input, env) {
        Value::Scalar(n) => n,
        _ => panic!("expected scalar"),
    }
}

fn vec_with(input: &str, env: &Env) -> Vec<f64> {
    match eval_with(input, env) {
        Value::Matrix(m) => m.iter().copied().collect(),
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_index_vector_scalar() {
    let env = index_env();
    assert_eq!(scalar_with("v(1)", &env), 10.0);
    assert_eq!(scalar_with("v(3)", &env), 30.0);
    assert_eq!(scalar_with("v(5)", &env), 50.0);
}

#[test]
fn test_index_vector_range() {
    let env = index_env();
    assert_eq!(vec_with("v(2:4)", &env), vec![20.0, 30.0, 40.0]);
    assert_eq!(vec_with("v(1:3)", &env), vec![10.0, 20.0, 30.0]);
}

#[test]
fn test_index_vector_colon() {
    // v(:) returns column vector
    let env = index_env();
    let m = match eval_with("v(:)", &env) {
        Value::Matrix(m) => m,
        _ => panic!("expected matrix"),
    };
    assert_eq!(m.shape(), &[5, 1]);
    assert_eq!(m[[0, 0]], 10.0);
    assert_eq!(m[[4, 0]], 50.0);
}

#[test]
fn test_index_matrix_scalar() {
    let env = index_env();
    assert_eq!(scalar_with("A(1,1)", &env), 1.0);
    assert_eq!(scalar_with("A(2,3)", &env), 6.0);
    assert_eq!(scalar_with("A(3,3)", &env), 9.0);
}

#[test]
fn test_index_matrix_colon_row() {
    // A(1,:) → row 1 as 1×3
    let env = index_env();
    let m = match eval_with("A(1,:)", &env) {
        Value::Matrix(m) => m,
        _ => panic!("expected matrix"),
    };
    assert_eq!(m.shape(), &[1, 3]);
    assert_eq!(m[[0, 0]], 1.0);
    assert_eq!(m[[0, 1]], 2.0);
    assert_eq!(m[[0, 2]], 3.0);
}

#[test]
fn test_index_matrix_colon_col() {
    // A(:,2) → column 2 as 3×1
    let env = index_env();
    let m = match eval_with("A(:,2)", &env) {
        Value::Matrix(m) => m,
        _ => panic!("expected matrix"),
    };
    assert_eq!(m.shape(), &[3, 1]);
    assert_eq!(m[[0, 0]], 2.0);
    assert_eq!(m[[1, 0]], 5.0);
    assert_eq!(m[[2, 0]], 8.0);
}

#[test]
fn test_index_submatrix() {
    // A(1:2, 2:3) → [2 3; 5 6]
    let env = index_env();
    let m = match eval_with("A(1:2, 2:3)", &env) {
        Value::Matrix(m) => m,
        _ => panic!("expected matrix"),
    };
    assert_eq!(m.shape(), &[2, 2]);
    assert_eq!(m[[0, 0]], 2.0);
    assert_eq!(m[[0, 1]], 3.0);
    assert_eq!(m[[1, 0]], 5.0);
    assert_eq!(m[[1, 1]], 6.0);
}

#[test]
fn test_index_scalar_in_env() {
    // Scalar in env can be indexed as 1×1
    let env = index_env();
    assert_eq!(scalar_with("x(1)", &env), 42.0);
    assert_eq!(scalar_with("x(1,1)", &env), 42.0);
}

#[test]
fn test_index_out_of_bounds_error() {
    let env = index_env();
    assert!(try_eval_with("v(6)", &env).is_err());
    assert!(try_eval_with("A(4,1)", &env).is_err());
}

#[test]
fn test_function_call_not_affected() {
    // zeros, ones, eye are not in env → treated as function calls
    let env = Env::new();
    assert!(matches!(eval_with("zeros(2,2)", &env), Value::Matrix(_)));
    assert!(matches!(eval_with("eye(3)", &env), Value::Matrix(_)));
}

#[test]
fn test_index_with_expr_arg() {
    // A(1+1, 3) == A(2,3) == 6
    let env = index_env();
    assert_eq!(scalar_with("A(1+1, 3)", &env), 6.0);
}

#[test]
fn test_colon_standalone_is_error() {
    // Bare ':' as a standalone expression (not inside an index) is an error at eval time.
    // parse(":") fails at the parser level (unexpected token).
    // If it somehow reached eval, Expr::Colon returns an error.
    let env = Env::new();
    assert!(try_eval_with(":", &env).is_err());
}

// --- Phase 7: Comparison and logical operators ---

#[test]
fn test_comparison_eq() {
    assert_eq!(calc("3 == 3"), 1.0);
    assert_eq!(calc("3 == 4"), 0.0);
}

#[test]
fn test_comparison_noteq() {
    assert_eq!(calc("3 ~= 4"), 1.0);
    assert_eq!(calc("3 ~= 3"), 0.0);
}

#[test]
fn test_comparison_lt_gt() {
    assert_eq!(calc("2 < 3"), 1.0);
    assert_eq!(calc("3 < 2"), 0.0);
    assert_eq!(calc("3 > 2"), 1.0);
    assert_eq!(calc("2 > 3"), 0.0);
}

#[test]
fn test_comparison_le_ge() {
    assert_eq!(calc("3 <= 3"), 1.0);
    assert_eq!(calc("4 <= 3"), 0.0);
    assert_eq!(calc("3 >= 3"), 1.0);
    assert_eq!(calc("2 >= 3"), 0.0);
}

#[test]
fn test_comparison_with_arithmetic() {
    // comparison has lower precedence than +/-
    assert_eq!(calc("1 + 1 == 2"), 1.0);
    assert_eq!(calc("2 * 3 > 5"), 1.0);
}

#[test]
fn test_logical_not_scalar() {
    assert_eq!(calc("~0"), 1.0);
    assert_eq!(calc("~1"), 0.0);
    assert_eq!(calc("~5"), 0.0);
}

#[test]
fn test_logical_not_of_comparison() {
    assert_eq!(calc("~(3 == 3)"), 0.0);
    assert_eq!(calc("~(3 == 4)"), 1.0);
}

#[test]
fn test_logical_and() {
    assert_eq!(calc("1 && 1"), 1.0);
    assert_eq!(calc("1 && 0"), 0.0);
    assert_eq!(calc("0 && 1"), 0.0);
    assert_eq!(calc("0 && 0"), 0.0);
}

#[test]
fn test_logical_or() {
    assert_eq!(calc("1 || 0"), 1.0);
    assert_eq!(calc("0 || 1"), 1.0);
    assert_eq!(calc("0 || 0"), 0.0);
    assert_eq!(calc("1 || 1"), 1.0);
}

#[test]
fn test_logical_precedence() {
    // '&&' binds tighter than '||'
    assert_eq!(calc("1 || 0 && 0"), 1.0); // 1 || (0 && 0) = 1 || 0 = 1
    assert_eq!(calc("0 && 0 || 1"), 1.0); // (0 && 0) || 1 = 0 || 1 = 1
}

#[test]
fn test_comparison_lower_than_arithmetic() {
    // `a + b > c` means `(a+b) > c`
    assert_eq!(calc("2 + 3 > 4"), 1.0);
    assert_eq!(calc("1 + 1 < 1"), 0.0);
}

#[test]
fn test_logical_combined() {
    // (2 > 1) && (3 > 2) → 1 && 1 = 1
    assert_eq!(calc("2 > 1 && 3 > 2"), 1.0);
    // (2 > 3) || (1 < 2) → 0 || 1 = 1
    assert_eq!(calc("2 > 3 || 1 < 2"), 1.0);
}

#[test]
fn test_comparison_matrix_scalar() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0, 4.0, 5.0]]),
    );
    // v > 3  → [0 0 0 1 1]
    let result = match eval_with("v > 3", &env) {
        Value::Matrix(m) => m.into_raw_vec(),
        _ => panic!("expected matrix"),
    };
    assert_eq!(result, vec![0.0, 0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn test_comparison_matrix_matrix() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("a".to_string(), Value::Matrix(array![[1.0, 5.0, 3.0]]));
    env.insert("b".to_string(), Value::Matrix(array![[2.0, 4.0, 3.0]]));
    // a == b → [0 0 1]
    let result = match eval_with("a == b", &env) {
        Value::Matrix(m) => m.into_raw_vec(),
        _ => panic!("expected matrix"),
    };
    assert_eq!(result, vec![0.0, 0.0, 1.0]);
}

#[test]
fn test_not_matrix() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[0.0, 1.0, 0.0, 5.0]]));
    let result = match eval_with("~v", &env) {
        Value::Matrix(m) => m.into_raw_vec(),
        _ => panic!("expected matrix"),
    };
    assert_eq!(result, vec![1.0, 0.0, 1.0, 0.0]);
}

#[test]
fn test_double_eq_not_assignment() {
    // `3 == 3` must not be parsed as assignment
    assert_eq!(calc("3 == 3"), 1.0);
}

#[test]
fn test_single_eq_in_expression_is_error() {
    // A bare `=` (not `==`) inside an expression is a tokenizer error
    assert!(parse("3 = 3").is_err());
}

// --- Bitwise functions ---

#[test]
fn test_bitand() {
    assert_eq!(calc("bitand(0xFF, 0x0F)"), 15.0);
    assert_eq!(calc("bitand(0b1111, 0b1010)"), 10.0);
    assert_eq!(calc("bitand(255, 0)"), 0.0);
    assert_eq!(calc("bitand(255, 255)"), 255.0);
}

#[test]
fn test_bitor() {
    assert_eq!(calc("bitor(0b1010, 0b0101)"), 15.0);
    assert_eq!(calc("bitor(0, 255)"), 255.0);
    assert_eq!(calc("bitor(0xFF00, 0x00FF)"), 65535.0);
}

#[test]
fn test_bitxor() {
    assert_eq!(calc("bitxor(0xFF, 0x0F)"), 240.0);
    assert_eq!(calc("bitxor(0b1010, 0b1010)"), 0.0); // XOR with itself = 0
    assert_eq!(calc("bitxor(0, 255)"), 255.0);
}

#[test]
fn test_bitshift_left() {
    assert_eq!(calc("bitshift(1, 8)"), 256.0);
    assert_eq!(calc("bitshift(1, 0)"), 1.0);
    assert_eq!(calc("bitshift(3, 4)"), 48.0);
}

#[test]
fn test_bitshift_right() {
    assert_eq!(calc("bitshift(256, -4)"), 16.0);
    assert_eq!(calc("bitshift(255, -4)"), 15.0);
    assert_eq!(calc("bitshift(1, -1)"), 0.0);
}

#[test]
fn test_bitshift_overflow() {
    // Shift of 64 or more returns 0
    assert_eq!(calc("bitshift(1, 64)"), 0.0);
    assert_eq!(calc("bitshift(255, -64)"), 0.0);
}

#[test]
fn test_bitnot_default_32bit() {
    // bitnot(5) = ~5 within 32-bit window = 4294967290
    assert_eq!(calc("bitnot(5)"), 4294967290.0);
    // bitnot(0) = 0xFFFFFFFF
    assert_eq!(calc("bitnot(0)"), 4294967295.0);
}

#[test]
fn test_bitnot_explicit_width() {
    // bitnot(5, 8): ~5 within 8 bits = 0b11111010 = 250
    assert_eq!(calc("bitnot(5, 8)"), 250.0);
    // bitnot(0, 4): ~0 within 4 bits = 0b1111 = 15
    assert_eq!(calc("bitnot(0, 4)"), 15.0);
    // bitnot(15, 4): ~15 within 4 bits = 0
    assert_eq!(calc("bitnot(15, 4)"), 0.0);
    // bitnot(0, 32) = 0xFFFFFFFF = 4294967295
    assert_eq!(calc("bitnot(0, 32)"), 4294967295.0);
}

#[test]
fn test_bitwise_with_hex_literals() {
    // Natural use: combine with hex/bin input literals
    assert_eq!(calc("bitor(0xFF00, 0x00FF)"), 65535.0);
    assert_eq!(calc("bitand(0xDEAD, 0xFF00)"), 56832.0); // 0xDE00
    assert_eq!(calc("bitxor(0xFFFF, 0x0F0F)"), 61680.0); // 0xF0F0
}

#[test]
fn test_bitshift_in_expression() {
    // Shift result used in further arithmetic
    assert_eq!(calc("bitshift(1, 4) + bitshift(1, 0)"), 17.0); // 16 + 1
    // Building a bitmask: (1 << n) - 1
    assert_eq!(calc("bitshift(1, 8) - 1"), 255.0);
}

#[test]
fn test_bitwise_error_negative() {
    assert!(parse("bitand(-1, 5)").is_ok()); // parses OK
    // eval must fail for negative args
    let env = Env::new();
    assert!(match parse("bitand(-1, 5)").unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).is_err(),
    });
}

#[test]
fn test_bitwise_error_noninteger() {
    let env = Env::new();
    assert!(match parse("bitand(1.5, 2)").unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).is_err(),
    });
}

#[test]
fn test_bitnot_error_invalid_width() {
    let env = Env::new();
    assert!(match parse("bitnot(5, 0)").unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).is_err(),
    });
    assert!(match parse("bitnot(5, 54)").unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).is_err(),
    });
}

// --- Phase 7.5a: Special constants ---

#[test]
fn test_isnan_scalar() {
    // nan/inf are parser-level constants, no env setup needed
    assert_eq!(calc("isnan(nan)"), 1.0);
    assert_eq!(calc("isnan(0)"), 0.0);
    assert_eq!(calc("isnan(1)"), 0.0);
}

#[test]
fn test_isinf_scalar() {
    assert_eq!(calc("isinf(inf)"), 1.0);
    assert_eq!(calc("isinf(0)"), 0.0);
    assert_eq!(calc("isinf(-inf)"), 1.0);
}

#[test]
fn test_isfinite_scalar() {
    assert_eq!(calc("isfinite(1)"), 1.0);
    assert_eq!(calc("isfinite(inf)"), 0.0);
    assert_eq!(calc("isfinite(nan)"), 0.0);
}

#[test]
fn test_nan_inf_constants() {
    // nan/inf parse as constants without being in env
    assert!(calc("nan").is_nan());
    assert_eq!(calc("inf"), f64::INFINITY);
    assert_eq!(calc("-inf"), f64::NEG_INFINITY);
    // NaN arithmetic
    assert!(calc("nan + 5").is_nan());
    // nan == nan is false (IEEE 754)
    assert_eq!(calc("nan == nan"), 0.0);
}

#[test]
fn test_nan_constructor() {
    match eval_with("nan(2, 3)", &Env::new()) {
        Value::Matrix(m) => {
            assert_eq!(m.nrows(), 2);
            assert_eq!(m.ncols(), 3);
            assert!(m.iter().all(|x| x.is_nan()));
        }
        _ => panic!("expected matrix"),
    }
    match eval_with("nan(2)", &Env::new()) {
        Value::Matrix(m) => {
            assert_eq!(m.nrows(), 2);
            assert_eq!(m.ncols(), 2);
        }
        _ => panic!("expected matrix"),
    }
}

// --- Phase 7.5b: Vector reductions ---

#[test]
fn test_sum_vector() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0, 4.0]]));
    assert_eq!(eval_with("sum(v)", &env).as_scalar().unwrap(), 10.0);
    // Column vector
    env.insert("c".to_string(), Value::Matrix(array![[1.0], [2.0], [3.0]]));
    assert_eq!(eval_with("sum(c)", &env).as_scalar().unwrap(), 6.0);
}

#[test]
fn test_sum_matrix_columnwise() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "m".to_string(),
        Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]),
    );
    match eval_with("sum(m)", &env) {
        Value::Matrix(r) => {
            assert_eq!(r.nrows(), 1);
            assert_eq!(r.ncols(), 2);
            assert_eq!(r[[0, 0]], 4.0); // col 0: 1+3
            assert_eq!(r[[0, 1]], 6.0); // col 1: 2+4
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_prod_vector() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0, 4.0]]));
    assert_eq!(eval_with("prod(v)", &env).as_scalar().unwrap(), 24.0);
}

#[test]
fn test_any_all() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[0.0, 1.0, 0.0]]));
    env.insert("w".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0]]));
    env.insert("z".to_string(), Value::Matrix(array![[0.0, 0.0, 0.0]]));
    assert_eq!(eval_with("any(v)", &env).as_scalar().unwrap(), 1.0);
    assert_eq!(eval_with("any(z)", &env).as_scalar().unwrap(), 0.0);
    assert_eq!(eval_with("all(v)", &env).as_scalar().unwrap(), 0.0);
    assert_eq!(eval_with("all(w)", &env).as_scalar().unwrap(), 1.0);
}

#[test]
fn test_mean_vector() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0, 4.0]]));
    assert_eq!(eval_with("mean(v)", &env).as_scalar().unwrap(), 2.5);
}

#[test]
fn test_min_max_one_arg() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[3.0, 1.0, 4.0, 1.0, 5.0]]),
    );
    assert_eq!(eval_with("min(v)", &env).as_scalar().unwrap(), 1.0);
    assert_eq!(eval_with("max(v)", &env).as_scalar().unwrap(), 5.0);
}

#[test]
fn test_norm_l2() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[3.0, 4.0]]));
    assert!((eval_with("norm(v)", &env).as_scalar().unwrap() - 5.0).abs() < 1e-10);
}

#[test]
fn test_norm_lp() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0]]));
    // L1 norm = 1+2+3 = 6
    assert!((eval_with("norm(v, 1)", &env).as_scalar().unwrap() - 6.0).abs() < 1e-10);
}

#[test]
fn test_cumsum_vector() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0, 4.0]]));
    match eval_with("cumsum(v)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![1.0, 3.0, 6.0, 10.0]);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_cumprod_vector() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0, 4.0]]));
    match eval_with("cumprod(v)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![1.0, 2.0, 6.0, 24.0]);
        }
        _ => panic!("expected matrix"),
    }
}

// --- Phase 7.5d: Sort, reshape, flip, find, unique ---

#[test]
fn test_sort_ascending() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[3.0, 1.0, 4.0, 1.0, 5.0, 9.0]]),
    );
    match eval_with("sort(v)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![1.0, 1.0, 3.0, 4.0, 5.0, 9.0]);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_reshape() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]]),
    );
    match eval_with("reshape(v, 2, 3)", &env) {
        Value::Matrix(r) => {
            assert_eq!(r.nrows(), 2);
            assert_eq!(r.ncols(), 3);
            // Column-major: col0=[1,2], col1=[3,4], col2=[5,6]
            assert_eq!(r[[0, 0]], 1.0);
            assert_eq!(r[[1, 0]], 2.0);
            assert_eq!(r[[0, 1]], 3.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_reshape_wrong_size() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0]]));
    assert!(match parse("reshape(v, 2, 2)").unwrap() {
        Stmt::Expr(expr) | Stmt::Assign(_, expr) => eval(&expr, &env).is_err(),
    });
}

#[test]
fn test_fliplr() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[1.0, 2.0, 3.0]]));
    match eval_with("fliplr(v)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![3.0, 2.0, 1.0]);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_flipud() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "m".to_string(),
        Value::Matrix(array![[1.0, 2.0], [3.0, 4.0]]),
    );
    match eval_with("flipud(m)", &env) {
        Value::Matrix(r) => {
            assert_eq!(r[[0, 0]], 3.0);
            assert_eq!(r[[0, 1]], 4.0);
            assert_eq!(r[[1, 0]], 1.0);
            assert_eq!(r[[1, 1]], 2.0);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_find_basic() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[0.0, 3.0, 0.0, 5.0]]));
    match eval_with("find(v)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![2.0, 4.0]); // 1-based indices
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_find_with_k() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[1.0, 0.0, 2.0, 0.0, 3.0]]),
    );
    match eval_with("find(v, 2)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![1.0, 3.0]); // first 2 non-zero indices
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_unique_basic() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[3.0, 1.0, 2.0, 1.0, 3.0]]),
    );
    match eval_with("unique(v)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![1.0, 2.0, 3.0]);
        }
        _ => panic!("expected matrix"),
    }
}

// --- Phase 7.5c: `end` keyword in indexing ---

#[test]
fn test_index_end_last_element() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert("v".to_string(), Value::Matrix(array![[10.0, 20.0, 30.0]]));
    assert_eq!(eval_with("v(end)", &env).as_scalar().unwrap(), 30.0);
}

#[test]
fn test_index_end_minus_one() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[10.0, 20.0, 30.0, 40.0]]),
    );
    assert_eq!(eval_with("v(end-1)", &env).as_scalar().unwrap(), 30.0);
}

#[test]
fn test_index_range_to_end() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "v".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0, 4.0, 5.0]]),
    );
    match eval_with("v(3:end)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![3.0, 4.0, 5.0]);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_index_end_two_dim() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "A".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
    );
    // A(end, :) → last row: [4 5 6]
    match eval_with("A(end, :)", &env) {
        Value::Matrix(r) => {
            let vals: Vec<f64> = r.iter().copied().collect();
            assert_eq!(vals, vec![4.0, 5.0, 6.0]);
        }
        _ => panic!("expected matrix"),
    }
}

#[test]
fn test_index_one_to_end() {
    use ndarray::array;
    let mut env = Env::new();
    env.insert(
        "A".to_string(),
        Value::Matrix(array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
    );
    // A(1:end, 2) → column 2 (1-based): [2; 5]
    match eval_with("A(1:end, 2)", &env) {
        Value::Matrix(r) => {
            assert_eq!(r.nrows(), 2);
            assert_eq!(r.ncols(), 1);
        }
        _ => panic!("expected matrix"),
    }
}
