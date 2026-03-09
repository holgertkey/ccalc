mod eval;
mod parser;
mod repl;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && (args[1] == "-v" || args[1] == "--version") {
        println!("ccalc v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    repl::run();
}
