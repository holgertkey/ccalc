pub fn print() {
    println!(
        "\
ccalc v{ver} — command-line calculator

USAGE:
    ccalc [OPTIONS]           start interactive REPL
    ccalc \"EXPR\"              evaluate expression and print result
    echo \"EXPR\" | ccalc       pipe mode — silent, result only
    ccalc < formulas.txt      read expressions from file

OPTIONS:
    -h, --help       Show this help message
    -v, --version    Show version information

PIPE / NON-INTERACTIVE MODE:
    When stdin is not a terminal (pipe or file redirect), ccalc runs in
    silent mode: no prompt is shown, one result is printed per line.
    The accumulator carries over across lines, so you can chain steps:

        printf \"100\\n/ 4\\n+ 5\" | ccalc
        100
        25
        30

    Commands supported in pipe mode: q (stop), c (reset accumulator),
    mc (clear all memory), mc[1-9], m[1-9] (memory store/clear),
    p / p<N> (precision), hex / dec / bin / oct / base (number base).
    cls and m are ignored.

REPL COMMANDS:
    q                Quit
    c                Clear accumulator (reset to 0)
    cls              Clear the screen
    Ctrl+C, Ctrl+D   Quit

PRECISION:
    p                Show current precision (number of decimal places)
    p<N>             Set precision to N decimal places (0–15, default 10)

        [ 0 ]: 1 / 3       → 0.3333333333
        [ 0 ]: p4
        [ 0 ]: 1 / 3       → 0.3333

NUMBER BASES:
    Input literals
        0xFF           hex integer
        0b1010         binary integer
        0o17           octal integer

    Display base commands (apply to prompt and all subsequent results)
        hex            switch display to hexadecimal
        dec            switch display to decimal (default)
        bin            switch display to binary
        oct            switch display to octal

    Inline base suffix — evaluate expression then switch display base
        expr hex       e.g. 0xFF + 0b1010 hex  → 0x109

    base               show current accumulator value in all four bases

        [ 10 ]: base
        2  - 0b1010
        8  - 012
        10 - 10
        16 - A

KEYBOARD SHORTCUTS:
    ↑ / ↓            Browse input history
    Ctrl+R           Reverse history search
    ← → Home End     Cursor movement
    Ctrl+W           Delete word before cursor
    Ctrl+U           Clear line

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
    acc         current accumulator value (explicit alias)

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

  Compound assignment  (cell = cell OP expr; accumulator = new cell value)
    expr m[1-9]+        m[1-9] += expr
    expr m[1-9]-        m[1-9] -= expr
    expr m[1-9]*        m[1-9] *= expr
    expr m[1-9]/        m[1-9] /= expr
    expr m[1-9]%        m[1-9] %= expr
    expr m[1-9]^        m[1-9] ^= expr

  Recall (use inside any expression)
    m[1-9]              Read cell value, e.g.:  m1 + 8 + m1

  View / clear
    m                   Show all non-zero memory cells
    mc                  Clear all memory cells
    mc[1-9]             Clear a specific cell, e.g. mc1

EXAMPLES:

  Single expression (argument mode):
    $ ccalc \"2 ^ 32\"
    4294967296

    $ ccalc \"sqrt(2)\"
    1.4142135624

  Pipe mode:
    $ echo \"sin(pi / 6)\" | ccalc
    0.5

    $ printf \"10\\n+ 5\\n* 2\" | ccalc
    10
    15
    30

  Power and modulo:
    [ 0 ]: 2 ^ 10          accumulator = 1024
    [ 1024 ]: % 1000       accumulator = 24

  Functions and constants:
    [ 0 ]: sqrt(144)       accumulator = 12
    [ 0 ]: sin(pi / 6)     accumulator = 0.5
    [ 0 ]: log(1000)       accumulator = 3
    [ 0 ]: ln(e)           accumulator = 1

  acc and empty-arg calls:
    [ 4 ]: sqrt()          same as sqrt(4); accumulator = 2
    [ 9 ]: sqrt(acc)       same as sqrt(9); accumulator = 3
    [ 3 ]: acc * 2         accumulator = 6

  Number bases:
    [ 0 ]: 0xFF + 0b1010   accumulator = 265
    [ 265 ]: hex           switch display to hex
    [ 0x109 ]: + 0b10      accumulator = 0x10B
    [ 0x10B ]: dec         switch back to decimal
    [ 267 ]:
    [ 0 ]: 0xFF + 0b1010 hex   inline: evaluate and switch to hex → 0x109

  Precision:
    [ 0 ]: 1 / 3           0.3333333333
    [ 0 ]: p4
    [ 0 ]: 1 / 3           0.3333

  Store and recall:
    [ 0 ]: (1 + 1) * 3 m1  stores 6 in m1; accumulator = 6
    [ 6 ]: m1 + 8 + m1     expands to 6 + 8 + 6; accumulator = 20

  Copy cell to cell:
    [ 0 ]: m1 m2           stores value of m1 into m2

  Compound assignment:
    [ 0 ]: 100 m1          m1 = 100; accumulator = 100
    [ 100 ]: 2 m1*         m1 = 200; accumulator = 200
    [ 200 ]: 50 m1-        m1 = 150; accumulator = 150
    [ 150 ]: 3 m1/         m1 = 50;  accumulator = 50

  View and clear memory:
    [ 10 ]: m
    m1: 50
    [ 10 ]: mc1            clears m1
    [ 10 ]: mc             clears all cells",
        ver = env!("CARGO_PKG_VERSION")
    );
}
