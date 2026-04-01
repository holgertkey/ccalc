mod config;
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
                help::print(None);
                return;
            }
            arg if !arg.starts_with('-') => {
                let path = std::path::Path::new(arg);
                if path.is_file() {
                    let file = std::fs::File::open(path).unwrap_or_else(|e| {
                        eprintln!("Error opening '{}': {e}", path.display());
                        std::process::exit(1);
                    });
                    repl::run_pipe(std::io::BufReader::new(file));
                } else {
                    repl::run_expr(arg);
                }
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
