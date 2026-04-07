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
        Some("matrices" | "matrix" | "mat") => print_matrices(),
        Some("logic" | "logical" | "comparison") => print_logic(),
        Some("examples" | "ex") => print_examples(),
        Some("vectors" | "vector" | "utils") => print_vectors(),
        Some("complex" | "cplx" | "imag") => print_complex(),
        Some("strings" | "string" | "str" | "char") => print_strings(),
        Some(unknown) => {
            eprintln!("Unknown help topic: '{unknown}'");
            eprintln!(
                "Available topics: syntax  functions  bases  vars  script  matrices  logic  vectors  complex  strings  examples"
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
Comparison  ==  ~=  <  >  <=  >=     return 1 (true) or 0 (false)
Logical     ~expr  &&  ||             NOT, AND, OR
Constants   pi  e  ans  nan  inf  i  j  (imaginary unit)
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
Complex 3+4i  3+4j  complex(re,im)
        real(z) imag(z) abs(z) angle(z) conj(z) isreal(z)
        z' = conj(z)  (conjugate transpose for scalars)
Strings 'char array'  \"string object\"
        num2str  str2num  str2double  strcat  strcmp  strcmpi
        lower  upper  strtrim  strrep  sprintf  ischar  isstring
Bitwise bitand(a,b)  bitor(a,b)  bitxor(a,b)
        bitshift(a,n)  bitnot(a)  bitnot(a,bits)

Vars    x = expr              shows: x = <val>  (ans unchanged)
        x = expr;             silent assignment
        who   clear   clear x
        ws (save workspace)   wl (load workspace)

Output  disp(expr)            fprintf('text\\n')
Prec    p<N>  (0-15 decimal places, default 10)
Config  config                show config path and active settings
        config reload         re-read config.toml and apply changes
REPL    exit  quit  cls  Ctrl+L (clear screen)
Keys    ↑↓ history  Ctrl+R search  Ctrl+A/E line start/end
        Ctrl+W del word  Ctrl+U del to start  Ctrl+K del to end

  help syntax      operators, precedence, implicit multiplication
  help functions   full function reference with examples
  help bases       number bases, display switching
  help vars        variables and workspace
  help script      pipe/script mode, semicolons, disp, fprintf
  help matrices    matrix literals, arithmetic, ranges, indexing
  help vectors     nan/inf, reductions, sort/find/unique, end, reshape
  help logic       comparison and logical operators, masks
  help complex     complex numbers, i/j unit, abs/angle/conj/real/imag
  help strings     char arrays, string objects, strcmp, num2str, ...
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
    Comparison:   ==  ~=  <  >  <=  >=    (return 1.0 or 0.0)
    Logical:      ~expr   &&   ||

    Precedence (high to low):
      postfix '   transpose (matrices)
      ^           exponentiation (right-associative)
      unary -  ~  negation, logical NOT
      *  /  .*  ./  .^   multiply, divide, element-wise
      +  -        addition, subtraction
      :           range (a:b, a:step:b)
      ==  ~=  <  >  <=  >=   comparison (non-associative)
      &&          short-circuit logical AND
      ||          short-circuit logical OR

    2 > 1 && 3 > 2     →  1    AND of two comparisons
    1 + 1 == 2         →  1    arithmetic evaluated first
    ~0                 →  1    logical NOT
    v > 3              →  element-wise mask (0/1 matrix)

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
    % this is a comment
    10 * 5  % inline comment — expression still evaluates

Semicolon
    Trailing ; suppresses output.
    Expressions — ans is still updated:    0.06 / 12;
    Assignments — ans is never updated:    rate = 0.06 / 12;

    Multiple statements on one line (; as separator):
    a = 1; b = 2         a = 1 silent,  b = 2 shown
    a = 1; b = 2;        both silent

    Inside a matrix literal [ ], ; is always a row separator:
    [1 2; 3 4]           2×2 matrix — the ; is not a statement separator

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
    sprintf(fmt)       process escape sequences (\\n \\t \\\\); 1-arg form
    ischar(s)          1 if s is a char array, else 0
    isstring(s)        1 if s is a string object, else 0

See also: help vectors    (sum, min, max, sort, find, norm, cumsum, ...)
          help complex    (full complex number reference)
          help strings    (char arrays, string objects, full reference)"
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
    a == b      equal
    a ~= b      not equal
    a <  b      less than
    a >  b      greater than
    a <= b      less than or equal
    a >= b      greater than or equal

Logical
    ~expr       NOT: 1 if expr == 0, else 0
    a && b      AND: 1 if both non-zero (scalar or element-wise)
    a || b      OR:  1 if either non-zero (scalar or element-wise)

Precedence (low to high inside an expression)
    ||  →  &&  →  comparisons  →  :  →  +/-  →  *//  →  ^  →  unary

Scalar examples
    3 > 2             →  1
    3 == 4            →  0
    5 ~= 5            →  0
    ~0                →  1
    ~1                →  0
    2 > 1 && 3 > 2    →  1
    0 || 1            →  1
    ~(3 == 3)         →  0

Arithmetic + comparison
    1 + 1 == 2        →  1    (arithmetic first, then ==)
    2 * 3 > 5         →  1
    2 > 3 || 1 < 2    →  1

Element-wise on matrices
    v = [1 2 3 4 5]
    v > 3             →  [0 0 0 1 1]
    v == 3            →  [0 0 1 0 0]
    v ~= 3            →  [1 1 0 1 1]

Soft masking — zero out elements that fail a condition
    v .* (v > 3)      →  [0 0 0 4 5]    keep elements > 3 only

Combining two masks with .*  (element-wise AND)
    lo = v >= 2;  hi = v <= 4;
    v .* (lo .* hi)   →  [0 2 3 4 0]

See also: help matrices, help syntax
Example:  ccalc examples/logic.calc"
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
    3 + 4*i          →  3 + 4i    (i is pre-set to the imaginary unit)
    3 + 4*j          →  3 + 4i    (j is also the imaginary unit)
    complex(3, 4)    →  3 + 4i    (construct from real and imaginary parts)
    5*i              →  5i         (pure imaginary)
    2 - 3*i          →  2 - 3i

    4i works via implicit multiplication: 4 * i.
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

Conjugate transpose
    z = 3 + 4*i
    z'         →  3 - 4i      (conjugate for complex scalars)
    conj(z)    →  3 - 4i      (same result)

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
    str2num(s)          char array → number  (error if not parseable)
    str2double(s)       char array → number  (NaN if not parseable)
    strcat(a, b, ...)   concatenate two or more strings
    strcmp(a, b)        1 if equal (case-sensitive), else 0
    strcmpi(a, b)       1 if equal (case-insensitive), else 0
    lower(s)            convert to lowercase
    upper(s)            convert to uppercase
    strtrim(s)          strip leading and trailing whitespace
    strrep(s, old, new) replace all occurrences of old with new
    sprintf(fmt)        process escape sequences (\\\\n \\\\t ...); 1-arg form
    ischar(s)           1 if s is a char array, else 0
    isstring(s)         1 if s is a string object, else 0

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
    ccalc examples/strings.calc"
    );
}
