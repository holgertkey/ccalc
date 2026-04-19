use super::*;
use crate::env::{Env, Value};
use crate::eval::eval;

fn eval_s(expr: &Expr, env: &Env) -> f64 {
    match eval(expr, env).unwrap() {
        Value::Scalar(n) => n,
        _ => panic!("expected scalar"),
    }
}

/// Extracts the inner `Expr` from a simple `Stmt::Assign` or `Stmt::Expr`.
/// Panics if the statement is a block (`If`/`For`/`While`/`Break`/`Continue`).
fn unwrap_expr(stmt: Stmt) -> Expr {
    match stmt {
        Stmt::Expr(e) | Stmt::Assign(_, e) => e,
        _ => panic!("expected simple Expr or Assign, got a block statement"),
    }
}

fn calc(input: &str) -> f64 {
    let env = Env::new();
    eval_s(&unwrap_expr(parse(input).unwrap()), &env)
}

fn calc_with_ans(input: &str, ans: f64) -> f64 {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(ans));
    eval_s(&unwrap_expr(parse(input).unwrap()), &env)
}

fn calc_with_var(input: &str, name: &str, val: f64) -> f64 {
    let mut env = Env::new();
    env.insert(name.to_string(), Value::Scalar(val));
    eval_s(&unwrap_expr(parse(input).unwrap()), &env)
}

#[allow(clippy::approx_constant)]
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
    match eval(&unwrap_expr(parse(input).unwrap()), &env).unwrap() {
        Value::Matrix(m) => m.iter().copied().collect(),
        _ => panic!("expected matrix"),
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
    eval(&unwrap_expr(parse(input).unwrap()), env).unwrap()
}

fn try_eval_with(input: &str, env: &Env) -> Result<Value, String> {
    eval(&unwrap_expr(parse(input)?), env)
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
        Value::Matrix(m) => m.into_raw_vec_and_offset().0,
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
        Value::Matrix(m) => m.into_raw_vec_and_offset().0,
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
        Value::Matrix(m) => m.into_raw_vec_and_offset().0,
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
    assert!(eval(&unwrap_expr(parse("bitand(-1, 5)").unwrap()), &env).is_err());
}

#[test]
fn test_bitwise_error_noninteger() {
    let env = Env::new();
    assert!(eval(&unwrap_expr(parse("bitand(1.5, 2)").unwrap()), &env).is_err());
}

#[test]
fn test_bitnot_error_invalid_width() {
    let env = Env::new();
    assert!(eval(&unwrap_expr(parse("bitnot(5, 0)").unwrap()), &env).is_err());
    assert!(eval(&unwrap_expr(parse("bitnot(5, 54)").unwrap()), &env).is_err());
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
    assert!(eval(&unwrap_expr(parse("reshape(v, 2, 2)").unwrap()), &env).is_err());
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

// ---------------------------------------------------------------------------
// Phase 9 — String tokenizer and parser tests
// ---------------------------------------------------------------------------

#[test]
fn test_parse_char_array() {
    match parse("'hello'").unwrap() {
        Stmt::Expr(Expr::StrLiteral(s)) => assert_eq!(s, "hello"),
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_parse_string_obj() {
    match parse("\"hello\"").unwrap() {
        Stmt::Expr(Expr::StringObjLiteral(s)) => assert_eq!(s, "hello"),
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_parse_char_array_with_escaped_quote() {
    // 'it''s' should parse as the string "it's"
    match parse("'it''s'").unwrap() {
        Stmt::Expr(Expr::StrLiteral(s)) => assert_eq!(s, "it's"),
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_char_array_assignment() {
    match parse("x = 'hello'").unwrap() {
        Stmt::Assign(name, Expr::StrLiteral(s)) => {
            assert_eq!(name, "x");
            assert_eq!(s, "hello");
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_transpose_after_ident_not_string() {
    // x' should be transpose of variable x, not a string
    match parse("x'").unwrap() {
        Stmt::Expr(Expr::Transpose(inner)) => {
            assert!(matches!(*inner, Expr::Var(ref s) if s == "x"));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_char_array_escaped_quote() {
    // 'A''' (5 chars: opening ', A, '', closing ') → char array containing "A'"
    // '' inside a string literal is an escaped single quote
    match parse("'A'''").unwrap() {
        Stmt::Expr(Expr::StrLiteral(s)) => assert_eq!(s, "A'"),
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_char_array_then_transpose() {
    // ('abc')' → transpose of a char array (use parens to force grouping)
    match parse("('abc')'").unwrap() {
        Stmt::Expr(Expr::Transpose(inner)) => {
            assert!(matches!(*inner, Expr::StrLiteral(ref s) if s == "abc"));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_string_obj_escape_sequences() {
    // "hello\nworld" should contain a newline
    match parse("\"hello\\nworld\"").unwrap() {
        Stmt::Expr(Expr::StringObjLiteral(s)) => assert_eq!(s, "hello\nworld"),
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_parse_empty_char_array() {
    match parse("''").unwrap() {
        Stmt::Expr(Expr::StrLiteral(s)) => assert_eq!(s, ""),
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn test_parse_empty_string_obj() {
    match parse("\"\"").unwrap() {
        Stmt::Expr(Expr::StringObjLiteral(s)) => assert_eq!(s, ""),
        other => panic!("unexpected: {:?}", other),
    }
}

// ── Phase 11a: Multi-line block parsing ──────────────────────────────────────

use crate::eval::Base;
use crate::eval::FormatMode;
use crate::exec::exec_stmts;
use crate::io::IoContext;

fn run_block(src: &str) -> Env {
    crate::exec::init();
    let stmts = parse_stmts(src).expect("parse_stmts failed");
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    env.insert("j".to_string(), Value::Complex(0.0, 1.0));
    let mut io = IoContext::new();
    exec_stmts(
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

fn scalar(env: &Env, name: &str) -> f64 {
    match env.get(name) {
        Some(Value::Scalar(n)) => *n,
        v => panic!("expected scalar for '{name}', got {v:?}"),
    }
}

#[test]
fn test_parse_stmts_simple_assign() {
    let env = run_block("x = 42");
    assert_eq!(scalar(&env, "x"), 42.0);
}

#[test]
fn test_parse_stmts_multiline_assign() {
    let env = run_block("x = 1\ny = 2\nz = x + y");
    assert_eq!(scalar(&env, "x"), 1.0);
    assert_eq!(scalar(&env, "y"), 2.0);
    assert_eq!(scalar(&env, "z"), 3.0);
}

#[test]
fn test_if_true_branch() {
    let env = run_block("x = 5\nif x > 0\n  y = 1\nend");
    assert_eq!(scalar(&env, "y"), 1.0);
}

#[test]
fn test_if_false_branch() {
    let env = run_block("x = -1\nif x > 0\n  y = 1\nelse\n  y = 0\nend");
    assert_eq!(scalar(&env, "y"), 0.0);
}

#[test]
fn test_if_elseif_chain() {
    let src = "x = 0\nif x > 0\n  r = 1\nelseif x == 0\n  r = 0\nelse\n  r = -1\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 0.0);
}

#[test]
fn test_if_only_else() {
    let env = run_block("x = -5\nif x > 0\n  r = 1\nelse\n  r = -1\nend");
    assert_eq!(scalar(&env, "r"), -1.0);
}

#[test]
fn test_for_loop_sum() {
    let src = "s = 0\nfor k = 1:5\n  s = s + k\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "s"), 15.0);
}

#[test]
fn test_for_loop_variable() {
    let src = "last = 0\nfor k = 1:4\n  last = k\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "last"), 4.0);
}

#[test]
fn test_while_loop() {
    let src = "x = 1\nwhile x < 8\n  x = x * 2\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "x"), 8.0);
}

#[test]
fn test_while_false_from_start() {
    let env = run_block("x = 10\nwhile x < 0\n  x = x - 1\nend");
    assert_eq!(scalar(&env, "x"), 10.0);
}

#[test]
fn test_break_exits_loop() {
    let src = "s = 0\nfor k = 1:10\n  if k > 3\n    break\n  end\n  s = s + k\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "s"), 6.0); // 1+2+3
}

#[test]
fn test_continue_skips_iteration() {
    let src = "s = 0\nfor k = 1:5\n  if k == 3\n    continue\n  end\n  s = s + k\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "s"), 12.0); // 1+2+4+5
}

#[test]
fn test_nested_for_loops() {
    let src = "s = 0\nfor i = 1:3\n  for j = 1:3\n    s = s + 1\n  end\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "s"), 9.0);
}

#[test]
fn test_block_depth_delta_keywords() {
    assert_eq!(block_depth_delta("if x > 0"), 1);
    assert_eq!(block_depth_delta("for k = 1:10"), 1);
    assert_eq!(block_depth_delta("while x > 0"), 1);
    assert_eq!(block_depth_delta("end"), -1);
    assert_eq!(block_depth_delta("else"), 0);
    assert_eq!(block_depth_delta("elseif x > 0"), 0);
    assert_eq!(block_depth_delta("x = 1"), 0);
    assert_eq!(block_depth_delta("end_value = 5"), 0); // not a keyword
    assert_eq!(block_depth_delta("if_flag = 1"), 0); // not a keyword
    assert_eq!(block_depth_delta("% if comment"), 0); // inside comment
}

// ── Phase 11b: Compound assignment operators ──────────────────────────────────

fn parse_assign(input: &str) -> (String, Expr) {
    match parse(input).unwrap() {
        Stmt::Assign(name, expr) => (name, expr),
        other => panic!("expected Stmt::Assign, got {other:?}"),
    }
}

fn exec_with_var(src: &str, var: &str, init: f64) -> f64 {
    let stmts = parse_stmts(src).expect("parse_stmts failed");
    let mut env = Env::new();
    env.insert(var.to_string(), Value::Scalar(init));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .expect("exec_stmts failed");
    scalar(&env, var)
}

#[test]
fn test_plus_eq() {
    // Parse check
    let (name, _) = parse_assign("x += 5");
    assert_eq!(name, "x");
    // Execution: x = 10, then x += 5 → 15
    assert_eq!(exec_with_var("x += 5", "x", 10.0), 15.0);
}

#[test]
fn test_minus_eq() {
    let stmts = parse_stmts("x -= 3").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(10.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 7.0);
}

#[test]
fn test_star_eq() {
    let stmts = parse_stmts("x *= 3").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(4.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 12.0);
}

#[test]
fn test_slash_eq() {
    let stmts = parse_stmts("x /= 2").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(8.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 4.0);
}

#[test]
fn test_plus_plus_suffix() {
    let stmts = parse_stmts("x++").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(5.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 6.0);
}

#[test]
fn test_minus_minus_suffix() {
    let stmts = parse_stmts("x--").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(5.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 4.0);
}

#[test]
fn test_plus_plus_prefix() {
    let stmts = parse_stmts("++x").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(5.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 6.0);
}

#[test]
fn test_minus_minus_prefix() {
    let stmts = parse_stmts("--x").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(5.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 4.0);
}

#[test]
fn test_compound_in_for_loop() {
    let src = "s = 0\nfor k = 1:5\n  s += k\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "s"), 15.0);
}

#[test]
fn test_increment_in_while_loop() {
    let src = "i = 0\nwhile i < 5\n  i++\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "i"), 5.0);
}

#[test]
fn test_compound_rhs_expression() {
    // x *= 2 + 3  →  x = x * (2 + 3) = x * 5
    let stmts = parse_stmts("x *= 2 + 3").unwrap();
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(4.0));
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "x"), 20.0);
}

#[test]
fn test_arithmetic_still_works_after_tokenizer_changes() {
    // Ensure regular + and - still work (not confused with ++ or +=)
    assert_eq!(calc("1 + 2"), 3.0);
    assert_eq!(calc("5 - 3"), 2.0);
    assert_eq!(calc("2 * 4"), 8.0);
    assert_eq!(calc("10 / 4"), 2.5);
    assert_eq!(calc("1e-3"), 0.001); // sci notation with minus
    assert_eq!(calc("1e+3"), 1000.0); // sci notation with plus
    assert_eq!(calc("-5 + 3"), -2.0); // unary minus + binary plus
}

// ── Aliases: # comment, ! NOT, != not-equal ───────────────────────────────

#[test]
fn test_hash_comment_full_line() {
    // A line that is only a comment produces no tokens → parse error (empty)
    // but split_stmts strips it entirely
    let pairs = split_stmts("# this is a comment");
    assert!(pairs.is_empty());
}

#[test]
fn test_hash_comment_inline() {
    // Inline # comment: only the part before # is parsed
    assert_eq!(calc("2 + 3 # plus three"), 5.0);
}

#[test]
fn test_bang_not() {
    // !x  →  same as ~x
    assert_eq!(calc_with_var("!x", "x", 5.0), 0.0); // nonzero → 0
    assert_eq!(calc_with_var("!x", "x", 0.0), 1.0); // zero    → 1
}

#[test]
fn test_bang_not_eq() {
    // !=  →  same as ~=
    assert_eq!(calc("3 != 4"), 1.0);
    assert_eq!(calc("3 != 3"), 0.0);
}

// ── Phase 11.5a: switch/case/otherwise ───────────────────────────────────────

#[test]
fn test_switch_scalar_first_case() {
    let src =
        "x = 2\nswitch x\n  case 1\n    r = 10\n  case 2\n    r = 20\n  case 3\n    r = 30\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 20.0);
}

#[test]
fn test_switch_scalar_last_case() {
    let src =
        "x = 3\nswitch x\n  case 1\n    r = 10\n  case 2\n    r = 20\n  case 3\n    r = 30\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 30.0);
}

#[test]
fn test_switch_no_match_no_otherwise() {
    // No match and no otherwise — r stays at its initial value
    let src = "r = 99\nswitch 5\n  case 1\n    r = 1\n  case 2\n    r = 2\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 99.0);
}

#[test]
fn test_switch_otherwise() {
    let src = "x = 7\nr = 0\nswitch x\n  case 1\n    r = 1\n  case 2\n    r = 2\n  otherwise\n    r = -1\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), -1.0);
}

#[test]
fn test_switch_string_match() {
    let src = "mode = 'fast'\nswitch mode\n  case 'slow'\n    r = 1\n  case 'fast'\n    r = 2\n  otherwise\n    r = 0\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 2.0);
}

#[test]
fn test_switch_string_otherwise() {
    let src = "mode = 'turbo'\nr = 0\nswitch mode\n  case 'slow'\n    r = 1\n  case 'fast'\n    r = 2\n  otherwise\n    r = -1\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), -1.0);
}

#[test]
fn test_switch_empty_no_cases() {
    // switch with no cases and no otherwise — nothing happens
    let src = "r = 42\nswitch 1\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 42.0);
}

#[test]
fn test_switch_only_otherwise() {
    let src = "r = 0\nswitch 99\n  otherwise\n    r = 7\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 7.0);
}

#[test]
fn test_switch_first_match_wins() {
    // Only the first matching case executes (no fall-through)
    let src = "x = 1\nr = 0\nswitch x\n  case 1\n    r = 100\n  case 1\n    r = 200\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "r"), 100.0);
}

#[test]
fn test_switch_inside_for_loop() {
    let src = "s = 0\nfor k = 1:3\n  switch k\n    case 1\n      s += 10\n    case 2\n      s += 20\n    otherwise\n      s += 1\n  end\nend";
    let env = run_block(src);
    assert_eq!(scalar(&env, "s"), 31.0); // 10 + 20 + 1
}

// ── Phase 11.5c: do...until ──────────────────────────────────────────────────

#[test]
fn test_do_until_basic() {
    // Counts from 1 to 5; stops when x == 5
    let src = "x = 0\ndo\n  x = x + 1\nuntil (x >= 5)";
    let env = run_block(src);
    assert_eq!(scalar(&env, "x"), 5.0);
}

#[test]
fn test_do_until_executes_at_least_once() {
    // Condition is true from the start — body still runs once
    let src = "x = 10\ndo\n  x = x + 1\nuntil (x > 0)";
    let env = run_block(src);
    assert_eq!(scalar(&env, "x"), 11.0);
}

#[test]
fn test_do_until_multiple_iterations() {
    // Doubles x until it exceeds 50
    let src = "x = 1\ndo\n  x = x * 2\nuntil (x > 50)";
    let env = run_block(src);
    assert_eq!(scalar(&env, "x"), 64.0);
}

#[test]
fn test_do_until_break() {
    // Break exits the loop before condition
    let src = "x = 0\ndo\n  x = x + 1\n  if x == 3\n    break\n  end\nuntil (x >= 10)";
    let env = run_block(src);
    assert_eq!(scalar(&env, "x"), 3.0);
}

#[test]
fn test_do_until_continue() {
    // continue skips the rest of the body; condition is then checked
    let src = "s = 0\nx = 0\ndo\n  x = x + 1\n  if x == 3\n    continue\n  end\n  s = s + x\nuntil (x >= 5)";
    let env = run_block(src);
    // x goes 1,2,3(skipped),4,5  →  s = 1+2+4+5 = 12
    assert_eq!(scalar(&env, "s"), 12.0);
    assert_eq!(scalar(&env, "x"), 5.0);
}

#[test]
fn test_do_until_no_parens() {
    // Octave allows until without parens
    let src = "x = 0\ndo\n  x += 1\nuntil x == 4";
    let env = run_block(src);
    assert_eq!(scalar(&env, "x"), 4.0);
}

// ── Phase 11.5a/c: block_depth_delta extended ────────────────────────────────

#[test]
fn test_block_depth_delta_switch_do_until() {
    assert_eq!(block_depth_delta("switch x"), 1);
    assert_eq!(block_depth_delta("do"), 1);
    assert_eq!(block_depth_delta("until (x < 1)"), -1);
    assert_eq!(block_depth_delta("until x < 1"), -1);
    // These should not be confused with keywords when part of identifiers
    assert_eq!(block_depth_delta("switch_val = 1"), 0);
    assert_eq!(block_depth_delta("do_something"), 0);
}

// ── Phase 11.5e: run() / source() ────────────────────────────────────────────

fn run_block_with_env(src: &str, env: &mut Env) {
    crate::exec::init();
    let stmts = parse_stmts(src).expect("parse_stmts failed");
    let mut io = IoContext::new();
    exec_stmts(&stmts, env, &mut io, &FormatMode::Short, Base::Dec, true)
        .expect("exec_stmts failed");
}

#[test]
fn test_run_calc_script() {
    let dir = std::env::temp_dir().join("ccalc_test_run");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("helper.calc");
    std::fs::write(&script, "result = 42\n").unwrap();

    let path = script.to_string_lossy().replace('\\', "/");
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env(&format!("run('{path}')"), &mut env);
    assert_eq!(
        match env.get("result") {
            Some(Value::Scalar(n)) => *n,
            other => panic!("expected scalar, got {other:?}"),
        },
        42.0
    );
    std::fs::remove_file(script).ok();
}

#[test]
fn test_run_m_script() {
    let dir = std::env::temp_dir().join("ccalc_test_run");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("helper_m.m");
    std::fs::write(&script, "mval = 7\n").unwrap();

    let path = script.to_string_lossy().replace('\\', "/");
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env(&format!("run('{path}')"), &mut env);
    assert_eq!(
        match env.get("mval") {
            Some(Value::Scalar(n)) => *n,
            other => panic!("expected scalar, got {other:?}"),
        },
        7.0
    );
    std::fs::remove_file(script).ok();
}

#[test]
fn test_run_no_extension_prefers_calc() {
    // When no extension given, .calc is preferred over .m
    let dir = std::env::temp_dir().join("ccalc_test_run_ext");
    std::fs::create_dir_all(&dir).unwrap();
    let calc_script = dir.join("ambiguous.calc");
    let m_script = dir.join("ambiguous.m");
    std::fs::write(&calc_script, "chosen = 1\n").unwrap();
    std::fs::write(&m_script, "chosen = 2\n").unwrap();

    let base = dir.join("ambiguous").to_string_lossy().replace('\\', "/");
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env(&format!("run('{base}')"), &mut env);
    assert_eq!(
        match env.get("chosen") {
            Some(Value::Scalar(n)) => *n,
            other => panic!("expected scalar, got {other:?}"),
        },
        1.0, // .calc wins
    );
    std::fs::remove_file(calc_script).ok();
    std::fs::remove_file(m_script).ok();
}

#[test]
fn test_run_script_shares_env() {
    // Variables defined in the script persist in the caller's scope
    let dir = std::env::temp_dir().join("ccalc_test_run_env");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("env_test.calc");
    std::fs::write(&script, "shared = x * 2\n").unwrap();

    let path = script.to_string_lossy().replace('\\', "/");
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    env.insert("x".to_string(), Value::Scalar(5.0));
    run_block_with_env(&format!("run('{path}')"), &mut env);
    assert_eq!(
        match env.get("shared") {
            Some(Value::Scalar(n)) => *n,
            other => panic!("expected scalar, got {other:?}"),
        },
        10.0
    );
    std::fs::remove_file(script).ok();
}

#[test]
fn test_source_alias() {
    // source() is an alias for run()
    let dir = std::env::temp_dir().join("ccalc_test_source");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("src_test.calc");
    std::fs::write(&script, "sourced = 99\n").unwrap();

    let path = script.to_string_lossy().replace('\\', "/");
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env(&format!("source('{path}')"), &mut env);
    assert_eq!(
        match env.get("sourced") {
            Some(Value::Scalar(n)) => *n,
            other => panic!("expected scalar, got {other:?}"),
        },
        99.0
    );
    std::fs::remove_file(script).ok();
}

// ── Phase 12.6 tests ──────────────────────────────────────────────────────────

#[test]
fn test_unary_plus_noop() {
    assert_eq!(calc("+5"), 5.0);
    assert_eq!(calc("+-3"), -3.0);
}

#[test]
fn test_starstar_pow() {
    assert_eq!(calc("2 ** 8"), 256.0);
    assert_eq!(calc("3 ** 3"), 27.0);
}

#[test]
fn test_elem_and_scalar() {
    assert_eq!(calc("1 & 1"), 1.0);
    assert_eq!(calc("1 & 0"), 0.0);
    assert_eq!(calc("0 & 0"), 0.0);
}

#[test]
fn test_elem_or_scalar() {
    assert_eq!(calc("1 | 0"), 1.0);
    assert_eq!(calc("0 | 0"), 0.0);
}

#[test]
fn test_elem_and_matrix() {
    use crate::env::Value;
    use crate::eval::eval;
    use ndarray::array;
    let env = Env::new();
    let expr = unwrap_expr(parse("[1 0 1] & [1 1 0]").unwrap());
    match eval(&expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m, array![[1.0, 0.0, 0.0]]);
        }
        other => panic!("expected matrix, got {other:?}"),
    }
}

#[test]
fn test_xor_builtin() {
    assert_eq!(calc("xor(1, 0)"), 1.0);
    assert_eq!(calc("xor(1, 1)"), 0.0);
    assert_eq!(calc("xor(0, 0)"), 0.0);
}

#[test]
fn test_not_builtin() {
    assert_eq!(calc("not(0)"), 1.0);
    assert_eq!(calc("not(5)"), 0.0);
}

#[test]
fn test_plain_transpose_real() {
    use crate::env::Value;
    use crate::eval::eval;
    use ndarray::array;
    let env = Env::new();
    let expr = unwrap_expr(parse("[1 2; 3 4].'").unwrap());
    match eval(&expr, &env).unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m, array![[1.0, 3.0], [2.0, 4.0]]);
        }
        other => panic!("expected matrix, got {other:?}"),
    }
}

#[test]
fn test_plain_transpose_complex_no_conjugate() {
    use crate::env::Value;
    use crate::eval::eval;
    let mut env = Env::new();
    env.insert("z".to_string(), Value::Complex(1.0, 2.0));
    let expr = unwrap_expr(parse("z.'").unwrap());
    // Plain transpose: z.' = z (no sign flip on imaginary part)
    assert_eq!(eval(&expr, &env).unwrap(), Value::Complex(1.0, 2.0));
}

#[test]
fn test_conjugate_transpose_complex() {
    use crate::env::Value;
    use crate::eval::eval;
    let mut env = Env::new();
    env.insert("z".to_string(), Value::Complex(1.0, 2.0));
    let expr = unwrap_expr(parse("z'").unwrap());
    // Conjugate transpose: z' = conj(z)
    assert_eq!(eval(&expr, &env).unwrap(), Value::Complex(1.0, -2.0));
}

#[test]
fn test_lambda_display() {
    use crate::eval::eval;
    use crate::eval::{Base, FormatMode, format_value};
    let env = Env::new();
    let expr = unwrap_expr(parse("@(x) x + 1").unwrap());
    let val = eval(&expr, &env).unwrap();
    let displayed = format_value(&val, Base::Dec, &FormatMode::Short);
    assert!(
        displayed.starts_with("@(x)"),
        "lambda display should show source, got: {displayed}"
    );
}

#[test]
fn test_single_line_if() {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env("if 1 > 0; y = 42; end", &mut env);
    assert_eq!(
        match env.get("y") {
            Some(Value::Scalar(n)) => *n,
            _ => panic!(),
        },
        42.0
    );
}

#[test]
fn test_single_line_for() {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env("for k = 1:3; s = k; end", &mut env);
    assert_eq!(
        match env.get("s") {
            Some(Value::Scalar(n)) => *n,
            _ => panic!(),
        },
        3.0
    );
}

#[test]
fn test_single_line_while() {
    let mut env = Env::new();
    env.insert("x".to_string(), Value::Scalar(3.0));
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env("while x > 0; x = x - 1; end", &mut env);
    assert_eq!(
        match env.get("x") {
            Some(Value::Scalar(n)) => *n,
            _ => panic!(),
        },
        0.0
    );
}

#[test]
fn test_line_continuation() {
    let block = "x = 1 + ...\n  2 + ...\n  3";
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    run_block_with_env(block, &mut env);
    assert_eq!(
        match env.get("x") {
            Some(Value::Scalar(n)) => *n,
            _ => panic!(),
        },
        6.0
    );
}

#[test]
fn test_comma_separator() {
    let stmts = split_stmts("a = 1, b = 2");
    // 'a = 1' non-silent, 'b = 2' non-silent
    assert_eq!(stmts.len(), 2);
    assert!(!stmts[0].1); // non-silent (shown)
    assert!(!stmts[1].1); // non-silent (shown)
}

#[test]
fn test_int2str() {
    use crate::env::Value;
    use crate::eval::eval;
    let env = Env::new();
    let expr = unwrap_expr(parse("int2str(3.7)").unwrap());
    assert_eq!(eval(&expr, &env).unwrap(), Value::Str("4".to_string()));
}

#[test]
fn test_mat2str() {
    use crate::env::Value;
    use crate::eval::eval;
    let env = Env::new();
    let expr = unwrap_expr(parse("mat2str([1 2; 3 4])").unwrap());
    assert_eq!(
        eval(&expr, &env).unwrap(),
        Value::Str("[1 2;3 4]".to_string())
    );
}

#[test]
fn test_strsplit_basic() {
    use crate::env::Value;
    use crate::eval::eval;
    let env = Env::new();
    let expr = unwrap_expr(parse("strsplit('a,b,c', ',')").unwrap());
    match eval(&expr, &env).unwrap() {
        Value::Cell(parts) => {
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0], Value::Str("a".to_string()));
            assert_eq!(parts[1], Value::Str("b".to_string()));
            assert_eq!(parts[2], Value::Str("c".to_string()));
        }
        other => panic!("expected cell, got {other:?}"),
    }
}

// ── Phase 12: User-defined functions and lambdas ─────────────────────────────

// Helper: runs a block and calls a function defined in it.
fn run_block_fn(src: &str, call: &str) -> Value {
    let mut env = run_block(src); // run_block already calls init()
    env.entry("ans".to_string()).or_insert(Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts(call).expect("parse call failed");
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .expect("exec call failed");
    env.get("ans").cloned().unwrap_or(Value::Scalar(0.0))
}

// ── 12a: Named functions ──────────────────────────────────────────────────────

#[test]
fn test_named_function_single_return() {
    let src = "function y = double(x)\ny = x * 2\nend";
    let val = run_block_fn(src, "ans = double(5)");
    assert_eq!(val, Value::Scalar(10.0));
}

#[test]
fn test_named_function_no_outputs() {
    // Function with no outputs stores nothing visible to the caller
    let src = "function foo(x)\nend";
    let env = run_block(src);
    assert!(matches!(env.get("foo"), Some(Value::Function { .. })));
}

#[test]
fn test_named_function_nargin() {
    let src = "function y = nargin_check(a, b)\ny = nargin\nend";
    let val = run_block_fn(src, "ans = nargin_check(1)");
    assert_eq!(val, Value::Scalar(1.0));
}

#[test]
fn test_named_function_with_if() {
    let src =
        "function y = sign_of(x)\nif x > 0\ny = 1\nelseif x < 0\ny = -1\nelse\ny = 0\nend\nend";
    let mut env = run_block(src);
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("ans = sign_of(5)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(1.0)));

    let stmts = parse_stmts("ans = sign_of(-3)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(-1.0)));

    let stmts = parse_stmts("ans = sign_of(0)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(0.0)));
}

#[test]
fn test_named_function_isolated_scope() {
    // Variables from the caller's scope must not leak into the function body
    let src = "x = 99\nfunction y = f(a)\ny = a + 1\nend";
    let mut env = run_block(src);
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("ans = f(3)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    // x should still be 99, not changed by function
    assert_eq!(env.get("x").cloned(), Some(Value::Scalar(99.0)));
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(4.0)));
}

#[test]
fn test_named_function_return_statement() {
    let src = "function y = early(x)\nif x > 0\ny = 1\nreturn\nend\ny = -1\nend";
    let mut env = run_block(src);
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("ans = early(5)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(1.0)));

    let stmts = parse_stmts("ans = early(-1)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(-1.0)));
}

// ── 12b: Multiple return values ───────────────────────────────────────────────

#[test]
fn test_named_function_multi_return() {
    let src = "function [mn, mx] = bounds(v)\nmn = min(v)\nmx = max(v)\nend";
    let mut env = run_block(src);
    env.insert("ans".to_string(), Value::Scalar(0.0));
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    env.insert("j".to_string(), Value::Complex(0.0, 1.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("[lo, hi] = bounds([3, 1, 4, 1, 5])").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("lo").cloned(), Some(Value::Scalar(1.0)));
    assert_eq!(env.get("hi").cloned(), Some(Value::Scalar(5.0)));
}

#[test]
fn test_multi_assign_extra_discarded() {
    // Calling a function that returns a tuple with only one target — extras discarded
    let src = "function [a, b] = pair()\na = 10\nb = 20\nend";
    let mut env = run_block(src);
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("[x] = pair()").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("x").cloned(), Some(Value::Scalar(10.0)));
    assert!(!env.contains_key("b") || env.get("b").cloned() != Some(Value::Scalar(20.0)));
}

#[test]
fn test_multi_assign_tilde_discard() {
    let src = "function [a, b, c] = triple()\na = 1\nb = 2\nc = 3\nend";
    let mut env = run_block(src);
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("[x, ~, z] = triple()").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("x").cloned(), Some(Value::Scalar(1.0)));
    assert_eq!(env.get("z").cloned(), Some(Value::Scalar(3.0)));
}

// ── 12c: Anonymous functions (lambdas) ────────────────────────────────────────

#[test]
fn test_lambda_parse() {
    // @(x) x * 2 should parse to Expr::Lambda
    let tokens = tokenize("@(x) x * 2").unwrap();
    let mut pos = 0;
    let expr = parse_logical_or(&tokens, &mut pos).unwrap();
    assert!(matches!(expr, Expr::Lambda { .. }));
}

#[test]
fn test_lambda_eval_single_arg() {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    // Assign f = @(x) x^2, then call f(4)
    let stmts = parse_stmts("f = @(x) x^2\nans = f(4)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(16.0)));
}

#[test]
fn test_lambda_eval_two_args() {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("add = @(a, b) a + b\nans = add(3, 7)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(10.0)));
}

#[test]
fn test_lambda_no_args() {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("k = @() 42\nans = k()").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(42.0)));
}

#[test]
fn test_lambda_captures_lexical_env() {
    // The lambda should capture the value of `c` at definition time
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("c = 10\nfn = @(x) x + c\nc = 99\nans = fn(5)").unwrap();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    // Should capture c=10, not c=99
    assert_eq!(env.get("ans").cloned(), Some(Value::Scalar(15.0)));
}

// ── Parser-level tests ────────────────────────────────────────────────────────

#[test]
fn test_parse_return_stmt() {
    let stmts = parse_stmts("return").unwrap();
    assert!(matches!(stmts.as_slice(), [(Stmt::Return, false)]));
}

#[test]
fn test_parse_multi_assign() {
    let stmt = parse("[a, b] = f()").unwrap();
    assert!(matches!(stmt, Stmt::MultiAssign { targets, .. } if targets == vec!["a", "b"]));
}

#[test]
fn test_parse_function_def() {
    let stmts = parse_stmts("function y = sq(x)\ny = x*x\nend").unwrap();
    assert!(
        matches!(&stmts[0].0, Stmt::FunctionDef { name, outputs, params, .. }
            if name == "sq" && outputs == &["y"] && params == &["x"])
    );
}

#[test]
fn test_parse_function_multi_output() {
    let stmts = parse_stmts("function [a, b] = swap(x, y)\na = y\nb = x\nend").unwrap();
    assert!(
        matches!(&stmts[0].0, Stmt::FunctionDef { name, outputs, params, .. }
            if name == "swap" && outputs == &["a", "b"] && params == &["x", "y"])
    );
}

#[test]
fn test_block_depth_function() {
    assert_eq!(block_depth_delta("function y = f(x)"), 1);
    assert_eq!(block_depth_delta("end"), -1);
}

#[test]
fn test_too_many_args_error() {
    crate::exec::init();
    let src = "function y = f(x)\ny = x\nend";
    let mut env = run_block(src);
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("ans = f(1, 2, 3)").unwrap();
    let result = exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    );
    assert!(result.is_err());
}

// ── Phase 12.5 — Cell arrays ─────────────────────────────────────────────────

#[test]
fn test_cell_literal_basic() {
    // {1, 'hello', [1 2 3]}
    let env = Env::new();
    let stmt = parse("{1, 2, 3}").unwrap();
    let expr = unwrap_expr(stmt);
    let val = eval(&expr, &env).unwrap();
    assert!(matches!(val, Value::Cell(ref v) if v.len() == 3));
}

#[test]
fn test_cell_literal_empty() {
    let env = Env::new();
    let stmt = parse("{}").unwrap();
    let expr = unwrap_expr(stmt);
    let val = eval(&expr, &env).unwrap();
    assert!(matches!(val, Value::Cell(ref v) if v.is_empty()));
}

#[test]
fn test_cell_index_basic() {
    crate::exec::init();
    let mut env = run_block("c = {10, 20, 30}");
    let stmts = parse_stmts("ans = c{2}").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans"), Some(&Value::Scalar(20.0)));
}

#[test]
fn test_cell_index_first() {
    crate::exec::init();
    let mut env = run_block("c = {42, 'hello'}");
    let stmts = parse_stmts("ans = c{1}").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans"), Some(&Value::Scalar(42.0)));
}

#[test]
fn test_cell_index_string() {
    crate::exec::init();
    let mut env = run_block("c = {1, 'world', 3}");
    let stmts = parse_stmts("ans = c{2}").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans"), Some(&Value::Str("world".to_string())));
}

#[test]
fn test_cell_set_basic() {
    crate::exec::init();
    let mut env = run_block("c = {1, 2, 3}");
    let stmts = parse_stmts("c{2} = 99").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    match env.get("c") {
        Some(Value::Cell(v)) => {
            assert_eq!(v[1], Value::Scalar(99.0));
        }
        _ => panic!("expected Cell"),
    }
}

#[test]
fn test_cell_set_grows() {
    crate::exec::init();
    let mut env = run_block("c = {1}");
    let stmts = parse_stmts("c{3} = 5").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    match env.get("c") {
        Some(Value::Cell(v)) => {
            assert_eq!(v.len(), 3);
            assert_eq!(v[2], Value::Scalar(5.0));
        }
        _ => panic!("expected Cell"),
    }
}

#[test]
fn test_iscell() {
    crate::exec::init();
    let mut env = run_block("c = {1, 2}");
    let stmts = parse_stmts("ans = iscell(c)").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans"), Some(&Value::Scalar(1.0)));
}

#[test]
fn test_iscell_on_scalar() {
    let env = Env::new();
    let stmt = parse("iscell(5)").unwrap();
    let val = eval(&unwrap_expr(stmt), &env).unwrap();
    assert_eq!(val, Value::Scalar(0.0));
}

#[test]
fn test_cell_constructor() {
    let env = Env::new();
    let stmt = parse("cell(3)").unwrap();
    let val = eval(&unwrap_expr(stmt), &env).unwrap();
    match val {
        Value::Cell(v) => {
            assert_eq!(v.len(), 3);
            assert!(v.iter().all(|x| matches!(x, Value::Scalar(n) if *n == 0.0)));
        }
        _ => panic!("expected Cell"),
    }
}

#[test]
fn test_numel_cell() {
    crate::exec::init();
    let mut env = run_block("c = {1, 2, 3, 4}");
    let stmts = parse_stmts("ans = numel(c)").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans"), Some(&Value::Scalar(4.0)));
}

#[test]
fn test_length_cell() {
    crate::exec::init();
    let mut env = run_block("c = {1, 2, 3}");
    let stmts = parse_stmts("ans = length(c)").unwrap();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(env.get("ans"), Some(&Value::Scalar(3.0)));
}

#[test]
fn test_cellfun_basic() {
    crate::exec::init();
    let src = "c = {1, 4, 9}\nans = cellfun(@sqrt, c)";
    let env = run_block(src);
    match env.get("ans") {
        Some(Value::Matrix(m)) => {
            assert_eq!(m.nrows(), 1);
            assert_eq!(m.ncols(), 3);
            assert!((m[[0, 0]] - 1.0).abs() < 1e-10);
            assert!((m[[0, 1]] - 2.0).abs() < 1e-10);
            assert!((m[[0, 2]] - 3.0).abs() < 1e-10);
        }
        _ => panic!("expected Matrix from cellfun"),
    }
}

#[test]
fn test_arrayfun_basic() {
    crate::exec::init();
    let src = "ans = arrayfun(@(x) x^2, [1 2 3])";
    let env = run_block(src);
    match env.get("ans") {
        Some(Value::Matrix(m)) => {
            assert_eq!(m.ncols(), 3);
            assert_eq!(m[[0, 0]], 1.0);
            assert_eq!(m[[0, 1]], 4.0);
            assert_eq!(m[[0, 2]], 9.0);
        }
        _ => panic!("expected Matrix from arrayfun"),
    }
}

#[test]
fn test_switch_cell_case() {
    crate::exec::init();
    let src = "x = 3\nswitch x\n  case {2, 3}\n    ans = 1\n  otherwise\n    ans = 0\nend";
    let env = run_block(src);
    assert_eq!(env.get("ans"), Some(&Value::Scalar(1.0)));
}

#[test]
fn test_switch_cell_case_no_match() {
    crate::exec::init();
    let src = "x = 5\nswitch x\n  case {2, 3}\n    ans = 1\n  otherwise\n    ans = 0\nend";
    let env = run_block(src);
    assert_eq!(env.get("ans"), Some(&Value::Scalar(0.0)));
}

#[test]
fn test_varargin_basic() {
    crate::exec::init();
    let src = "function out = mysum(varargin)\nout = 0\nfor k = 1:numel(varargin)\n  out = out + varargin{k}\nend\nend\nans = mysum(1, 2, 3)";
    let env = run_block(src);
    assert_eq!(env.get("ans"), Some(&Value::Scalar(6.0)));
}

// ── Phase 12.6 bug-fix tests ─────────────────────────────────────────────────

#[test]
fn test_imag_literal_4i() {
    // `4i` must tokenize as 4 * i, giving Complex(0, 4)
    let mut env = Env::new();
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    let expr = unwrap_expr(parse("4i").unwrap());
    assert_eq!(eval(&expr, &env).unwrap(), Value::Complex(0.0, 4.0));
}

#[test]
fn test_imag_literal_in_expression() {
    // `3 + 4i` must give Complex(3, 4)
    let mut env = Env::new();
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    let expr = unwrap_expr(parse("3 + 4i").unwrap());
    assert_eq!(eval(&expr, &env).unwrap(), Value::Complex(3.0, 4.0));
}

#[test]
fn test_imag_literal_not_confused_with_ident() {
    // `inside` must NOT strip leading `i` — only bare `i`/`j` get the suffix treatment
    let mut env = Env::new();
    env.insert("inside".to_string(), Value::Scalar(42.0));
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    let expr = unwrap_expr(parse("inside").unwrap());
    assert_eq!(eval(&expr, &env).unwrap(), Value::Scalar(42.0));
}

#[test]
fn test_split_stmts_dot_apostrophe_not_string() {
    // `B.';` must split into one silent statement, not treat `'` as a string start
    let parts: Vec<_> = crate::parser::split_stmts("Bt = B.';");
    assert_eq!(parts.len(), 1);
    assert!(parts[0].1); // silent
}

#[test]
fn test_cell_index_out_of_bounds() {
    crate::exec::init();
    let mut env = run_block("c = {1, 2}");
    let mut io = IoContext::new();
    let stmts = parse_stmts("ans = c{5}").unwrap();
    let result = exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    );
    assert!(result.is_err());
}

// ── Phase 13: Structs ────────────────────────────────────────────────────────

#[test]
fn test_struct_field_assign_basic() {
    let env = run_block("s.x = 42");
    match env.get("s").unwrap() {
        Value::Struct(map) => assert_eq!(map.get("x"), Some(&Value::Scalar(42.0))),
        other => panic!("expected Struct, got {other:?}"),
    }
}

#[test]
fn test_struct_field_read() {
    let env = run_block("s.x = 7; ans = s.x");
    assert_eq!(env.get("ans"), Some(&Value::Scalar(7.0)));
}

#[test]
fn test_struct_multiple_fields() {
    let env = run_block("s.a = 1; s.b = 2; s.c = 3");
    let map = match env.get("s").unwrap() {
        Value::Struct(m) => m,
        other => panic!("expected Struct, got {other:?}"),
    };
    assert_eq!(map.get("a"), Some(&Value::Scalar(1.0)));
    assert_eq!(map.get("b"), Some(&Value::Scalar(2.0)));
    assert_eq!(map.get("c"), Some(&Value::Scalar(3.0)));
}

#[test]
fn test_struct_field_overwrite() {
    let env = run_block("s.x = 1; s.x = 99");
    match env.get("s").unwrap() {
        Value::Struct(map) => assert_eq!(map.get("x"), Some(&Value::Scalar(99.0))),
        other => panic!("expected Struct, got {other:?}"),
    }
}

#[test]
fn test_struct_nested_assign() {
    let env = run_block("s.a.b = 5");
    let outer = match env.get("s").unwrap() {
        Value::Struct(m) => m,
        other => panic!("expected Struct, got {other:?}"),
    };
    let inner = match outer.get("a").unwrap() {
        Value::Struct(m) => m,
        other => panic!("expected nested Struct, got {other:?}"),
    };
    assert_eq!(inner.get("b"), Some(&Value::Scalar(5.0)));
}

#[test]
fn test_struct_nested_read() {
    let env = run_block("s.a.b = 10; ans = s.a.b");
    assert_eq!(env.get("ans"), Some(&Value::Scalar(10.0)));
}

#[test]
fn test_struct_constructor_basic() {
    let env = run_block("s = struct('x', 1, 'y', 2)");
    let map = match env.get("s").unwrap() {
        Value::Struct(m) => m,
        other => panic!("expected Struct, got {other:?}"),
    };
    assert_eq!(map.get("x"), Some(&Value::Scalar(1.0)));
    assert_eq!(map.get("y"), Some(&Value::Scalar(2.0)));
}

#[test]
fn test_struct_constructor_empty() {
    let env = run_block("s = struct()");
    match env.get("s").unwrap() {
        Value::Struct(map) => assert!(map.is_empty()),
        other => panic!("expected Struct, got {other:?}"),
    }
}

#[test]
fn test_struct_fieldnames() {
    let env = run_block("s.a = 1; s.b = 2; fn = fieldnames(s)");
    match env.get("fn").unwrap() {
        Value::Cell(v) => {
            assert_eq!(v.len(), 2);
            assert!(matches!(&v[0], Value::Str(s) if s == "a"));
            assert!(matches!(&v[1], Value::Str(s) if s == "b"));
        }
        other => panic!("expected Cell, got {other:?}"),
    }
}

#[test]
fn test_struct_isfield_true() {
    let env = run_block("s.x = 1; ans = isfield(s, 'x')");
    assert_eq!(env.get("ans"), Some(&Value::Scalar(1.0)));
}

#[test]
fn test_struct_isfield_false() {
    let env = run_block("s.x = 1; ans = isfield(s, 'y')");
    assert_eq!(env.get("ans"), Some(&Value::Scalar(0.0)));
}

#[test]
fn test_struct_rmfield() {
    let env = run_block("s.a = 1; s.b = 2; s = rmfield(s, 'a')");
    let map = match env.get("s").unwrap() {
        Value::Struct(m) => m,
        other => panic!("expected Struct, got {other:?}"),
    };
    assert!(!map.contains_key("a"));
    assert_eq!(map.get("b"), Some(&Value::Scalar(2.0)));
}

#[test]
fn test_struct_isstruct_true() {
    let env = run_block("s.x = 1; ans = isstruct(s)");
    assert_eq!(env.get("ans"), Some(&Value::Scalar(1.0)));
}

#[test]
fn test_struct_isstruct_false() {
    let env = run_block("ans = isstruct(42)");
    assert_eq!(env.get("ans"), Some(&Value::Scalar(0.0)));
}

#[test]
fn test_struct_field_missing_error() {
    crate::exec::init();
    let mut env = run_block("s.x = 1");
    let mut io = IoContext::new();
    let stmts = parse_stmts("ans = s.z").unwrap();
    let result = exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    );
    assert!(result.is_err());
}

#[test]
fn test_struct_field_on_non_struct_error() {
    crate::exec::init();
    let mut env = run_block("x = 5");
    let mut io = IoContext::new();
    let stmts = parse_stmts("ans = x.field").unwrap();
    let result = exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    );
    assert!(result.is_err());
}

#[test]
fn test_struct_constructor_odd_args_error() {
    crate::exec::init();
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    let mut io = IoContext::new();
    let stmts = parse_stmts("s = struct('x')").unwrap();
    let result = exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    );
    assert!(result.is_err());
}

#[test]
fn test_struct_rmfield_missing_error() {
    crate::exec::init();
    let mut env = run_block("s.x = 1");
    let mut io = IoContext::new();
    let stmts = parse_stmts("s = rmfield(s, 'z')").unwrap();
    let result = exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    );
    assert!(result.is_err());
}

#[test]
fn test_struct_field_insertion_order() {
    let env = run_block("s.c = 3; s.a = 1; s.b = 2");
    let map = match env.get("s").unwrap() {
        Value::Struct(m) => m,
        other => panic!("expected Struct, got {other:?}"),
    };
    let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["c", "a", "b"]);
}

// ── Phase 13.5 — Struct arrays ────────────────────────────────────────────────

#[test]
fn test_struct_array_basic_create_and_read() {
    // s(1).x = 1; s(2).x = 5 → StructArray with 2 elements
    let env = run_block("s(1).x = 1; s(2).x = 5");
    match env.get("s").unwrap() {
        Value::StructArray(arr) => {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0].get("x"), Some(&Value::Scalar(1.0)));
            assert_eq!(arr[1].get("x"), Some(&Value::Scalar(5.0)));
        }
        other => panic!("expected StructArray, got {other:?}"),
    }
}

#[test]
fn test_struct_array_index_read() {
    // s(1).x should return the scalar value 1.0
    let env = run_block("s(1).x = 1; s(2).x = 5");
    let stmts = parse_stmts("ans = s(1).x").unwrap();
    let mut env = env;
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "ans"), 1.0);
}

#[test]
fn test_struct_array_collect_field() {
    // s.x on a struct array returns a 1×N matrix when all fields are scalars
    let env = run_block("s(1).x = 1; s(2).x = 5");
    let stmts = parse_stmts("v = s.x").unwrap();
    let mut env = env;
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    match env.get("v").unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.nrows(), 1);
            assert_eq!(m.ncols(), 2);
            assert_eq!(m[[0, 0]], 1.0);
            assert_eq!(m[[0, 1]], 5.0);
        }
        other => panic!("expected Matrix for s.x, got {other:?}"),
    }
}

#[test]
fn test_struct_array_numel() {
    let env = run_block("s(1).x = 1; s(2).x = 5; s(3).x = 9");
    let stmts = parse_stmts("n = numel(s)").unwrap();
    let mut env = env;
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "n"), 3.0);
}

#[test]
fn test_struct_array_isstruct() {
    let env = run_block("s(1).x = 1; s(2).x = 5");
    let stmts = parse_stmts("r = isstruct(s)").unwrap();
    let mut env = env;
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    assert_eq!(scalar(&env, "r"), 1.0);
}

#[test]
fn test_struct_array_fieldnames() {
    let env = run_block("s(1).x = 1; s(1).y = 2; s(2).x = 3; s(2).y = 4");
    let stmts = parse_stmts("fn = fieldnames(s)").unwrap();
    let mut env = env;
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    match env.get("fn").unwrap() {
        Value::Cell(v) => {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0], Value::Str("x".to_string()));
            assert_eq!(v[1], Value::Str("y".to_string()));
        }
        other => panic!("expected Cell, got {other:?}"),
    }
}

#[test]
fn test_struct_array_growing() {
    // Assigning s(3).x = 7 when s only has 2 elements should grow it to 3
    let env = run_block("s(1).x = 1; s(2).x = 5; s(3).x = 7");
    match env.get("s").unwrap() {
        Value::StructArray(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[2].get("x"), Some(&Value::Scalar(7.0)));
        }
        other => panic!("expected StructArray, got {other:?}"),
    }
}

#[test]
fn test_struct_array_collect_mixed_field_gives_cell() {
    // When field values are not all scalars, s.field returns a Cell
    let env = run_block("s(1).name = 'Alice'; s(2).name = 'Bob'");
    let stmts = parse_stmts("c = s.name").unwrap();
    let mut env = env;
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::Short,
        Base::Dec,
        true,
    )
    .unwrap();
    match env.get("c").unwrap() {
        Value::Cell(v) => {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0], Value::Str("Alice".to_string()));
            assert_eq!(v[1], Value::Str("Bob".to_string()));
        }
        other => panic!("expected Cell, got {other:?}"),
    }
}

// ── Phase 13.6a — Backslash operator (left division / linear solve) ──────────

#[test]
fn test_ldiv_scalar() {
    // a \ b = b / a  (use raw strings to avoid Rust escape interpretation)
    assert_eq!(calc(r"3 \ 12"), 4.0);
    assert_eq!(calc(r"2 \ 8"), 4.0);
    assert_eq!(calc(r"4 \ 1"), 0.25);
}

#[test]
fn test_ldiv_scalar_precedence() {
    // Same precedence as *, left-associative:
    // 6 \ 12 * 2  =>  (6 \ 12) * 2  =>  2 * 2  =>  4
    assert_eq!(calc(r"6 \ 12 * 2"), 4.0);
}

#[test]
fn test_ldiv_zero_divisor() {
    let env = Env::new();
    let result = parse(r"0 \ 5").and_then(|s| match s {
        Stmt::Expr(e) => eval(&e, &env),
        _ => Err("unexpected".to_string()),
    });
    assert!(result.is_err(), "0 \\ 5 should be an error");
}

#[test]
fn test_ldiv_matrix_solve() {
    // A = [2 1; 1 3]; b = [5; 10] => x = [1; 3]
    use crate::eval::{Base, FormatMode};
    use crate::exec::exec_stmts;
    use crate::io::IoContext;
    let stmts = crate::parser::parse_stmts(r"A = [2 1; 1 3]; b = [5; 10]; x = A \ b").unwrap();
    let mut env = Env::new();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::default(),
        Base::Dec,
        false,
    )
    .unwrap();
    match env.get("x").unwrap() {
        Value::Matrix(m) => {
            assert_eq!(m.nrows(), 2);
            assert_eq!(m.ncols(), 1);
            assert!((m[[0, 0]] - 1.0).abs() < 1e-10, "x[0] = {}", m[[0, 0]]);
            assert!((m[[1, 0]] - 3.0).abs() < 1e-10, "x[1] = {}", m[[1, 0]]);
        }
        other => panic!("expected Matrix, got {other:?}"),
    }
}

#[test]
fn test_ldiv_matrix_solve_identity() {
    // I \ b = b
    use crate::eval::{Base, FormatMode};
    use crate::exec::exec_stmts;
    use crate::io::IoContext;
    let stmts = crate::parser::parse_stmts(r"b = [3; 7]; x = eye(2) \ b").unwrap();
    let mut env = Env::new();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::default(),
        Base::Dec,
        false,
    )
    .unwrap();
    match env.get("x").unwrap() {
        Value::Matrix(m) => {
            assert!((m[[0, 0]] - 3.0).abs() < 1e-10);
            assert!((m[[1, 0]] - 7.0).abs() < 1e-10);
        }
        other => panic!("expected Matrix, got {other:?}"),
    }
}

#[test]
fn test_ldiv_scalar_times_matrix() {
    // 2 \ [4; 8] = [2; 4]
    use crate::eval::{Base, FormatMode};
    use crate::exec::exec_stmts;
    use crate::io::IoContext;
    let stmts = crate::parser::parse_stmts(r"x = 2 \ [4; 8]").unwrap();
    let mut env = Env::new();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::default(),
        Base::Dec,
        false,
    )
    .unwrap();
    match env.get("x").unwrap() {
        Value::Matrix(m) => {
            assert!((m[[0, 0]] - 2.0).abs() < 1e-10);
            assert!((m[[1, 0]] - 4.0).abs() < 1e-10);
        }
        other => panic!("expected Matrix, got {other:?}"),
    }
}

// ── Phase 13.6b — Path system ────────────────────────────────────────────────

#[test]
fn test_session_path_init_and_list() {
    use crate::exec::{session_path_init, session_path_list};
    let paths = vec![
        std::path::PathBuf::from("/a/b"),
        std::path::PathBuf::from("/c/d"),
    ];
    session_path_init(paths.clone());
    let got = session_path_list();
    assert_eq!(got, paths);
    // cleanup
    session_path_init(vec![]);
}

#[test]
fn test_session_path_add_prepend() {
    use crate::exec::{session_path_add, session_path_init, session_path_list};
    session_path_init(vec![std::path::PathBuf::from("/existing")]);
    session_path_add(std::path::PathBuf::from("/new"), false); // prepend
    let got = session_path_list();
    assert_eq!(got[0], std::path::PathBuf::from("/new"));
    assert_eq!(got[1], std::path::PathBuf::from("/existing"));
    session_path_init(vec![]);
}

#[test]
fn test_session_path_add_append() {
    use crate::exec::{session_path_add, session_path_init, session_path_list};
    session_path_init(vec![std::path::PathBuf::from("/existing")]);
    session_path_add(std::path::PathBuf::from("/new"), true); // append
    let got = session_path_list();
    assert_eq!(got[0], std::path::PathBuf::from("/existing"));
    assert_eq!(got[1], std::path::PathBuf::from("/new"));
    session_path_init(vec![]);
}

#[test]
fn test_session_path_add_deduplicates() {
    use crate::exec::{session_path_add, session_path_init, session_path_list};
    session_path_init(vec![
        std::path::PathBuf::from("/a"),
        std::path::PathBuf::from("/b"),
    ]);
    // Adding /a again (prepend) should remove it from position 0, re-insert at front
    session_path_add(std::path::PathBuf::from("/a"), false);
    let got = session_path_list();
    assert_eq!(got.len(), 2, "no duplicates: {got:?}");
    assert_eq!(got[0], std::path::PathBuf::from("/a"));
    session_path_init(vec![]);
}

#[test]
fn test_session_path_remove() {
    use crate::exec::{session_path_init, session_path_list, session_path_remove};
    session_path_init(vec![
        std::path::PathBuf::from("/a"),
        std::path::PathBuf::from("/b"),
    ]);
    session_path_remove(std::path::Path::new("/a"));
    let got = session_path_list();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0], std::path::PathBuf::from("/b"));
    session_path_init(vec![]);
}

#[test]
fn test_addpath_via_exec() {
    use crate::eval::{Base, FormatMode};
    use crate::exec::{exec_stmts, session_path_init, session_path_list};
    use crate::io::IoContext;
    session_path_init(vec![]);
    let stmts = crate::parser::parse_stmts(r#"addpath('/tmp/mylib')"#).unwrap();
    let mut env = Env::new();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::default(),
        Base::Dec,
        true,
    )
    .unwrap();
    let got = session_path_list();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0], std::path::PathBuf::from("/tmp/mylib"));
    session_path_init(vec![]);
}

#[test]
fn test_rmpath_via_exec() {
    use crate::eval::{Base, FormatMode};
    use crate::exec::{exec_stmts, session_path_init, session_path_list};
    use crate::io::IoContext;
    session_path_init(vec![std::path::PathBuf::from("/tmp/mylib")]);
    let stmts = crate::parser::parse_stmts(r#"rmpath('/tmp/mylib')"#).unwrap();
    let mut env = Env::new();
    let mut io = IoContext::new();
    exec_stmts(
        &stmts,
        &mut env,
        &mut io,
        &FormatMode::default(),
        Base::Dec,
        true,
    )
    .unwrap();
    let got = session_path_list();
    assert!(got.is_empty());
    session_path_init(vec![]);
}

// ── Phase 14 — Error handling ────────────────────────────────────────────────

fn run_block_result(src: &str) -> Result<Env, String> {
    crate::exec::init();
    let stmts = parse_stmts(src).expect("parse_stmts failed");
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    env.insert("j".to_string(), Value::Complex(0.0, 1.0));
    let mut io = IoContext::new();
    exec_stmts(&stmts, &mut env, &mut io, &FormatMode::Short, Base::Dec, true)?;
    Ok(env)
}

#[test]
fn test_error_builtin_raises() {
    let result = run_block_result("error('something went wrong')");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("something went wrong"));
}

#[test]
fn test_error_builtin_format() {
    let result = run_block_result("error('expected %d args', 2)");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expected 2 args"));
}

#[test]
fn test_warning_does_not_stop_execution() {
    let env = run_block("warning('just a warning'); x = 42;");
    assert_eq!(scalar(&env, "x"), 42.0);
}

#[test]
fn test_lasterr_returns_empty_initially() {
    crate::eval::set_last_err("");
    let v = crate::eval::get_last_err();
    assert_eq!(v, "");
}

#[test]
fn test_lasterr_set_and_get() {
    crate::eval::set_last_err("test error");
    assert_eq!(crate::eval::get_last_err(), "test error");
    crate::eval::set_last_err("");
}

#[test]
fn test_try_catch_anonymous() {
    let env = run_block(
        "try\n  error('boom')\ncatch\n  x = 99;\nend",
    );
    assert_eq!(scalar(&env, "x"), 99.0);
}

#[test]
fn test_try_catch_named_binds_message() {
    let env = run_block(
        "try\n  error('oops')\ncatch e\n  msg = e.message;\nend",
    );
    match env.get("msg") {
        Some(Value::Str(s)) => assert_eq!(s, "oops"),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn test_try_no_error_skips_catch() {
    let env = run_block(
        "x = 1;\ntry\n  x = 2;\ncatch\n  x = 99;\nend",
    );
    assert_eq!(scalar(&env, "x"), 2.0);
}

#[test]
fn test_try_end_no_catch() {
    // try block with no catch arm — error is silently swallowed
    let env = run_block("x = 1;\ntry\n  error('silent')\nend");
    assert_eq!(scalar(&env, "x"), 1.0);
}

#[test]
fn test_try_functional_fallback() {
    let env = run_block("x = try(1/0, -1);");
    assert!(scalar(&env, "x").is_infinite() || scalar(&env, "x") == -1.0);
}

#[test]
fn test_try_functional_catches_error() {
    let env = run_block("x = try(error('bad'), 42);");
    assert_eq!(scalar(&env, "x"), 42.0);
}

#[test]
fn test_try_functional_no_error_returns_value() {
    let env = run_block("x = try(2 + 3, 99);");
    assert_eq!(scalar(&env, "x"), 5.0);
}

#[test]
fn test_pcall_success() {
    let env = run_block("f = @(x) x * 2; [ok, v] = pcall(f, 5);");
    assert_eq!(scalar(&env, "ok"), 1.0);
    assert_eq!(scalar(&env, "v"), 10.0);
}

#[test]
fn test_pcall_failure() {
    let env = run_block("[ok, msg] = pcall(@(x) error('bad %d', x), 7);");
    assert_eq!(scalar(&env, "ok"), 0.0);
    match env.get("msg") {
        Some(Value::Str(s)) => assert!(s.contains("bad 7"), "msg = {s}"),
        other => panic!("expected Str msg, got {other:?}"),
    }
}

// ── Bug regression: split_stmts must handle '' (escaped quote) correctly ──

#[test]
fn test_split_stmts_escaped_quote_no_false_split() {
    // A string with '' and a comma inside must NOT be split on the comma.
    // Before fix: the '' confused in_sq tracking, making the comma appear
    // to be at depth-0, producing a spurious second statement.
    let stmts = crate::parser::split_stmts(r"fprintf('hello ''world'', no split')");
    assert_eq!(stmts.len(), 1, "must be one statement, got: {stmts:?}");
    assert_eq!(stmts[0].0, r"fprintf('hello ''world'', no split')");
}

#[test]
fn test_split_stmts_escaped_quote_with_semicolon_split() {
    // A real semicolon outside the string should still split.
    let stmts = crate::parser::split_stmts(r"x = 'it''s fine'; y = 2");
    assert_eq!(stmts.len(), 2, "must be two statements, got: {stmts:?}");
    assert!(stmts[0].1, "first stmt (x=...) should be silent");
    assert!(!stmts[1].1, "second stmt (y=2) should be non-silent");
}
