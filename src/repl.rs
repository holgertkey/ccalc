use std::io::{BufRead, Write};

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::eval::{eval, format_number};
use crate::memory::{
    extract_directive, expand_memory_refs, parse_standalone_cmd, CompoundOp, Directive, Memory,
    StandaloneCmd,
};
use crate::parser::{is_partial, parse};

pub fn run() {
    let mut accumulator: f64 = 0.0;
    let mut memory = Memory::new();
    let mut rl = DefaultEditor::new().expect("Failed to initialize line editor");

    loop {
        let prompt = format!("[ {} ]: ", format_number(accumulator));
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

        // Built-in commands
        match trimmed {
            "q" => break,
            "c" => {
                accumulator = 0.0;
                continue;
            }
            "cls" => {
                clear_screen();
                continue;
            }
            "m" => {
                memory.display_nonzero();
                continue;
            }
            "mc" => {
                memory.clear_all();
                continue;
            }
            _ => {}
        }

        // Standalone memory commands: m[1-9], mc[1-9]
        if let Some(cmd) = parse_standalone_cmd(trimmed) {
            match cmd {
                StandaloneCmd::StoreAcc(idx) => memory.set(idx, accumulator),
                StandaloneCmd::ClearOne(idx) => memory.clear_one(idx),
            }
            continue;
        }

        // Expression (with optional trailing memory directive and/or m[1-9] value refs)
        let (expr_part, directive) = extract_directive(trimmed);

        let base_expr = if is_partial(expr_part) {
            format!("{} {}", format_number(accumulator), expr_part)
        } else {
            expr_part.to_string()
        };

        let (mem_expanded, mem_display) = expand_memory_refs(&base_expr, &memory);
        let (full_expanded, acc_display) = expand_acc(&mem_expanded, accumulator);

        if let Some(display) = acc_display.or(mem_display) {
            println!("{}", display);
        }

        match evaluate_expanded(&full_expanded, directive, &mut accumulator, &mut memory) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}

/// Evaluate a single expression string in pipe/non-interactive mode.
/// Prints the result and exits with code 1 on error.
pub fn run_expr(expr: &str) {
    let mut acc: f64 = 0.0;
    let mut mem = Memory::new();
    match evaluate(expr.trim(), &mut acc, &mut mem) {
        Ok(result) => println!("{}", format_number(result)),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Process lines from a non-interactive reader (pipe, file redirect).
/// Prints one result per expression line; no prompts.
pub fn run_pipe(reader: impl BufRead) {
    let mut acc: f64 = 0.0;
    let mut mem = Memory::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Built-in commands (subset relevant in pipe mode)
        match trimmed {
            "q" => break,
            "c" => {
                acc = 0.0;
                continue;
            }
            "mc" => {
                mem.clear_all();
                continue;
            }
            "cls" | "m" => continue, // no-op in pipe mode
            _ => {}
        }

        // Standalone memory commands: m[1-9], mc[1-9]
        if let Some(cmd) = parse_standalone_cmd(trimmed) {
            match cmd {
                StandaloneCmd::StoreAcc(idx) => mem.set(idx, acc),
                StandaloneCmd::ClearOne(idx) => mem.clear_one(idx),
            }
            continue;
        }

        match evaluate(trimmed, &mut acc, &mut mem) {
            Ok(result) => println!("{}", format_number(result)),
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}

/// Evaluate a line (expression + optional directive) updating acc and memory.
/// Handles partial expressions, memory ref expansion, and acc substitution.
fn evaluate(trimmed: &str, acc: &mut f64, mem: &mut Memory) -> Result<f64, String> {
    let (expr_part, directive) = extract_directive(trimmed);

    let base_expr = if is_partial(expr_part) {
        format!("{} {}", format_number(*acc), expr_part)
    } else {
        expr_part.to_string()
    };

    let (mem_expanded, _) = expand_memory_refs(&base_expr, mem);
    let (full_expanded, _) = expand_acc(&mem_expanded, *acc);

    evaluate_expanded(&full_expanded, directive, acc, mem)
}

/// Parse and evaluate an already-expanded expression string, applying the directive.
fn evaluate_expanded(
    full_expanded: &str,
    directive: Option<Directive>,
    acc: &mut f64,
    mem: &mut Memory,
) -> Result<f64, String> {
    let result = parse(full_expanded, *acc).and_then(|ast| eval(&ast))?;
    *acc = result;

    match directive {
        Some(Directive::Store(idx)) => {
            mem.set(idx, result);
        }
        Some(Directive::Compound(idx, op)) => {
            let cell = mem.get(idx);
            let new_val = apply_compound(cell, result, op)?;
            mem.set(idx, new_val);
            *acc = new_val;
        }
        None => {}
    }

    Ok(*acc)
}

fn apply_compound(cell: f64, result: f64, op: CompoundOp) -> Result<f64, String> {
    match op {
        CompoundOp::Add => Ok(cell + result),
        CompoundOp::Sub => Ok(cell - result),
        CompoundOp::Mul => Ok(cell * result),
        CompoundOp::Div => {
            if result == 0.0 {
                Err("Division by zero".to_string())
            } else {
                Ok(cell / result)
            }
        }
        CompoundOp::Mod => {
            if result == 0.0 {
                Err("Modulo by zero".to_string())
            } else {
                Ok(cell % result)
            }
        }
        CompoundOp::Pow => Ok(cell.powf(result)),
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().expect("Failed to flush stdout");
}

/// Replaces `acc` (word-boundary) and `fn()` (empty-arg calls) with the numeric
/// accumulator value. Returns `(expanded, display)` where `display` is `Some`
/// only when at least one substitution was performed.
fn expand_acc(input: &str, acc: f64) -> (String, Option<String>) {
    let acc_str = format_number(acc);
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut result = String::with_capacity(input.len() + 4);
    let mut had = false;
    let mut i = 0;

    while i < n {
        // `acc` as a whole word (not part of a longer identifier)
        if i + 3 <= n
            && chars[i] == 'a'
            && chars[i + 1] == 'c'
            && chars[i + 2] == 'c'
            && (i == 0 || !is_ident_char(chars[i - 1]))
            && (i + 3 >= n || !is_ident_char(chars[i + 3]))
        {
            result.push_str(&acc_str);
            had = true;
            i += 3;
            continue;
        }
        // `fn()` — empty-arg call: `(` immediately followed by `)`, preceded by identifier
        if chars[i] == '('
            && i + 1 < n
            && chars[i + 1] == ')'
            && i > 0
            && is_ident_char(chars[i - 1])
        {
            result.push('(');
            result.push_str(&acc_str);
            result.push(')');
            had = true;
            i += 2;
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }

    let display = if had { Some(result.clone()) } else { None };
    (result, display)
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- expand_acc tests ---

    #[test]
    fn test_expand_acc_standalone() {
        let (expr, display) = expand_acc("acc", 42.0);
        assert_eq!(expr, "42");
        assert_eq!(display, Some("42".to_string()));
    }

    #[test]
    fn test_expand_acc_in_expr() {
        let (expr, display) = expand_acc("2 + acc + 2", 10.0);
        assert_eq!(expr, "2 + 10 + 2");
        assert_eq!(display, Some("2 + 10 + 2".to_string()));
    }

    #[test]
    fn test_expand_acc_no_match() {
        let (expr, display) = expand_acc("2 + 3", 10.0);
        assert_eq!(expr, "2 + 3");
        assert!(display.is_none());
    }

    #[test]
    fn test_expand_acc_empty_call() {
        let (expr, display) = expand_acc("sin()", 40.0);
        assert_eq!(expr, "sin(40)");
        assert_eq!(display, Some("sin(40)".to_string()));
    }

    #[test]
    fn test_expand_acc_in_call_arg() {
        let (expr, display) = expand_acc("sqrt(acc)", 9.0);
        assert_eq!(expr, "sqrt(9)");
        assert_eq!(display, Some("sqrt(9)".to_string()));
    }

    #[test]
    fn test_expand_acc_combined_empty_call_and_acc() {
        let (expr, display) = expand_acc("sqrt() + acc", 9.0);
        assert_eq!(expr, "sqrt(9) + 9");
        assert_eq!(display, Some("sqrt(9) + 9".to_string()));
    }

    #[test]
    fn test_expand_acc_word_boundary() {
        // longer identifiers containing "acc" should not be affected
        let (expr, display) = expand_acc("access", 10.0);
        assert_eq!(expr, "access");
        assert!(display.is_none());
    }

    #[test]
    fn test_expand_acc_with_mem_already_expanded() {
        // simulate combined: m1 was already replaced by 14, acc=14
        let (expr, display) = expand_acc("12 + acc + 14", 14.0);
        assert_eq!(expr, "12 + 14 + 14");
        assert_eq!(display, Some("12 + 14 + 14".to_string()));
    }

    // --- evaluate / run_pipe tests ---

    fn pipe_output(input: &str) -> Vec<String> {
        use std::io::Cursor;
        let mut output = Vec::new();
        let mut acc: f64 = 0.0;
        let mut mem = Memory::new();
        let reader = Cursor::new(input);
        for line in reader.lines() {
            let line = line.unwrap();
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match trimmed {
                "q" => break,
                "c" => { acc = 0.0; continue; }
                "mc" => { mem.clear_all(); continue; }
                "cls" | "m" => continue,
                _ => {}
            }
            if let Some(cmd) = parse_standalone_cmd(trimmed) {
                match cmd {
                    StandaloneCmd::StoreAcc(idx) => mem.set(idx, acc),
                    StandaloneCmd::ClearOne(idx) => mem.clear_one(idx),
                }
                continue;
            }
            match evaluate(trimmed, &mut acc, &mut mem) {
                Ok(result) => output.push(format_number(result)),
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
    fn test_pipe_reset_with_c() {
        let lines = "10\nc\n+ 5";
        assert_eq!(pipe_output(lines), vec!["10", "5"]);
    }

    #[test]
    fn test_pipe_quit_with_q() {
        let lines = "1\n2\nq\n3";
        assert_eq!(pipe_output(lines), vec!["1", "2"]);
    }

    #[test]
    fn test_pipe_empty_lines_skipped() {
        let lines = "1\n\n\n2";
        assert_eq!(pipe_output(lines), vec!["1", "2"]);
    }

    #[test]
    fn test_pipe_error_reported() {
        let out = pipe_output("1 / 0");
        assert!(out[0].starts_with("Error:"));
    }

    #[test]
    fn test_pipe_memory_store_and_use() {
        // Store 7 in m1, then use m1
        let lines = "7 m1\nm1 + 3";
        assert_eq!(pipe_output(lines), vec!["7", "10"]);
    }

    #[test]
    fn test_evaluate_simple() {
        let mut acc = 0.0;
        let mut mem = Memory::new();
        assert_eq!(evaluate("3 * 4", &mut acc, &mut mem).unwrap(), 12.0);
        assert_eq!(acc, 12.0);
    }

    #[test]
    fn test_evaluate_partial_adds_to_acc() {
        let mut acc = 10.0;
        let mut mem = Memory::new();
        assert_eq!(evaluate("+ 5", &mut acc, &mut mem).unwrap(), 15.0);
    }
}
