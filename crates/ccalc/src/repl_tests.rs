use super::*;

// --- who_format_columns tests ---

#[test]
fn test_who_columns_empty() {
    assert!(who_format_columns(&[], 80).is_empty());
}

#[test]
fn test_who_columns_single_entry() {
    let entries = vec!["x = 3.14".to_string()];
    assert_eq!(who_format_columns(&entries, 80), vec!["x = 3.14"]);
}

#[test]
fn test_who_columns_fits_one_row() {
    // Each entry "a = 1" is 5 chars, +2 = 7 col_width; 3 entries × 7 = 21 < 80 → one row
    let entries = vec![
        "a = 1".to_string(),
        "b = 2".to_string(),
        "c = 3".to_string(),
    ];
    let lines = who_format_columns(&entries, 80);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("a = 1"));
    assert!(lines[0].contains("b = 2"));
    assert!(lines[0].contains("c = 3"));
}

#[test]
fn test_who_columns_wraps_to_two_columns() {
    // col_width = len("val = 1") + 2 = 9; term_width = 18 → 2 cols, 3 rows
    let entries: Vec<String> = (1..=6).map(|i| format!("v{} = {}", i, i)).collect();
    let lines = who_format_columns(&entries, 18);
    // column-major: col0=[v1,v2,v3], col1=[v4,v5,v6]
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("v1 = 1"));
    assert!(lines[0].contains("v4 = 4"));
    assert!(lines[1].starts_with("v2 = 2"));
    assert!(lines[2].starts_with("v3 = 3"));
}

#[test]
fn test_who_columns_narrow_terminal_one_col() {
    // term_width smaller than any entry → max 1 column
    let entries = vec!["longname = 123".to_string(), "other = 456".to_string()];
    let lines = who_format_columns(&entries, 5);
    assert_eq!(lines.len(), 2);
}

// --- split_stmts tests ---

#[test]
fn test_split_stmts_single_no_semi() {
    assert_eq!(split_stmts("a = 1"), vec![("a = 1", false)]);
}

#[test]
fn test_split_stmts_single_trailing_semi() {
    assert_eq!(split_stmts("a = 1;"), vec![("a = 1", true)]);
}

#[test]
fn test_split_stmts_two_stmts_last_no_semi() {
    assert_eq!(
        split_stmts("a = 1; b = 2"),
        vec![("a = 1", true), ("b = 2", false)]
    );
}

#[test]
fn test_split_stmts_all_silent() {
    assert_eq!(
        split_stmts("a = 1; b = 2; c = 3;"),
        vec![("a = 1", true), ("b = 2", true), ("c = 3", true)]
    );
}

#[test]
fn test_split_stmts_comment_stripped() {
    assert_eq!(split_stmts("10; % comment"), vec![("10", true)]);
    assert_eq!(split_stmts("10 % comment"), vec![("10", false)]);
}

#[test]
fn test_split_stmts_empty_input() {
    assert_eq!(split_stmts(""), Vec::<(&str, bool)>::new());
    assert_eq!(split_stmts("  % only comment"), Vec::<(&str, bool)>::new());
}

#[test]
fn test_split_stmts_whitespace_segments_skipped() {
    // Double semicolon creates empty segment — should be ignored
    assert_eq!(
        split_stmts("a = 1;; b = 2"),
        vec![("a = 1", true), ("b = 2", false)]
    );
}

#[test]
fn test_split_stmts_semi_in_string_not_split() {
    // ';' inside a string literal must not split the statement
    assert_eq!(
        split_stmts("fprintf('hello; world')"),
        vec![("fprintf('hello; world')", false)]
    );
}

#[test]
fn test_split_stmts_semi_in_matrix_not_split() {
    // ';' inside '[...]' must not split the statement
    assert_eq!(split_stmts("[1 2; 3 4]"), vec![("[1 2; 3 4]", false)]);
}

#[test]
fn test_split_stmts_transpose_semi_splits() {
    // R' * q; — the ';' must not be swallowed by the transpose apostrophe
    assert_eq!(split_stmts("p = R' * q;"), vec![("p = R' * q", true)]);
}

#[test]
fn test_split_stmts_transpose_multi_stmt() {
    // A'; B — two statements
    assert_eq!(split_stmts("A'; B"), vec![("A'", true), ("B", false)]);
}

#[test]
fn test_split_stmts_matrix_then_semi() {
    // Matrix literal followed by outer ';'
    assert_eq!(
        split_stmts("A = [1 2; 3 4]; B = 5"),
        vec![("A = [1 2; 3 4]", true), ("B = 5", false)]
    );
}

// --- extract_base_suffix tests ---

#[test]
fn test_extract_base_suffix_hex() {
    let (expr, suffix) = extract_base_suffix("255 hex");
    assert_eq!(expr, "255");
    assert_eq!(suffix, Some(BaseSuffix::Switch(Base::Hex)));
}

#[test]
fn test_extract_base_suffix_bin() {
    let (expr, suffix) = extract_base_suffix("10 bin");
    assert_eq!(expr, "10");
    assert_eq!(suffix, Some(BaseSuffix::Switch(Base::Bin)));
}

#[test]
fn test_extract_base_suffix_oct() {
    let (expr, suffix) = extract_base_suffix("8 oct");
    assert_eq!(expr, "8");
    assert_eq!(suffix, Some(BaseSuffix::Switch(Base::Oct)));
}

#[test]
fn test_extract_base_suffix_dec() {
    let (expr, suffix) = extract_base_suffix("255 dec");
    assert_eq!(expr, "255");
    assert_eq!(suffix, Some(BaseSuffix::Switch(Base::Dec)));
}

#[test]
fn test_extract_base_suffix_show_all() {
    let (expr, suffix) = extract_base_suffix("10 base");
    assert_eq!(expr, "10");
    assert_eq!(suffix, Some(BaseSuffix::ShowAll));
}

#[test]
fn test_extract_base_suffix_none() {
    let (expr, suffix) = extract_base_suffix("255 + 10");
    assert_eq!(expr, "255 + 10");
    assert!(suffix.is_none());
}

#[test]
fn test_extract_base_suffix_complex() {
    let (expr, suffix) = extract_base_suffix("0xFF + 0b1010 hex");
    assert_eq!(expr, "0xFF + 0b1010");
    assert_eq!(suffix, Some(BaseSuffix::Switch(Base::Hex)));
}

#[test]
fn test_extract_base_suffix_no_space() {
    let (expr, suffix) = extract_base_suffix("hex");
    assert_eq!(expr, "hex");
    assert!(suffix.is_none());
}

// --- format_for_base tests ---

#[test]
fn test_format_for_base_hex() {
    assert_eq!(format_for_base(10.0, Base::Hex), "0xA");
    assert_eq!(format_for_base(255.0, Base::Hex), "0xFF");
    assert_eq!(format_for_base(0.0, Base::Hex), "0x0");
}

#[test]
fn test_format_for_base_bin() {
    assert_eq!(format_for_base(10.0, Base::Bin), "0b1010");
    assert_eq!(format_for_base(1.0, Base::Bin), "0b1");
}

#[test]
fn test_format_for_base_oct() {
    assert_eq!(format_for_base(8.0, Base::Oct), "0o10");
    assert_eq!(format_for_base(255.0, Base::Oct), "0o377");
}

#[test]
fn test_format_for_base_dec() {
    assert_eq!(format_for_base(42.0, Base::Dec), "42");
    assert_eq!(format_for_base(3.14, Base::Dec), "3.14");
}

// --- format_expr_for_display tests ---

#[test]
fn test_format_expr_hex_converts_bin_and_dec() {
    assert_eq!(
        format_expr_for_display("0xFF + 0b1010 + 10", Base::Hex),
        Some("0xFF + 0xA + 0xA".to_string())
    );
}

#[test]
fn test_format_expr_hex_keeps_hex_literals() {
    assert_eq!(
        format_expr_for_display("0xFF + 0b1010", Base::Hex),
        Some("0xFF + 0xA".to_string())
    );
}

#[test]
fn test_format_expr_dec_converts_hex() {
    assert_eq!(
        format_expr_for_display("0xFF + 10", Base::Dec),
        Some("255 + 10".to_string())
    );
}

#[test]
fn test_format_expr_no_change_when_all_match() {
    assert_eq!(format_expr_for_display("10 + 5", Base::Dec), None);
    assert_eq!(format_expr_for_display("0xFF + 0xA", Base::Hex), None);
}

#[test]
fn test_format_expr_preserves_identifiers() {
    assert_eq!(
        format_expr_for_display("sin(pi) + 0b1010", Base::Hex),
        Some("sin(pi) + 0xA".to_string())
    );
}

#[test]
fn test_format_expr_bin_accumulator_mixed_bases() {
    assert_eq!(
        format_expr_for_display("2 + 0b110 + 0xa", Base::Bin),
        Some("0b10 + 0b110 + 0b1010".to_string())
    );
}

#[test]
fn test_format_expr_hex_accumulator_bin_literals() {
    assert_eq!(
        format_expr_for_display("0b11 + 0b11", Base::Hex),
        Some("0x3 + 0x3".to_string())
    );
}

// --- parse_precision_cmd tests ---

#[test]
fn test_parse_precision_cmd_valid() {
    assert_eq!(parse_precision_cmd("p6"), Some(6));
    assert_eq!(parse_precision_cmd("p0"), Some(0));
    assert_eq!(parse_precision_cmd("p15"), Some(15));
    assert_eq!(parse_precision_cmd("p10"), Some(10));
}

#[test]
fn test_parse_precision_cmd_invalid() {
    assert_eq!(parse_precision_cmd("p"), None);
    assert_eq!(parse_precision_cmd("p16"), None);
    assert_eq!(parse_precision_cmd("pi"), None);
    assert_eq!(parse_precision_cmd("6"), None);
}

// --- parse_disp_cmd tests ---

#[test]
fn test_parse_disp_cmd_simple() {
    assert_eq!(parse_disp_cmd("disp(42)"), Some("42"));
    assert_eq!(parse_disp_cmd("disp(x + 1)"), Some("x + 1"));
    assert_eq!(parse_disp_cmd("disp(sin(pi/2))"), Some("sin(pi/2)"));
}

#[test]
fn test_parse_disp_cmd_not_matched() {
    assert!(parse_disp_cmd("display(42)").is_none());
    assert!(parse_disp_cmd("disp()").is_none());
    assert!(parse_disp_cmd("disp 42").is_none());
}

// --- parse_fprintf_cmd tests ---

#[test]
fn test_parse_fprintf_cmd_string() {
    assert_eq!(parse_fprintf_cmd("fprintf('hello')"), Some("'hello'"));
    assert_eq!(parse_fprintf_cmd("fprintf(\"hi\")"), Some("\"hi\""));
}

#[test]
fn test_parse_fprintf_cmd_not_matched() {
    assert!(parse_fprintf_cmd("printf('x')").is_none());
    assert!(parse_fprintf_cmd("fprintf 'x'").is_none());
}

// --- process_escapes tests ---

#[test]
fn test_process_escapes_newline() {
    assert_eq!(process_escapes("a\\nb"), "a\nb");
}

#[test]
fn test_process_escapes_tab() {
    assert_eq!(process_escapes("a\\tb"), "a\tb");
}

#[test]
fn test_process_escapes_backslash() {
    assert_eq!(process_escapes("a\\\\b"), "a\\b");
}

#[test]
fn test_process_escapes_no_escape() {
    assert_eq!(process_escapes("hello"), "hello");
}

// --- expand_vars_for_display tests ---

#[test]
fn test_expand_vars_no_vars() {
    let env = new_env();
    assert_eq!(expand_vars_for_display("2 + 3", &env, Base::Dec), None);
}

#[test]
fn test_expand_vars_single() {
    let mut env = new_env();
    env.insert("x".to_string(), Value::Scalar(10.0));
    assert_eq!(
        expand_vars_for_display("x + 5", &env, Base::Dec),
        Some("10 + 5".to_string())
    );
}

#[test]
fn test_expand_vars_multiple() {
    let mut env = new_env();
    env.insert("ans".to_string(), Value::Scalar(13.0));
    env.insert("x".to_string(), Value::Scalar(10.0));
    env.insert("y".to_string(), Value::Scalar(20.0));
    assert_eq!(
        expand_vars_for_display("ans + x + y", &env, Base::Dec),
        Some("13 + 10 + 20".to_string())
    );
}

#[test]
fn test_expand_vars_unknown_ident_preserved() {
    let mut env = new_env();
    env.insert("x".to_string(), Value::Scalar(5.0));
    // sqrt is not in env — should stay as-is
    assert_eq!(
        expand_vars_for_display("sqrt(x)", &env, Base::Dec),
        Some("sqrt(5)".to_string())
    );
}

#[test]
fn test_expand_vars_in_hex_base() {
    let mut env = new_env();
    env.insert("x".to_string(), Value::Scalar(255.0));
    assert_eq!(
        expand_vars_for_display("x + 1", &env, Base::Hex),
        Some("0xFF + 1".to_string())
    );
}

#[test]
fn test_expand_vars_matrix_not_expanded() {
    // Build a matrix value via evaluate so we don't need ndarray directly
    let mut env = new_env();
    evaluate("[1 2; 3 4]", &mut env).unwrap();
    // ans is now a matrix — move it to "m"
    let mat_val = env.get("ans").unwrap().clone();
    env.insert("m".to_string(), mat_val);
    // Matrix variables are not substituted in display expressions
    assert_eq!(expand_vars_for_display("m + 1", &env, Base::Dec), None);
}

// --- evaluate tests ---

#[test]
fn test_evaluate_simple() {
    let mut env = Env::new();
    let result = evaluate("3 * 4", &mut env).unwrap();
    assert!(matches!(result, EvalResult::Value(Value::Scalar(12.0))));
    assert_eq!(ans(&env), 12.0);
}

#[test]
fn test_evaluate_partial_adds_to_ans() {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(10.0));
    let result = evaluate("+ 5", &mut env).unwrap();
    assert!(matches!(result, EvalResult::Value(Value::Scalar(15.0))));
    assert_eq!(ans(&env), 15.0);
}

#[test]
fn test_evaluate_assignment() {
    let mut env = Env::new();
    let result = evaluate("x = 7", &mut env).unwrap();
    assert!(
        matches!(&result, EvalResult::Assigned(n, Value::Scalar(v)) if n == "x" && *v == 7.0)
    );
    assert_eq!(env.get("x"), Some(&Value::Scalar(7.0)));
}

#[test]
fn test_evaluate_expression_always_updates_ans() {
    let mut env = new_env();
    let result = evaluate("3 * 4", &mut env).unwrap();
    assert!(matches!(result, EvalResult::Value(Value::Scalar(12.0))));
    assert_eq!(ans(&env), 12.0);
}

#[test]
fn test_evaluate_assignment_does_not_update_ans() {
    let mut env = new_env();
    let result = evaluate("x = 7", &mut env).unwrap();
    assert!(
        matches!(&result, EvalResult::Assigned(n, Value::Scalar(v)) if n == "x" && *v == 7.0)
    );
    assert_eq!(env.get("x"), Some(&Value::Scalar(7.0)));
    assert_eq!(ans(&env), 0.0);
}

#[test]
fn test_evaluate_sets_base_via_suffix() {
    let mut env = Env::new();
    let mut base = Base::Dec;
    let (to_eval, suffix) = extract_base_suffix("255 hex");
    if let Some(BaseSuffix::Switch(b)) = suffix {
        base = b;
    }
    evaluate(to_eval, &mut env).unwrap();
    assert_eq!(base, Base::Hex);
    assert_eq!(ans(&env), 255.0);
}

// --- pipe_output helper + tests ---

fn pipe_output(input: &str) -> Vec<String> {
    use ccalc_engine::eval::eval;
    use ccalc_engine::parser::{Stmt, parse};
    use std::io::Cursor;

    let mut output = Vec::new();
    let mut env = new_env();
    let mut precision: usize = 10;
    let mut base = Base::Dec;
    let reader = Cursor::new(input);

    'lines: for line in reader.lines() {
        let line = line.unwrap();
        let trimmed = line.trim();

        for (stmt, silent) in split_stmts(trimmed) {
            match stmt {
                "exit" | "quit" => break 'lines,
                "clear" => {
                    env.clear();
                    continue;
                }
                "cls" | "who" => continue,
                "hex" => {
                    base = Base::Hex;
                    continue;
                }
                "dec" => {
                    base = Base::Dec;
                    continue;
                }
                "bin" => {
                    base = Base::Bin;
                    continue;
                }
                "oct" => {
                    base = Base::Oct;
                    continue;
                }
                _ => {}
            }
            if let Some(name) = stmt.strip_prefix("clear ").map(str::trim) {
                if !name.is_empty() {
                    env.remove(name);
                }
                continue;
            }
            if let Some(p) = parse_precision_cmd(stmt) {
                precision = p;
                continue;
            }
            // disp(expr) — push formatted value without updating ans
            if let Some(arg) = parse_disp_cmd(stmt) {
                let result = parse(arg.trim()).and_then(|stmt| {
                    let expr = match stmt {
                        Stmt::Expr(e) => e,
                        Stmt::Assign(_, e) => e,
                    };
                    eval(&expr, &env)
                });
                match result {
                    Ok(v) => match &v {
                        Value::Matrix(_) => {
                            if let Some(full) = format_value_full(&v, precision) {
                                output.push(full);
                            }
                        }
                        Value::Scalar(n) => output.push(format_scalar(*n, precision, base)),
                    },
                    Err(e) => output.push(format!("Error: {e}")),
                }
                continue;
            }
            // fprintf('fmt') — push processed string
            if let Some(arg) = parse_fprintf_cmd(stmt) {
                let s = arg.trim();
                let content = if let Some(inner) =
                    s.strip_prefix('\'').and_then(|s| s.strip_suffix('\''))
                {
                    process_escapes(inner)
                } else if let Some(inner) =
                    s.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
                {
                    process_escapes(inner)
                } else {
                    "Error: fprintf requires a string literal".to_string()
                };
                output.push(content);
                continue;
            }
            let (to_eval, base_suffix) = extract_base_suffix(stmt);
            let show_all = matches!(base_suffix, Some(BaseSuffix::ShowAll));
            if let Some(BaseSuffix::Switch(b)) = base_suffix {
                base = b;
            }
            match evaluate(to_eval, &mut env) {
                Ok(result) => {
                    if !silent {
                        match result {
                            EvalResult::Assigned(name, v) => match &v {
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&v, precision) {
                                        output.push(format!("{name} ="));
                                        output.push(full);
                                    }
                                }
                                Value::Scalar(n) => {
                                    output.push(format!(
                                        "{} = {}",
                                        name,
                                        format_scalar(*n, precision, base)
                                    ));
                                }
                            },
                            EvalResult::Value(v) => match &v {
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&v, precision) {
                                        output.push("ans =".to_string());
                                        output.push(full);
                                    }
                                }
                                Value::Scalar(n) => {
                                    if show_all {
                                        let i = n.round() as i64;
                                        let u = i.unsigned_abs();
                                        let sign = if i < 0 { "-" } else { "" };
                                        output.push(format!("2  - {}0b{:b}", sign, u));
                                        output.push(format!("8  - {}0o{:o}", sign, u));
                                        output.push(format!(
                                            "10 - {}",
                                            format_scalar(*n, precision, Base::Dec)
                                        ));
                                        output.push(format!("16 - {}0x{:X}", sign, u));
                                    } else {
                                        output.push(format_scalar(*n, precision, base));
                                    }
                                }
                            },
                        }
                    }
                }
                Err(e) => output.push(format!("Error: {e}")),
            }
        }
    }
    output
}

#[test]
fn test_pipe_simple_expression() {
    assert_eq!(pipe_output("2 + 2"), vec!["4"]);
}

#[test]
fn test_pipe_power() {
    assert_eq!(pipe_output("2 ^ 32"), vec!["4294967296"]);
}

#[test]
fn test_pipe_sqrt() {
    assert_eq!(pipe_output("sqrt(2)"), vec!["1.4142135624"]);
}

#[test]
fn test_pipe_multi_line_accumulates() {
    let lines = "10\n+ 5\n* 2";
    assert_eq!(pipe_output(lines), vec!["10", "15", "30"]);
}

#[test]
fn test_pipe_quit_with_exit() {
    let lines = "1\n2\nexit\n3";
    assert_eq!(pipe_output(lines), vec!["1", "2"]);
}

#[test]
fn test_pipe_quit_with_quit() {
    let lines = "1\n2\nquit\n3";
    assert_eq!(pipe_output(lines), vec!["1", "2"]);
}

#[test]
fn test_pipe_empty_lines_skipped() {
    let lines = "1\n\n\n2";
    assert_eq!(pipe_output(lines), vec!["1", "2"]);
}

#[test]
fn test_pipe_comments_skipped() {
    let lines = "% header comment\n1\n% inline comment\n+ 2\n% trailing comment";
    assert_eq!(pipe_output(lines), vec!["1", "3"]);
}

#[test]
fn test_pipe_inline_comments_stripped() {
    let lines = "10  % first value\n+ 5 % add five";
    assert_eq!(pipe_output(lines), vec!["10", "15"]);
}

#[test]
fn test_pipe_error_reported() {
    let out = pipe_output("1 / 0");
    assert!(out[0].starts_with("Error:"));
}

#[test]
fn test_pipe_variable_assignment() {
    let lines = "x = 7\nx + 3";
    assert_eq!(pipe_output(lines), vec!["x = 7", "10"]);
}

#[test]
fn test_pipe_hex_literals() {
    assert_eq!(pipe_output("0xFF"), vec!["255"]);
    assert_eq!(pipe_output("0xFF + 0b1010"), vec!["265"]);
}

#[test]
fn test_pipe_hex_base_suffix_changes_display() {
    let lines = "0xFF + 0b1010 hex";
    assert_eq!(pipe_output(lines), vec!["0x109"]);
}

#[test]
fn test_pipe_base_persists() {
    let lines = "0xFF + 0b1010 hex\n+ 0b10";
    assert_eq!(pipe_output(lines), vec!["0x109", "0x10B"]);
}

#[test]
fn test_pipe_base_switch_dec() {
    let lines = "255 hex\ndec";
    let out = pipe_output(lines);
    assert_eq!(out, vec!["0xFF"]);
}

#[test]
fn test_pipe_bin_literals() {
    assert_eq!(pipe_output("0b1010"), vec!["10"]);
}

#[test]
fn test_pipe_oct_literals() {
    assert_eq!(pipe_output("0o17"), vec!["15"]);
}

#[test]
fn test_pipe_base_suffix_shows_all() {
    let out = pipe_output("10 base");
    assert_eq!(out, vec!["2  - 0b1010", "8  - 0o12", "10 - 10", "16 - 0xA"]);
}

#[test]
fn test_pipe_base_suffix_evaluates_expression() {
    let out = pipe_output("0xFF + 0b1010 base");
    assert_eq!(
        out,
        vec!["2  - 0b100001001", "8  - 0o411", "10 - 265", "16 - 0x109"]
    );
}

#[test]
fn test_pipe_base_suffix_accumulator_set() {
    let out = pipe_output("10 base\n+ 5");
    assert_eq!(out[4], "15");
}

#[test]
fn test_pipe_sci_partial_expression() {
    let out = pipe_output("1e-12\n* 1000");
    assert_eq!(out[0], "1e-12");
    assert_eq!(out[1], "0.000000001");
    let out2 = pipe_output("1e-12\n* 1000\n* 1000");
    assert_eq!(out2[2], "0.000001");
    let out3 = pipe_output("1e-12\n* 10");
    assert_eq!(out3[1], "1e-11");
}

#[test]
fn test_pipe_semicolon_suppresses_output() {
    // ';' suppresses output but ans IS still updated (MATLAB semantics)
    let out = pipe_output("10;\n+ 5");
    assert_eq!(out, vec!["15"]);
}

#[test]
fn test_pipe_semicolon_still_updates_ans() {
    // ';' suppresses output but ans is updated — disp(ans) sees 10
    let out = pipe_output("10;\ndisp(ans)");
    assert_eq!(out, vec!["10"]);
}

#[test]
fn test_pipe_semicolon_with_comment() {
    // Comment stripped; ';' suppresses output but ans is updated
    let out = pipe_output("10; % intermediate\ndisp(ans)");
    assert_eq!(out, vec!["10"]);
}

#[test]
fn test_pipe_semicolon_variable_store() {
    // 7; → ans=7 (silent). x = ans; → x=7, ans unchanged (assignment). x + 3 = 10.
    let out = pipe_output("7;\nx = ans;\nx + 3");
    assert_eq!(out, vec!["10"]);
}

#[test]
fn test_pipe_multi_stmt_last_shown() {
    // Only the last non-silent statement produces output;
    // assignments never update ans (MATLAB semantics)
    let out = pipe_output("a = 1; b = 2");
    assert_eq!(out, vec!["b = 2"]);
}

#[test]
fn test_pipe_multi_stmt_all_silent() {
    // Trailing ';' makes everything silent — no output at all
    let out = pipe_output("a = 1; b = 2;");
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn test_pipe_multi_stmt_ans_not_updated_by_assignment() {
    // Assignments never update ans; disp(ans) still sees 0
    let out = pipe_output("a = 5; b = 10\ndisp(ans)");
    assert_eq!(out, vec!["b = 10", "0"]);
}

#[test]
fn test_pipe_multi_stmt_expr_last() {
    // Expression as the last non-silent statement updates ans
    let out = pipe_output("x = 3; x * 2");
    assert_eq!(out, vec!["6"]);
}

// --- disp tests ---

#[test]
fn test_pipe_disp_simple() {
    let out = pipe_output("disp(42)");
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_pipe_disp_expression() {
    let out = pipe_output("disp(sqrt(16))");
    assert_eq!(out, vec!["4"]);
}

#[test]
fn test_pipe_disp_does_not_change_ans() {
    let out = pipe_output("10\ndisp(42)\n+ 5");
    assert_eq!(out, vec!["10", "42", "15"]);
}

#[test]
fn test_pipe_disp_variable() {
    let out = pipe_output("x = 7;\ndisp(x)");
    assert_eq!(out, vec!["7"]);
}

// --- fprintf tests ---

#[test]
fn test_pipe_fprintf_single_quotes() {
    let out = pipe_output("fprintf('hello\\n')");
    assert_eq!(out, vec!["hello\n"]);
}

#[test]
fn test_pipe_fprintf_double_quotes() {
    let out = pipe_output("fprintf(\"hi\\n\")");
    assert_eq!(out, vec!["hi\n"]);
}

#[test]
fn test_pipe_fprintf_no_newline() {
    let out = pipe_output("fprintf('result: ')");
    assert_eq!(out, vec!["result: "]);
}

#[test]
fn test_pipe_base_suffix_accumulator_set_uses_ans() {
    // Verify partial expression uses ans, not a stale accumulator
    let out = pipe_output("ans\n+ 5");
    assert_eq!(out, vec!["0", "5"]);
}

// --- Matrix pipe tests ---

#[test]
fn test_pipe_matrix_assignment() {
    let out = pipe_output("A = [1 2; 3 4]");
    // Should show "A =" and the matrix body
    assert_eq!(out[0], "A =");
    assert!(out[1].contains("1"));
    assert!(out[1].contains("2"));
}

#[test]
fn test_pipe_matrix_literal() {
    let out = pipe_output("[1 2 3]");
    assert_eq!(out[0], "ans =");
    assert!(out[1].contains("1"));
}

#[test]
fn test_pipe_matrix_semicolon_not_split() {
    // The ';' inside [1 2; 3 4] must not be treated as a statement separator
    let out = pipe_output("[1 2; 3 4]");
    assert_eq!(out[0], "ans =");
    // Two rows of output
    assert_eq!(out[1].lines().count(), 2);
}
