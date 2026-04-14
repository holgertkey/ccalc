mod config;
mod help;
mod repl;

use std::io::IsTerminal;

fn main() {
    // Register exec-level hooks in eval.rs so user function calls are dispatched correctly.
    ccalc_engine::exec::init();

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
                    // Push the script's directory so run()/source() calls inside the
                    // script resolve helper files relative to the script's location.
                    if let Some(dir) = path
                        .canonicalize()
                        .ok()
                        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                    {
                        ccalc_engine::exec::script_dir_push(&dir);
                    }
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
