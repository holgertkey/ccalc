use std::io::{BufRead, Write};

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use ccalc_engine::env::{Env, config_dir, load_workspace_default, save_workspace_default};
use ccalc_engine::eval::{Base, eval, format_number, format_value};
use ccalc_engine::parser::{Stmt, is_partial, parse};

/// Result of evaluating one input line.
enum EvalResult {
    /// Assignment `name = expr` was executed; `name` was set to `val`.
    Assigned(String, f64),
    /// Standalone expression; result stored in `ans`.
    Value(f64),
}

/// Parse and evaluate one input string, updating `env`.
/// Handles partial expressions (starting with an operator) by prepending `ans`.
fn evaluate(input: &str, env: &mut Env) -> Result<EvalResult, String> {
    let expanded = if is_partial(input) {
        format!("ans {}", input)
    } else {
        input.to_string()
    };

    match parse(&expanded)? {
        Stmt::Assign(name, expr) => {
            let val = eval(&expr, env)?;
            env.insert(name.clone(), val);
            Ok(EvalResult::Assigned(name, val))
        }
        Stmt::Expr(expr) => {
            let val = eval(&expr, env)?;
            env.insert("ans".to_string(), val);
            Ok(EvalResult::Value(val))
        }
    }
}

fn ans(env: &Env) -> f64 {
    env.get("ans").copied().unwrap_or(0.0)
}

fn new_env() -> Env {
    let mut env = Env::new();
    env.insert("ans".to_string(), 0.0);
    env
}

pub fn run() {
    let mut env = new_env();
    let mut precision: usize = 10;
    let mut base = Base::Dec;
    let mut rl = DefaultEditor::new().expect("Failed to initialize line editor");

    let history_path = config_dir().join("history");
    rl.load_history(&history_path).ok();

    println!("ccalc v{}", env!("CARGO_PKG_VERSION"));
    println!();

    loop {
        let prompt = format!("[ {} ]: ", format_value(ans(&env), precision, base));
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

        let (trimmed, silent) = if let Some(t) = trimmed.strip_suffix(';') {
            (t.trim_end(), true)
        } else {
            (trimmed, false)
        };
        if trimmed.is_empty() {
            continue;
        }

        // Built-in commands
        match trimmed {
            "exit" | "quit" => break,
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
            _ => {}
        }

        // clear <name>
        if let Some(name) = trimmed.strip_prefix("clear ").map(str::trim) {
            if !name.is_empty() {
                env.remove(name);
            }
            continue;
        }

        // Precision command: p<N>
        if let Some(p) = parse_precision_cmd(trimmed) {
            precision = p;
            continue;
        }

        // disp(expr) — print value without updating ans
        if let Some(arg) = parse_disp_cmd(trimmed) {
            handle_disp(arg, &env, precision, base);
            continue;
        }

        // fprintf('fmt') — print formatted string
        if let Some(arg) = parse_fprintf_cmd(trimmed) {
            handle_fprintf(arg);
            continue;
        }

        // Extract trailing base suffix (e.g. "0xFF + 0b10 hex", "10 base")
        let (to_eval, base_suffix) = extract_base_suffix(trimmed);
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
                let val = match &result {
                    EvalResult::Assigned(_, v) | EvalResult::Value(v) => *v,
                };
                if !silent {
                    match result {
                        EvalResult::Assigned(_, _) => {}
                        EvalResult::Value(_) => {
                            // Show expanded expression only for plain expressions
                            let to_show: Option<&str> = if let Some(ref s) = base_display {
                                Some(s.as_str())
                            } else {
                                expanded.as_deref()
                            };
                            if let Some(display) = to_show {
                                println!("{display}");
                            }
                            if show_all_bases {
                                print_all_bases(val, precision);
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {e}"),
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
        Ok(result) => {
            let val = match result {
                EvalResult::Assigned(name, v) => {
                    println!("{} = {}", name, format_value(v, 10, base));
                    return;
                }
                EvalResult::Value(v) => v,
            };
            if show_all {
                print_all_bases(val, 10);
            } else {
                println!("{}", format_value(val, 10, base));
            }
        }
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

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        };
        let trimmed = line.trim();
        // Strip inline comments
        let trimmed = trimmed.split('%').next().unwrap_or("").trim_end();
        let (trimmed, silent) = if let Some(t) = trimmed.strip_suffix(';') {
            (t.trim_end(), true)
        } else {
            (trimmed, false)
        };
        if trimmed.is_empty() {
            continue;
        }

        // Built-in commands (subset relevant in pipe mode)
        match trimmed {
            "exit" | "quit" => break,
            "clear" => {
                env.clear();
                continue;
            }
            "cls" | "who" => continue, // no-op in pipe mode
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

        // clear <name>
        if let Some(name) = trimmed.strip_prefix("clear ").map(str::trim) {
            if !name.is_empty() {
                env.remove(name);
            }
            continue;
        }

        // Precision command: p<N>
        if let Some(p) = parse_precision_cmd(trimmed) {
            precision = p;
            continue;
        }

        // disp(expr) — print value without updating ans
        if let Some(arg) = parse_disp_cmd(trimmed) {
            handle_disp(arg, &env, precision, base);
            continue;
        }

        // fprintf('fmt') — print formatted string
        if let Some(arg) = parse_fprintf_cmd(trimmed) {
            handle_fprintf(arg);
            continue;
        }

        let (to_eval, base_suffix) = extract_base_suffix(trimmed);
        let show_all = matches!(base_suffix, Some(BaseSuffix::ShowAll));
        if let Some(BaseSuffix::Switch(b)) = base_suffix {
            base = b;
        }

        match evaluate(to_eval, &mut env) {
            Ok(result) => {
                if !silent {
                    let val = match result {
                        EvalResult::Assigned(name, v) => {
                            println!("{} = {}", name, format_value(v, precision, base));
                            v
                        }
                        EvalResult::Value(v) => {
                            if show_all {
                                let i = v.round() as i64;
                                let u = i.unsigned_abs();
                                let sign = if i < 0 { "-" } else { "" };
                                println!("2  - {}0b{:b}", sign, u);
                                println!("8  - {}0o{:o}", sign, u);
                                println!("10 - {}", format_value(v, precision, Base::Dec));
                                println!("16 - {}0x{:X}", sign, u);
                            } else {
                                println!("{}", format_value(v, precision, base));
                            }
                            v
                        }
                    };
                    let _ = val;
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}

fn print_who(env: &Env, precision: usize, base: Base) {
    let mut vars: Vec<(&String, &f64)> = env.iter().collect();
    vars.sort_by_key(|(k, _)| k.as_str());
    for (name, val) in vars {
        println!("{} = {}", name, format_value(*val, precision, base));
    }
}

/// Prints a value in all four bases.
fn print_all_bases(n: f64, precision: usize) {
    let i = n.round() as i64;
    let u = i.unsigned_abs();
    let sign = if i < 0 { "-" } else { "" };
    println!("2  - {}0b{:b}", sign, u);
    println!("8  - {}0o{:o}", sign, u);
    println!("10 - {}", format_value(n, precision, Base::Dec));
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
            if let Some(&val) = env.get(&ident) {
                result.push_str(&format_for_base(val, base));
                replaced = true;
            } else {
                result.push_str(&ident);
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
        Ok(v) => println!("{}", format_value(v, precision, base)),
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
mod tests {
    use super::*;

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
        env.insert("x".to_string(), 10.0);
        assert_eq!(
            expand_vars_for_display("x + 5", &env, Base::Dec),
            Some("10 + 5".to_string())
        );
    }

    #[test]
    fn test_expand_vars_multiple() {
        let mut env = new_env();
        env.insert("ans".to_string(), 13.0);
        env.insert("x".to_string(), 10.0);
        env.insert("y".to_string(), 20.0);
        assert_eq!(
            expand_vars_for_display("ans + x + y", &env, Base::Dec),
            Some("13 + 10 + 20".to_string())
        );
    }

    #[test]
    fn test_expand_vars_unknown_ident_preserved() {
        let mut env = new_env();
        env.insert("x".to_string(), 5.0);
        // sqrt is not in env — should stay as-is
        assert_eq!(
            expand_vars_for_display("sqrt(x)", &env, Base::Dec),
            Some("sqrt(5)".to_string())
        );
    }

    #[test]
    fn test_expand_vars_in_hex_base() {
        let mut env = new_env();
        env.insert("x".to_string(), 255.0);
        assert_eq!(
            expand_vars_for_display("x + 1", &env, Base::Hex),
            Some("0xFF + 1".to_string())
        );
    }

    // --- evaluate tests ---

    #[test]
    fn test_evaluate_simple() {
        let mut env = Env::new();
        let result = evaluate("3 * 4", &mut env).unwrap();
        assert!(matches!(result, EvalResult::Value(12.0)));
        assert_eq!(ans(&env), 12.0);
    }

    #[test]
    fn test_evaluate_partial_adds_to_ans() {
        let mut env = Env::new();
        env.insert("ans".to_string(), 10.0);
        let result = evaluate("+ 5", &mut env).unwrap();
        assert!(matches!(result, EvalResult::Value(15.0)));
        assert_eq!(ans(&env), 15.0);
    }

    #[test]
    fn test_evaluate_assignment() {
        let mut env = Env::new();
        let result = evaluate("x = 7", &mut env).unwrap();
        assert!(matches!(result, EvalResult::Assigned(ref n, 7.0) if n == "x"));
        assert_eq!(env.get("x"), Some(&7.0));
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

        for line in reader.lines() {
            let line = line.unwrap();
            let trimmed = line.trim();
            let trimmed = trimmed.split('%').next().unwrap_or("").trim_end();
            let (trimmed, silent) = if let Some(t) = trimmed.strip_suffix(';') {
                (t.trim_end(), true)
            } else {
                (trimmed, false)
            };
            if trimmed.is_empty() {
                continue;
            }
            match trimmed {
                "exit" | "quit" => break,
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
            if let Some(name) = trimmed.strip_prefix("clear ").map(str::trim) {
                if !name.is_empty() {
                    env.remove(name);
                }
                continue;
            }
            if let Some(p) = parse_precision_cmd(trimmed) {
                precision = p;
                continue;
            }
            // disp(expr) — push formatted value without updating ans
            if let Some(arg) = parse_disp_cmd(trimmed) {
                let result = parse(arg.trim()).and_then(|stmt| {
                    let expr = match stmt {
                        Stmt::Expr(e) => e,
                        Stmt::Assign(_, e) => e,
                    };
                    eval(&expr, &env)
                });
                match result {
                    Ok(v) => output.push(format_value(v, precision, base)),
                    Err(e) => output.push(format!("Error: {e}")),
                }
                continue;
            }
            // fprintf('fmt') — push processed string
            if let Some(arg) = parse_fprintf_cmd(trimmed) {
                let s = arg.trim();
                let content = if let Some(inner) =
                    s.strip_prefix('\'').and_then(|s| s.strip_suffix('\''))
                {
                    process_escapes(inner)
                } else if let Some(inner) = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                    process_escapes(inner)
                } else {
                    "Error: fprintf requires a string literal".to_string()
                };
                output.push(content);
                continue;
            }
            let (to_eval, base_suffix) = extract_base_suffix(trimmed);
            let show_all = matches!(base_suffix, Some(BaseSuffix::ShowAll));
            if let Some(BaseSuffix::Switch(b)) = base_suffix {
                base = b;
            }
            match evaluate(to_eval, &mut env) {
                Ok(result) => {
                    if !silent {
                        match result {
                            EvalResult::Assigned(name, v) => {
                                output.push(format!(
                                    "{} = {}",
                                    name,
                                    format_value(v, precision, base)
                                ));
                            }
                            EvalResult::Value(v) => {
                                if show_all {
                                    let i = v.round() as i64;
                                    let u = i.unsigned_abs();
                                    let sign = if i < 0 { "-" } else { "" };
                                    output.push(format!("2  - {}0b{:b}", sign, u));
                                    output.push(format!("8  - {}0o{:o}", sign, u));
                                    output.push(format!(
                                        "10 - {}",
                                        format_value(v, precision, Base::Dec)
                                    ));
                                    output.push(format!("16 - {}0x{:X}", sign, u));
                                } else {
                                    output.push(format_value(v, precision, base));
                                }
                            }
                        }
                    }
                }
                Err(e) => output.push(format!("Error: {e}")),
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
        let out = pipe_output("10;\n+ 5");
        assert_eq!(out, vec!["15"]);
    }

    #[test]
    fn test_pipe_semicolon_still_updates_ans() {
        let out = pipe_output("10;\ndisp(ans)");
        assert_eq!(out, vec!["10"]);
    }

    #[test]
    fn test_pipe_semicolon_with_comment() {
        let out = pipe_output("10; % intermediate\ndisp(ans)");
        assert_eq!(out, vec!["10"]);
    }

    #[test]
    fn test_pipe_semicolon_variable_store() {
        let out = pipe_output("7;\nx = ans;\nx + 3");
        assert_eq!(out, vec!["10"]);
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
}
