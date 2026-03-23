mod eval;
mod help;
mod memory;
mod parser;
mod repl;

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
            flag => {
                eprintln!("Unknown option: {flag}");
                eprintln!("Run 'ccalc -h' for usage.");
                std::process::exit(1);
            }
        }
    }

    repl::run();
}
