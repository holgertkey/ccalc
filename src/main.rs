mod eval;
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
                print_help();
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

fn print_help() {
    println!(
        "\
ccalc v{ver} — command-line calculator

USAGE:
    ccalc [OPTIONS]

OPTIONS:
    -h, --help       Show this help message
    -v, --version    Show version information

REPL COMMANDS:
    q                Quit
    c                Clear accumulator (reset to 0)
    cls              Clear the screen

ARITHMETIC:
    Operators:  +  -  *  /  ^  %
    Precedence (high to low):  ^  (right-associative)
                               *  /  %
                               +  -
    Grouping:    parentheses, e.g. (3 + 2) * 4
    Unary minus: -5,  -(3 + 2)

    The prompt [ value ]: shows the current accumulator — the result of the last
    expression. Expressions starting with an operator use the accumulator as
    the left-hand operand (partial expressions):

        [ 6 ]: * 2         accumulator = 12
        [ 12 ]: ^ 2        accumulator = 144
        [ 144 ]: % 100     accumulator = 44

CONSTANTS:
    pi          3.14159265358979...
    e           2.71828182845904...
    ans         current accumulator value (explicit alias)

MATH FUNCTIONS:
    sqrt(x)     square root
    abs(x)      absolute value
    floor(x)    round down to integer
    ceil(x)     round up to integer
    round(x)    round to nearest integer
    log(x)      base-10 logarithm
    ln(x)       natural logarithm
    exp(x)      e raised to the power x
    sin(x)      sine (radians)
    cos(x)      cosine (radians)
    tan(x)      tangent (radians)

    If called with empty parentheses, the accumulator is used as the argument:
        sqrt()   →   sqrt(accumulator)

MEMORY CELLS  m1 – m9:

  Store
    m[1-9]              Store accumulator into cell
    expr m[1-9]         Evaluate expression, store result into cell

  Recall (use inside any expression)
    m[1-9]              Read cell value, e.g.:  m1 + 8 + m1

  View / clear
    m                   Show all non-zero memory cells
    mc                  Clear all memory cells
    mc[1-9]             Clear a specific cell, e.g. mc1

EXAMPLES:

  Power and modulo:
    [ 0 ]: 2 ^ 10          accumulator = 1024
    [ 1024 ]: % 1000       accumulator = 24

  Functions and constants:
    [ 0 ]: sqrt(144)       accumulator = 12
    [ 0 ]: sin(pi / 6)     accumulator = 0.5
    [ 0 ]: log(1000)       accumulator = 3
    [ 0 ]: ln(e)           accumulator = 1

  ans and empty-arg calls:
    [ 4 ]: sqrt()          same as sqrt(4); accumulator = 2
    [ 9 ]: sqrt(ans)       same as sqrt(9); accumulator = 3
    [ 3 ]: ans * 2         accumulator = 6

  Store and recall:
    [ 0 ]: (1 + 1) * 3 m1  stores 6 in m1; accumulator = 6
    [ 6 ]: m1 + 8 + m1     expands to 6 + 8 + 6; accumulator = 20

  Copy cell to cell:
    [ 0 ]: m1 m2           stores value of m1 into m2

  View and clear memory:
    [ 10 ]: m
    m1: 85
    [ 10 ]: mc1            clears m1
    [ 10 ]: mc             clears all cells",
        ver = env!("CARGO_PKG_VERSION")
    );
}
