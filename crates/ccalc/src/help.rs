pub fn print() {
    println!(
        "\
ccalc v{ver} — command-line calculator

USAGE:
    ccalc [OPTIONS]           start interactive REPL
    ccalc \"EXPR\"              evaluate expression and print result
    ccalc script.m            run a script file
    echo \"EXPR\" | ccalc       pipe mode — silent, result only
    ccalc < formulas.txt      read expressions from file

OPTIONS:
    -h, --help       Show this help message
    -v, --version    Show version information

PIPE / NON-INTERACTIVE MODE:
    When stdin is not a terminal (pipe or file redirect), ccalc runs in
    silent mode: no prompt is shown, one result is printed per line.
    ans carries over across lines, so you can chain steps:

        printf \"100\\n/ 4\\n+ 5\" | ccalc
        100
        25
        30

    Commands supported in pipe mode: exit / quit (stop), c (reset ans),
    who, clear, clear <name>, ws, wl,
    p / p<N> (precision), hex / dec / bin / oct / base (number base).
    cls is ignored.

SCRIPT FILES (ccalc < formula.txt):
    Three tools for controlling output in pipe/file mode:

    Comments  (% — Octave/MATLAB convention)
        % full-line comment
        10 * 5  % inline comment — expression still evaluates

    Semicolon — suppress output of a line
        0.06 / 12;    evaluates and updates ans, prints nothing
        rate = ans;   store to variable silently
        1 + rate;     still updates ans

    print — explicit output
        print                   print current ans value
        print \"label\"           print label then value (no separator added)
                                write any punctuation you want in the label:
                                  print \"Result:\"   →  Result: 42
                                  print \"Sum =\"      →  Sum = 42

    print after a blank line — section header (label only, no value)
        Placing print \"label\" right after a blank line prints only the label.
        Use this for headings between calculation blocks:

            print \"=== Section ===\"

            10 + 5
            print \"Sum:\"

        Output:
            === Section ===
            15
            Sum: 15

REPL COMMANDS:
    exit, quit       Quit
    c                Reset ans to 0
    cls              Clear the screen
    who              List all defined variables
    clear            Clear all variables
    clear <name>     Clear a single variable
    ws               Save workspace to ~/.config/ccalc/workspace.toml
    wl               Load workspace from file
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

    base               show current ans value in all four bases

        [ 10 ]: base
        2  - 0b1010
        8  - 0o12
        10 - 10
        16 - 0xA

    Expression conversion — when the current base is non-decimal and the expression
    contains literals in other bases, the converted expression is shown before the result:

        [ 0x6 ]: 0b11 + 0b11
        0x3 + 0x3
        [ 0x6 ]:

        [ 0b110 ]: 2 + 0b110 + 0xa
        0b10 + 0b110 + 0b1010
        [ 0b10010 ]:

KEYBOARD SHORTCUTS:
    ↑ / ↓            Browse input history
    Ctrl+R           Reverse history search
    ← → Home End     Cursor movement
    Ctrl+W           Delete word before cursor
    Ctrl+U           Clear line

ARITHMETIC:
    Operators:  +  -  *  /  ^
    Precedence (high to low):  ^  (right-associative)
                               *  /   (and implicit multiplication)
                               +  -
    Grouping:    parentheses, e.g. (3 + 2) * 4
    Unary minus: -5,  -(3 + 2)

    The prompt [ value ]: shows ans — the result of the last expression.
    Expressions starting with an operator use ans as the left-hand operand
    (partial expressions):

        [ 6 ]: * 2         ans = 12
        [ 12 ]: ^ 2        ans = 144

IMPLICIT MULTIPLICATION:
    A number or closing parenthesis immediately before ( triggers implicit *:

        2(3 + 1)    →  8     (same as 2 * (3 + 1))
        (2+1)(4-1)  →  9     (same as (2+1) * (4-1))

CONSTANTS:
    pi          3.14159265358979...
    e           2.71828182845904...
    ans         result of last expression (Octave/MATLAB convention)

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

    If called with empty parentheses, ans is used as the argument:
        sqrt()   →   sqrt(ans)

VARIABLES:

  Assignment
    name = expr         Evaluate expr, store result in variable; print as \"name = value\"
    name = expr;        Same, but suppress output

  Using variables
    Any defined variable can be used inside expressions by name.
    ans is the implicit variable — set after every expression.

        [ 0 ]: rate = 0.06 / 12
        rate = 0.005
        [ 0.005 ]: 1 + rate
        [ 1.005 ]:

  Built-in variables
    ans             Result of last expression (reset to 0 by c)
    pi              3.14159265358979...
    e               2.71828182845904...

  View and clear
    who             List all defined variables and their values
    clear           Clear all variables
    clear <name>    Clear a single variable by name

  Workspace persistence
    ws              Save all variables to ~/.config/ccalc/workspace.toml
    wl              Load variables from file (replaces current workspace)

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

  Implicit multiplication:
    [ 0 ]: 2(3 + 1)        ans = 8
    [ 0 ]: (2+1)(4-1)      ans = 9

  Power:
    [ 0 ]: 2 ^ 10          ans = 1024

  Functions and constants:
    [ 0 ]: sqrt(144)       ans = 12
    [ 0 ]: sin(pi / 6)     ans = 0.5
    [ 0 ]: log(1000)       ans = 3
    [ 0 ]: ln(e)           ans = 1

  ans and empty-arg calls:
    [ 4 ]: sqrt()          same as sqrt(4); ans = 2
    [ 9 ]: sqrt(ans)       same as sqrt(9); ans = 3
    [ 3 ]: ans * 2         ans = 6

  Number bases:
    [ 0 ]: 0xFF + 0b1010   ans = 265
    [ 265 ]: hex           switch display to hex
    [ 0x109 ]: + 0b10      ans = 0x10B
    [ 0x10B ]: dec         switch back to decimal
    [ 267 ]:
    [ 0 ]: 0xFF + 0b1010 hex   inline: evaluate and switch to hex → 0x109

  Precision:
    [ 0 ]: 1 / 3           0.3333333333
    [ 0 ]: p4
    [ 0 ]: 1 / 3           0.3333

  Variables — store and reuse:
    [ 0 ]: rate = 0.06 / 12
    rate = 0.005
    [ 0.005 ]: n = 360
    n = 360
    [ 360 ]: factor = (1 + rate) ^ n
    factor = 6.0226...
    [ 6.0226 ]: 200000 * rate * factor / (factor - 1)
    [ 1199.10 ]:

  Script file (ccalc < formula.txt):
    % Monthly mortgage payment
    rate = 0.06 / 12;      % monthly rate — silent
    n = 360;               % 30 years in months — silent
    factor = (1 + rate) ^ n;
    200000 * rate * factor / (factor - 1)
    print \"Monthly payment ($):\"",
        ver = env!("CARGO_PKG_VERSION")
    );
}
