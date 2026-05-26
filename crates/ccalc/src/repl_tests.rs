use super::*;

fn test_io() -> IoContext {
    IoContext::new()
}

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

#[allow(clippy::approx_constant)]
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
    evaluate("[1 2; 3 4]", &mut env, &mut test_io()).unwrap();
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
    let result = evaluate("3 * 4", &mut env, &mut test_io()).unwrap();
    assert!(matches!(result, EvalResult::Value(_, Value::Scalar(12.0))));
    assert_eq!(ans(&env), 12.0);
}

#[test]
fn test_evaluate_partial_adds_to_ans() {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(10.0));
    let result = evaluate("+ 5", &mut env, &mut test_io()).unwrap();
    assert!(matches!(result, EvalResult::Value(_, Value::Scalar(15.0))));
    assert_eq!(ans(&env), 15.0);
}

#[test]
fn test_evaluate_assignment() {
    let mut env = Env::new();
    let result = evaluate("x = 7", &mut env, &mut test_io()).unwrap();
    assert!(matches!(&result, EvalResult::Assigned(n, Value::Scalar(v)) if n == "x" && *v == 7.0));
    assert_eq!(env.get("x"), Some(&Value::Scalar(7.0)));
}

#[test]
fn test_evaluate_expression_always_updates_ans() {
    let mut env = new_env();
    let result = evaluate("3 * 4", &mut env, &mut test_io()).unwrap();
    assert!(matches!(result, EvalResult::Value(_, Value::Scalar(12.0))));
    assert_eq!(ans(&env), 12.0);
}

#[test]
fn test_evaluate_assignment_does_not_update_ans() {
    let mut env = new_env();
    let result = evaluate("x = 7", &mut env, &mut test_io()).unwrap();
    assert!(matches!(&result, EvalResult::Assigned(n, Value::Scalar(v)) if n == "x" && *v == 7.0));
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
    evaluate(to_eval, &mut env, &mut test_io()).unwrap();
    assert_eq!(base, Base::Hex);
    assert_eq!(ans(&env), 255.0);
}

// --- pipe_output helper + tests ---

fn pipe_output(input: &str) -> Vec<String> {
    use ccalc_engine::eval::{FormatMode, eval};
    use ccalc_engine::parser::{Stmt, parse};
    use std::io::Cursor;

    let mut output = Vec::new();
    let mut env = new_env();
    let fmt = FormatMode::default();
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
            if let Some(cmd) = try_parse_save_load(stmt, &env) {
                use ccalc_engine::env::{load_workspace, save_workspace, save_workspace_vars};
                match cmd {
                    SaveLoadCmd::Save { path, vars } => {
                        let _ = match (&path, vars.is_empty()) {
                            (None, _) => save_workspace_default(&env),
                            (Some(p), true) => save_workspace(&env, std::path::Path::new(p)),
                            (Some(p), false) => {
                                let var_refs: Vec<&str> = vars.iter().map(String::as_str).collect();
                                save_workspace_vars(&env, std::path::Path::new(p), &var_refs)
                            }
                        };
                    }
                    SaveLoadCmd::Load { path } => {
                        let result = match path {
                            None => load_workspace_default(),
                            Some(p) => load_workspace(std::path::Path::new(&p)),
                        };
                        if let Ok(loaded) = result {
                            env = loaded;
                        }
                    }
                }
                continue;
            }
            // disp(expr) — push formatted value without updating ans
            if let Some(arg) = parse_disp_cmd(stmt) {
                let result = parse(arg.trim()).and_then(|stmt| {
                    let expr = match stmt {
                        Stmt::Expr(e) | Stmt::Assign(_, e) => e,
                        _ => return Err("Block statements not valid in disp()".to_string()),
                    };
                    eval(&expr, &env)
                });
                match result {
                    Ok(v) => match &v {
                        Value::Void => {}
                        Value::Matrix(_) | Value::ComplexMatrix(_) => {
                            if let Some(full) = format_value_full(&v, &fmt) {
                                output.push(full);
                            }
                        }
                        Value::Scalar(n) => output.push(format_scalar(*n, base, &fmt)),
                        Value::Complex(re, im) => output.push(format_complex(*re, *im, &fmt)),
                        Value::Str(s) | Value::StringObj(s) => output.push(s.clone()),
                        Value::Lambda(_) => output.push("@<lambda>".to_string()),
                        Value::Function { .. } => output.push("@function".to_string()),
                        Value::Tuple(_) => {}
                        Value::Cell(_) | Value::Struct(_) | Value::StructArray(_) => {
                            if let Some(full) = format_value_full(&v, &fmt) {
                                output.push(full);
                            }
                        }
                        Value::DateTime(ts) => {
                            output.push(ccalc_engine::datetime::format_datetime(*ts));
                        }
                        Value::Duration(s) => {
                            output.push(ccalc_engine::datetime::format_duration(*s));
                        }
                        Value::DateTimeArray(_) | Value::DurationArray(_) => {
                            if let Some(full) = format_value_full(&v, &fmt) {
                                output.push(full);
                            }
                        }
                    },
                    Err(e) => output.push(format!("Error: {e}")),
                }
                continue;
            }
            let (to_eval, base_suffix) = extract_base_suffix(stmt);
            let show_all = matches!(base_suffix, Some(BaseSuffix::ShowAll));
            if let Some(BaseSuffix::Switch(b)) = base_suffix {
                base = b;
            }
            match evaluate(to_eval, &mut env, &mut test_io()) {
                Ok(result) => {
                    if !silent {
                        match result {
                            EvalResult::Assigned(name, v) => match &v {
                                Value::Void => {}
                                Value::Matrix(_) | Value::ComplexMatrix(_) => {
                                    if let Some(full) = format_value_full(&v, &fmt) {
                                        output.push(format!("{name} ="));
                                        output.push(full);
                                    }
                                }
                                Value::Scalar(n) => {
                                    output.push(format!(
                                        "{} = {}",
                                        name,
                                        format_scalar(*n, base, &fmt)
                                    ));
                                }
                                Value::Complex(re, im) => {
                                    output.push(format!(
                                        "{} = {}",
                                        name,
                                        format_complex(*re, *im, &fmt)
                                    ));
                                }
                                Value::Str(s) => output.push(format!("{name} = {s}")),
                                Value::StringObj(s) => output.push(format!("{name} = {s}")),
                                Value::Lambda(_) => output.push(format!("{name} = @<lambda>")),
                                Value::Function { .. } => {
                                    output.push(format!("{name} = @function"))
                                }
                                Value::Tuple(_) => {}
                                Value::Cell(_) | Value::Struct(_) | Value::StructArray(_) => {
                                    if let Some(full) = format_value_full(&v, &fmt) {
                                        output.push(format!("{name} ="));
                                        output.push(full);
                                    }
                                }
                                Value::DateTime(ts) => {
                                    output.push(format!(
                                        "{name} = {}",
                                        ccalc_engine::datetime::format_datetime(*ts)
                                    ));
                                }
                                Value::Duration(s) => {
                                    output.push(format!(
                                        "{name} = {}",
                                        ccalc_engine::datetime::format_duration(*s)
                                    ));
                                }
                                Value::DateTimeArray(_) | Value::DurationArray(_) => {
                                    if let Some(full) = format_value_full(&v, &fmt) {
                                        output.push(format!("{name} ="));
                                        output.push(full);
                                    }
                                }
                            },
                            EvalResult::Value(label, v) => {
                                let prefix = label.as_deref().unwrap_or("ans");
                                match &v {
                                    Value::Void => {}
                                    Value::Matrix(_) | Value::ComplexMatrix(_) => {
                                        if let Some(full) = format_value_full(&v, &fmt) {
                                            output.push(format!("{prefix} ="));
                                            output.push(full);
                                        }
                                    }
                                    Value::Scalar(n) => {
                                        if let Some(ref name) = label {
                                            output.push(format!(
                                                "{name} = {}",
                                                format_scalar(*n, base, &fmt)
                                            ));
                                        } else if show_all {
                                            let i = n.round() as i64;
                                            let u = i.unsigned_abs();
                                            let sign = if i < 0 { "-" } else { "" };
                                            output.push(format!("2  - {}0b{:b}", sign, u));
                                            output.push(format!("8  - {}0o{:o}", sign, u));
                                            output.push(format!(
                                                "10 - {}",
                                                format_scalar(*n, Base::Dec, &fmt)
                                            ));
                                            output.push(format!("16 - {}0x{:X}", sign, u));
                                        } else {
                                            output.push(format_scalar(*n, base, &fmt));
                                        }
                                    }
                                    Value::Complex(re, im) => {
                                        output.push(format_complex(*re, *im, &fmt));
                                    }
                                    Value::Str(s) | Value::StringObj(s) => output.push(s.clone()),
                                    Value::Lambda(_) => output.push("@<lambda>".to_string()),
                                    Value::Function { .. } => output.push("@function".to_string()),
                                    Value::Tuple(_) => {}
                                    Value::Cell(_) | Value::Struct(_) | Value::StructArray(_) => {
                                        if let Some(full) = format_value_full(&v, &fmt) {
                                            output.push(format!("{prefix} ="));
                                            output.push(full);
                                        }
                                    }
                                    Value::DateTime(ts) => {
                                        output.push(ccalc_engine::datetime::format_datetime(*ts));
                                    }
                                    Value::Duration(s) => {
                                        output.push(ccalc_engine::datetime::format_duration(*s));
                                    }
                                    Value::DateTimeArray(_) | Value::DurationArray(_) => {
                                        if let Some(full) = format_value_full(&v, &fmt) {
                                            output.push(format!("{prefix} ="));
                                            output.push(full);
                                        }
                                    }
                                } // match &v
                            } // EvalResult::Value
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
fn test_pipe_div_by_zero_gives_inf() {
    // IEEE 754: 1/0 = Inf, not an error.
    let out = pipe_output("1 / 0");
    assert_eq!(out, vec!["Inf"]);
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
fn test_pipe_fprintf_returns_void_no_captured_output() {
    // fprintf writes directly to stdout — no captured output in test harness
    let out = pipe_output("fprintf('hello\\n')");
    assert!(out.is_empty());
}

#[test]
fn test_pipe_fprintf_double_quotes_void() {
    let out = pipe_output("fprintf(\"hi\\n\")");
    assert!(out.is_empty());
}

#[test]
fn test_pipe_fprintf_with_arg_void() {
    let out = pipe_output("fprintf('%d\\n', 42)");
    assert!(out.is_empty());
}

#[test]
fn test_pipe_sprintf_single_quotes() {
    // sprintf returns a char array — captured in output
    let out = pipe_output("sprintf('hello\\n')");
    assert_eq!(out, vec!["hello\n"]);
}

#[test]
fn test_pipe_sprintf_format_arg() {
    let out = pipe_output("sprintf('%d items', 5)");
    assert_eq!(out, vec!["5 items"]);
}

#[test]
fn test_pipe_base_suffix_accumulator_set_uses_ans() {
    // Verify partial expression uses ans, not a stale accumulator.
    // Bare `ans` now shows "ans = 0" (variable name label, MATLAB semantics).
    let out = pipe_output("ans\n+ 5");
    assert_eq!(out, vec!["ans = 0", "5"]);
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

// --- Phase 10.5d: save / load with path ---

#[test]
fn test_try_parse_save_bare() {
    let env = ccalc_engine::env::Env::new();
    let cmd = try_parse_save_load("save", &env);
    assert!(matches!(cmd, Some(SaveLoadCmd::Save { path: None, .. })));
}

#[test]
fn test_try_parse_load_bare() {
    let env = ccalc_engine::env::Env::new();
    let cmd = try_parse_save_load("load", &env);
    assert!(matches!(cmd, Some(SaveLoadCmd::Load { path: None })));
}

#[test]
fn test_try_parse_save_with_path() {
    let env = ccalc_engine::env::Env::new();
    let cmd = try_parse_save_load("save('session.mat')", &env);
    match cmd {
        Some(SaveLoadCmd::Save {
            path: Some(p),
            vars,
        }) => {
            assert_eq!(p, "session.mat");
            assert!(vars.is_empty());
        }
        _ => panic!("expected Save with path"),
    }
}

#[test]
fn test_try_parse_save_with_vars() {
    let env = ccalc_engine::env::Env::new();
    let cmd = try_parse_save_load("save('out.mat', 'x', 'y')", &env);
    match cmd {
        Some(SaveLoadCmd::Save {
            path: Some(p),
            vars,
        }) => {
            assert_eq!(p, "out.mat");
            assert_eq!(vars, vec!["x", "y"]);
        }
        _ => panic!("expected Save with path and vars"),
    }
}

#[test]
fn test_try_parse_load_with_path() {
    let env = ccalc_engine::env::Env::new();
    let cmd = try_parse_save_load("load('session.mat')", &env);
    match cmd {
        Some(SaveLoadCmd::Load { path: Some(p) }) => assert_eq!(p, "session.mat"),
        _ => panic!("expected Load with path"),
    }
}

#[test]
fn test_try_parse_unrelated_returns_none() {
    let env = ccalc_engine::env::Env::new();
    assert!(try_parse_save_load("x = 3", &env).is_none());
    assert!(try_parse_save_load("fprintf('hi')", &env).is_none());
}

#[allow(clippy::approx_constant)]
#[test]
fn test_pipe_save_load_roundtrip() {
    use ccalc_engine::env::load_workspace;
    let tmp = std::env::temp_dir().join("ccalc_test_pipe_saveload.mat");
    let path = tmp.to_string_lossy().to_string();

    // Save x and y to file
    let save_script = format!("x = 42\ny = 3.14\nsave('{path}')");
    pipe_output(&save_script);

    // Load and check
    let loaded = load_workspace(std::path::Path::new(&path)).unwrap();
    assert_eq!(loaded.get("x"), Some(&Value::Scalar(42.0)));
    assert_eq!(loaded.get("y"), Some(&Value::Scalar(3.14)));

    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_pipe_save_selective_vars() {
    use ccalc_engine::env::load_workspace;
    let tmp = std::env::temp_dir().join("ccalc_test_pipe_save_selective.mat");
    let path = tmp.to_string_lossy().to_string();

    let save_script = format!("x = 1\ny = 2\nz = 3\nsave('{path}', 'x', 'z')");
    pipe_output(&save_script);

    let loaded = load_workspace(std::path::Path::new(&path)).unwrap();
    assert_eq!(loaded.get("x"), Some(&Value::Scalar(1.0)));
    assert_eq!(loaded.get("z"), Some(&Value::Scalar(3.0)));
    assert!(!loaded.contains_key("y"));

    std::fs::remove_file(&tmp).ok();
}

// --- render_prompt tests ---
//
// render_prompt returns (plain, colored):
//   plain   — no ANSI codes; used by rustyline for cursor width
//   colored — ANSI codes included; used by highlight_prompt for display
//
// Content placeholders appear in BOTH strings.
// Color/style placeholders appear in `colored` only; `plain` is unaffected.

fn make_env_with_ans(v: Value) -> Env {
    let mut env = new_env();
    env.insert("ans".to_string(), v);
    env
}

#[test]
fn test_render_prompt_literal() {
    let env = new_env();
    let (plain, colored) = render_prompt("$ ", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, "$ ");
    assert_eq!(colored, "$ ");
}

#[test]
fn test_render_prompt_ans_scalar() {
    let env = make_env_with_ans(Value::Scalar(42.0));
    let (plain, colored) = render_prompt("[{ans}] ", &env, 1, Base::Dec, &FormatMode::Short);
    assert!(plain.contains("42"), "expected '42' in plain '{plain}'");
    assert!(
        colored.contains("42"),
        "expected '42' in colored '{colored}'"
    );
}

#[test]
fn test_render_prompt_line_counter() {
    let env = new_env();
    let (plain, colored) = render_prompt("({line}) ", &env, 7, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, "(7) ");
    assert_eq!(colored, "(7) ");
}

#[test]
fn test_render_prompt_unknown_placeholder() {
    let env = new_env();
    let (plain, colored) = render_prompt("{foo}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, "{foo}");
    assert_eq!(colored, "{foo}");
}

#[test]
fn test_render_prompt_ansi_escape_passthrough() {
    // \e is plain text — pass through unchanged to both strings
    let env = new_env();
    let (plain, colored) = render_prompt(r"\e[0m", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, r"\e[0m");
    assert_eq!(colored, r"\e[0m");
}

#[test]
fn test_render_prompt_color_reset() {
    let env = new_env();
    let (plain, colored) = render_prompt("{reset}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, ""); // no visible text in plain
    assert_eq!(colored, "\x1b[0m"); // ANSI code in colored
}

#[test]
fn test_render_prompt_color_bold_and_dim() {
    let env = new_env();
    let (p, c) = render_prompt("{bold}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(p, "");
    assert_eq!(c, "\x1b[1m");
    let (p, c) = render_prompt("{dim}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(p, "");
    assert_eq!(c, "\x1b[2m");
}

#[test]
fn test_render_prompt_standard_colors() {
    let env = new_env();
    let cases = [
        ("{black}", "\x1b[30m"),
        ("{red}", "\x1b[31m"),
        ("{green}", "\x1b[32m"),
        ("{yellow}", "\x1b[33m"),
        ("{blue}", "\x1b[34m"),
        ("{magenta}", "\x1b[35m"),
        ("{cyan}", "\x1b[36m"),
        ("{white}", "\x1b[37m"),
    ];
    for (tmpl, expected_ansi) in cases {
        let (plain, colored) = render_prompt(tmpl, &env, 1, Base::Dec, &FormatMode::Short);
        assert_eq!(plain, "", "plain should be empty for {tmpl}");
        assert_eq!(colored, expected_ansi, "wrong ANSI for {tmpl}");
    }
}

#[test]
fn test_render_prompt_bright_colors() {
    let env = new_env();
    let cases = [
        ("{gray}", "\x1b[90m"),
        ("{bright_red}", "\x1b[91m"),
        ("{bright_green}", "\x1b[92m"),
        ("{bright_yellow}", "\x1b[93m"),
        ("{bright_blue}", "\x1b[94m"),
        ("{bright_magenta}", "\x1b[95m"),
        ("{bright_cyan}", "\x1b[96m"),
        ("{bright_white}", "\x1b[97m"),
    ];
    for (tmpl, expected_ansi) in cases {
        let (plain, colored) = render_prompt(tmpl, &env, 1, Base::Dec, &FormatMode::Short);
        assert_eq!(plain, "", "plain should be empty for {tmpl}");
        assert_eq!(colored, expected_ansi, "wrong ANSI for {tmpl}");
    }
}

#[test]
fn test_render_prompt_rgb_color() {
    let env = new_env();
    // {#FF8800} → truecolor orange foreground
    let (plain, colored) = render_prompt("{#FF8800}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, "");
    assert_eq!(colored, "\x1b[38;2;255;136;0m");
    // lowercase hex also works
    let (_, colored2) = render_prompt("{#ff8800}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(colored2, "\x1b[38;2;255;136;0m");
}

#[test]
fn test_render_prompt_color256() {
    let env = new_env();
    // {color256(220)} → 8-bit palette gold
    let (plain, colored) = render_prompt("{color256(220)}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, "");
    assert_eq!(colored, "\x1b[38;5;220m");
    // out-of-range N → unknown placeholder, passed through literally
    let (plain2, colored2) =
        render_prompt("{color256(999)}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain2, "{color256(999)}");
    assert_eq!(colored2, "{color256(999)}");
}

#[test]
fn test_render_prompt_rgb_invalid_passthrough() {
    let env = new_env();
    // Too short — treated as unknown placeholder, passed through to both
    let (plain, colored) = render_prompt("{#FFF}", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, "{#FFF}");
    assert_eq!(colored, "{#FFF}");
}

#[test]
fn test_render_prompt_color_splits_plain_and_colored() {
    // Key property: plain has no ANSI, colored has all content + ANSI
    let env = make_env_with_ans(Value::Scalar(1.0));
    let (plain, colored) = render_prompt(
        "{gray}({line}){reset} [ {ans} ]: ",
        &env,
        3,
        Base::Dec,
        &FormatMode::Short,
    );
    // plain: only visible text
    assert_eq!(plain, "(3) [ 1 ]: ");
    // colored: ANSI codes + same visible text
    assert!(
        colored.starts_with("\x1b[90m(3)\x1b[0m"),
        "got: {colored:?}"
    );
    assert!(colored.contains("1"), "ans missing in colored: {colored:?}");
}

#[test]
fn test_render_prompt_cwd_short_nonempty() {
    let env = new_env();
    let (plain, _) = render_prompt("{cwd_short}", &env, 1, Base::Dec, &FormatMode::Short);
    assert!(!plain.is_empty());
}

#[test]
fn test_render_prompt_time_format() {
    let env = new_env();
    let (plain, _) = render_prompt("{time}", &env, 1, Base::Dec, &FormatMode::Short);
    // HH:MM:SS — exactly 8 chars
    assert_eq!(plain.len(), 8, "time should be HH:MM:SS, got '{plain}'");
    assert_eq!(&plain[2..3], ":");
    assert_eq!(&plain[5..6], ":");
}

#[test]
fn test_render_prompt_unclosed_brace() {
    let env = new_env();
    // Unclosed brace is passed through literally to both
    let (plain, colored) = render_prompt("{ans", &env, 1, Base::Dec, &FormatMode::Short);
    assert_eq!(plain, "{ans");
    assert_eq!(colored, "{ans");
}
