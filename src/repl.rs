use std::io::{self, Write};

use crate::eval::{eval, format_number};
use crate::memory::{
    extract_directive, expand_memory_refs, parse_standalone_cmd, Directive, Memory, StandaloneCmd,
};
use crate::parser::{is_partial, parse};

pub fn run() {
    let mut accumulator: f64 = 0.0;
    let mut memory = Memory::new();

    loop {
        print_prompt(accumulator);

        let input = read_line();
        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

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

        // Standalone memory commands: m[1-9], ma[1-9], ms[1-9], mc[1-9]
        if let Some(cmd) = parse_standalone_cmd(trimmed) {
            match cmd {
                StandaloneCmd::StoreAcc(idx) => memory.set(idx, accumulator),
                StandaloneCmd::AddAcc(idx) => {
                    println!("{}", format_number(memory.add(idx, accumulator)));
                }
                StandaloneCmd::SubAcc(idx) => {
                    println!("{}", format_number(memory.sub(idx, accumulator)));
                }
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

        match parse(&expanded_expr).and_then(|ast| eval(&ast)) {
            Ok(result) => {
                accumulator = result;
                match directive {
                    Some(Directive::Store(idx)) => memory.set(idx, result),
                    Some(Directive::Add(idx)) => {
                        println!("{}", format_number(memory.add(idx, result)));
                    }
                    Some(Directive::Sub(idx)) => {
                        println!("{}", format_number(memory.sub(idx, result)));
                    }
                    None => {}
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}

fn print_prompt(value: f64) {
    print!("[ {} ]: ", format_number(value));
    io::stdout().flush().expect("Failed to flush stdout");
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().expect("Failed to flush stdout");
}
