use ndarray::Array2;

use crate::env::{Env, Value};
use crate::eval::{
    Base, Expr, FormatMode, eval_with_io, format_complex, format_scalar, format_value_full,
    get_display_base, get_display_compact, get_display_fmt, set_display_ctx, set_fn_call_hook,
};
use crate::io::IoContext;
use crate::parser::{Stmt, parse_stmts};

thread_local! {
    /// Tracks the current script nesting depth to prevent infinite recursion via `run()`.
    static RUN_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Flow control signal returned by [`exec_stmts`].
///
/// Used to propagate `break`, `continue`, and `return` through nested block calls.
/// Loop implementations catch `Break`/`Continue`; function call implementation catches `Return`.
/// Uncaught signals at the top level are reported as errors.
pub enum Signal {
    Break,
    Continue,
    /// `return` inside a named function — carries no value (outputs are read from env).
    Return,
}

/// Initialises exec-level hooks in `eval.rs` so that `eval_inner` can call user functions.
///
/// Must be called once at program startup before any evaluation takes place.
pub fn init() {
    set_fn_call_hook(call_user_function);
}

/// Called by `eval_inner` whenever a user function (`Value::Function`) is invoked.
///
/// Executes the function body in an isolated scope containing only the parameters plus
/// any callable values (`Function`/`Lambda`) from the caller's environment, enabling
/// recursion and mutual recursion.
/// Multi-return: if the function has >1 output, returns `Value::Tuple`.
fn call_user_function(
    func: &Value,
    args: &[Value],
    caller_env: &Env,
    io: &mut IoContext,
) -> Result<Value, String> {
    let Value::Function {
        outputs,
        params,
        body_source,
    } = func
    else {
        return Err("call_user_function: not a Function value".to_string());
    };

    // Build isolated scope: seed imaginary unit and ans, then copy all callable
    // values (Function/Lambda) from the caller's environment so that recursion
    // and mutual recursion work correctly.
    let mut local_env = Env::new();
    local_env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    local_env.insert("j".to_string(), Value::Complex(0.0, 1.0));
    local_env.insert("ans".to_string(), Value::Scalar(0.0));
    for (name, val) in caller_env.iter() {
        if matches!(val, Value::Function { .. } | Value::Lambda(_)) {
            local_env.insert(name.clone(), val.clone());
        }
    }

    // Trim any trailing args beyond what the function declares.
    // The parser injects `ans` for empty `f()` calls; for 0-param functions
    // we must silently ignore it. For N-param functions, allow up to N+1
    // args before complaining (one implicit `ans` is always present).
    let effective_args = if args.len() > params.len() {
        if args.len() > params.len() + 1 {
            return Err(format!(
                "Too many arguments: expected at most {}, got {}",
                params.len(),
                args.len()
            ));
        }
        // Exactly 1 extra: the implicit `ans` from empty-call injection — trim it.
        &args[..params.len()]
    } else {
        args
    };
    for (p, a) in params.iter().zip(effective_args.iter()) {
        local_env.insert(p.clone(), a.clone());
    }
    local_env.insert(
        "nargin".to_string(),
        Value::Scalar(effective_args.len() as f64),
    );
    local_env.insert("nargout".to_string(), Value::Scalar(outputs.len() as f64));

    // Parse and execute body
    let stmts = parse_stmts(body_source).map_err(|e| format!("function body parse error: {e}"))?;
    let fmt = get_display_fmt();
    let base = get_display_base();
    let compact = get_display_compact();
    // Function bodies execute silently (no output printing)
    let silent_stmts: Vec<(Stmt, bool)> = stmts.into_iter().map(|(s, _)| (s, true)).collect();
    match exec_stmts(&silent_stmts, &mut local_env, io, &fmt, base, compact)? {
        None | Some(Signal::Return) => {}
        Some(Signal::Break) => return Err("'break' outside loop".to_string()),
        Some(Signal::Continue) => return Err("'continue' outside loop".to_string()),
    }

    // Collect return values
    if outputs.is_empty() {
        return Ok(Value::Void);
    }
    if outputs.len() == 1 {
        return Ok(local_env.remove(&outputs[0]).unwrap_or(Value::Void));
    }
    let vals: Vec<Value> = outputs
        .iter()
        .map(|o| local_env.remove(o).unwrap_or(Value::Void))
        .collect();
    Ok(Value::Tuple(vals))
}

/// Resolves a script filename to an existing path.
///
/// If `name` already has an extension, it is used verbatim.
/// Otherwise, `.calc` is tried first (native ccalc format), then `.m` (Octave/MATLAB compatibility).
/// The search is relative to the current working directory.
fn resolve_script_path(name: &str) -> Option<std::path::PathBuf> {
    let p = std::path::Path::new(name);
    if p.extension().is_some() {
        return if p.exists() {
            Some(p.to_path_buf())
        } else {
            None
        };
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
        // Functions are truthy (they exist), but comparing them to 0 makes no sense.
        // Treat as truthy so that `if f` doesn't silently fail.
        Value::Lambda(_) | Value::Function { .. } | Value::Tuple(_) => true,
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
        Value::Lambda(_) => {
            if let Some(name) = label {
                println!("{name} = @<lambda>");
            } else {
                println!("@<lambda>");
            }
        }
        Value::Function {
            outputs, params, ..
        } => {
            let params_str = params.join(", ");
            let out_str = match outputs.len() {
                0 => String::new(),
                1 => format!("{} = ", outputs[0]),
                _ => format!("[{}] = ", outputs.join(", ")),
            };
            if let Some(name) = label {
                println!("{name} = @function {out_str}{name}({params_str})");
            } else {
                println!("@function {out_str}f({params_str})");
            }
        }
        Value::Tuple(vals) => {
            // Tuples are internal and shouldn't normally be displayed at the REPL level.
            // This can happen if a multi-output function is called without multi-assign.
            for (i, v) in vals.iter().enumerate() {
                print_value(label.map(|_| "ans").or(Some("ans")), v, fmt, base, compact);
                let _ = i;
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
    // Propagate display settings to eval.rs so named function bodies can use them.
    set_display_ctx(fmt, base, compact);

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
                            return Err(format!("{fn_name}: argument must be a string (filename)"));
                        }
                    };
                    let script_path = resolve_script_path(&filename)
                        .ok_or_else(|| format!("{fn_name}: script not found: '{filename}'"))?;
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
                        Some(Signal::Return) => return Ok(Some(Signal::Return)),
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
                    Some(Signal::Return) => return Ok(Some(Signal::Return)),
                }
            },

            Stmt::Break => return Ok(Some(Signal::Break)),
            Stmt::Continue => return Ok(Some(Signal::Continue)),

            // ── switch / case / otherwise / end ──────────────────────────────
            Stmt::Switch {
                expr,
                cases,
                otherwise_body,
            } => {
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
                            if let Some(sig) = exec_stmts(case_body, env, io, fmt, base, compact)? {
                                return Ok(Some(sig));
                            }
                            matched = true;
                            break 'switch_loop;
                        }
                    }
                }
                if !matched
                    && let Some(ob) = otherwise_body
                    && let Some(sig) = exec_stmts(ob, env, io, fmt, base, compact)?
                {
                    return Ok(Some(sig));
                }
            }

            // ── do...until ───────────────────────────────────────────────────
            Stmt::DoUntil { body, cond } => loop {
                match exec_stmts(body, env, io, fmt, base, compact)? {
                    Some(Signal::Break) => break,
                    Some(Signal::Continue) | None => {}
                    Some(Signal::Return) => return Ok(Some(Signal::Return)),
                }
                if is_truthy(&eval_with_io(cond, env, io)?) {
                    break;
                }
            },

            // ── function definition ──────────────────────────────────────────
            Stmt::FunctionDef {
                name,
                outputs,
                params,
                body_source,
            } => {
                env.insert(
                    name.clone(),
                    Value::Function {
                        outputs: outputs.clone(),
                        params: params.clone(),
                        body_source: body_source.clone(),
                    },
                );
            }

            // ── return ───────────────────────────────────────────────────────
            Stmt::Return => return Ok(Some(Signal::Return)),

            // ── multi-assign ─────────────────────────────────────────────────
            Stmt::MultiAssign { targets, expr } => {
                let val = eval_with_io(expr, env, io)?;
                let vals: Vec<Value> = match val {
                    Value::Tuple(v) => v,
                    other => vec![other],
                };
                for (i, target) in targets.iter().enumerate() {
                    if target == "~" {
                        continue; // discard output
                    }
                    let v = vals.get(i).cloned().unwrap_or(Value::Void);
                    env.insert(target.clone(), v.clone());
                    if !silent && !matches!(v, Value::Void) {
                        print_value(Some(target), &v, fmt, base, compact);
                    }
                }
            }
        }
    }
    Ok(None)
}
