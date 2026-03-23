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

        let (expanded_expr, display_str) = expand_memory_refs(&base_expr, &memory);

        if let Some(display) = display_str {
            println!("{}", display);
        }

        match parse(&expanded_expr, accumulator).and_then(|ast| eval(&ast)) {
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
