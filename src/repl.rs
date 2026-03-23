use std::io::Write;

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

        match parse(&full_expanded, accumulator).and_then(|ast| eval(&ast)) {
            Ok(result) => {
                accumulator = result;
                match directive {
                    Some(Directive::Store(idx)) => memory.set(idx, result),
                    Some(Directive::Compound(idx, op)) => {
                        let cell = memory.get(idx);
                        let new_val = match op {
                            CompoundOp::Add => Ok(cell + result),
                            CompoundOp::Sub => Ok(cell - result),
                            CompoundOp::Mul => Ok(cell * result),
                            CompoundOp::Div => {
                                if result == 0.0 {
                                    Err("Division by zero")
                                } else {
                                    Ok(cell / result)
                                }
                            }
                            CompoundOp::Mod => {
                                if result == 0.0 {
                                    Err("Modulo by zero")
                                } else {
                                    Ok(cell % result)
                                }
                            }
                            CompoundOp::Pow => Ok(cell.powf(result)),
                        };
                        match new_val {
                            Ok(v) => {
                                memory.set(idx, v);
                                accumulator = v;
                            }
                            Err(e) => eprintln!("Error: {e}"),
                        }
                    }
                    None => {}
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }
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
}
