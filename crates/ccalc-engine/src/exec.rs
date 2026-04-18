use std::collections::HashMap;
use std::rc::Rc;

/// Parsed function body cache: body source string → pre-parsed, all-silent statements.
type BodyCache = HashMap<String, Rc<Vec<(Stmt, bool)>>>;

/// Expands a leading `~` to the user's home directory.
///
/// On Windows `USERPROFILE` is tried as a fallback for `HOME`. If neither is set the
/// string is returned unchanged.
fn expand_tilde(path: &str) -> String {
    if path == "~" || path.starts_with("~/") || path.starts_with("~\\") {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();
        if home.is_empty() {
            return path.to_string();
        }
        if path == "~" {
            home
        } else {
            format!("{}{}", home, &path[1..])
        }
    } else {
        path.to_string()
    }
}

use indexmap::IndexMap;
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

    /// Stack of directories for currently executing scripts.
    ///
    /// When `run()`/`source()` starts executing a script, the script's parent directory is
    /// pushed here. `resolve_script_path` searches this stack (top-first) so that helper
    /// scripts can be referenced by bare name relative to the calling script's directory.
    static SCRIPT_DIR_STACK: std::cell::RefCell<Vec<std::path::PathBuf>> =
        const { std::cell::RefCell::new(Vec::new()) };

    /// Session search path — initialized from `config.toml` at startup.
    ///
    /// `addpath`/`rmpath` mutate this list for the current session; changes are never
    /// written back to `config.toml`. `resolve_script_path` searches here after CWD.
    static SESSION_PATH: std::cell::RefCell<Vec<std::path::PathBuf>> =
        const { std::cell::RefCell::new(Vec::new()) };

    /// Parse cache for named function bodies.
    ///
    /// Key: body source string (verbatim text between `function` and `end`).
    /// Value: pre-parsed, all-silent statement sequence.
    ///
    /// Populated on the first call to any function; subsequent calls with the
    /// same body string skip parsing entirely and reuse the shared `Rc`.
    /// Cache entries are never evicted — acceptable because the number of
    /// unique function bodies in a session is small.
    static BODY_CACHE: std::cell::RefCell<BodyCache> =
        std::cell::RefCell::new(HashMap::new());
}

/// Returns a parsed, all-silent body for `body_source`, using the cache when possible.
///
/// "All-silent" means every `(Stmt, bool)` has `bool = true` — function bodies
/// never print output directly. The parse result is shared via `Rc` so that
/// repeated calls to the same function avoid both allocation and parsing work.
fn get_or_parse_body(body_source: &str) -> Result<Rc<Vec<(Stmt, bool)>>, String> {
    BODY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(body) = cache.get(body_source) {
            return Ok(Rc::clone(body));
        }
        let stmts =
            parse_stmts(body_source).map_err(|e| format!("function body parse error: {e}"))?;
        let silent: Vec<(Stmt, bool)> = stmts.into_iter().map(|(s, _)| (s, true)).collect();
        let rc = Rc::new(silent);
        cache.insert(body_source.to_string(), Rc::clone(&rc));
        Ok(rc)
    })
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

/// Push a script directory onto the search stack.
///
/// Call this before executing a top-level script file so that `run()`/`source()` calls
/// inside the script can find helper files by bare name relative to the script's directory.
/// Always paired with a matching `script_dir_pop`.
pub fn script_dir_push(dir: &std::path::Path) {
    SCRIPT_DIR_STACK.with(|s| s.borrow_mut().push(dir.to_path_buf()));
}

/// Pop the most recently pushed script directory from the search stack.
pub fn script_dir_pop() {
    SCRIPT_DIR_STACK.with(|s| s.borrow_mut().pop());
}

/// Initializes the session search path from the config `path` array.
///
/// Called once at startup (after loading `config.toml`). Each entry has `~` already
/// expanded by the caller.
pub fn session_path_init(paths: Vec<std::path::PathBuf>) {
    SESSION_PATH.with(|p| *p.borrow_mut() = paths);
}

/// Prepends (default) or appends a directory to the session search path.
///
/// If the same path is already present it is removed from its current position
/// before being re-inserted, so the path list contains no duplicates.
pub fn session_path_add(path: std::path::PathBuf, append: bool) {
    SESSION_PATH.with(|p| {
        let mut v = p.borrow_mut();
        v.retain(|e| e != &path);
        if append {
            v.push(path);
        } else {
            v.insert(0, path);
        }
    });
}

/// Removes a directory from the session search path (exact match).
pub fn session_path_remove(path: &std::path::Path) {
    SESSION_PATH.with(|p| p.borrow_mut().retain(|e| e.as_path() != path));
}

/// Returns a snapshot of the current session search path.
pub fn session_path_list() -> Vec<std::path::PathBuf> {
    SESSION_PATH.with(|p| p.borrow().clone())
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

    // Check for varargin: last parameter is 'varargin' → variadic function.
    let has_varargin = params.last().is_some_and(|p| p == "varargin");
    let fixed_params = if has_varargin {
        &params[..params.len() - 1]
    } else {
        params.as_slice()
    };

    // Trim any trailing args beyond what the function declares.
    // The parser injects `ans` for empty `f()` calls; for 0-param functions
    // we must silently ignore it. For N-param functions, allow up to N+1
    // args before complaining (one implicit `ans` is always present).
    let effective_args = if args.len() > params.len() {
        if !has_varargin && args.len() > params.len() + 1 {
            return Err(format!(
                "Too many arguments: expected at most {}, got {}",
                params.len(),
                args.len()
            ));
        }
        if has_varargin {
            args
        } else {
            // Exactly 1 extra: the implicit `ans` — trim it.
            &args[..params.len()]
        }
    } else {
        args
    };

    // Bind fixed parameters
    for (p, a) in fixed_params.iter().zip(effective_args.iter()) {
        local_env.insert(p.clone(), a.clone());
    }

    // If varargin, collect remaining args into a Cell
    if has_varargin {
        // Collect extra args beyond the fixed parameters into varargin.
        // User functions do not receive an injected `ans`; the arg list reflects
        // exactly what the caller passed.
        let extra: Vec<Value> = effective_args
            .get(fixed_params.len()..)
            .unwrap_or(&[])
            .to_vec();
        let varargin = Value::Cell(extra);
        local_env.insert("varargin".to_string(), varargin);
    }

    let nargin = effective_args.len().min(params.len());
    local_env.insert("nargin".to_string(), Value::Scalar(nargin as f64));
    local_env.insert("nargout".to_string(), Value::Scalar(outputs.len() as f64));

    // Retrieve (or parse-and-cache) the function body, then execute it.
    let body = get_or_parse_body(body_source)?;
    let fmt = get_display_fmt();
    let base = get_display_base();
    let compact = get_display_compact();
    match exec_stmts(&body, &mut local_env, io, &fmt, base, compact)? {
        None | Some(Signal::Return) => {}
        Some(Signal::Break) => return Err("'break' outside loop".to_string()),
        Some(Signal::Continue) => return Err("'continue' outside loop".to_string()),
    }

    // Collect return values
    if outputs.is_empty() {
        return Ok(Value::Void);
    }

    // varargout: single output named 'varargout' — expand from cell
    if outputs.len() == 1 && outputs[0] == "varargout" {
        let cell = local_env.remove("varargout").unwrap_or(Value::Cell(vec![]));
        return match cell {
            Value::Cell(mut v) => {
                if v.is_empty() {
                    Ok(Value::Void)
                } else if v.len() == 1 {
                    Ok(v.remove(0))
                } else {
                    Ok(Value::Tuple(v))
                }
            }
            other => Ok(other),
        };
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
pub fn resolve_script_path(name: &str) -> Option<std::path::PathBuf> {
    // Build candidate base paths: CWD-relative first, then each stacked script dir (top-first),
    // then the session search path entries in order.
    let p = std::path::Path::new(name);
    let mut bases: Vec<std::path::PathBuf> = vec![p.to_path_buf()];
    SCRIPT_DIR_STACK.with(|stack| {
        for dir in stack.borrow().iter().rev() {
            bases.push(dir.join(p));
        }
    });
    SESSION_PATH.with(|sp| {
        for dir in sp.borrow().iter() {
            bases.push(dir.join(p));
        }
    });

    for base in &bases {
        if base.extension().is_some() {
            if base.exists() {
                return Some(base.clone());
            }
            // Explicit extension given but not found — try next base.
            continue;
        }
        let with_calc = base.with_extension("calc");
        if with_calc.exists() {
            return Some(with_calc);
        }
        let with_m = base.with_extension("m");
        if with_m.exists() {
            return Some(with_m);
        }
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
        // A cell is truthy if nonempty.
        Value::Cell(v) => !v.is_empty(),
        // A struct / struct array is always truthy.
        Value::Struct(_) | Value::StructArray(_) => true,
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
        Value::Cell(_) | Value::Struct(_) | Value::StructArray(_) => {
            if let Some(full) = format_value_full(val, fmt) {
                let prefix = label.unwrap_or("ans");
                println!("{prefix} =");
                println!("{full}");
                if !compact {
                    println!();
                }
            }
        }
    }
}

/// Recursively sets a value at `path` inside a nested struct map.
///
/// Ownership-by-value approach: consumes the map, updates it, and returns the updated map.
/// Intermediate structs are created on demand if a path segment does not yet exist.
fn set_nested(
    mut map: IndexMap<String, Value>,
    path: &[String],
    val: Value,
) -> Result<IndexMap<String, Value>, String> {
    let (first, rest) = path.split_first().expect("set_nested: empty path");
    if rest.is_empty() {
        map.insert(first.clone(), val);
    } else {
        let inner = match map.shift_remove(first) {
            Some(Value::Struct(m)) => m,
            None => IndexMap::new(),
            Some(other) => {
                map.insert(first.clone(), other);
                return Err(format!("'{first}' is not a struct"));
            }
        };
        let updated = set_nested(inner, rest, val)?;
        map.insert(first.clone(), Value::Struct(updated));
    }
    Ok(map)
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
                // Intercept addpath()/rmpath()/path() — mutate the session search path.
                if let Expr::Call(fn_name, args) = expr
                    && matches!(fn_name.as_str(), "addpath" | "rmpath" | "path")
                {
                    match fn_name.as_str() {
                        "addpath" => {
                            if args.is_empty() || args.len() > 2 {
                                return Err(
                                    "addpath: expects 1 or 2 arguments: addpath(dir) or addpath(dir, '-end')".to_string()
                                );
                            }
                            let path_val = eval_with_io(&args[0], env, io)?;
                            let path_str = match &path_val {
                                Value::Str(s) | Value::StringObj(s) => s.clone(),
                                _ => {
                                    return Err(
                                        "addpath: argument must be a string (directory path)"
                                            .to_string(),
                                    );
                                }
                            };
                            let append = if args.len() == 2 {
                                let flag_val = eval_with_io(&args[1], env, io)?;
                                match &flag_val {
                                    Value::Str(s) | Value::StringObj(s) if s == "-end" => true,
                                    Value::Str(_) | Value::StringObj(_) => {
                                        return Err(
                                            "addpath: second argument must be '-end' (to append) or omitted (to prepend)".to_string()
                                        );
                                    }
                                    _ => {
                                        return Err(
                                            "addpath: second argument must be a string '-end'"
                                                .to_string(),
                                        );
                                    }
                                }
                            } else {
                                false
                            };
                            let expanded = expand_tilde(&path_str);
                            let pb = std::path::PathBuf::from(&expanded);
                            session_path_add(pb, append);
                            if !silent {
                                for p in session_path_list() {
                                    println!("{}", p.display());
                                }
                            }
                        }
                        "rmpath" => {
                            if args.len() != 1 {
                                return Err("rmpath: expects exactly 1 argument".to_string());
                            }
                            let path_val = eval_with_io(&args[0], env, io)?;
                            let path_str = match &path_val {
                                Value::Str(s) | Value::StringObj(s) => s.clone(),
                                _ => {
                                    return Err(
                                        "rmpath: argument must be a string (directory path)"
                                            .to_string(),
                                    );
                                }
                            };
                            let expanded = expand_tilde(&path_str);
                            session_path_remove(std::path::Path::new(&expanded));
                        }
                        "path" => {
                            if !args.is_empty() {
                                return Err("path: takes no arguments".to_string());
                            }
                            if !silent {
                                let paths = session_path_list();
                                if paths.is_empty() {
                                    println!("(search path is empty)");
                                } else {
                                    for p in &paths {
                                        println!("{}", p.display());
                                    }
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                    continue;
                }

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
                    // Push the script's directory so nested run()/source() calls resolve
                    // helper scripts relative to the calling script's location.
                    if let Some(dir) = script_path.parent() {
                        SCRIPT_DIR_STACK.with(|s| s.borrow_mut().push(dir.to_path_buf()));
                    }
                    let run_stmts = parse_stmts(&content).map_err(|e| {
                        format!("{fn_name}: parse error in '{}': {e}", script_path.display())
                    })?;
                    let result = exec_stmts(&run_stmts, env, io, fmt, base, compact);
                    SCRIPT_DIR_STACK.with(|s| s.borrow_mut().pop());
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
                        // When the case expression is a Cell, check if switch_val
                        // matches any element of the cell (Phase 12.5c).
                        let is_match = if let Value::Cell(cell_elems) = &case_val {
                            cell_elems.iter().any(|elem| match (&switch_val, elem) {
                                (Value::Scalar(a), Value::Scalar(b)) => a == b,
                                _ => {
                                    let sv = match &switch_val {
                                        Value::Str(s) | Value::StringObj(s) => Some(s.as_str()),
                                        _ => None,
                                    };
                                    let cv = match elem {
                                        Value::Str(s) | Value::StringObj(s) => Some(s.as_str()),
                                        _ => None,
                                    };
                                    matches!((sv, cv), (Some(a), Some(b)) if a == b)
                                }
                            })
                        } else {
                            match (&switch_val, &case_val) {
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

            // ── cell element assignment ──────────────────────────────────────
            Stmt::CellSet(cell_name, idx_expr, val_expr) => {
                let idx = eval_with_io(idx_expr, env, io)?;
                let rhs = eval_with_io(val_expr, env, io)?;
                let i = match idx {
                    Value::Scalar(n) => n as isize,
                    _ => return Err(format!("{cell_name}{{}}: index must be a scalar integer")),
                };
                match env.get_mut(cell_name) {
                    Some(Value::Cell(v)) => {
                        if i < 1 {
                            return Err(format!(
                                "{cell_name}{{}}: index {i} out of range (1..{})",
                                v.len()
                            ));
                        }
                        let idx = (i - 1) as usize;
                        // Grow the cell if needed (MATLAB semantics: assigning beyond end grows it)
                        if idx >= v.len() {
                            v.resize(idx + 1, Value::Scalar(0.0));
                        }
                        v[idx] = rhs.clone();
                    }
                    Some(_) => {
                        return Err(format!(
                            "'{cell_name}' is not a cell array; use () for regular indexing"
                        ));
                    }
                    None => {
                        // Auto-create cell if not defined (MATLAB semantics)
                        if i < 1 {
                            return Err(format!("{cell_name}{{}}: index {i} must be >= 1"));
                        }
                        let idx = (i - 1) as usize;
                        let mut v = vec![Value::Scalar(0.0); idx + 1];
                        v[idx] = rhs.clone();
                        env.insert(cell_name.clone(), Value::Cell(v));
                    }
                }
                if !silent && let Some(val) = env.get(cell_name) {
                    print_value(Some(cell_name), val, fmt, base, compact);
                }
            }

            // ── struct field assignment ──────────────────────────────────────
            Stmt::FieldSet(base_name, path, rhs_expr) => {
                let rhs = eval_with_io(rhs_expr, env, io)?;
                let root = match env.remove(base_name) {
                    Some(Value::Struct(m)) => m,
                    None => IndexMap::new(),
                    Some(other) => {
                        env.insert(base_name.clone(), other);
                        return Err(format!("'{base_name}' is not a struct"));
                    }
                };
                let updated = set_nested(root, path, rhs)?;
                let struct_val = Value::Struct(updated);
                if !silent {
                    print_value(Some(base_name), &struct_val, fmt, base, compact);
                }
                env.insert(base_name.clone(), struct_val);
            }

            // ── struct array element field assignment ────────────────────────
            Stmt::StructArrayFieldSet(base_name, idx_expr, path, rhs_expr) => {
                let rhs = eval_with_io(rhs_expr, env, io)?;
                let idx_val = eval_with_io(idx_expr, env, io)?;
                let idx = match &idx_val {
                    Value::Scalar(n) => {
                        let i = *n as isize;
                        if i < 1 {
                            return Err(format!(
                                "Struct array index must be a positive integer, got {n}"
                            ));
                        }
                        i as usize
                    }
                    _ => return Err("Struct array index must be a scalar integer".to_string()),
                };
                // Load or create the struct array
                let mut arr: Vec<IndexMap<String, Value>> = match env.remove(base_name) {
                    Some(Value::StructArray(v)) => v,
                    // A scalar struct with no index yet — promote to 1-element array
                    Some(Value::Struct(m)) => vec![m],
                    None => Vec::new(),
                    Some(other) => {
                        env.insert(base_name.clone(), other);
                        return Err(format!("'{base_name}' is not a struct array"));
                    }
                };
                // Grow the array if needed (fill with empty structs)
                while arr.len() < idx {
                    arr.push(IndexMap::new());
                }
                // Set the field(s) on element idx (1-based → 0-based)
                let elem = arr[idx - 1].clone();
                let updated_elem = set_nested(elem, path, rhs)?;
                arr[idx - 1] = updated_elem;
                let arr_val = Value::StructArray(arr);
                if !silent {
                    print_value(Some(base_name), &arr_val, fmt, base, compact);
                }
                env.insert(base_name.clone(), arr_val);
            }

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
