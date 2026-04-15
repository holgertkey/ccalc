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
        Some("script" | "pipe" | "printf") => print_script(),
        Some("format" | "fmt") => print_format(),
        Some("matrices" | "matrix" | "mat") => print_matrices(),
        Some("logic" | "logical" | "comparison") => print_logic(),
        Some("examples" | "ex") => print_examples(),
        Some("vectors" | "vector" | "utils") => print_vectors(),
        Some("complex" | "cplx" | "imag") => print_complex(),
        Some("strings" | "string" | "str" | "char") => print_strings(),
        Some("files" | "file" | "fileio" | "io" | "fopen" | "fclose") => print_fileio(),
        Some("control" | "flow" | "if" | "for" | "while" | "switch" | "do" | "run" | "source") => {
            print_control()
        }
        Some(
            "userfuncs" | "userfunc" | "ufunc" | "lambda" | "lambdas" | "anon" | "user"
            | "function" | "closures",
        ) => print_userfuncs(),
        Some(
            "cells" | "cell" | "cellfun" | "arrayfun" | "varargin" | "varargout" | "cell-arrays",
        ) => print_cells(),
        Some("structs" | "struct" | "fieldnames" | "isfield" | "rmfield" | "isstruct") => {
            print_structs()
        }
        Some(unknown) => {
            eprintln!("Unknown help topic: '{unknown}'");
            eprintln!(
                "Available topics: syntax  functions  userfuncs  cells  structs  bases  vars  script  format  matrices  logic  vectors  complex  strings  files  io  control  examples"
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

Operators   + - * / ^ **     ^ and ** are right-associative
            2(3+1) → 8       implicit multiplication
            +x               unary + (no-op)   ...  line continuation
Comparison  ==  ~= (!=)  <  >  <=  >=     return 1 (true) or 0 (false)
Logical     ~expr (!expr)  &&  ||          NOT, AND, OR (short-circuit)
            &   |                          element-wise AND, OR (matrices)
            xor(a,b)  not(a)               exclusive OR, logical NOT
Constants   pi  e  ans  nan  inf  i  j  (imaginary unit, 4i works)
Partial     [ 100 ]: / 4     starts with operator → uses ans

1-arg   sqrt abs floor ceil round sign exp ln log
        sin cos tan  asin acos atan
2-arg   atan2(y,x)  mod(a,b)  rem(a,b)  max(a,b)  min(a,b)
        hypot(a,b)  log(x,base)
        fn() → fn(ans)

Bases   0xFF  0b1010  0o17    hex dec bin oct base

Matrix  [1 2 3]   [1;2;3]   [1 2;3 4]
        A*B (matmul)  A' (transpose)  A.*B  A./B  A.^n
        zeros(m,n)  ones(m,n)  eye(n)  size  det  inv  trace
Range   1:5  →  [1 2 3 4 5]     1:2:9  →  [1 3 5 7 9]
        linspace(a,b,n)   [1:3, 10]  →  [1 2 3 10]
Index   v(3)  v(2:4)  v(:)  v(end)  v(end-1:end)   1-based
        A(i,j)  A(:,j)  A(i,:)  A(end,:)  A(1:end-1, 2:end)
Vector  sum prod mean min max any all norm(v) norm(v,p)
        cumsum cumprod  sort  find  unique
        reshape(A,m,n)  fliplr  flipud
NaN/Inf nan  inf  isnan  isinf  isfinite  nan(m,n)
Complex 3+4i  3+4j  4i  complex(re,im)    (Ni syntax works directly)
        real(z) imag(z) abs(z) angle(z) conj(z) isreal(z)
        z' = conj(z)   z.' = plain transpose (no conjugation)
Strings 'char array'  \"string object\"
        num2str  str2num  str2double  strcat  strcmp  strcmpi
        lower  upper  strtrim  strrep  ischar  isstring
        strsplit(s,delim)  int2str(x)  mat2str(A)
Bitwise bitand(a,b)  bitor(a,b)  bitxor(a,b)
        bitshift(a,n)  bitnot(a)  bitnot(a,bits)

Compound  x += e  x -= e  x *= e  x /= e    desugar to x = x op e
          x++  x--  ++x  --x                statement-level only
Control   if cond / elseif / else / end
          for var = range / end
          while cond / end
          switch expr / case val / otherwise / end
          do / until (cond)                 body runs at least once
          break   continue
Scripts   run('file.calc')  run('file')     .calc first, then .m
          source('file')                    Octave alias for run()
Functions function y = f(x) ... end         named, single return
          function [a,b] = f(x) ... end     multiple return values
          [a,b] = f(x)   [~,b] = f(x)      multi-assign, ~ discards
          nargin                            # args actually passed
          return                            early exit
          varargin / varargout              variadic args via cell array
Lambda    f = @(x) expr                    anonymous function
          g = @(x,y) expr                  multi-arg lambda
          h = @funcname                    function handle (wraps builtin/named)
Cells     c = {{1, 'hi', [1 2 3]}}        cell literal (heterogeneous)
          c{{2}}                           brace-index: returns element
          c{{2}} = val                     assign to element (auto-grows)
          iscell(c)  cell(n)  numel(c)     predicates, constructor, size
          cellfun(@f, c)                   apply f to each cell element
          arrayfun(@f, v)                  apply f to each vector element
          case {{2, 3}}                    multi-value switch case
Vars    x = expr              shows: x = <val>  (ans unchanged)
        x = expr;             silent assignment
        who   clear   clear x
        ws/save (save workspace)   wl/load (load workspace)
        save('f.mat')  save('f.mat','x','y')  load('f.mat')

Output  disp(expr)
        fprintf('fmt', v1, v2, ...)   print formatted  (C printf)
        sprintf('fmt', v1, v2, ...)   return formatted string
        Specifiers: %d %i %f %e %g %s %%   Width/prec: %8.3f %-10s
Files   fd = fopen('f.txt','w')   fclose(fd)   fclose('all')
        fprintf(fd,'fmt',v1,...)  fgetl(fd)  fgets(fd)
        dlmwrite('f.csv',A)  dlmwrite('f.tsv',A,'\t')
        data = dlmread('f.csv')  data = dlmread('f.tsv','\t')
        isfile(p)  isfolder(p)  pwd()  exist('x','var')  exist('f','file')
Format  format short   5 sig digits (default)   format long    15 sig digits
        format shortE  always scientific         format longE
        format shortG  same as short             format bank    2 decimal places
        format rat     rational (p/q)            format hex     IEEE 754 hex
        format +       sign only (+/-/space)
        format compact suppress blank lines      format loose   add blank lines
        format N       N decimal places (e.g. format 4)
Config  config                show config path and active settings
        config reload         re-read config.toml and apply changes
REPL    exit  quit  cls  Ctrl+L (clear screen)
Keys    ↑↓ history  Ctrl+R search  Ctrl+A/E line start/end
        Ctrl+W del word  Ctrl+U del to start  Ctrl+K del to end

  help syntax      operators, precedence, implicit multiplication
  help functions   built-in function reference with examples
  help userfuncs   user-defined functions, multiple return, lambdas
  help cells       cell arrays, varargin/varargout, cellfun, arrayfun
  help structs     scalar structs, field access, fieldnames/isfield/rmfield
  help bases       number bases, display switching
  help format      number display format modes (short/long/bank/rat/hex/+)
  help vars        variables and workspace
  help script      pipe/script mode, semicolons, disp, fprintf
  help matrices    matrix literals, arithmetic, ranges, indexing
  help vectors     nan/inf, reductions, sort/find/unique, end, reshape
  help logic       comparison and logical operators, masks
  help complex     complex numbers, i/j unit, abs/angle/conj/real/imag
  help strings     char arrays, string objects, strcmp, num2str, ...
  help files       file I/O: fopen/fclose/fgetl/fgets, dlmread/dlmwrite, isfile, pwd
  help control     if/for/while, break/continue, compound assignment
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
    +  -  *  /  ^  **   (** is Octave alias for ^)
    Comparison:   ==  ~=  <  >  <=  >=    (return 1.0 or 0.0)
    Logical:      ~expr   &&   ||          short-circuit scalars
                  &    |                   element-wise (matrices OK)
                  xor(a,b)  not(a)

    Precedence (high to low):
      postfix '  .'  transpose / plain transpose
      ^  **       exponentiation (right-associative)
      unary +  -  ~  no-op, negation, logical NOT
      *  /  .*  ./  .^   multiply, divide, element-wise
      +  -        addition, subtraction
      :           range (a:b, a:step:b)
      ==  ~=  <  >  <=  >=   comparison (non-associative)
      &           element-wise AND
      |           element-wise OR
      &&          short-circuit logical AND
      ||          short-circuit logical OR

    2 > 1 && 3 > 2     →  1    AND of two comparisons
    1 + 1 == 2         →  1    arithmetic evaluated first
    ~0                 →  1    logical NOT
    v > 3              →  element-wise mask (0/1 matrix)
    +x                 →  x    unary + is a no-op

Grouping
    (2 + 3) * 4     →  20
    ~(x == 0)       →  negate comparison

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
    % this is a comment           (Octave/MATLAB style)
    # this is a comment           (Octave/shell alias)
    10 * 5  % inline comment — expression still evaluates
    10 * 5  # hash-style inline comment

Semicolons and commas
    Trailing ; suppresses output.
    Expressions — ans is still updated:    0.06 / 12;
    Assignments — ans is never updated:    rate = 0.06 / 12;

    Statement separators on one line:
    a = 1; b = 2         ;  → a silent, b shown
    a = 1, b = 2         ,  → both shown (comma is non-silent separator)
    a = 1; b = 2;        both silent

    Inside a matrix literal [ ], ; is always a row separator:
    [1 2; 3 4]           2×2 matrix — the ; is not a statement separator

Line continuation  (...)
    Long lines can continue on the next line using ...:
    result = 1 + ...
             2 + ...
             3;               → result = 6
    A = [1 2 3; ...
         4 5 6];              → 2×3 matrix
    if value > 0 && ...
       value < 100
      disp('ok')
    end

Range operator
    a:b               row vector  [a, a+1, ..., b]   (step = 1)
    a:step:b          row vector with explicit step
    1:5               →  [1 2 3 4 5]
    0:0.5:2           →  [0 0.5 1 1.5 2]
    5:-1:1            →  [5 4 3 2 1]
    Range is lower precedence than arithmetic:
    1+1:2+2           →  2:4  →  [2 3 4]"
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

Bitwise (non-negative integers only; works naturally with 0x/0b/0o literals)
    bitand(a, b)      bitwise AND
    bitor(a, b)       bitwise OR
    bitxor(a, b)      bitwise XOR
    bitshift(a, n)    left shift (n>0), logical right shift (n<0); 0 if |n|>=64
    bitnot(a)         NOT in 32-bit window  (Octave uint32 default)
    bitnot(a, bits)   NOT in explicit bit-width window (bits in [1, 53])

    bitand(0xFF, 0x0F)      →  15
    bitor(0b1010, 0b0101)   →  15
    bitxor(0xFF, 0x0F)      →  240
    bitshift(1, 8)          →  256     (1 << 8)
    bitshift(256, -4)       →  16      (256 >> 4)
    bitnot(5, 8)            →  250     (~5 in 8 bits = 0b11111010)
    bitnot(0, 32)           →  4294967295

Constants
    pi     3.14159265358979...
    e      2.71828182845904...
    nan    IEEE 754 Not-a-Number — propagates through all arithmetic
           nan + 5  →  NaN     nan == nan  →  0  (always false)
    inf    positive infinity   -inf for negative
    i, j   imaginary unit: 0 + 1i  (can be reassigned; restart to restore)
    ans    result of last expression

Complex functions  (see also: help complex)
    real(z)          real part  (real(5) = 5)
    imag(z)          imaginary part  (imag(5) = 0)
    abs(z)           modulus sqrt(re²+im²)  (overloads scalar/matrix abs)
    angle(z)         argument atan2(im, re), in radians
    conj(z)          complex conjugate  re - im*i
    complex(re, im)  construct from two real scalars
    isreal(z)        1 if im == 0, else 0

Examples
    hypot(3, 4)                →   5
    atan2(1, 1) * 180 / pi     →  45
    log(8, 2)                  →   3
    mod(370, 360)              →  10
    abs(3 + 4*i)               →   5
    angle(i)                   →   1.5707963...  (π/2)

String functions  (see also: help strings)
    num2str(x)         number → char array ('3.1416' for pi)
    num2str(x, N)      number → char array with N decimal digits
    str2num(s)         char array → number  (error if not parseable)
    str2double(s)      char array → number  (NaN if not parseable)
    strcat(a, b, ...)  concatenate two or more strings
    strcmp(a, b)       1 if equal (case-sensitive), else 0
    strcmpi(a, b)      1 if equal (case-insensitive), else 0
    lower(s)           convert to lowercase
    upper(s)           convert to uppercase
    strtrim(s)         strip leading and trailing whitespace
    strrep(s, old, new)  replace all occurrences of old with new
    sprintf(fmt, ...)  format string (C printf); returns char array
    fprintf(fmt, ...)  format and print to stdout
    ischar(s)          1 if s is a char array, else 0
    isstring(s)        1 if s is a string object, else 0

Higher-order  (see also: help cells)
    cellfun(f, c)   apply f to each element of cell c; returns Matrix if all scalar
    arrayfun(f, v)  apply f to each element of numeric vector v; returns Matrix
    @funcname       function handle — wraps a builtin or named function as a lambda

    cellfun(@sqrt, {{1, 4, 9}})     →  [1 2 3]
    arrayfun(@(x) x^2, [1 2 3])    →  [1 4 9]
    f = @abs; f(-5)                 →  5

See also: help vectors    (sum, min, max, sort, find, norm, cumsum, ...)
          help complex    (full complex number reference)
          help strings    (char arrays, string objects, full reference)
          help script     (fprintf/sprintf reference with format specifiers)
          help cells      (cell arrays, varargin, cellfun, arrayfun)"
    );
}

// ---------------------------------------------------------------------------
// help format
// ---------------------------------------------------------------------------

fn print_format() {
    println!(
        "\
NUMBER DISPLAY FORMAT  (help format)

Change how numbers are displayed in the REPL and pipe/script mode.
The format command affects disp(), variable assignment output, and
the REPL prompt — but not fprintf/sprintf (which use their own specifiers).

SYNTAX
    format               reset to 'short' (5 significant digits)
    format <mode>        switch to named mode
    format <N>           N decimal places (e.g. format 4)

MODES
    short     5 significant digits, auto fixed/scientific  (default)
    long      15 significant digits, auto fixed/scientific
    shortE    always scientific notation, 4 decimal places
    longE     always scientific notation, 14 decimal places
    shortG    same as short  (MATLAB shortG alias)
    longG     same as long   (MATLAB longG alias)
    bank      fixed 2 decimal places  (currency)
    rat       rational approximation  p/q  (e.g. 1/3, 22/7)
    hex       IEEE 754 double-precision bit pattern (16 hex digits)
    +         sign character only: + for positive, - for negative, space for 0
    compact   suppress blank lines between outputs
    loose     add blank line after every output
    N         N decimal places (fixed or scientific as needed)

EXAMPLES
    >> format short
    >> pi
    3.1416

    >> format long
    >> pi
    3.14159265358979

    >> format shortE
    >> pi
    3.1416e+00

    >> format bank
    >> 1/3
    0.33

    >> format rat
    >> pi
    355/113

    >> format hex
    >> 1.0
    3FF0000000000000

    >> format +
    >> [1 -2 0 3]
    +- +

    >> format 4
    >> 1/3
    0.3333

    >> format compact
    (blank lines between matrix outputs are suppressed)

    >> format loose
    (blank line added after every output)

Note: 'format hex' (IEEE 754 bits) and 'hex' (integer display base) are
different commands. 'hex' changes the base for integer display; 'format hex'
shows the raw double-precision bit pattern of any number."
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
    nan    IEEE 754 Not-a-Number  (propagates through arithmetic)
    inf    positive infinity  (use -inf for negative)
    i, j   imaginary unit: 0 + 1i  (can be reassigned; restart to restore)

View and clear
    who            list all defined variables and their values
    clear          clear all variables  (ans reset to 0)
    clear name     clear a single variable by name

Workspace persistence
    ws / save               save all variables to ~/.config/ccalc/workspace.toml
    wl / load               load variables from file  (replaces the current workspace)
    save('path.mat')        save all variables to named file
    save('path.mat','x','y')  save specific variables only
    load('path.mat')        load from named file

    Only scalars and strings are persisted; matrices and complex values are skipped.
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

fprintf(fmt, v1, v2, ...) — print formatted output (C printf)
    fprintf('x = %d\\n', 42)
    fprintf('%.4f\\n', pi)
    fprintf('%s = %.2f\\n', 'rate', 0.065)
    fprintf('%8.3f  %-10s\\n', 3.14159, 'pi')

sprintf(fmt, v1, v2, ...) — format and return as string
    s = sprintf('R = %.1f Ohm', 47.5)
    disp(s)

Format specifiers
    %d  %i    integer (truncated)
    %f        fixed decimal  (default 6 places)
    %.Nf      fixed with N decimal places
    %e        scientific  1.23e+04
    %g        shorter of %%f and %%e  (trailing zeros trimmed)
    %s        string
    %%        literal percent sign
    Width/flags:  %8.3f   %-10s   %+.4e   %05d

Escape sequences inside strings
    \\n   newline
    \\t   horizontal tab
    \\\\   literal backslash

Octave behaviour: if more arguments than specifiers, the format
string repeats for the remaining arguments.

Commands that work in pipe/script mode
    exit / quit              stop processing
    who / clear              manage variables
    ws / wl / save / load    workspace save and load
    save('f.mat')            save to explicit path
    save('f.mat','x','y')    save specific variables
    load('f.mat')            load from explicit path
    hex/dec/bin/oct          display base
    Precision is set in config.toml (default 10)

Example script
    % Monthly mortgage payment
    rate = 0.06 / 12;
    n = 360;
    factor = (1 + rate) ^ n;
    payment = 200000 * rate * factor / (factor - 1);
    fprintf('Monthly payment: $%.2f\\n', payment)"
    );
}

// ---------------------------------------------------------------------------
// help matrices
// ---------------------------------------------------------------------------

fn print_matrices() {
    println!(
        "\
MATRICES

Literals
    [1 2 3]           row vector  (1×3)
    [1; 2; 3]         column vector  (3×1)
    [1 2; 3 4]        2×2 matrix
    [1, 2; 3, 4]      commas also separate elements

    Elements can be expressions:
    [sqrt(4), 2^3]    →  [2, 8]

Arithmetic  (scalar operations are element-wise)
    A + B   A - B     element-wise  (shapes must match)
    2 * A             scale all elements
    A / 10   A ^ 2    element-wise divide / power

Matrix multiplication and transpose
    A * B             matrix multiplication  (inner dims must agree)
    A'                transpose  (postfix, highest precedence)
    v' * v            dot product  (row × column → scalar-like 1×1)
    v * v'            outer product  (column × row → matrix)

Element-wise operators  (.* ./ .^ — shapes must match)
    A .* B            element-wise product  (Hadamard product)
    A ./ B            element-wise division
    A .^ 2            element-wise power  (same as A .* A)

Built-in functions
    zeros(m,n)        m×n matrix of zeros
    ones(m,n)         m×n matrix of ones
    eye(n)            n×n identity matrix
    size(A)           [rows cols] as a 1×2 row vector
    size(A, dim)      rows (dim=1) or cols (dim=2) as scalar
    length(A)         max(rows, cols)
    numel(A)          total element count
    trace(A)          sum of diagonal elements
    det(A)            determinant  (square matrices only)
    inv(A)            inverse  (square, non-singular)

Range operator
    a:b               row vector from a to b with step 1
    a:step:b          row vector with explicit step (may be negative)
    1:5               →  [1 2 3 4 5]
    1:2:9             →  [1 3 5 7 9]
    5:-1:1            →  [5 4 3 2 1]
    0:0.5:2           →  [0 0.5 1 1.5 2]
    Ranges work inside [ ]:
    [1:3, 10]         →  [1 2 3 10]
    [1:2:7]           →  [1 3 5 7]

linspace
    linspace(a,b,n)   n evenly spaced values from a to b (inclusive)
    linspace(0,1,5)   →  [0 0.25 0.5 0.75 1]

Display
    A =
       1   2
       3   4
    Prompt shows size when ans is a matrix:  [ [2×2] ]:

Indexing  (1-based — Octave convention)
    v(3)              scalar element (3rd)
    v(2:4)            sub-vector  (elements 2, 3, 4)
    v(:)              all elements as a column vector
    A(i,j)            scalar element at row i, column j
    A(:,j)            entire column j  (result: Nx1)
    A(i,:)            entire row i     (result: 1xM)
    A(1:2, 2:3)       submatrix via range indices
    Variables in env shadow function names (same as Octave):
    v = [10 20 30]; v(2)  →  20

end keyword — resolves to the size of the indexed dimension
    v(end)            last element
    v(end-2:end)      last three elements
    A(end, :)         last row
    A(1:end-1, 2:end) all rows except last, columns 2 to end

Display
    A =
       1   2
       3   4
    Prompt shows size when ans is a matrix:  [ [2×2] ]:

Workspace
    ws  saves only scalar variables — matrices are not persisted.
    who shows dimensions:  A = [2×2 double]"
    );
}

// ---------------------------------------------------------------------------
// help logic
// ---------------------------------------------------------------------------

fn print_logic() {
    println!(
        "\
COMPARISON AND LOGICAL OPERATORS

Comparison  (return 1.0 = true, 0.0 = false)
    a == b           equal
    a ~= b  (a != b) not equal        ~= and != are equivalent
    a <  b           less than
    a >  b           greater than
    a <= b           less than or equal
    a >= b           greater than or equal

Logical
    ~expr   (!expr)  NOT: 1 if expr == 0, else 0
    a && b           AND: short-circuit scalar (scalars only)
    a || b           OR:  short-circuit scalar (scalars only)
    a & b            element-wise AND (works on matrices, always evaluates both)
    a | b            element-wise OR  (works on matrices, always evaluates both)
    xor(a, b)        element-wise XOR
    not(a)           element-wise NOT (alias for ~)

  ! and != are C/shell-style aliases for ~ and ~= (Octave extension).
  Use & and | for matrix logical masks; && and || for scalar conditions.

Precedence (low to high inside an expression)
    ||  →  &&  →  |  →  &  →  comparisons  →  :  →  +/-  →  *//  →  ^  →  unary

Scalar examples
    3 > 2             →  1
    3 == 4            →  0
    5 ~= 5            →  0
    ~0                →  1
    ~1                →  0
    2 > 1 && 3 > 2    →  1
    0 || 1            →  1
    xor(1, 0)         →  1
    not(5)            →  0

Arithmetic + comparison
    1 + 1 == 2        →  1    (arithmetic first, then ==)
    2 * 3 > 5         →  1
    2 > 3 || 1 < 2    →  1

Element-wise on matrices
    v = [1 2 3 4 5]
    v > 3                      →  [0 0 0 1 1]
    v == 3                     →  [0 0 1 0 0]
    v ~= 3                     →  [1 1 0 1 1]

    % & and | work on boolean matrices:
    a = [1 0 1 0];  b = [1 1 0 0];
    a & b                      →  [1 0 0 0]
    a | b                      →  [1 1 1 0]
    xor(a, b)                  →  [0 1 1 0]

Logical mask pattern
    v = [3, -1, 8, 0, 5, -2, 7];
    mask = v > 0 & v < 6       →  [1 0 0 0 1 0 0]

Soft masking — zero out elements that fail a condition
    v .* (v > 3)               →  [0 0 0 4 5]  keep elements > 3 only

See also: help matrices, help syntax
Example:  ccalc examples/logic.calc   ccalc examples/language_polish.calc"
    );
}

// ---------------------------------------------------------------------------
// help vectors
// ---------------------------------------------------------------------------

fn print_vectors() {
    println!(
        "\
VECTOR UTILITIES & SPECIAL CONSTANTS

Special constants
    nan    IEEE 754 Not-a-Number — propagates through all arithmetic
           nan + 5  →  NaN     nan == nan  →  0  (always false)
    inf    Positive infinity.  Use -inf for negative infinity.
    nan(n)          n×n matrix of NaN
    nan(m, n)       m×n matrix of NaN

Predicates  (element-wise — work on scalars and matrices)
    isnan(x)        1.0 if NaN, else 0.0
    isinf(x)        1.0 if ±Inf, else 0.0
    isfinite(x)     1.0 if finite, else 0.0

Reductions
    For vectors (1×N or N×1) these collapse to a scalar.
    For M×N matrices (M>1, N>1) they operate column-wise, returning 1×N.

    sum(v)          sum of elements
    prod(v)         product of elements
    mean(v)         arithmetic mean
    min(v)          minimum  (1-arg form; 2-arg min(a,b) still works)
    max(v)          maximum  (1-arg form)
    any(v)          1 if any element is non-zero, else 0
    all(v)          1 if all elements are non-zero, else 0
    norm(v)         Euclidean (L2) norm
    norm(v, p)      Lp norm  (norm(v, inf) = max of absolute values)

    sum([1 2 3 4])       →  10
    sum([1 2; 3 4])      →  [4  6]     (column sums)
    any([0 1 0])         →  1
    all([1 2 3] > 0)     →  1
    norm([3 4])          →  5

Cumulative operations  (return same shape as input)
    cumsum(v)       cumulative sum
    cumprod(v)      cumulative product

    cumsum([1 2 3 4])    →  [1  3  6  10]
    cumprod([1 2 3 4])   →  [1  2  6  24]

Sorting and searching
    sort(v)             sort ascending  (vectors only)
    find(v)             1-based column-major indices of non-zero elements
    find(v, k)          first k non-zero indices
    unique(v)           sorted unique elements as a 1×N row vector

    sort([3 1 4 1 5])          →  [1  1  3  4  5]
    find([0 3 0 5])            →  [2  4]
    find([1 0 2 0 3], 2)       →  [1  3]
    unique([3 1 4 1 5 9 2 6])  →  [1  2  3  4  5  6  9]

Reshape and flip
    reshape(A, m, n)    reshape to m×n  (column-major element order)
    fliplr(v)           reverse column order  (mirror left↔right)
    flipud(v)           reverse row order    (mirror up↔down)

    reshape(1:6, 2, 3)  →  [1 3 5; 2 4 6]
    fliplr([1 2 3])     →  [3 2 1]
    flipud([1;2;3])     →  [3;2;1]

end in index expressions
    Inside index parentheses, end resolves to the size of that dimension.
    Arithmetic on end is fully supported.

    v(end)              last element
    v(end-1)            second to last
    v(end-2:end)        last three elements
    v(1:2:end)          every other element (first to last)
    A(end, :)           last row
    A(:, end)           last column
    A(1:end-1, 2:end)   submatrix: all rows except last, col 2 to end

See also: help matrices  help functions
Example:  ccalc examples/vector_utils.calc"
    );
}

// ---------------------------------------------------------------------------
// help complex
// ---------------------------------------------------------------------------

fn print_complex() {
    println!(
        "\
COMPLEX NUMBERS

Creating complex numbers
    3 + 4i           →  3 + 4i    (Ni suffix: 4i = 4*i, tokenizer handles this)
    3 + 4*i          →  3 + 4i    (explicit multiply — also works)
    3 + 4*j          →  3 + 4i    (j is also the imaginary unit)
    complex(3, 4)    →  3 + 4i    (construct from real and imaginary parts)
    5i               →  5i         (pure imaginary; 5*i also works)
    2 - 3i           →  2 - 3i

    Ni suffix: any decimal number immediately followed by i or j (no space,
    no further alphanumeric chars) is treated as a complex literal.
    When im is exactly 0, the result collapses to a real scalar.

Arithmetic
    z1 = 3 + 4*i;  z2 = 1 - 2*i
    z1 + z2    →  4 + 2i
    z1 - z2    →  2 + 6i
    z1 * z2    →  11 - 2i     (ac-bd) + (ad+bc)i
    z1 / z2    →  -1 + 2i
    z1 ^ 2     →  -7 + 24i
    2 * z1     →  6 + 8i

Powers
    i^2        →  -1          (exact integer exponentiation)
    i^3        →  -i
    i^4        →   1
    (1+i)^-1   →  0.5 - 0.5i
    i^0.5      →  0.7071... + 0.7071...i   (polar form for non-integers)

Conjugate and plain transpose
    z = 3 + 4i
    z'         →  3 - 4i      (conjugate transpose — flips sign of imaginary part)
    z.'        →  3 + 4i      (plain transpose — no conjugation)
    conj(z)    →  3 - 4i      (same as z')

Polar form
    abs(z)     →  5           modulus  sqrt(re² + im²)
    angle(z)   →  0.9272...   argument atan2(im, re), in radians

Built-in functions
    real(z)          real part                real(3+4i)  →  3
    imag(z)          imaginary part           imag(3+4i)  →  4
    abs(z)           modulus                  abs(3+4i)   →  5
    angle(z)         argument in radians      angle(i)    →  π/2
    conj(z)          complex conjugate        conj(3+4i)  →  3-4i
    complex(re, im)  construct               complex(3,4) →  3+4i
    isreal(z)        1 if im==0, else 0      isreal(5)   →  1

    real(5) = 5  (real of a scalar is itself)
    imag(5) = 0  (imaginary part of a real scalar is 0)

Comparison
    ==  and  ~=  compare both real and imaginary parts
    (3+4i) == (3+4i)   →  1
    (3+4i) == (3-4i)   →  0
    <  >  <=  >=  on complex numbers → error (ordering not defined)

Variables i and j
    i and j are initialized to 0+1i at startup.
    You can reassign them:  i = 5   (shadows the imaginary unit)
    Restart ccalc or use complex(0,1) to restore the imaginary unit.

Limitations
    Complex matrices [1+2i, 3] are not yet supported (returns an error).
    ws/wl do not persist complex variables (same policy as matrices).

Example: ccalc examples/complex_numbers.calc"
    );
}

// ---------------------------------------------------------------------------
// help strings
// ---------------------------------------------------------------------------

fn print_strings() {
    println!(
        "\
STRINGS

Two types — both display as plain text (no surrounding quotes).

Char arrays — single quotes  (MATLAB classic, numeric-compatible)
    'hello'            →  Str(\"hello\")   1×5 char array
    'it''s ok'         →  it's ok         '' inside '' = escaped quote
    length('hello')    →  5
    size('hello')      →  [1  5]
    numel('hello')     →  5

    Arithmetic converts chars to their ASCII codes:
    'a' + 0            →  97
    'abc' + 1          →  [98  99  100]
    'abc' == 'aXc'     →  [1  0  1]      element-wise comparison

String objects — double quotes  (modern style, scalar element)
    \"hello\"            →  StringObj(\"hello\")
    \"it\"\"s ok\"         →  it\"s ok          \"\" inside \"\" = escaped quote
    \"a\\n\" + \"b\"         →  a<newline>b      escape sequences in double-quoted strings
    length(\"hello\")    →  1            scalar element — not a char array
    \"abc\" + \"def\"       →  \"abcdef\"        + concatenates string objects

Escape sequences inside \"...\"  (also work in fprintf/sprintf)
    \\n    newline
    \\t    horizontal tab
    \\\\    literal backslash
    \\\"    literal double-quote

String built-in functions
    num2str(x)          number → char array ('3.1416' for pi)
    num2str(x, N)       number → char array with N decimal digits
    int2str(x)          round to integer, then → char array ('4' for 3.7)
    mat2str(A)          matrix → MATLAB literal string ('[1 2;3 4]')
    str2num(s)          char array → number  (error if not parseable)
    str2double(s)       char array → number  (NaN if not parseable)
    strsplit(s)         split on whitespace → cell array of char arrays
    strsplit(s, delim)  split on delimiter  → cell array of char arrays
    strcat(a, b, ...)   concatenate two or more strings
    strcmp(a, b)        1 if equal (case-sensitive), else 0
    strcmpi(a, b)       1 if equal (case-insensitive), else 0
    lower(s)            convert to lowercase
    upper(s)            convert to uppercase
    strtrim(s)          strip leading and trailing whitespace
    strrep(s, old, new) replace all occurrences of old with new
    sprintf(fmt, ...)   format string using C printf specifiers; returns char array
    ischar(s)           1 if s is a char array, else 0
    isstring(s)         1 if s is a string object, else 0

    strsplit examples
    parts = strsplit('a,b,c', ',')   → {{'a', 'b', 'c'}}  (cell array)
    parts{{1}}                       → 'a'
    words = strsplit('hello world')  → {{'hello', 'world'}}

Type checking
    ischar('hello')     →  1
    isstring(\"hello\")  →  1
    ischar(\"hello\")    →  0    string object is not a char array
    ischar(42)          →  0

Practical — building labeled output
    num2str(4700) + ' Ohm'
    strcat('R = ', num2str(R), ' kOhm')

Comparison
    strcmp('abc', 'abc')    →  1
    strcmpi('ABC', 'abc')   →  1
    \"hello\" == \"hello\"      →  1
    \"hello\" == \"world\"      →  0

Workspace
    ws/wl do not persist string variables (same policy as matrices).
    who shows: name [1×N char]  or  name [string]

Example: ccalc examples/strings.calc"
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

REPL — matrices
    [ 0 ]: A = [1 2; 3 4]
    A =
       1   2
       3   4
    [ [2×2] ]: A'
    ans =
       1   3
       2   4
    [ [2×2] ]: det(A)
    [ -2 ]: inv(A)
    ans =
       -2    1
       1.5  -0.5

REPL — ranges and indexing
    [ 0 ]: v = 1:5
    v =
       1   2   3   4   5
    [ [1×5] ]: v(3)
    [ 3 ]: v(2:4)
    ans =
       2   3   4
    [ [1×3] ]: A = [1:3; 4:6; 7:9]
    A =
       1   2   3
       4   5   6
       7   8   9
    [ [3×3] ]: A(:,2)
    ans =
       2
       5
       8
    [ [3×1] ]: A(1:2, 2:3)
    ans =
       2   3
       5   6

REPL — bitwise operations (combine with hex/bin literals)
    [ 0 ]: bitand(0xFF, 0x0F)
    [ 15 ]: bitor(0b1010, 0b0101)
    [ 15 ]: bitxor(0xFF, 0x0F)
    [ 240 ]: bitshift(1, 8)
    [ 256 ]: bitshift(256, -4)
    [ 16 ]: bitnot(5, 8)
    [ 250 ]:

REPL — comparison and logical operators
    [ 0 ]: 3 > 2
    [ 1 ]:
    [ 0 ]: 5 ~= 5
    [ 0 ]:
    [ 0 ]: ~0
    [ 1 ]:
    [ 0 ]: 2 > 1 && 3 > 2
    [ 1 ]:
    [ 0 ]: v = [1 2 3 4 5];
    [ 0 ]: v > 3
    ans =
       0   0   0   1   1
    [ [1×5] ]: v .* (v > 3)
    ans =
       0   0   0   4   5

Script files  (see examples/ directory)
    ccalc examples/mortgage.calc
    ccalc examples/resistors.calc
    ccalc examples/matrix_ops.calc
    ccalc examples/sequences.calc
    ccalc examples/logic.calc
    ccalc examples/bitwise.calc
    ccalc examples/vector_utils.calc
    ccalc examples/complex_numbers.calc
    ccalc examples/strings.calc
    ccalc examples/file_io.calc
    ccalc examples/control_flow.calc
    ccalc examples/extended_control_flow.calc
    ccalc examples/user_functions.calc
    ccalc examples/cell_arrays.calc"
    );
}

// ---------------------------------------------------------------------------
// help files
// ---------------------------------------------------------------------------

fn print_fileio() {
    println!(
        "\
FILE I/O

File handles
    fd = fopen(path, mode)    open file; returns fd (≥3) or -1 on failure
    fclose(fd)                close by fd; returns 0 on success, -1 on failure
    fclose('all')             close all open handles

  Modes: 'r' read  'w' write (create/truncate)  'a' append  'r+' read+write
  fd 1 = stdout, fd 2 = stderr

Read / write
    fprintf(fd, fmt, v1, ...)  write formatted output to fd
    fprintf(fmt, v1, ...)      write to stdout  (fd 1)
    line = fgetl(fd)           read one line; newline stripped; returns -1 at EOF
    raw  = fgets(fd)           read one line; newline kept; returns -1 at EOF

  Example — write then read back:
    fd = fopen('log.txt', 'w');
    fprintf(fd, 'x = %.4f\\n', 3.14159);
    fclose(fd);
    fd = fopen('log.txt', 'r');
    line = fgetl(fd);          % 'x = 3.1416'
    fclose(fd);

Delimiter-separated data
    dlmwrite(path, A)          write matrix with ',' separator
    dlmwrite(path, A, delim)   explicit delimiter  (',' or '\\t')
    A = dlmread(path)          read; auto-detect ',' / '\\t' / whitespace
    A = dlmread(path, delim)   explicit delimiter

  Example:
    data = [1, 3.3; 2, 4.7];
    dlmwrite('meas.csv', data);
    loaded = dlmread('meas.csv');

Filesystem queries
    isfile(path)            1 if path is an existing file, else 0
    isfolder(path)          1 if path is an existing directory, else 0
    pwd()                   current working directory as a char array
    exist(name)             1 if variable in workspace, 2 if file on disk
    exist(name, 'var')      check workspace only; 1 or 0
    exist(name, 'file')     check filesystem only; 2 if found, 0 otherwise

Workspace with explicit path
    save                         save to ~/.config/ccalc/workspace.toml
    save('path.mat')             save all variables to named file
    save('path.mat', 'x', 'y')  save only named variables
    load('path.mat')             load from named file

  The path argument may be a variable reference:
    p = 'session.mat';
    save(p);
    load(p);

  Persisted types: Scalar, char array, string object.
  Skipped always:  Matrix, Complex.

See also: help script  help vars"
    );
}

// ---------------------------------------------------------------------------
// help control
// ---------------------------------------------------------------------------

fn print_control() {
    println!(
        "\
CONTROL FLOW

All block constructs use the keyword `end` to close. Blocks may be nested
to any depth. Multi-line blocks work in both REPL and script/pipe mode.

─── if / elseif / else ────────────────────────────────────────────────────────

  if cond
    ...
  elseif cond2
    ...
  else
    ...
  end

  Condition is truthy when:
    Scalar:       non-zero and not NaN
    Matrix:       all elements non-zero and not NaN
    Str/StringObj: non-empty

  Example:
    score = 73;
    if score >= 90
      grade = 'A';
    elseif score >= 70
      grade = 'C';
    else
      grade = 'F';
    end

─── for ───────────────────────────────────────────────────────────────────────

  for var = range_expr
    ...
  end

  Range is evaluated once before the loop. Iteration over matrix columns:
    Row vector  →  each element as a scalar
    M×N matrix  →  each column as an M×1 column vector

  Examples:
    for k = 1:5
      fprintf('%d\\n', k)
    end

    for k = 1:2:9           % step = 2  →  1 3 5 7 9
      fprintf('%d ', k)
    end

─── while ─────────────────────────────────────────────────────────────────────

  while cond
    ...
  end

  Example:
    x = 1.0;
    while abs(x ^ 2 - 2) > 1e-12
      x = (x + 2 / x) / 2;
    end

─── break / continue ──────────────────────────────────────────────────────────

  break      exit the innermost loop immediately
  continue   skip to the next iteration of the innermost loop

  Example:
    for n = 1:20
      if mod(n, 2) == 0
        continue
      end
      if n > 9
        break
      end
      fprintf('%d ', n)      % prints: 1 3 5 7 9
    end

─── Compound assignment operators ─────────────────────────────────────────────

  All forms desugar at parse time to a plain assignment — no new AST nodes.

    x += e     →  x = x + e
    x -= e     →  x = x - e
    x *= e     →  x = x * e
    x /= e     →  x = x / e
    x++        →  x = x + 1   (suffix)
    x--        →  x = x - 1   (suffix)
    ++x        →  x = x + 1   (prefix)
    --x        →  x = x - 1   (prefix)

  RHS is a full expression:
    x *= 2 + 3    →  x = x * (2 + 3)   (not x * 2 + 3)

  Limitation: ++ and -- are statement-level only.
    b = a - b--   is NOT supported (use two statements instead).

─── switch / case / otherwise ─────────────────────────────────────────────────

  switch expr
    case val1
      ...
    case val2
      ...
    otherwise       % optional default branch
      ...
  end

  No fall-through: only the first matching case runs.
  Scalars: exact == comparison.
  Strings: Str and StringObj are interchangeable.
  break/continue inside switch propagate to the nearest enclosing loop.

  Example:
    switch code
      case 200
        msg = 'OK';
      case 404
        msg = 'Not Found';
      otherwise
        msg = 'Unknown';
    end

─── do...until ────────────────────────────────────────────────────────────────

  do
    ...
  until (cond)

  Octave post-test loop: body always executes at least once, then cond is
  checked. Parentheses around cond are optional.
  break and continue work as in while.
  until closes the block (no separate end needed).

  Example:
    x = 1;
    do
      x *= 2;
    until (x > 100)
    % x == 128

─── run() / source() ──────────────────────────────────────────────────────────

  run('filename')
  source('filename')    % Octave alias for run()

  Execute a script file in the current workspace (MATLAB run semantics):
  variables defined in the script persist in the caller's scope.

  Extension resolution for bare names (no extension):
    1. <name>.calc   native ccalc format (checked first)
    2. <name>.m      Octave/MATLAB compatibility

  With explicit extension: used verbatim.
  Maximum nesting depth: 64 levels (catches accidental recursion).

  Example:
    a = 252; b = 105;
    run('euclid_helper')       % defines g = gcd(a, b) in workspace
    fprintf('gcd = %d\\n', g)   % 21

─── REPL multi-line input ─────────────────────────────────────────────────────

  The REPL detects unclosed blocks by tracking depth changes:
    Keywords that open a block (+1): if  for  while  switch  do
    Keywords that close a block (-1): end  until
  Lines accumulate with a continuation prompt until the block is complete.
  Press Ctrl+C to cancel an in-progress block.

See also: help syntax  help logic  help userfuncs
Examples: ccalc examples/control_flow.calc
          ccalc examples/extended_control_flow.calc"
    );
}

// ---------------------------------------------------------------------------
// help userfuncs
// ---------------------------------------------------------------------------

fn print_userfuncs() {
    println!(
        "\
USER-DEFINED FUNCTIONS AND LAMBDAS  (help userfuncs)

─── Named functions ───────────────────────────────────────────────────────────

  function result = name(p1, p2)
    ...
    result = expr;
  end

  Defined at the top level or in a script file. Stored in the workspace
  like any variable. Functions persist until cleared.

  Single return value:
    function y = square(x)
      y = x ^ 2;
    end
    square(5)    →  25

  Multiple return values:
    function [mn, mx] = bounds(v)
      mn = min(v);
      mx = max(v);
    end
    [lo, hi] = bounds([3 1 4 1 5])   →  lo = 1, hi = 5

  Discard outputs with ~:
    [~, hi] = bounds([3 1 4 1 5])    →  hi = 5

─── nargin — optional arguments ───────────────────────────────────────────────

  nargin holds the number of arguments actually passed by the caller.
  Use it to implement optional parameters with defaults:

    function y = power_fn(base, exp)
      if nargin < 2
        exp = 2;
      end
      y = base ^ exp;
    end
    power_fn(5)     →  25   (uses default exp = 2)
    power_fn(2, 8)  →  256

─── return — early exit ───────────────────────────────────────────────────────

  return immediately exits the current function. Output variables must
  be assigned before return is reached:

    function g = gcd_fn(a, b)
      while b ~= 0
        r = mod(a, b);
        a = b;
        b = r;
      end
      g = a;
    end

─── Scope ─────────────────────────────────────────────────────────────────────

  Each call creates a fresh local scope. The caller's data variables
  (scalars, matrices, strings) are NOT visible inside the function.
  Parameters are bound to the local scope.

  However, all Function and Lambda values from the caller's workspace
  are forwarded, enabling:
    - self-recursion: a function can call itself by name
    - mutual recursion: two functions can call each other

─── Anonymous functions (lambdas) ─────────────────────────────────────────────

  Syntax:  @(param1, param2, ...) expr

    sq    = @(x) x ^ 2;
    hyp   = @(a, b) sqrt(a^2 + b^2);
    add   = @(a, b) a + b;

    sq(7)        →  49
    hyp(3, 4)    →   5

  Zero-argument lambda:
    const_pi = @() pi;
    const_pi()   →  3.14159...

  Lambdas are stored in variables and passed like any value.

─── Lexical capture ───────────────────────────────────────────────────────────

  A lambda captures the enclosing environment at DEFINITION time.
  Changing a captured variable later has no effect:

    rate = 0.05;
    interest = @(p, n) p * (1 + rate) ^ n;
    rate = 0.99;              % too late — the lambda captured 0.05
    interest(1000, 10)   →  1628.89

─── Lambdas as arguments ──────────────────────────────────────────────────────

  Pass a lambda to a function using @:

    function s = midpoint(f, a, b, n)
      h = (b - a) / n;
      s = 0;
      for k = 1:n
        xm = a + (k - 0.5) * h;
        s += f(xm);
      end
      s *= h;
    end

    midpoint(@(x) x^2,    0, 1, 1000)   →  0.333333
    midpoint(@(x) sin(x), 0, pi, 1000)  →  2.000001

─── Functions returning functions ─────────────────────────────────────────────

  A named function can return a lambda (higher-order programming):

    function f = make_adder(c)
      f = @(x) x + c;
    end

    add5  = make_adder(5);
    add10 = make_adder(10);
    add5(3)         →   8
    add10(7)        →  17
    add5(add10(1))  →  16

─── varargin — variadic input ─────────────────────────────────────────────────

  When the last parameter is named varargin, all extra call arguments are
  collected into a cell array bound to that name:

    function s = sum_all(varargin)
      s = 0;
      for k = 1:numel(varargin)
        s += varargin{{k}};
      end
    end

    sum_all(1, 2, 3)        →  6
    sum_all(10, 20, 30)     →  60
    sum_all()               →  0   (empty varargin cell)

  Fixed and variadic parameters may be mixed:

    function show(label, varargin)
      fprintf('[%s]', label)
      for k = 1:numel(varargin)
        fprintf(' %g', varargin{{k}})
      end
      fprintf('\\n')
    end

─── varargout — variadic output ───────────────────────────────────────────────

  When the sole output variable is varargout, fill it as a cell array and
  the caller receives one output per cell element:

    function varargout = first_n(v, n)
      for k = 1:n
        varargout{{k}} = v(k);
      end
    end

    [a, b, c] = first_n([10 20 30 40], 3)   →  a=10  b=20  c=30

See also: help control  help functions  help cells
Example:  ccalc examples/user_functions.calc
          ccalc examples/cell_arrays.calc"
    );
}

// ---------------------------------------------------------------------------
// help cells
// ---------------------------------------------------------------------------

fn print_cells() {
    println!(
        "\
CELL ARRAYS  (help cells)

A cell array is a heterogeneous 1-D container: each element can be any value
(scalar, matrix, string, complex, another cell, or a function handle).

─── Creating cell arrays ──────────────────────────────────────────────────────

  {{e1, e2, e3}}            cell literal — one expression per element
  cell(n)                   1×n cell pre-filled with zeros
  cell(m, n)                1×(m*n) cell pre-filled with zeros

  c = {{1, 'hello', [1 2 3]}}
  c{{1}}                    →  1         (scalar)
  c{{2}}                    →  hello     (char array)
  c{{3}}                    →  [1 2 3]   (matrix)

─── Brace indexing ────────────────────────────────────────────────────────────

  c{{i}}                    access element i (1-based); returns its VALUE
  c{{i}} = v               assign to element i; auto-grows if i > numel(c)

  iscell(c)                 1 if c is a cell array, else 0
  numel(c)                  number of elements
  length(c)                 number of elements (same as numel for 1-D)
  size(c)                   [1  numel(c)] as a 1×2 matrix

  Note: c(i) with round parentheses returns an error — use c{{i}} for content.

─── varargin / varargout ──────────────────────────────────────────────────────

  varargin   — last parameter that collects all extra call arguments into a cell:
    function s = sum_all(varargin)
      s = 0;
      for k = 1:numel(varargin)
        s += varargin{{k}};
      end
    end
    sum_all(1, 2, 3)    →  6

  varargout  — sole output that is expanded into multiple return values:
    function varargout = swap(a, b)
      varargout{{1}} = b;
      varargout{{2}} = a;
    end
    [x, y] = swap(10, 20)   →  x=20  y=10

─── case with cell array ──────────────────────────────────────────────────────

  Inside a switch block, case {{v1, v2}} matches if the switch expression
  equals any element of the cell array:

    switch x
      case {{1, 2, 3}}
        disp('small')
      case {{4, 5, 6}}
        disp('medium')
      otherwise
        disp('large')
    end

─── cellfun ───────────────────────────────────────────────────────────────────

  cellfun(f, c)    apply f to each element of cell c
  Returns Value::Matrix when all results are scalar; Value::Cell otherwise.

    c = {{1, 4, 9}};
    cellfun(@sqrt, c)          →  [1  2  3]
    cellfun(@(x) x*2, c)       →  [2  8  18]

─── arrayfun ──────────────────────────────────────────────────────────────────

  arrayfun(f, v)   apply f to each element of numeric vector v
  Returns a same-shape matrix (function must return a scalar per element).

    arrayfun(@(x) x^2, [1 2 3])       →  [1  4  9]
    arrayfun(@(x) x > 2, [1 2 3 4])   →  [0  0  1  1]

─── @funcname — function handles ──────────────────────────────────────────────

  @funcname creates a lambda that forwards its arguments to funcname.
  Works with builtins and user-defined functions.

    f = @sqrt;       f(16)       →  4
    g = @abs;        g(-7.5)     →  7.5
    h = @clamp01;    h(-0.5)     →  0   (user function)

  Compose handles via a lambda that calls them sequentially:
    compose = @(f, g) @(x) f(g(x));
    sqrt_abs = compose(@sqrt, @abs);
    sqrt_abs(-9)    →  3

─── Workspace ─────────────────────────────────────────────────────────────────

  Cell arrays are NOT persisted by ws/save — same policy as matrices.
  who shows: c = {{1×N cell}}

See also: help userfuncs  help functions  help control  help structs
Example:  ccalc examples/cell_arrays.calc"
    );
}

// ---------------------------------------------------------------------------
// help structs
// ---------------------------------------------------------------------------

fn print_structs() {
    println!(
        "\
STRUCTS  (help structs)

A scalar struct groups named fields, each holding any value (scalar, matrix,
string, complex, cell, or another struct).  Fields are ordered by insertion.

─── Creating structs ──────────────────────────────────────────────────────────

  s.x = 1               field assignment; creates struct if s doesn't exist yet
  s.y = [1 2 3]         field can hold any Value
  s.a.b = 42            nested field — creates s.a as an empty struct if needed

  s = struct()                  empty struct
  s = struct('x', 1, 'y', 2)   constructor; pairs: string key + value

─── Reading fields ────────────────────────────────────────────────────────────

  s.x                   read field value
  s.a.b                 chained: read nested field (any depth)
  v = s.x + s.y         fields are ordinary values — use them in expressions

─── Built-in utilities ────────────────────────────────────────────────────────

  fieldnames(s)         Value::Cell of Value::Str field names, insertion order
  isfield(s, 'x')       1 if field 'x' exists, else 0
  rmfield(s, 'x')       copy of s with field 'x' removed; error if absent
  isstruct(v)           1 if v is a struct, else 0

─── Display ───────────────────────────────────────────────────────────────────

  s =

    struct with fields:

      x: 1
      y: [1×3 double]
      inner: [1×1 struct]

  Nested struct fields are shown as [1×1 struct] inline; expanded when accessed.

─── Example ───────────────────────────────────────────────────────────────────

  % 3-D point struct
  pt.x = 3;  pt.y = 4;  pt.z = 0;
  dist = sqrt(pt.x^2 + pt.y^2 + pt.z^2)   →  5

  % struct() constructor + fieldnames
  p = struct('name', 'Alice', 'score', 98.5);
  fn = fieldnames(p);
  fn{{1}}    →  name
  fn{{2}}    →  score

  % Check, remove, iterate
  isfield(p, 'score')       →  1
  isfield(p, 'rank')        →  0
  p2 = rmfield(p, 'score');
  numel(fieldnames(p2))     →  1

─── Workspace ─────────────────────────────────────────────────────────────────

  Structs are NOT persisted by ws/save — same policy as matrices and cells.
  who shows: s = [1×1 struct]

See also: help cells  help userfuncs  help control
Example:  ccalc examples/structs.calc"
    );
}
