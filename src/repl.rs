use std::io::{self, Write};

use crate::eval::{eval, format_number};
use crate::parser::{is_partial, parse};

pub fn run() {
    let mut accumulator: f64 = 0.0;

    loop {
        print_prompt(accumulator);

        let input = read_line();
        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

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
            _ => {}
        }

        let expr_str = if is_partial(trimmed) {
            format!("{} {}", format_number(accumulator), trimmed)
        } else {
            trimmed.to_string()
        };

        match parse(&expr_str).and_then(|ast| eval(&ast)) {
            Ok(result) => accumulator = result,
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}

fn print_prompt(value: f64) {
    print!("[{}]: ", format_number(value));
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
