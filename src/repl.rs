use std::io::{BufRead, Write};

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::eval::{Base, eval, format_number, format_value};
use crate::memory::{
    extract_directive, expand_memory_refs, parse_standalone_cmd, CompoundOp, Directive, Memory,
    StandaloneCmd,
};
use crate::parser::{is_partial, parse};

pub fn run() {
    let mut accumulator: f64 = 0.0;
    let mut memory = Memory::new();
    let mut precision: usize = 10;
    let mut base = Base::Dec;
    let mut rl = DefaultEditor::new().expect("Failed to initialize line editor");

    loop {
        let prompt = format!("[ {} ]: ", format_value(accumulator, precision, base));
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
                memory.display_nonzero(|v| format_value(v, precision, base));
                continue;
            }
            "mc" => {
                memory.clear_all();
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
                print_all_bases(accumulator, precision);
                continue;
            }
            _ => {}
        }

        // Precision command: p<N>
        if let Some(p) = parse_precision_cmd(trimmed) {
            precision = p;
            continue;
        }

        // Standalone memory commands: m[1-9], mc[1-9]
        if let Some(cmd) = parse_standalone_cmd(trimmed) {
            match cmd {
                StandaloneCmd::StoreAcc(idx) => memory.set(idx, accumulator),
                StandaloneCmd::ClearOne(idx) => memory.clear_one(idx),
            }
            continue;
        }

        // Extract trailing base suffix (e.g. "0xFF + 0b10 hex")
        let (trimmed_no_base, new_base) = extract_base_suffix(trimmed);
        if let Some(b) = new_base {
            base = b;
        }

        // Expression (with optional trailing memory directive and/or m[1-9] value refs)
        let (expr_part, directive) = extract_directive(trimmed_no_base);

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
    let mut base = Base::Dec;
    match evaluate(expr.trim(), &mut acc, &mut mem, &mut base) {
        Ok(result) => println!("{}", format_value(result, 10, base)),
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
                print_all_bases(acc, precision);
                continue;
            }
            _ => {}
        }

        // Precision command: p<N>
        if let Some(p) = parse_precision_cmd(trimmed) {
            precision = p;
            continue;
        }

        // Standalone memory commands: m[1-9], mc[1-9]
        if let Some(cmd) = parse_standalone_cmd(trimmed) {
            match cmd {
                StandaloneCmd::StoreAcc(idx) => mem.set(idx, acc),
                StandaloneCmd::ClearOne(idx) => mem.clear_one(idx),
            }
            continue;
        }

        match evaluate(trimmed, &mut acc, &mut mem, &mut base) {
            Ok(result) => println!("{}", format_value(result, precision, base)),
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}

/// Evaluate a line (expression + optional base suffix + optional directive)
/// updating acc, mem, and base state.
fn evaluate(
    trimmed: &str,
    acc: &mut f64,
    mem: &mut Memory,
    base: &mut Base,
) -> Result<f64, String> {
    let (after_base, new_base) = extract_base_suffix(trimmed);
    if let Some(b) = new_base {
        *base = b;
    }

    let (expr_part, directive) = extract_directive(after_base);

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

/// Prints the current accumulator value in all four bases.
fn print_all_bases(n: f64, precision: usize) {
    let i = n.round() as i64;
    let u = i.unsigned_abs();
    let sign = if i < 0 { "-" } else { "" };
    println!("2  - {}0b{:b}", sign, u);
    println!("8  - {}0{:o}", sign, u);
    println!("10 - {}", format_value(n, precision, Base::Dec));
    println!("16 - {}{:X}", sign, u);
}

/// Strips a trailing base keyword (`hex`, `dec`, `bin`, `oct`) from an expression.
/// Returns `(remaining_expr, Some(Base))` or `(input, None)` if no suffix found.
fn extract_base_suffix(input: &str) -> (&str, Option<Base>) {
    if let Some(pos) = input.rfind(' ') {
        let token = &input[pos + 1..];
        let before = input[..pos].trim_end();
        if !before.is_empty() {
            let b = match token {
                "hex" => Some(Base::Hex),
                "dec" => Some(Base::Dec),
                "bin" => Some(Base::Bin),
                "oct" => Some(Base::Oct),
                _ => None,
            };
            if b.is_some() {
                return (before, b);
            }
        }
    }
    (input, None)
}

/// Parses a precision command of the form `p<N>` where N is 0–15.
/// Returns `Some(N)` on match, `None` otherwise.
fn parse_precision_cmd(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.first() == Some(&b'p') && bytes.len() > 1 {
        input[1..].parse::<usize>().ok().filter(|&n| n <= 15)
    } else {
        None
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

    // --- extract_base_suffix tests ---

    #[test]
    fn test_extract_base_suffix_hex() {
        let (expr, base) = extract_base_suffix("255 hex");
        assert_eq!(expr, "255");
        assert_eq!(base, Some(Base::Hex));
    }

    #[test]
    fn test_extract_base_suffix_bin() {
        let (expr, base) = extract_base_suffix("10 bin");
        assert_eq!(expr, "10");
        assert_eq!(base, Some(Base::Bin));
    }

    #[test]
    fn test_extract_base_suffix_oct() {
        let (expr, base) = extract_base_suffix("8 oct");
        assert_eq!(expr, "8");
        assert_eq!(base, Some(Base::Oct));
    }

    #[test]
    fn test_extract_base_suffix_dec() {
        let (expr, base) = extract_base_suffix("255 dec");
        assert_eq!(expr, "255");
        assert_eq!(base, Some(Base::Dec));
    }

    #[test]
    fn test_extract_base_suffix_none() {
        let (expr, base) = extract_base_suffix("255 + 10");
        assert_eq!(expr, "255 + 10");
        assert!(base.is_none());
    }

    #[test]
    fn test_extract_base_suffix_complex() {
        let (expr, base) = extract_base_suffix("0xFF + 0b1010 hex");
        assert_eq!(expr, "0xFF + 0b1010");
        assert_eq!(base, Some(Base::Hex));
    }

    #[test]
    fn test_extract_base_suffix_no_space() {
        let (expr, base) = extract_base_suffix("hex");
        assert_eq!(expr, "hex");
        assert!(base.is_none());
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
        assert_eq!(parse_precision_cmd("p16"), None); // exceeds max
        assert_eq!(parse_precision_cmd("pi"), None);  // not numeric
        assert_eq!(parse_precision_cmd("6"), None);   // no 'p' prefix
    }

    // --- evaluate / run_pipe tests ---

    fn pipe_output(input: &str) -> Vec<String> {
        use std::io::Cursor;
        let mut output = Vec::new();
        let mut acc: f64 = 0.0;
        let mut mem = Memory::new();
        let mut base = Base::Dec;
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
                "hex" => { base = Base::Hex; continue; }
                "dec" => { base = Base::Dec; continue; }
                "bin" => { base = Base::Bin; continue; }
                "oct" => { base = Base::Oct; continue; }
                _ => {}
            }
            if let Some(p) = parse_precision_cmd(trimmed) {
                let _ = p; // precision changes not tracked in this helper
                continue;
            }
            if let Some(cmd) = parse_standalone_cmd(trimmed) {
                match cmd {
                    StandaloneCmd::StoreAcc(idx) => mem.set(idx, acc),
                    StandaloneCmd::ClearOne(idx) => mem.clear_one(idx),
                }
                continue;
            }
            match evaluate(trimmed, &mut acc, &mut mem, &mut base) {
                Ok(result) => output.push(format_value(result, 10, base)),
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
    fn test_pipe_hex_literals() {
        assert_eq!(pipe_output("0xFF"), vec!["255"]);
        assert_eq!(pipe_output("0xFF + 0b1010"), vec!["265"]);
    }

    #[test]
    fn test_pipe_hex_base_suffix_changes_display() {
        // After "0xFF + 0b1010 hex", result should display as hex
        let lines = "0xFF + 0b1010 hex";
        assert_eq!(pipe_output(lines), vec!["0x109"]);
    }

    #[test]
    fn test_pipe_base_persists() {
        // Set hex base, subsequent result also in hex
        let lines = "0xFF + 0b1010 hex\n+ 0b10";
        assert_eq!(pipe_output(lines), vec!["0x109", "0x10B"]);
    }

    #[test]
    fn test_pipe_base_switch_dec() {
        let lines = "255 hex\ndec";
        // After hex, 255 shows as 0xFF. After dec, nothing printed (standalone command).
        // Next expression would print in dec.
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
    fn test_evaluate_simple() {
        let mut acc = 0.0;
        let mut mem = Memory::new();
        let mut base = Base::Dec;
        assert_eq!(evaluate("3 * 4", &mut acc, &mut mem, &mut base).unwrap(), 12.0);
        assert_eq!(acc, 12.0);
    }

    #[test]
    fn test_evaluate_partial_adds_to_acc() {
        let mut acc = 10.0;
        let mut mem = Memory::new();
        let mut base = Base::Dec;
        assert_eq!(evaluate("+ 5", &mut acc, &mut mem, &mut base).unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_sets_base_via_suffix() {
        let mut acc = 0.0;
        let mut mem = Memory::new();
        let mut base = Base::Dec;
        evaluate("255 hex", &mut acc, &mut mem, &mut base).unwrap();
        assert_eq!(base, Base::Hex);
        assert_eq!(acc, 255.0);
    }
}
