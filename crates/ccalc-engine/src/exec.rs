use ndarray::Array2;

use crate::env::{Env, Value};
use crate::eval::{
    Base, Expr, FormatMode, eval_with_io, format_complex, format_scalar, format_value_full,
};
use crate::io::IoContext;
use crate::parser::{Stmt, parse_stmts};

thread_local! {
    /// Tracks the current script nesting depth to prevent infinite recursion via `run()`.
    static RUN_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Flow control signal returned by [`exec_stmts`].
///
/// Used to propagate `break` and `continue` through nested block calls.
/// The caller (loop implementation) catches the signal; uncaught signals at
/// the top level are reported as errors.
pub enum Signal {
    Break,
    Continue,
}

/// Resolves a script filename to an existing path.
///
/// If `name` already has an extension, it is used verbatim.
/// Otherwise, `.calc` is tried first (native ccalc format), then `.m` (Octave/MATLAB compatibility).
/// The search is relative to the current working directory.
fn resolve_script_path(name: &str) -> Option<std::path::PathBuf> {
    let p = std::path::Path::new(name);
    if p.extension().is_some() {
        return if p.exists() { Some(p.to_path_buf()) } else { None };
    }
    let with_calc = p.with_extension("calc");
    if with_calc.exists() {
        return Some(with_calc);
    }
    let with_m = p.with_extension("m");
    if with_m.exists() {
        return Some(with_m);
    }
    None
}

/// Returns `true` if `val` is considered truthy by MATLAB `if`/`while` semantics.
///
/// - Scalar: nonzero and not NaN.
/// - Matrix: all elements nonzero and not NaN.
/// - Complex: either part nonzero.
/// - Str/StringObj: nonempty.
/// - Void: always false.
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Scalar(n) => *n != 0.0 && !n.is_nan(),
        Value::Matrix(m) => m.iter().all(|&x| x != 0.0 && !x.is_nan()),
        Value::Complex(re, im) => *re != 0.0 || *im != 0.0,
        Value::Str(s) | Value::StringObj(s) => !s.is_empty(),
        Value::Void => false,
    }
}

/// Prints a value to stdout with MATLAB-style formatting.
///
/// `label` is `Some("name")` for assignment output and `None` for expression output.
/// In expression context scalars/complex print without a label; matrices print `ans =`.
fn print_value(label: Option<&str>, val: &Value, fmt: &FormatMode, base: Base, compact: bool) {
    match val {
        Value::Void => {}
        Value::Scalar(n) => {
            if let Some(name) = label {
                println!("{name} = {}", format_scalar(*n, base, fmt));
            } else {
                println!("{}", format_scalar(*n, base, fmt));
            }
        }
        Value::Matrix(_) => {
            if let Some(full) = format_value_full(val, fmt) {
                let prefix = label.unwrap_or("ans");
                println!("{prefix} =");
                println!("{full}");
                if !compact {
                    println!();
                }
            }
        }
        Value::Complex(re, im) => {
            if let Some(name) = label {
                println!("{name} = {}", format_complex(*re, *im, fmt));
            } else {
                println!("{}", format_complex(*re, *im, fmt));
            }
        }
        Value::Str(s) | Value::StringObj(s) => {
            if let Some(name) = label {
                println!("{name} = {s}");
            } else {
                println!("{s}");
            }
        }
    }
}

/// Executes a sequence of parsed statements, handling flow control signals.
///
/// Returns:
/// - `Ok(None)` — normal completion
/// - `Ok(Some(Signal::Break))` — `break` statement executed
/// - `Ok(Some(Signal::Continue))` — `continue` statement executed
/// - `Err(e)` — runtime error
///
/// Loop implementations (`For`, `While`) catch `Break`/`Continue` internally.
/// A signal that escapes to the top-level caller should be reported as an error.
pub fn exec_stmts(
    stmts: &[(Stmt, bool)],
    env: &mut Env,
    io: &mut IoContext,
    fmt: &FormatMode,
    base: Base,
    compact: bool,
) -> Result<Option<Signal>, String> {
    for (stmt, silent) in stmts {
        match stmt {
            Stmt::Assign(name, expr) => {
                let val = eval_with_io(expr, env, io)?;
                env.insert(name.clone(), val.clone());
                if !silent && !matches!(val, Value::Void) {
                    print_value(Some(name), &val, fmt, base, compact);
                }
            }

            Stmt::Expr(expr) => {
                // Intercept run()/source() — execute a script file in the current workspace.
                // Variables defined in the script persist in the caller's scope (MATLAB `run` semantics).
                if let Expr::Call(fn_name, args) = expr
                    && matches!(fn_name.as_str(), "run" | "source")
                    && args.len() == 1
                {
                    let path_val = eval_with_io(&args[0], env, io)?;
                    let filename = match &path_val {
                        Value::Str(s) | Value::StringObj(s) => s.clone(),
                        _ => {
                            return Err(format!(
                                "{fn_name}: argument must be a string (filename)"
                            ));
                        }
                    };
                    let script_path = resolve_script_path(&filename).ok_or_else(|| {
                        format!("{fn_name}: script not found: '{filename}'")
                    })?;
                    let content = std::fs::read_to_string(&script_path).map_err(|e| {
                        format!("{fn_name}: cannot read '{}': {e}", script_path.display())
                    })?;
                    let depth = RUN_DEPTH.with(|d| d.get());
                    if depth >= 64 {
                        return Err(format!(
                            "{fn_name}: maximum script nesting depth (64) exceeded"
                        ));
                    }
                    RUN_DEPTH.with(|d| d.set(depth + 1));
                    let run_stmts = parse_stmts(&content).map_err(|e| {
                        format!("{fn_name}: parse error in '{}': {e}", script_path.display())
                    })?;
                    let result = exec_stmts(&run_stmts, env, io, fmt, base, compact);
                    RUN_DEPTH.with(|d| d.set(depth));
                    return result;
                }

                let val = eval_with_io(expr, env, io)?;
                env.insert("ans".to_string(), val.clone());
                if !silent && !matches!(val, Value::Void) {
                    print_value(None, &val, fmt, base, compact);
                }
            }

            Stmt::If {
                cond,
                body,
                elseif_branches,
                else_body,
            } => {
                let cond_val = eval_with_io(cond, env, io)?;
                let chosen: Option<&[(Stmt, bool)]> = if is_truthy(&cond_val) {
                    Some(body)
                } else {
                    let mut found = None;
                    for (ei_cond, ei_body) in elseif_branches {
                        if is_truthy(&eval_with_io(ei_cond, env, io)?) {
                            found = Some(ei_body.as_slice());
                            break;
                        }
                    }
                    if found.is_none() {
                        found = else_body.as_deref();
                    }
                    found
                };
                if let Some(body_stmts) = chosen
                    && let Some(sig) = exec_stmts(body_stmts, env, io, fmt, base, compact)?
                {
                    return Ok(Some(sig));
                }
            }

            Stmt::For {
                var,
                range_expr,
                body,
            } => {
                let range_val = eval_with_io(range_expr, env, io)?;
                let iter_cols: Vec<Value> = match range_val {
                    Value::Scalar(n) => vec![Value::Scalar(n)],
                    Value::Matrix(m) => {
                        let nrows = m.nrows();
                        let ncols = m.ncols();
                        (0..ncols)
                            .map(|j| {
                                if nrows == 1 {
                                    // Row vector: yield each element as a scalar
                                    Value::Scalar(m[[0, j]])
                                } else {
                                    // General matrix: yield each column as an M×1 matrix
                                    let mut col = Array2::zeros((nrows, 1));
                                    for i in 0..nrows {
                                        col[[i, 0]] = m[[i, j]];
                                    }
                                    Value::Matrix(col)
                                }
                            })
                            .collect()
                    }
                    _ => return Err("'for' range must evaluate to a scalar or matrix".to_string()),
                };

                'for_loop: for col_val in iter_cols {
                    env.insert(var.clone(), col_val);
                    match exec_stmts(body, env, io, fmt, base, compact)? {
                        None => {}
                        Some(Signal::Break) => break 'for_loop,
                        Some(Signal::Continue) => continue 'for_loop,
                    }
                }
            }

            Stmt::While { cond, body } => loop {
                if !is_truthy(&eval_with_io(cond, env, io)?) {
                    break;
                }
                match exec_stmts(body, env, io, fmt, base, compact)? {
                    None => {}
                    Some(Signal::Break) => break,
                    Some(Signal::Continue) => continue,
                }
            },

            Stmt::Break => return Ok(Some(Signal::Break)),
            Stmt::Continue => return Ok(Some(Signal::Continue)),

            // ── switch / case / otherwise / end ──────────────────────────────
            Stmt::Switch { expr, cases, otherwise_body } => {
                let switch_val = eval_with_io(expr, env, io)?;
                let mut matched = false;
                'switch_loop: for (case_exprs, case_body) in cases {
                    for case_expr in case_exprs {
                        let case_val = eval_with_io(case_expr, env, io)?;
                        let is_match = match (&switch_val, &case_val) {
                            (Value::Scalar(a), Value::Scalar(b)) => a == b,
                            _ => {
                                let sv = match &switch_val {
                                    Value::Str(s) | Value::StringObj(s) => Some(s.as_str()),
                                    _ => None,
                                };
                                let cv = match &case_val {
                                    Value::Str(s) | Value::StringObj(s) => Some(s.as_str()),
                                    _ => None,
                                };
                                matches!((sv, cv), (Some(a), Some(b)) if a == b)
                            }
                        };
                        if is_match {
                            if let Some(sig) =
                                exec_stmts(case_body, env, io, fmt, base, compact)?
                            {
                                return Ok(Some(sig));
                            }
                            matched = true;
                            break 'switch_loop;
                        }
                    }
                }
                if !matched {
                    if let Some(ob) = otherwise_body {
                        if let Some(sig) = exec_stmts(ob, env, io, fmt, base, compact)? {
                            return Ok(Some(sig));
                        }
                    }
                }
            }

            // ── do...until ───────────────────────────────────────────────────
            Stmt::DoUntil { body, cond } => loop {
                match exec_stmts(body, env, io, fmt, base, compact)? {
                    Some(Signal::Break) => break,
                    Some(Signal::Continue) | None => {}
                }
                if is_truthy(&eval_with_io(cond, env, io)?) {
                    break;
                }
            },
        }
    }
    Ok(None)
}
