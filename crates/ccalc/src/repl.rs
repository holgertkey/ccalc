use std::io::{BufRead, Write};

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use ccalc_engine::env::{
    Env, Value, config_dir, load_workspace, load_workspace_default, save_workspace,
    save_workspace_default, save_workspace_vars,
};
use ccalc_engine::eval::{
    Base, Expr, FormatMode, eval, eval_with_io, format_complex, format_number, format_scalar,
    format_value_full,
};
use ccalc_engine::exec::{Signal, exec_stmts};
use ccalc_engine::io::IoContext;
use ccalc_engine::parser::{Stmt, block_depth_delta, is_partial, parse, parse_stmts, split_stmts};

/// Result of evaluating one input line.
enum EvalResult {
    /// Assignment `name = expr` was executed; `name` was set to `val`.
    Assigned(String, Value),
    /// Standalone expression; result stored in `ans`.
    Value(Value),
}

/// Parse and evaluate one input string, updating `env`.
/// Handles partial expressions (starting with an operator) by prepending `ans`.
///
/// MATLAB semantics: expressions always update `ans`; assignments never do.
/// The caller controls whether output is printed (silent flag), but `ans` is
/// always updated by expressions regardless of silence.
fn evaluate(input: &str, env: &mut Env, io: &mut IoContext) -> Result<EvalResult, String> {
    let expanded = if is_partial(input) {
        format!("ans {}", input)
    } else {
        input.to_string()
    };

    match parse(&expanded)? {
        Stmt::Assign(name, expr) => {
            let val = eval_with_io(&expr, env, io)?;
            env.insert(name.clone(), val.clone());
            // Assignments do not update ans (MATLAB semantics)
            Ok(EvalResult::Assigned(name, val))
        }
        Stmt::Expr(expr) => {
            let val = eval_with_io(&expr, env, io)?;
            env.insert("ans".to_string(), val.clone()); // always update ans
            Ok(EvalResult::Value(val))
        }
        _ => Err("Block statements must be entered in multi-line mode".to_string()),
    }
}

fn ans(env: &Env) -> f64 {
    match env.get("ans") {
        Some(Value::Scalar(n)) => *n,
        _ => 0.0,
    }
}

fn new_env() -> Env {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    env.insert("j".to_string(), Value::Complex(0.0, 1.0));
    env
}

/// Parsed form of a `save` or `load` command.
enum SaveLoadCmd {
    Save {
        path: Option<String>,
        vars: Vec<String>,
    },
    Load {
        path: Option<String>,
    },
}

/// Tries to parse a statement as a `save`/`load` command (bare or with arguments).
///
/// Recognises:
/// - `save`  /  `load`  — bare aliases for `ws` / `wl`
/// - `save('path')`  /  `load('path')`
/// - `save('path', 'x', 'y')`  — selective save
///
/// Returns `None` if the statement is not a save/load command.
///
/// String arguments may be literals (`'path'`) or variables holding a string value.
/// `env` is used to resolve variable references.
fn try_parse_save_load(stmt: &str, env: &Env) -> Option<SaveLoadCmd> {
    match stmt.trim() {
        "save" => {
            return Some(SaveLoadCmd::Save {
                path: None,
                vars: vec![],
            });
        }
        "load" => return Some(SaveLoadCmd::Load { path: None }),
        _ => {}
    }
    let parsed = parse(stmt).ok()?;
    match parsed {
        Stmt::Expr(Expr::Call(name, args)) => {
            let mut str_args: Vec<String> = Vec::new();
            for arg in args {
                let s = match arg {
                    Expr::StrLiteral(s) | Expr::StringObjLiteral(s) => s,
                    Expr::Var(v) => match env.get(&v) {
                        Some(Value::Str(s)) | Some(Value::StringObj(s)) => s.clone(),
                        _ => return None,
                    },
                    _ => return None,
                };
                str_args.push(s);
            }
            match name.as_str() {
                "save" => {
                    let path = str_args.first().cloned();
                    let vars = if str_args.len() > 1 {
                        str_args[1..].to_vec()
                    } else {
                        vec![]
                    };
                    Some(SaveLoadCmd::Save { path, vars })
                }
                "load" => Some(SaveLoadCmd::Load {
                    path: str_args.into_iter().next(),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Appends a `% --- Session: YYYY-MM-DD HH:MM:SS UTC ---` line to the history
/// file before loading it, so the file acts as a timestamped session log.
/// The `%` prefix makes the line a no-op comment if the user ever recalls it.
fn append_session_marker(path: &std::path::Path) {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(path) {
        let _ = writeln!(f, "% --- Session: {} ---", format_utc(secs));
    }
}

/// Converts a Unix timestamp to a human-readable `YYYY-MM-DD HH:MM:SS UTC` string.
/// Uses only `std` — no external date/time crate needed.
fn format_utc(secs: u64) -> String {
    let s = (secs % 60) as u32;
    let m = (secs / 60 % 60) as u32;
    let h = (secs / 3600 % 24) as u32;
    let mut days = (secs / 86400) as u32;

    let mut year = 1970u32;
    loop {
        let y_days = if is_leap_year(year) { 366 } else { 365 };
        if days < y_days {
            break;
        }
        days -= y_days;
        year += 1;
    }

    let month_days = [
        31u32,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u32;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;

    format!("{year:04}-{month:02}-{day:02} {h:02}:{m:02}:{s:02} UTC")
}

fn is_leap_year(y: u32) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

fn format_prompt_ans(env: &Env, base: Base, fmt: &FormatMode) -> String {
    match env.get("ans") {
        Some(Value::Void) | None => "0".to_string(),
        Some(Value::Scalar(n)) => format_scalar(*n, base, fmt),
        Some(Value::Matrix(m)) => format!("[{}×{}]", m.nrows(), m.ncols()),
        Some(Value::Complex(re, im)) => format_complex(*re, *im, fmt),
        Some(Value::Str(s)) => {
            let display: String = s.chars().take(15).collect();
            if s.len() > 15 {
                format!("'{display}...'")
            } else {
                format!("'{display}'")
            }
        }
        Some(Value::StringObj(s)) => {
            let display: String = s.chars().take(15).collect();
            if s.len() > 15 {
                format!("\"{display}...\"")
            } else {
                format!("\"{display}\"")
            }
        }
    }
}

pub fn run() {
    let mut env = new_env();
    let mut io = IoContext::new();
    let config_path = config_dir().join("config.toml");
    let cfg = crate::config::load_or_create(&config_path);
    let mut fmt = FormatMode::Custom(cfg.precision());
    let mut compact = false;
    let mut base = cfg.base();
    let mut rl = DefaultEditor::new().expect("Failed to initialize line editor");

    let history_path = config_dir().join("history");
    append_session_marker(&history_path);
    rl.load_history(&history_path).ok();

    println!(
        "ccalc v{}  (type 'help' for reference)",
        env!("CARGO_PKG_VERSION")
    );
    println!();

    // Multi-line block buffering state
    let mut block_buf: Vec<String> = Vec::new();
    let mut block_depth: i32 = 0;

    'repl: loop {
        let prompt = if block_depth > 0 {
            "  >> ".to_string()
        } else {
            format!("[ {} ]: ", format_prompt_ans(&env, base, &fmt))
        };

        let input = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                if block_depth > 0 {
                    // Cancel current block
                    block_buf.clear();
                    block_depth = 0;
                    eprintln!("^C");
                    continue;
                }
                break;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(trimmed);

        // Exit/quit always works, even inside block mode
        if trimmed == "exit" || trimmed == "quit" {
            break 'repl;
        }

        // Block mode: accumulate lines until block closes
        let delta = block_depth_delta(trimmed);
        if block_depth > 0 || delta > 0 {
            block_buf.push(trimmed.to_string());
            block_depth += delta;
            if block_depth <= 0 {
                block_depth = 0;
                let block_input = block_buf.join("\n");
                block_buf.clear();
                match parse_stmts(&block_input) {
                    Ok(stmts) => match exec_stmts(&stmts, &mut env, &mut io, &fmt, base, compact) {
                        Ok(None) => {}
                        Ok(Some(Signal::Break | Signal::Continue)) => {
                            eprintln!("Error: 'break'/'continue' outside a loop");
                        }
                        Err(e) => eprintln!("Error: {e}"),
                    },
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            continue;
        }

        for (stmt, silent) in split_stmts(trimmed) {
            // Built-in commands
            match stmt {
                "exit" | "quit" => break 'repl,
                "cls" => {
                    clear_screen();
                    continue;
                }
                "who" => {
                    print_who(&env, base, &fmt);
                    continue;
                }
                "clear" => {
                    env.clear();
                    continue;
                }
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
                "base" => {
                    print_all_bases(ans(&env), &fmt);
                    continue;
                }
                "ws" | "save" => {
                    match save_workspace_default(&env) {
                        Ok(()) => println!("Workspace saved."),
                        Err(e) => eprintln!("Error: {e}"),
                    }
                    continue;
                }
                "wl" | "load" => {
                    match load_workspace_default() {
                        Ok(loaded) => {
                            env = loaded;
                            println!("Workspace loaded.");
                        }
                        Err(e) => eprintln!("Error: {e}"),
                    }
                    continue;
                }
                "help" | "?" => {
                    crate::help::print(Some(""));
                    continue;
                }
                "config" => {
                    println!("config file: {}", config_path.display());
                    println!("format:      {}", fmt.name());
                    println!("compact:     {compact}");
                    println!("base:        {}", format_base_name(base));
                    continue;
                }
                _ => {}
            }

            // config reload
            if stmt == "config reload" {
                match crate::config::load(&config_path) {
                    Ok(cfg) => {
                        fmt = FormatMode::Custom(cfg.precision());
                        base = cfg.base();
                        println!("Config reloaded.");
                        println!("format:      {}", fmt.name());
                        println!("base:        {}", format_base_name(base));
                    }
                    Err(e) => eprintln!("Error: {e}"),
                }
                continue;
            }

            // format [mode] — change number display format
            if stmt == "format" {
                fmt = FormatMode::Short;
                println!("format: short");
                continue;
            }
            if let Some(arg) = stmt.strip_prefix("format ").map(str::trim) {
                match arg {
                    "short" => {
                        fmt = FormatMode::Short;
                        println!("format: short");
                    }
                    "long" => {
                        fmt = FormatMode::Long;
                        println!("format: long");
                    }
                    "shorte" | "shortE" => {
                        fmt = FormatMode::ShortE;
                        println!("format: shortE");
                    }
                    "longe" | "longE" => {
                        fmt = FormatMode::LongE;
                        println!("format: longE");
                    }
                    "shortg" | "shortG" => {
                        fmt = FormatMode::ShortG;
                        println!("format: shortG");
                    }
                    "longg" | "longG" => {
                        fmt = FormatMode::LongG;
                        println!("format: longG");
                    }
                    "bank" => {
                        fmt = FormatMode::Bank;
                        println!("format: bank");
                    }
                    "rat" => {
                        fmt = FormatMode::Rat;
                        println!("format: rat");
                    }
                    "hex" => {
                        fmt = FormatMode::Hex;
                        println!("format: hex");
                    }
                    "+" => {
                        fmt = FormatMode::Plus;
                        println!("format: +");
                    }
                    "compact" => {
                        compact = true;
                        println!("format: compact");
                    }
                    "loose" => {
                        compact = false;
                        println!("format: loose");
                    }
                    s => {
                        if let Ok(n) = s.parse::<usize>() {
                            fmt = FormatMode::Custom(n);
                            println!("format: {n} decimal places");
                        } else {
                            eprintln!(
                                "Unknown format '{s}'. Options: short long shortE longE \
                                 shortG longG bank rat hex + compact loose <N>"
                            );
                        }
                    }
                }
                continue;
            }

            // help <topic>
            if let Some(topic) = stmt.strip_prefix("help ").map(str::trim) {
                crate::help::print(Some(topic));
                continue;
            }

            // clear <name>
            if let Some(name) = stmt.strip_prefix("clear ").map(str::trim) {
                if !name.is_empty() {
                    env.remove(name);
                }
                continue;
            }

            // save / load with optional path and variable list
            if let Some(cmd) = try_parse_save_load(stmt, &env) {
                match cmd {
                    SaveLoadCmd::Save { path, vars } => {
                        let result = match (&path, vars.is_empty()) {
                            (None, _) => save_workspace_default(&env),
                            (Some(p), true) => save_workspace(&env, std::path::Path::new(p)),
                            (Some(p), false) => {
                                let var_refs: Vec<&str> = vars.iter().map(String::as_str).collect();
                                save_workspace_vars(&env, std::path::Path::new(p), &var_refs)
                            }
                        };
                        match result {
                            Ok(()) => println!("Workspace saved."),
                            Err(e) => eprintln!("Error: {e}"),
                        }
                    }
                    SaveLoadCmd::Load { path } => {
                        let result = match path {
                            None => load_workspace_default(),
                            Some(p) => load_workspace(std::path::Path::new(&p)),
                        };
                        match result {
                            Ok(loaded) => {
                                env = loaded;
                                println!("Workspace loaded.");
                            }
                            Err(e) => eprintln!("Error: {e}"),
                        }
                    }
                }
                continue;
            }

            // disp(expr) — print value without updating ans
            if let Some(arg) = parse_disp_cmd(stmt) {
                handle_disp(arg, &env, base, &fmt);
                continue;
            }

            // run() / source() — execute a script file in the current workspace
            if try_run_source(stmt, silent, &mut env, &mut io, &fmt, base, compact) {
                continue;
            }

            // Extract trailing base suffix (e.g. "0xFF + 0b10 hex", "10 base")
            let (to_eval, base_suffix) = extract_base_suffix(stmt);
            let show_all_bases = matches!(base_suffix, Some(BaseSuffix::ShowAll));
            if let Some(BaseSuffix::Switch(b)) = base_suffix {
                base = b;
            }

            // Build display string: partial expressions show numeric ans, not the word "ans"
            let display_str = if is_partial(to_eval) {
                format!("{} {}", format_for_base(ans(&env), base), to_eval)
            } else {
                to_eval.to_string()
            };
            // Expand variable references, then apply base conversion on the result
            let expanded = expand_vars_for_display(&display_str, &env, base);
            let base_display =
                format_expr_for_display(expanded.as_deref().unwrap_or(&display_str), base);

            match evaluate(to_eval, &mut env, &mut io) {
                Ok(result) => {
                    if !silent {
                        match result {
                            EvalResult::Assigned(name, val) => match &val {
                                Value::Void => {}
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&val, &fmt) {
                                        println!("{name} =");
                                        println!("{full}");
                                        if !compact {
                                            println!();
                                        }
                                    }
                                }
                                Value::Scalar(v) => {
                                    println!("{name} = {}", format_scalar(*v, base, &fmt));
                                    if compact {
                                    } else if matches!(
                                        fmt,
                                        FormatMode::Hex | FormatMode::Rat | FormatMode::Bank
                                    ) {
                                        println!();
                                    }
                                }
                                Value::Complex(re, im) => {
                                    println!("{name} = {}", format_complex(*re, *im, &fmt));
                                }
                                Value::Str(s) => println!("{name} = {s}"),
                                Value::StringObj(s) => println!("{name} = {s}"),
                            },
                            EvalResult::Value(val) => match &val {
                                Value::Void => {}
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&val, &fmt) {
                                        println!("ans =");
                                        println!("{full}");
                                        if !compact {
                                            println!();
                                        }
                                    }
                                }
                                Value::Scalar(v) => {
                                    let to_show: Option<&str> = if let Some(ref s) = base_display {
                                        Some(s.as_str())
                                    } else {
                                        expanded.as_deref()
                                    };
                                    if let Some(display) = to_show {
                                        println!("{display}");
                                    }
                                    if show_all_bases {
                                        print_all_bases(*v, &fmt);
                                    }
                                }
                                Value::Complex(re, im) => {
                                    println!("{}", format_complex(*re, *im, &fmt));
                                }
                                Value::Str(s) | Value::StringObj(s) => println!("{s}"),
                            },
                        }
                    }
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    }

    rl.save_history(&history_path).ok();
}

/// Evaluate a single expression string in argument mode.
/// Prints the result and exits with code 1 on error.
pub fn run_expr(expr: &str) {
    let mut env = new_env();
    let mut io = IoContext::new();
    let mut base = Base::Dec;
    let trimmed = expr.trim();

    let fmt = FormatMode::default();
    if let Some(arg) = parse_disp_cmd(trimmed) {
        handle_disp(arg, &env, base, &fmt);
        return;
    }

    let (to_eval, base_suffix) = extract_base_suffix(trimmed);
    let show_all = matches!(base_suffix, Some(BaseSuffix::ShowAll));
    if let Some(BaseSuffix::Switch(b)) = base_suffix {
        base = b;
    }
    match evaluate(to_eval, &mut env, &mut io) {
        Ok(result) => match result {
            EvalResult::Assigned(name, v) => match &v {
                Value::Void => {}
                Value::Matrix(_) => {
                    if let Some(full) = format_value_full(&v, &fmt) {
                        println!("{name} =");
                        println!("{full}");
                    }
                }
                Value::Scalar(n) => {
                    println!("{} = {}", name, format_scalar(*n, base, &fmt));
                }
                Value::Complex(re, im) => {
                    println!("{} = {}", name, format_complex(*re, *im, &fmt));
                }
                Value::Str(s) => println!("{name} = {s}"),
                Value::StringObj(s) => println!("{name} = {s}"),
            },
            EvalResult::Value(v) => match &v {
                Value::Void => {}
                Value::Matrix(_) => {
                    if let Some(full) = format_value_full(&v, &fmt) {
                        println!("ans =");
                        println!("{full}");
                    }
                }
                Value::Scalar(n) => {
                    if show_all {
                        print_all_bases(*n, &fmt);
                    } else {
                        println!("{}", format_scalar(*n, base, &fmt));
                    }
                }
                Value::Complex(re, im) => {
                    println!("{}", format_complex(*re, *im, &fmt));
                }
                Value::Str(s) | Value::StringObj(s) => println!("{s}"),
            },
        },
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Handles `run('file')` / `source('file')` when encountered as a single
/// statement in pipe or REPL mode.
///
/// In those modes, statements normally go through [`evaluate()`] → `eval_with_io`.
/// But `run`/`source` must execute a script via [`exec_stmts`], which shares the
/// caller's `Env`. This function bridges that gap.
///
/// Returns `true` if the statement was intercepted (caller should `continue`),
/// `false` if normal evaluation should proceed.
fn try_run_source(
    stmt: &str,
    silent: bool,
    env: &mut Env,
    io: &mut IoContext,
    fmt: &FormatMode,
    base: Base,
    compact: bool,
) -> bool {
    let s = stmt.trim_start();
    if !s.starts_with("run(") && !s.starts_with("source(") {
        return false;
    }
    match parse(stmt) {
        Ok(parsed) => {
            match exec_stmts(&[(parsed, silent)], env, io, fmt, base, compact) {
                Ok(_) => {}
                Err(e) => eprintln!("Error: {e}"),
            }
            true
        }
        Err(_) => false, // fall through to evaluate() for a proper error message
    }
}

/// Process lines from a non-interactive reader (pipe, file redirect).
/// Prints one result per expression line; no prompts.
pub fn run_pipe(reader: impl BufRead) {
    let mut env = new_env();
    let mut io = IoContext::new();
    let mut fmt = FormatMode::default();
    let mut compact = false;
    let mut base = Base::Dec;

    // Multi-line block buffering state
    let mut block_buf: Vec<String> = Vec::new();
    let mut block_depth: i32 = 0;

    'lines: for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        };
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if block_depth > 0 {
                block_buf.push(String::new());
            }
            continue;
        }

        // Block mode: accumulate lines until block closes
        let delta = block_depth_delta(trimmed);
        if block_depth > 0 || delta > 0 {
            if matches!(trimmed, "exit" | "quit") {
                break 'lines;
            }
            block_buf.push(trimmed.to_string());
            block_depth += delta;
            if block_depth <= 0 {
                block_depth = 0;
                let block_input = block_buf.join("\n");
                block_buf.clear();
                match parse_stmts(&block_input) {
                    Ok(stmts) => {
                        if let Err(e) = exec_stmts(&stmts, &mut env, &mut io, &fmt, base, compact) {
                            eprintln!("Error: {e}");
                        }
                    }
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            continue;
        }

        for (stmt, silent) in split_stmts(trimmed) {
            // Built-in commands (subset relevant in pipe mode)
            match stmt {
                "exit" | "quit" => break 'lines,
                "clear" => {
                    env.clear();
                    continue;
                }
                "cls" | "who" | "help" | "?" | "config" => continue, // no-op in pipe mode
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
                "base" => {
                    print_all_bases(ans(&env), &fmt);
                    continue;
                }
                "ws" | "save" => {
                    let _ = save_workspace_default(&env);
                    continue;
                }
                "wl" | "load" => {
                    if let Ok(loaded) = load_workspace_default() {
                        env = loaded;
                    }
                    continue;
                }
                _ => {}
            }

            // help / config — no-op in pipe mode
            if stmt.starts_with("help ") || stmt == "config reload" {
                continue;
            }

            // format [mode] — change number display format (pipe mode)
            if stmt == "format" {
                fmt = FormatMode::Short;
                continue;
            }
            if let Some(arg) = stmt.strip_prefix("format ").map(str::trim) {
                match arg {
                    "short" => {
                        fmt = FormatMode::Short;
                    }
                    "long" => {
                        fmt = FormatMode::Long;
                    }
                    "shorte" | "shortE" => {
                        fmt = FormatMode::ShortE;
                    }
                    "longe" | "longE" => {
                        fmt = FormatMode::LongE;
                    }
                    "shortg" | "shortG" => {
                        fmt = FormatMode::ShortG;
                    }
                    "longg" | "longG" => {
                        fmt = FormatMode::LongG;
                    }
                    "bank" => {
                        fmt = FormatMode::Bank;
                    }
                    "rat" => {
                        fmt = FormatMode::Rat;
                    }
                    "hex" => {
                        fmt = FormatMode::Hex;
                    }
                    "+" => {
                        fmt = FormatMode::Plus;
                    }
                    "compact" => {
                        compact = true;
                    }
                    "loose" => {
                        compact = false;
                    }
                    s => {
                        if let Ok(n) = s.parse::<usize>() {
                            fmt = FormatMode::Custom(n);
                        }
                    }
                }
                continue;
            }

            // clear <name>
            if let Some(name) = stmt.strip_prefix("clear ").map(str::trim) {
                if !name.is_empty() {
                    env.remove(name);
                }
                continue;
            }

            // save / load with optional path and variable list
            if let Some(cmd) = try_parse_save_load(stmt, &env) {
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

            // disp(expr) — print value without updating ans
            if let Some(arg) = parse_disp_cmd(stmt) {
                handle_disp(arg, &env, base, &fmt);
                continue;
            }

            // run() / source() — execute a script file in the current workspace
            if try_run_source(stmt, silent, &mut env, &mut io, &fmt, base, compact) {
                continue;
            }

            let (to_eval, base_suffix) = extract_base_suffix(stmt);
            let show_all = matches!(base_suffix, Some(BaseSuffix::ShowAll));
            if let Some(BaseSuffix::Switch(b)) = base_suffix {
                base = b;
            }

            match evaluate(to_eval, &mut env, &mut io) {
                Ok(result) => {
                    if !silent {
                        match result {
                            EvalResult::Assigned(name, v) => match &v {
                                Value::Void => {}
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&v, &fmt) {
                                        println!("{name} =");
                                        println!("{full}");
                                        if !compact {
                                            println!();
                                        }
                                    }
                                }
                                Value::Scalar(n) => {
                                    println!("{} = {}", name, format_scalar(*n, base, &fmt));
                                }
                                Value::Complex(re, im) => {
                                    println!("{} = {}", name, format_complex(*re, *im, &fmt));
                                }
                                Value::Str(s) => println!("{name} = {s}"),
                                Value::StringObj(s) => println!("{name} = {s}"),
                            },
                            EvalResult::Value(v) => match &v {
                                Value::Void => {}
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&v, &fmt) {
                                        println!("ans =");
                                        println!("{full}");
                                        if !compact {
                                            println!();
                                        }
                                    }
                                }
                                Value::Scalar(n) => {
                                    if show_all {
                                        let i = n.round() as i64;
                                        let u = i.unsigned_abs();
                                        let sign = if i < 0 { "-" } else { "" };
                                        println!("2  - {}0b{:b}", sign, u);
                                        println!("8  - {}0o{:o}", sign, u);
                                        println!("10 - {}", format_scalar(*n, Base::Dec, &fmt));
                                        println!("16 - {}0x{:X}", sign, u);
                                    } else {
                                        println!("{}", format_scalar(*n, base, &fmt));
                                    }
                                }
                                Value::Complex(re, im) => {
                                    println!("{}", format_complex(*re, *im, &fmt));
                                }
                                Value::Str(s) | Value::StringObj(s) => println!("{s}"),
                            },
                        }
                    }
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    }
}

/// Lay out `entries` into multi-column lines that fit within `term_width`.
/// Column-major order (top-to-bottom, then left-to-right), like `ls`.
/// Returns one string per output line.
fn who_format_columns(entries: &[String], term_width: usize) -> Vec<String> {
    if entries.is_empty() {
        return vec![];
    }
    let col_width = entries.iter().map(|s| s.len()).max().unwrap_or(0) + 2;
    let num_cols = (term_width / col_width).max(1);
    let num_rows = entries.len().div_ceil(num_cols);

    let mut lines = Vec::with_capacity(num_rows);
    for row in 0..num_rows {
        let mut line = String::new();
        for col in 0..num_cols {
            let idx = col * num_rows + row;
            if idx < entries.len() {
                let is_last_in_row =
                    col + 1 == num_cols || (col + 1) * num_rows + row >= entries.len();
                if is_last_in_row {
                    line.push_str(&entries[idx]);
                } else {
                    line.push_str(&format!("{:<width$}", entries[idx], width = col_width));
                }
            }
        }
        lines.push(line);
    }
    lines
}

fn print_who(env: &Env, base: Base, fmt: &FormatMode) {
    if env.is_empty() {
        return;
    }

    println!("Variables visible from the current scope:");
    println!();

    let term_width: usize = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);

    // ans always first
    if let Some(val) = env.get("ans") {
        match val {
            Value::Void => {}
            Value::Scalar(n) => println!("ans = {}", format_scalar(*n, base, fmt)),
            Value::Matrix(m) => println!("ans = [{}×{} double]", m.nrows(), m.ncols()),
            Value::Complex(re, im) => println!("ans = {}", format_complex(*re, *im, fmt)),
            Value::Str(s) => println!("ans = {s}"),
            Value::StringObj(s) => println!("ans = {s}"),
        }
    }

    // Remaining variables sorted alphabetically, scalars and matrices separated
    let mut scalars: Vec<String> = Vec::new();
    let mut matrices: Vec<String> = Vec::new();

    let mut others: Vec<(&String, &Value)> =
        env.iter().filter(|(k, _)| k.as_str() != "ans").collect();
    others.sort_by_key(|(k, _)| k.as_str());

    for (name, val) in others {
        match val {
            Value::Void => {}
            Value::Scalar(n) => {
                scalars.push(format!("{} = {}", name, format_scalar(*n, base, fmt)));
            }
            Value::Complex(re, im) => {
                scalars.push(format!("{} = {}", name, format_complex(*re, *im, fmt)));
            }
            Value::Matrix(m) => {
                matrices.push(format!("{} = [{}×{} double]", name, m.nrows(), m.ncols()));
            }
            Value::Str(s) => {
                let n = s.chars().count();
                scalars.push(format!("{name} [1×{n} char]"));
            }
            Value::StringObj(_) => {
                scalars.push(format!("{name} [string]"));
            }
        }
    }

    // Scalars in columns
    for line in who_format_columns(&scalars, term_width) {
        println!("{}", line);
    }

    // Matrices each on its own line at the end
    for entry in &matrices {
        println!("{}", entry);
    }

    println!();
}

/// Prints a value in all four bases.
fn print_all_bases(n: f64, fmt: &FormatMode) {
    let i = n.round() as i64;
    let u = i.unsigned_abs();
    let sign = if i < 0 { "-" } else { "" };
    println!("2  - {}0b{:b}", sign, u);
    println!("8  - {}0o{:o}", sign, u);
    println!("10 - {}", format_scalar(n, Base::Dec, fmt));
    println!("16 - {}0x{:X}", sign, u);
}

/// Trailing base suffix: a base-change keyword or `base` (show all).
#[derive(Debug, Clone, Copy, PartialEq)]
enum BaseSuffix {
    Switch(Base),
    ShowAll,
}

/// Strips a trailing base keyword from an expression.
/// Returns `(remaining_expr, Some(suffix))` or `(input, None)` if no suffix found.
fn extract_base_suffix(input: &str) -> (&str, Option<BaseSuffix>) {
    if let Some(pos) = input.rfind(' ') {
        let token = &input[pos + 1..];
        let before = input[..pos].trim_end();
        if !before.is_empty() {
            let suffix = match token {
                "hex" => Some(BaseSuffix::Switch(Base::Hex)),
                "dec" => Some(BaseSuffix::Switch(Base::Dec)),
                "bin" => Some(BaseSuffix::Switch(Base::Bin)),
                "oct" => Some(BaseSuffix::Switch(Base::Oct)),
                "base" => Some(BaseSuffix::ShowAll),
                _ => None,
            };
            if suffix.is_some() {
                return (before, suffix);
            }
        }
    }
    (input, None)
}

/// Formats `val` in the given base for expression display.
fn format_for_base(val: f64, base: Base) -> String {
    let i = val.round() as i64;
    let u = i.unsigned_abs();
    let sign = if i < 0 { "-" } else { "" };
    match base {
        Base::Hex => format!("{}0x{:X}", sign, u),
        Base::Bin => format!("{}0b{:b}", sign, u),
        Base::Oct => format!("{}0o{:o}", sign, u),
        Base::Dec => format_number(val),
    }
}

/// Replaces identifiers that match a variable in `env` with their formatted values.
/// Returns `Some(expanded)` if any replacement was made, `None` otherwise.
fn expand_vars_for_display(expr: &str, env: &Env, base: Base) -> Option<String> {
    let mut result = String::with_capacity(expr.len());
    let mut chars = expr.chars().peekable();
    let mut replaced = false;

    while let Some(&c) = chars.peek() {
        if c.is_alphabetic() || c == '_' {
            let mut ident = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    ident.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            match env.get(&ident) {
                Some(Value::Scalar(val)) => {
                    result.push_str(&format_for_base(*val, base));
                    replaced = true;
                }
                Some(Value::Complex(re, im)) => {
                    result.push_str(&format_complex(*re, *im, &FormatMode::default()));
                    replaced = true;
                }
                _ => result.push_str(&ident),
            }
        } else {
            result.push(c);
            chars.next();
        }
    }

    if replaced { Some(result) } else { None }
}

/// Rewrites number literals in `expr` that are not in the target `base` to that base.
/// Returns `Some(rewritten)` if any conversion happened, `None` if nothing changed.
fn format_expr_for_display(expr: &str, base: Base) -> Option<String> {
    let mut result = String::with_capacity(expr.len());
    let mut chars = expr.chars().peekable();
    let mut changed = false;

    while let Some(&c) = chars.peek() {
        match c {
            '0' => {
                chars.next();
                match chars.peek().copied() {
                    Some('x') | Some('X') => {
                        let pfx = chars.next().unwrap();
                        let mut s = String::new();
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_hexdigit() {
                                s.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if s.is_empty() {
                            result.push('0');
                            result.push(pfx);
                        } else if base == Base::Hex {
                            result.push('0');
                            result.push(pfx);
                            result.push_str(&s);
                        } else {
                            let val = i64::from_str_radix(&s, 16).unwrap_or(0) as f64;
                            result.push_str(&format_for_base(val, base));
                            changed = true;
                        }
                    }
                    Some('b') | Some('B') => {
                        let pfx = chars.next().unwrap();
                        let mut s = String::new();
                        while let Some(&d) = chars.peek() {
                            if d == '0' || d == '1' {
                                s.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if s.is_empty() {
                            result.push('0');
                            result.push(pfx);
                        } else if base == Base::Bin {
                            result.push('0');
                            result.push(pfx);
                            result.push_str(&s);
                        } else {
                            let val = i64::from_str_radix(&s, 2).unwrap_or(0) as f64;
                            result.push_str(&format_for_base(val, base));
                            changed = true;
                        }
                    }
                    Some('o') | Some('O') => {
                        let pfx = chars.next().unwrap();
                        let mut s = String::new();
                        while let Some(&d) = chars.peek() {
                            if ('0'..='7').contains(&d) {
                                s.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if s.is_empty() {
                            result.push('0');
                            result.push(pfx);
                        } else if base == Base::Oct {
                            result.push('0');
                            result.push(pfx);
                            result.push_str(&s);
                        } else {
                            let val = i64::from_str_radix(&s, 8).unwrap_or(0) as f64;
                            result.push_str(&format_for_base(val, base));
                            changed = true;
                        }
                    }
                    _ => {
                        let mut num_str = String::from("0");
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_digit() || d == '.' {
                                num_str.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if base == Base::Dec {
                            result.push_str(&num_str);
                        } else {
                            let val: f64 = num_str.parse().unwrap_or(0.0);
                            let formatted = format_for_base(val, base);
                            if formatted != num_str {
                                changed = true;
                            }
                            result.push_str(&formatted);
                        }
                    }
                }
            }
            '1'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        num_str.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if base == Base::Dec {
                    result.push_str(&num_str);
                } else {
                    let val: f64 = num_str.parse().unwrap_or(0.0);
                    let formatted = format_for_base(val, base);
                    if formatted != num_str {
                        changed = true;
                    }
                    result.push_str(&formatted);
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                // Identifier (function name, variable, constant) — keep verbatim
                while let Some(&d) = chars.peek() {
                    if d.is_alphanumeric() || d == '_' {
                        result.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => {
                result.push(c);
                chars.next();
            }
        }
    }

    if changed { Some(result) } else { None }
}

/// Returns the display name of a `Base` value (used in `config` output).
fn format_base_name(base: Base) -> &'static str {
    match base {
        Base::Dec => "dec",
        Base::Hex => "hex",
        Base::Bin => "bin",
        Base::Oct => "oct",
    }
}

/// Parses a precision command of the form `p<N>` where N is 0–15.
/// Extracts the argument string from a `disp(...)` call.
/// Returns `None` if the input does not match the pattern.
fn parse_disp_cmd(input: &str) -> Option<&str> {
    let inner = input.strip_prefix("disp(")?.strip_suffix(')')?;
    if inner.is_empty() { None } else { Some(inner) }
}

/// Evaluates `arg` and prints the result. Does not update `ans`.
fn handle_disp(arg: &str, env: &Env, base: Base, fmt: &FormatMode) {
    let result = parse(arg.trim()).and_then(|stmt| {
        let expr = match stmt {
            Stmt::Expr(e) | Stmt::Assign(_, e) => e,
            _ => return Err("Block statements are not valid in disp()".to_string()),
        };
        eval(&expr, env)
    });
    match result {
        Ok(v) => match &v {
            Value::Void => {}
            Value::Matrix(_) => {
                if let Some(full) = format_value_full(&v, fmt) {
                    println!("{full}");
                }
            }
            Value::Scalar(n) => println!("{}", format_scalar(*n, base, fmt)),
            Value::Complex(re, im) => println!("{}", format_complex(*re, *im, fmt)),
            Value::Str(s) | Value::StringObj(s) => println!("{s}"),
        },
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().expect("Failed to flush stdout");
}

#[cfg(test)]
#[path = "repl_tests.rs"]
mod tests;
