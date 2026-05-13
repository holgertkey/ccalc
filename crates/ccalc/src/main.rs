//! `ccalc` — a command-line calculator with an Octave/MATLAB-compatible syntax.
//!
//! Supports single-expression mode, pipe mode, and an interactive REPL.
//! The computation engine lives in the `ccalc-engine` crate.

mod config;
mod help;
mod repl;

use std::io::IsTerminal;

fn main() {
    // Spawn on a 64 MB stack to support deep recursion in user functions.
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .expect("failed to spawn main thread")
        .join();
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

fn run() {
    // Register exec-level hooks in eval.rs so user function calls are dispatched correctly.
    ccalc_engine::exec::init();

    // Register built-in plugins.
    ccalc_engine::plugin::register_plugin(Box::new(ccalc_plot::PlotPlugin));

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
                // Load config and init the session search path so that script
                // files can be found by name even when they are not in CWD.
                let config_path = ccalc_engine::env::config_dir().join("config.toml");
                if let Ok(cfg) = config::load(&config_path) {
                    ccalc_engine::exec::session_path_init(cfg.search_path());
                }

                // Try to locate the file: first as a literal path, then via
                // the session search path (which includes the configured dirs).
                let resolved = std::path::Path::new(arg)
                    .is_file()
                    .then(|| std::path::PathBuf::from(arg))
                    .or_else(|| ccalc_engine::exec::resolve_script_path(arg));

                if let Some(path) = resolved {
                    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
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
                    repl::run_file_content(&content);
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
