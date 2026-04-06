use std::io::{BufRead, Write};

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use ccalc_engine::env::{Env, Value, config_dir, load_workspace_default, save_workspace_default};
use ccalc_engine::eval::{
    Base, eval, format_complex, format_number, format_scalar, format_value_full,
};
use ccalc_engine::parser::{Stmt, is_partial, parse};

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
fn evaluate(input: &str, env: &mut Env) -> Result<EvalResult, String> {
    let expanded = if is_partial(input) {
        format!("ans {}", input)
    } else {
        input.to_string()
    };

    match parse(&expanded)? {
        Stmt::Assign(name, expr) => {
            let val = eval(&expr, env)?;
            env.insert(name.clone(), val.clone());
            // Assignments do not update ans (MATLAB semantics)
            Ok(EvalResult::Assigned(name, val))
        }
        Stmt::Expr(expr) => {
            let val = eval(&expr, env)?;
            env.insert("ans".to_string(), val.clone()); // always update ans
            Ok(EvalResult::Value(val))
        }
    }
}

/// Splits a raw input line into `(statement, silent)` pairs.
///
/// - Strips inline `%` comments (outside string literals).
/// - Splits on `;` outside string literals and outside `[...]` brackets.
/// - `silent = true` when the statement was followed by `;`,
///   meaning output is suppressed.
fn split_stmts(input: &str) -> Vec<(&str, bool)> {
    let mut semis: Vec<usize> = Vec::new();
    let mut comment_at = input.len();
    let mut in_sq = false;
    let mut in_dq = false;

    let mut bracket_depth: i32 = 0;
    for (i, c) in input.char_indices() {
        match c {
            '\'' if !in_dq => {
                if in_sq {
                    in_sq = false; // closing string quote
                } else {
                    // Transpose operator if preceded by ident char, digit, ')', ']', or another '
                    // (i.e. the context is "rvalue '"); otherwise it opens a string literal.
                    let before = input[..i].trim_end_matches([' ', '\t']);
                    let is_transpose = before.ends_with(|c: char| {
                        c.is_alphanumeric() || c == '_' || c == ')' || c == ']' || c == '\''
                    });
                    if !is_transpose {
                        in_sq = true;
                    }
                }
            }
            '"' if !in_sq => in_dq = !in_dq,
            '[' if !in_sq && !in_dq => bracket_depth += 1,
            ']' if !in_sq && !in_dq => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            }
            '%' if !in_sq && !in_dq && bracket_depth == 0 => {
                comment_at = i;
                break;
            }
            ';' if !in_sq && !in_dq && bracket_depth == 0 => semis.push(i),
            _ => {}
        }
    }

    let content = input[..comment_at].trim_end();
    if content.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut start = 0;

    for &sc in &semis {
        if sc >= content.len() {
            break;
        }
        let part = content[start..sc].trim();
        if !part.is_empty() {
            result.push((part, true));
        }
        start = sc + 1;
    }

    if start <= content.len() {
        let last = content[start..].trim();
        if !last.is_empty() {
            result.push((last, false));
        }
    }

    result
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

fn format_prompt_ans(env: &Env, precision: usize, base: Base) -> String {
    match env.get("ans") {
        Some(Value::Scalar(n)) => format_scalar(*n, precision, base),
        Some(Value::Matrix(m)) => format!("[{}×{}]", m.nrows(), m.ncols()),
        Some(Value::Complex(re, im)) => format_complex(*re, *im, precision),
        None => "0".to_string(),
    }
}

pub fn run() {
    let mut env = new_env();
    let config_path = config_dir().join("config.toml");
    let cfg = crate::config::load_or_create(&config_path);
    let mut precision: usize = cfg.precision();
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

    'repl: loop {
        let prompt = format!("[ {} ]: ", format_prompt_ans(&env, precision, base));
        let input = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
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

        for (stmt, silent) in split_stmts(trimmed) {
            // Built-in commands
            match stmt {
                "exit" | "quit" => break 'repl,
                "cls" => {
                    clear_screen();
                    continue;
                }
                "who" => {
                    print_who(&env, precision, base);
                    continue;
                }
                "clear" => {
                    env.clear();
                    continue;
                }
                "p" => {
                    println!("precision: {precision}");
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
                    print_all_bases(ans(&env), precision);
                    continue;
                }
                "ws" => {
                    match save_workspace_default(&env) {
                        Ok(()) => println!("Workspace saved."),
                        Err(e) => eprintln!("Error: {e}"),
                    }
                    continue;
                }
                "wl" => {
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
                    println!("precision:   {precision}");
                    println!("base:        {}", format_base_name(base));
                    continue;
                }
                _ => {}
            }

            // config reload
            if stmt == "config reload" {
                match crate::config::load(&config_path) {
                    Ok(cfg) => {
                        precision = cfg.precision();
                        base = cfg.base();
                        println!("Config reloaded.");
                        println!("precision:   {precision}");
                        println!("base:        {}", format_base_name(base));
                    }
                    Err(e) => eprintln!("Error: {e}"),
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

            // Precision command: p<N>
            if let Some(p) = parse_precision_cmd(stmt) {
                precision = p;
                continue;
            }

            // disp(expr) — print value without updating ans
            if let Some(arg) = parse_disp_cmd(stmt) {
                handle_disp(arg, &env, precision, base);
                continue;
            }

            // fprintf('fmt') — print formatted string
            if let Some(arg) = parse_fprintf_cmd(stmt) {
                handle_fprintf(arg);
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

            match evaluate(to_eval, &mut env) {
                Ok(result) => {
                    if !silent {
                        match result {
                            EvalResult::Assigned(name, val) => match &val {
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&val, precision) {
                                        println!("{name} =");
                                        println!("{full}");
                                        println!();
                                    }
                                }
                                Value::Scalar(v) => {
                                    println!("{name} = {}", format_scalar(*v, precision, base));
                                }
                                Value::Complex(re, im) => {
                                    println!("{name} = {}", format_complex(*re, *im, precision));
                                }
                            },
                            EvalResult::Value(val) => match &val {
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&val, precision) {
                                        println!("ans =");
                                        println!("{full}");
                                        println!();
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
                                        print_all_bases(*v, precision);
                                    }
                                }
                                Value::Complex(re, im) => {
                                    println!("{}", format_complex(*re, *im, precision));
                                }
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
    let mut base = Base::Dec;
    let trimmed = expr.trim();

    if let Some(arg) = parse_disp_cmd(trimmed) {
        handle_disp(arg, &env, 10, base);
        return;
    }
    if let Some(arg) = parse_fprintf_cmd(trimmed) {
        handle_fprintf(arg);
        return;
    }

    let (to_eval, base_suffix) = extract_base_suffix(trimmed);
    let show_all = matches!(base_suffix, Some(BaseSuffix::ShowAll));
    if let Some(BaseSuffix::Switch(b)) = base_suffix {
        base = b;
    }
    match evaluate(to_eval, &mut env) {
        Ok(result) => match result {
            EvalResult::Assigned(name, v) => match &v {
                Value::Matrix(_) => {
                    if let Some(full) = format_value_full(&v, 10) {
                        println!("{name} =");
                        println!("{full}");
                    }
                }
                Value::Scalar(n) => {
                    println!("{} = {}", name, format_scalar(*n, 10, base));
                }
                Value::Complex(re, im) => {
                    println!("{} = {}", name, format_complex(*re, *im, 10));
                }
            },
            EvalResult::Value(v) => match &v {
                Value::Matrix(_) => {
                    if let Some(full) = format_value_full(&v, 10) {
                        println!("ans =");
                        println!("{full}");
                    }
                }
                Value::Scalar(n) => {
                    if show_all {
                        print_all_bases(*n, 10);
                    } else {
                        println!("{}", format_scalar(*n, 10, base));
                    }
                }
                Value::Complex(re, im) => {
                    println!("{}", format_complex(*re, *im, 10));
                }
            },
        },
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Process lines from a non-interactive reader (pipe, file redirect).
/// Prints one result per expression line; no prompts.
pub fn run_pipe(reader: impl BufRead) {
    let mut env = new_env();
    let mut precision: usize = 10;
    let mut base = Base::Dec;

    'lines: for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        };
        let trimmed = line.trim();

        for (stmt, silent) in split_stmts(trimmed) {
            // Built-in commands (subset relevant in pipe mode)
            match stmt {
                "exit" | "quit" => break 'lines,
                "clear" => {
                    env.clear();
                    continue;
                }
                "cls" | "who" | "help" | "?" | "config" => continue, // no-op in pipe mode
                "p" => {
                    println!("precision: {precision}");
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
                    print_all_bases(ans(&env), precision);
                    continue;
                }
                "ws" => {
                    let _ = save_workspace_default(&env);
                    continue;
                }
                "wl" => {
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

            // clear <name>
            if let Some(name) = stmt.strip_prefix("clear ").map(str::trim) {
                if !name.is_empty() {
                    env.remove(name);
                }
                continue;
            }

            // Precision command: p<N>
            if let Some(p) = parse_precision_cmd(stmt) {
                precision = p;
                continue;
            }

            // disp(expr) — print value without updating ans
            if let Some(arg) = parse_disp_cmd(stmt) {
                handle_disp(arg, &env, precision, base);
                continue;
            }

            // fprintf('fmt') — print formatted string
            if let Some(arg) = parse_fprintf_cmd(stmt) {
                handle_fprintf(arg);
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
                                        println!("{name} =");
                                        println!("{full}");
                                        println!();
                                    }
                                }
                                Value::Scalar(n) => {
                                    println!("{} = {}", name, format_scalar(*n, precision, base));
                                }
                                Value::Complex(re, im) => {
                                    println!("{} = {}", name, format_complex(*re, *im, precision));
                                }
                            },
                            EvalResult::Value(v) => match &v {
                                Value::Matrix(_) => {
                                    if let Some(full) = format_value_full(&v, precision) {
                                        println!("ans =");
                                        println!("{full}");
                                        println!();
                                    }
                                }
                                Value::Scalar(n) => {
                                    if show_all {
                                        let i = n.round() as i64;
                                        let u = i.unsigned_abs();
                                        let sign = if i < 0 { "-" } else { "" };
                                        println!("2  - {}0b{:b}", sign, u);
                                        println!("8  - {}0o{:o}", sign, u);
                                        println!(
                                            "10 - {}",
                                            format_scalar(*n, precision, Base::Dec)
                                        );
                                        println!("16 - {}0x{:X}", sign, u);
                                    } else {
                                        println!("{}", format_scalar(*n, precision, base));
                                    }
                                }
                                Value::Complex(re, im) => {
                                    println!("{}", format_complex(*re, *im, precision));
                                }
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

fn print_who(env: &Env, precision: usize, base: Base) {
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
            Value::Scalar(n) => println!("ans = {}", format_scalar(*n, precision, base)),
            Value::Matrix(m) => println!("ans = [{}×{} double]", m.nrows(), m.ncols()),
            Value::Complex(re, im) => println!("ans = {}", format_complex(*re, *im, precision)),
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
            Value::Scalar(n) => {
                scalars.push(format!("{} = {}", name, format_scalar(*n, precision, base)));
            }
            Value::Complex(re, im) => {
                scalars.push(format!(
                    "{} = {}",
                    name,
                    format_complex(*re, *im, precision)
                ));
            }
            Value::Matrix(m) => {
                matrices.push(format!("{} = [{}×{} double]", name, m.nrows(), m.ncols()));
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
fn print_all_bases(n: f64, precision: usize) {
    let i = n.round() as i64;
    let u = i.unsigned_abs();
    let sign = if i < 0 { "-" } else { "" };
    println!("2  - {}0b{:b}", sign, u);
    println!("8  - {}0o{:o}", sign, u);
    println!("10 - {}", format_scalar(n, precision, Base::Dec));
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
                    result.push_str(&format_complex(*re, *im, 10));
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
fn parse_precision_cmd(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.first() == Some(&b'p') && bytes.len() > 1 {
        input[1..].parse::<usize>().ok().filter(|&n| n <= 15)
    } else {
        None
    }
}

/// Extracts the argument string from a `disp(...)` call.
/// Returns `None` if the input does not match the pattern.
fn parse_disp_cmd(input: &str) -> Option<&str> {
    let inner = input.strip_prefix("disp(")?.strip_suffix(')')?;
    if inner.is_empty() { None } else { Some(inner) }
}

/// Extracts the argument string from a `fprintf(...)` call.
fn parse_fprintf_cmd(input: &str) -> Option<&str> {
    input.strip_prefix("fprintf(")?.strip_suffix(')')
}

/// Evaluates `arg` and prints the result. Does not update `ans`.
fn handle_disp(arg: &str, env: &Env, precision: usize, base: Base) {
    let result = parse(arg.trim()).and_then(|stmt| {
        let expr = match stmt {
            Stmt::Expr(e) => e,
            Stmt::Assign(_, e) => e,
        };
        eval(&expr, env)
    });
    match result {
        Ok(v) => match &v {
            Value::Matrix(_) => {
                if let Some(full) = format_value_full(&v, precision) {
                    println!("{full}");
                }
            }
            Value::Scalar(n) => println!("{}", format_scalar(*n, precision, base)),
            Value::Complex(re, im) => println!("{}", format_complex(*re, *im, precision)),
        },
        Err(e) => eprintln!("Error: {e}"),
    }
}

/// Prints a formatted string literal. Phase 1: single string arg, escape sequences only.
fn handle_fprintf(arg: &str) {
    let s = arg.trim();
    let content = if let Some(inner) = s.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        inner
    } else if let Some(inner) = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        inner
    } else {
        eprintln!("Error: fprintf requires a string literal");
        return;
    };
    print!("{}", process_escapes(content));
    let _ = std::io::stdout().flush();
}

/// Processes `\n`, `\t`, `\\` escape sequences in a string.
fn process_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('"'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().expect("Failed to flush stdout");
}

#[cfg(test)]
#[path = "repl_tests.rs"]
mod tests;
