/// Entry point for all help output.
///
/// - `None`       → brief usage (for `-h` / `--help` flag)
/// - `Some("")`   → one-screen cheatsheet (REPL `help`)
/// - `Some(topic)`→ detailed section (REPL `help <topic>`)
pub fn print(topic: Option<&str>) {
    match topic {
        None => print_usage(),
        Some("" | "help") => print_cheatsheet(),
        Some("syntax") => print_syntax(),
        Some("functions" | "fn" | "func") => print_functions(),
        Some("bases" | "base") => print_bases(),
        Some("vars" | "variables") => print_vars(),
        Some("script" | "pipe") => print_script(),
        Some("examples" | "ex") => print_examples(),
        Some(unknown) => {
            eprintln!("Unknown help topic: '{unknown}'");
            eprintln!(
                "Available topics: syntax  functions  bases  vars  script  examples"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// -h / --help  (CLI flag — modes and options only)
// ---------------------------------------------------------------------------

fn print_usage() {
    println!(
        "\
ccalc v{ver} — terminal calculator with Octave/MATLAB syntax

USAGE
    ccalc                   start interactive REPL
    ccalc \"EXPR\"            evaluate expression and print result
    ccalc script.m          run a script file
    echo \"EXPR\" | ccalc     pipe mode — silent, result only
    ccalc < formulas.txt    read expressions from a file

OPTIONS
    -h, --help      show this message
    -v, --version   show version

Start the REPL and type 'help' for the full reference.",
        ver = env!("CARGO_PKG_VERSION")
    );
}

// ---------------------------------------------------------------------------
// help  (REPL — one-screen cheatsheet)
// ---------------------------------------------------------------------------

fn print_cheatsheet() {
    println!(
        "\
ccalc v{ver} — terminal calculator with Octave/MATLAB syntax

Operators   + - * / ^        ^ is right-associative
            2(3+1) → 8       implicit multiplication
Constants   pi  e  ans
Partial     [ 100 ]: / 4     starts with operator → uses ans

1-arg   sqrt abs floor ceil round sign exp ln log
        sin cos tan  asin acos atan
2-arg   atan2(y,x)  mod(a,b)  rem(a,b)  max(a,b)  min(a,b)
        hypot(a,b)  log(x,base)
        fn() → fn(ans)

Bases   0xFF  0b1010  0o17    hex dec bin oct base

Vars    x = expr              shows: x = <val>  (ans unchanged)
        x = expr;             silent assignment
        who   clear   clear x
        ws (save workspace)   wl (load workspace)

Output  disp(expr)            fprintf('text\\n')
Prec    p<N>  (0-15 decimal places, default 10)
REPL    exit  quit  cls

  help syntax      operators, precedence, implicit multiplication
  help functions   full function reference with examples
  help bases       number bases, display switching
  help vars        variables and workspace
  help script      pipe/script mode, semicolons, disp, fprintf
  help examples    practical usage examples",
        ver = env!("CARGO_PKG_VERSION")
    );
}

// ---------------------------------------------------------------------------
// help syntax
// ---------------------------------------------------------------------------

fn print_syntax() {
    println!(
        "\
SYNTAX

Operators
    +  -  *  /  ^
    Precedence (high to low):  ^  (right-associative)
                                *  /  (and implicit multiplication)
                                +  -
    Unary minus:  -5    -(3 + 2)

Grouping
    (2 + 3) * 4     →  20

Implicit multiplication
    A number or closing parenthesis immediately before ( triggers *:
    2(3 + 1)        →   8    (same as 2 * (3 + 1))
    (2+1)(4-1)      →   9

Partial expressions
    An expression starting with an operator uses ans as the left operand:
    [ 100 ]: / 4      →  25
    [  25 ]: + 5      →  30
    [  30 ]: ^ 2      →  900

Comments
    % this is a comment
    10 * 5  % inline comment — expression still evaluates

Semicolon
    Trailing ; suppresses output.
    Expressions — ans is still updated:    0.06 / 12;
    Assignments — ans is never updated:    rate = 0.06 / 12;

    Multiple statements on one line (; as separator):
    a = 1; b = 2         a = 1 silent,  b = 2 shown
    a = 1; b = 2;        both silent"
    );
}

// ---------------------------------------------------------------------------
// help functions
// ---------------------------------------------------------------------------

fn print_functions() {
    println!(
        "\
FUNCTIONS

One-argument
    sqrt(x)       square root
    abs(x)        absolute value
    floor(x)      round down to integer
    ceil(x)       round up to integer
    round(x)      round to nearest integer
    sign(x)       sign: -1, 0, or 1
    log(x)        base-10 logarithm
    ln(x)         natural logarithm (base e)
    exp(x)        e raised to the power x
    sin(x)        sine (radians)
    cos(x)        cosine (radians)
    tan(x)        tangent (radians)
    asin(x)       inverse sine (radians)
    acos(x)       inverse cosine (radians)
    atan(x)       inverse tangent (radians)

Two-argument
    atan2(y, x)   four-quadrant inverse tangent (radians)
    mod(a, b)     remainder, sign follows divisor  (Octave convention)
    rem(a, b)     remainder, sign follows dividend
    max(a, b)     larger of two values
    min(a, b)     smaller of two values
    hypot(a, b)   sqrt(a^2 + b^2), numerically stable
    log(x, base)  logarithm of x to an arbitrary base

mod vs rem
    mod(-1, 3)  →   2    sign of the divisor  (use for angle wrapping)
    rem(-1, 3)  →  -1    sign of the dividend (IEEE 754 truncation)

Empty parentheses — ans is passed as the sole argument
    sqrt()   →   sqrt(ans)
    abs()    →   abs(ans)

Nesting
    sqrt(abs(-16))        →   4
    max(hypot(3, 4), 6)   →   6
    floor(log(1000))      →   3

Examples
    hypot(3, 4)                →   5
    atan2(1, 1) * 180 / pi     →  45
    log(8, 2)                  →   3
    mod(370, 360)              →  10"
    );
}

// ---------------------------------------------------------------------------
// help bases
// ---------------------------------------------------------------------------

fn print_bases() {
    println!(
        "\
NUMBER BASES

Input literals — mix freely in any expression
    0xFF        hexadecimal    (0x or 0X prefix)
    0b1010      binary         (0b or 0B prefix)
    0o17        octal          (0o or 0O prefix)

    0xFF + 0b1010   →  265

Display commands
    hex     switch display to hexadecimal
    dec     switch display to decimal  (default)
    bin     switch display to binary
    oct     switch display to octal

Inline base suffix — evaluate and switch display in one step
    0xFF + 0b1010 hex   →   0x109

base — show ans in all four bases at once
    [ 10 ]: base
    2  - 0b1010
    8  - 0o12
    10 - 10
    16 - 0xA

Expression conversion
    When the display base is non-decimal and the expression contains
    literals in other bases, the converted expression is shown first:
    [ 0x6 ]: 0b11 + 0b11
    0x3 + 0x3
    [ 0x6 ]:"
    );
}

// ---------------------------------------------------------------------------
// help vars
// ---------------------------------------------------------------------------

fn print_vars() {
    println!(
        "\
VARIABLES

Assignment  (never updates ans; ; suppresses display)
    x = expr       shows: x = <val>
    x = expr;      silent

Using variables
    [ 0 ]: rate = 0.06 / 12
    [ 0 ]: 1 + rate
    [ 1.005 ]:

Built-in variables
    ans    result of last expression  (initialized to 0 on startup)
    pi     3.14159265358979...
    e      2.71828182845904...

View and clear
    who            list all defined variables and their values
    clear          clear all variables  (ans reset to 0)
    clear name     clear a single variable by name

Workspace persistence
    ws    save all variables to ~/.config/ccalc/workspace.toml
    wl    load variables from file  (replaces the current workspace)

    The workspace file is plain text: one 'name = value' per line."
    );
}

// ---------------------------------------------------------------------------
// help script
// ---------------------------------------------------------------------------

fn print_script() {
    println!(
        "\
PIPE / SCRIPT MODE

Running non-interactively: no prompt, one result printed per line.
    echo \"2 ^ 10\" | ccalc
    ccalc script.m
    ccalc < formulas.txt

Semicolon — suppress output
    rate = 0.06 / 12;    silent assignment  (ans unchanged)
    n = 360;             silent assignment  (ans unchanged)
    0.06 / 12;           silent expression  (ans = 0.005)

    Assignments never update ans regardless of ;.
    Expressions always update ans; ; only hides the output.

Comments
    % full-line comment
    10 * 5  % inline comment — expression still evaluates

disp(expr) — print value without updating ans
    disp(ans)
    disp(rate * 12)

fprintf('fmt') — print a formatted string  (double quotes also work)
    fprintf('Monthly payment: ')
    fprintf(\"value: %g\\n\")

Escape sequences inside strings
    \\n   newline
    \\t   horizontal tab
    \\\\   literal backslash

Commands that work in pipe/script mode
    exit / quit      stop processing
    who / clear      manage variables
    ws / wl          workspace save and load
    p / p<N>         precision
    hex/dec/bin/oct  display base

Example script
    % Monthly mortgage payment
    rate = 0.06 / 12;
    n = 360;
    factor = (1 + rate) ^ n;
    payment = 200000 * rate * factor / (factor - 1);
    fprintf('Monthly payment ($): ')
    disp(payment)"
    );
}

// ---------------------------------------------------------------------------
// help examples
// ---------------------------------------------------------------------------

fn print_examples() {
    println!(
        "\
EXAMPLES

Single expression
    $ ccalc \"2 ^ 32\"                →  4294967296
    $ ccalc \"hypot(3, 4)\"           →  5
    $ ccalc \"0xFF + 0b1010\"         →  265
    $ ccalc \"atan2(1,1) * 180 / pi\" →  45

Pipe mode
    $ echo \"sin(pi / 6)\" | ccalc
    0.5

    $ printf \"100\\n/ 4\\n+ 5\" | ccalc
    100
    25
    30

REPL — chained calculations with ans
    [ 0 ]: 2 ^ 10
    [ 1024 ]: / 4
    [ 256 ]: sqrt()
    [ 16 ]:

REPL — variables
    [ 0 ]: rate = 0.07
    rate = 0.07
    [ 0 ]: 1000 * (1 + rate) ^ 10
    [ 1967.1513573 ]:

REPL — two-argument functions
    [ 0 ]: hypot(3, 4)              →  5
    [ 0 ]: atan2(1, 1) * 180 / pi  →  45
    [ 0 ]: log(8, 2)                →  3
    [ 0 ]: mod(-1, 3)               →  2
    [ 0 ]: max(hypot(3,4), 6)       →  6

REPL — number bases
    [ 0 ]: 0xFF + 0b1010            →  265
    [ 265 ]: hex
    [ 0x109 ]: + 0b10               →  0x10B
    [ 0x10B ]: base
    2  - 0b100001011
    8  - 0o413
    10 - 267
    16 - 0x10B

Script files  (see examples/ directory)
    ccalc examples/mortgage.ccalc
    ccalc examples/resistors.ccalc
    ccalc examples/ac_impedance.ccalc"
    );
}
