mod help;
mod repl;

use std::io::IsTerminal;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "-v" | "--version" => {
                println!("ccalc v{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "-h" | "--help" => {
                help::print();
                return;
            }
            expr if !expr.starts_with('-') => {
                repl::run_expr(expr);
                return;
            }
            flag => {
                eprintln!("Unknown option: {flag}");
                eprintln!("Run 'ccalc -h' for usage.");
                std::process::exit(1);
            }
        }
    }

    if !std::io::stdin().is_terminal() {
        repl::run_pipe(std::io::stdin().lock());
    } else {
        repl::run();
    }
}
