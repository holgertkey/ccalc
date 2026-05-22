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
        Some("index" | "indexing" | "indexed" | "assignment") => print_index_assign(),
        Some("logic" | "logical" | "comparison") => print_logic(),
        Some("examples" | "ex") => print_examples(),
        Some("vectors" | "vector" | "utils") => print_vectors(),
        Some("complex" | "cplx" | "imag") => print_complex(),
        Some("strings" | "string" | "str" | "char") => print_strings(),
        Some("files" | "file" | "fileio" | "io" | "fopen" | "fclose") => print_fileio(),
        Some(
            "control" | "flow" | "if" | "for" | "while" | "switch" | "do" | "run" | "source"
            | "path" | "addpath" | "rmpath",
        ) => print_control(),
        Some(
            "errors" | "error" | "warning" | "try" | "catch" | "pcall" | "lasterr"
            | "error-handling" | "exceptions",
        ) => print_errors(),
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
        Some(
            "scoping" | "scope" | "global" | "persistent" | "private" | "packages" | "package"
            | "namespace" | "namespaces" | "pkg",
        ) => print_scoping(),
        Some(
            "stats" | "stat" | "statistics" | "random" | "rand" | "randn" | "randi"
            | "distribution" | "distributions" | "normal" | "prctile" | "zscore" | "erf"
            | "skewness" | "kurtosis",
        ) => print_stats(),
        Some(
            "linalg" | "linear" | "linear-algebra" | "linearalgebra" | "decomp" | "decomposition"
            | "svd" | "qr" | "lu" | "eig" | "chol" | "cholesky" | "rank" | "null" | "orth" | "cond"
            | "pinv",
        ) => print_linalg(),
        Some("testing" | "assert" | "test" | "tests") => print_testing(),
        Some("csv" | "readmatrix" | "readtable" | "writetable" | "table") => print_csv(),
        Some("json" | "jsondecode" | "jsonencode") => print_json(),
        Some("matfile" | "mat-file" | "loadmat" | "load-mat") => print_matfile(),
        Some("regex" | "regexp" | "regexpi" | "regexprep" | "regular-expressions") => print_regex(),
        Some(
            "datetime" | "duration" | "nat" | "isdatetime" | "isduration" | "isnat" | "datestr"
            | "datevec" | "datenum" | "posixtime",
        ) => print_datetime(),
        Some(
            "setops" | "set" | "sets" | "triu" | "tril" | "repmat" | "kron" | "cross" | "dot"
            | "intersect" | "union" | "setdiff" | "ismember" | "sub2ind" | "ind2sub" | "repelem",
        ) => print_setops(),
        Some(
            "poly" | "polyfit" | "polyval" | "roots" | "conv" | "deconv" | "interp1" | "polynomial"
            | "polynomials" | "interpolation",
        ) => print_poly(),
        Some("eval" | "tic" | "toc" | "dynamic" | "timing" | "metaprogramming") => print_eval(),
        Some(
            "fft" | "ifft" | "fftshift" | "ifftshift" | "fftfreq" | "signal" | "spectrum"
            | "spectral",
        ) => print_fft(),
        Some(
            "plot" | "scatter" | "bar" | "stem" | "stairs" | "hist" | "loglog" | "semilogx"
            | "semilogy" | "plot3" | "scatter3" | "3d" | "xlabel" | "ylabel" | "zlabel" | "title"
            | "xlim" | "ylim" | "zlim" | "legend" | "grid" | "figurestate" | "plotting" | "charts"
            | "svg" | "png" | "colormap" | "colorbar" | "imagesc" | "heatmap" | "surf" | "mesh"
            | "meshgrid" | "surface" | "wireframe" | "subplot" | "hold" | "savefig" | "multipanel"
            | "panels" | "fill" | "area" | "polar" | "style" | "linestyle" | "color" | "stylestr"
            | "quiver" | "vector" | "vectorfield" | "text" | "annotation" | "annotations",
        ) => print_plot(),
        Some(unknown) => {
            eprintln!("Unknown help topic: '{unknown}'");
            eprintln!(
                "Available topics: syntax  functions  userfuncs  cells  structs  errors  testing  scoping  stats  linalg  bases  vars  script  format  matrices  index  logic  vectors  complex  strings  datetime  regex  files  csv  json  matfile  control  path  setops  poly  eval  fft  plot  examples"
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

Operators   + - * / ^ .^ **     ^ .^ ** right-assoc; -x^2 = -(x^2)
            2(3+1) → 8           implicit multiplication
            +x                   unary + (no-op)   ...  line continuation
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
        A\\b (left-divide / linear solve)  a\\b = b/a (scalar)
        zeros(m,n)  ones(m,n)  eye(n)  size  det  inv  trace
Range   1:5  →  [1 2 3 4 5]     1:2:9  →  [1 3 5 7 9]
        linspace(a,b,n)   [1:3, 10]  →  [1 2 3 10]
Index   v(3)  v(2:4)  v(:)  v(end)  v(end-1:end)   1-based
        A(i,j)  A(:,j)  A(i,:)  A(end,:)  A(1:end-1, 2:end)
        v(i) = x  v(1:3) = 0  v(end+1) = x  A(:,j) = col
        v(v > 0)  v(mask) = 0  (logical mask indexing)
Vector  sum prod mean min max any all norm(v) norm(v,p)
        cumsum cumprod  sort  find  unique
        reshape(A,m,n)  fliplr  flipud
NaN/Inf nan  inf  isnan  isinf  isfinite  nan(m,n)
Stats   rand randn randi rng(seed)  std var median mode cov
        prctile iqr zscore  skewness kurtosis
        hist histc  normcdf normpdf erf erfc
Linalg  qr lu chol svd eig         decompositions (see help linalg)
        rank null orth cond pinv    matrix properties
        norm(A)  norm(A,'fro')  norm(A,1)  norm(A,inf)
Complex 3+4i  3+4j  4i  complex(re,im)    (Ni syntax works directly)
        real(z) imag(z) abs(z) angle(z) conj(z) isreal(z)
        z' = conj(z)   z.' = plain transpose (no conjugation)
Strings 'char array'  \"string object\"
        num2str  str2num  str2double  strcat  strcmp  strcmpi
        lower  upper  strtrim  strrep  ischar  isstring
        strsplit(s,delim)  strjoin(c,delim)  int2str(x)  mat2str(A)
        contains(s,pat)  startsWith(s,pat)  endsWith(s,pat)
        regexp  regexpi  regexprep           (help regex for details)
Datetime NaT  datetime('2024-06-01')  datetime(y,m,d[,H,M,S])
         datetime(ts,'ConvertFrom','posixtime')
         duration(H,M,S)  hours(n) minutes(n) seconds(n) days(n) milliseconds(n) years(n)
         dt+dur→DateTime  dt-dur→DateTime  dt-dt→Duration  dur+dur  dur*scalar
         year(dt) month(dt) day(dt) hour(dt) minute(dt) second(dt)
         isdatetime(x)  isduration(x)  isnat(x)
         datestr(dt[,fmt])  datevec(dt)  datenum(dt)  posixtime(dt)
         [dt1;dt2]→DateTimeArray   [d1;d2]→DurationArray   diff(arr)
Set ops triu(A[,k])  tril(A[,k])  repmat(A,m,n)  kron(A,B)
        cross(a,b)  dot(a,b)
        intersect  union  setdiff  ismember
        sub2ind([r c],r,c)  ind2sub([r c],idx)  repelem(v,n)
FFT     fft(x)  fft(x,n)  ifft(X)              (requires --features fft)
        fftshift(x)  ifftshift(x)  fftfreq(n,d)  (always available)
Plot    plot(x,y)  scatter(x,y)              ASCII chart (requires --features plot)
        plot3(x,y,z)  scatter3(x,y,z)       3D line/scatter (orthographic projection)
        fill(x,y)  area(x,y)               filled polygon / filled area under curve
        polar(theta,r)                      polar coordinate line chart
        quiver(x,y,u,v)                     vector field arrows (Unicode / SVG)
        text(x,y,'label')                   text annotation at data coordinates
        plot(x,y,'r--')                     style string: color + linestyle (MATLAB)
        plot(x,y,'f.svg')  plot(x,y,'f.png') file export (requires --features plot-svg)
        plot3(x,y,z,'f.svg')               3D file export via plotters build_cartesian_3d
        title('t')  xlabel('x')  ylabel('y')  zlabel('z')  set annotations
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
Errors    error('msg')  error('fmt', v...)  raise a runtime error
          warning('fmt', v...)              print warning, continue
          lasterr()                         last error message
          try / catch / end                 protected block (anonymous)
          try / catch e / end              named: e.message = error string
          try(expr, default)               inline fallback (lazy default)
          pcall(@f, args...)               [ok, val] = pcall(...)
Testing   assert(cond)                     pass if cond is truthy
          assert(expected, actual)         exact equality check
          assert(expected, actual, tol)    tolerance check  |a-b| <= tol
Scripts   run('file.calc')  run('file')     .calc first, then .m
          source('file')                    Octave alias for run()
Path      addpath('/dir')                   prepend to session search path
          addpath('/dir', '-end')           append to session search path
          rmpath('/dir')                    remove from search path
          path()                            display current search path
          genpath('/dir')                   return dir + all subdirs as path string
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
Scoping global x              shared across all functions and the workspace
        persistent x         per-function value that survives between calls
        private/             directory-scoped helpers (visible only to parent dir)
        utils.func(args)     package call — searches +utils/func.calc on path
Vars    x = expr              shows: x = <val>  (ans unchanged)
        x = expr;             silent assignment
        who   clear   clear x
        ws/save (save workspace)   wl/load (load workspace)
        save('f.mat')  save('f.mat','x','y')  load('f.mat')

Output  disp(expr)
        fprintf('fmt', v1, v2, ...)   print formatted  (C printf)
        sprintf('fmt', v1, v2, ...)   return formatted string
        Specifiers: %d %i %f %e %g %x %X %s %%   Width/prec: %8.3f %-10s %04X
Files   fd = fopen('f.txt','w')   fclose(fd)   fclose('all')
        fprintf(fd,'fmt',v1,...)  fgetl(fd)  fgets(fd)
        dlmwrite('f.csv',A)  dlmwrite('f.tsv',A,'\t')
        data = dlmread('f.csv')  data = dlmread('f.tsv','\t')
        A = readmatrix('f.csv')          readmatrix(f,'Delimiter','\t')
        T = readtable('f.csv')           writetable(T,'out.csv')
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
  help structs     scalar structs + struct arrays, field access, fieldnames/isfield/rmfield
  help errors      error/warning, try/catch, try(expr,default), pcall
  help testing     assert(cond), assert(a,b), assert(a,b,tol) — unit testing
  help scoping     global/persistent variables, private/ dirs, packages (pkg.func)
  help bases       number bases, display switching
  help format      number display format modes (short/long/bank/rat/hex/+)
  help vars        variables and workspace
  help script      pipe/script mode, semicolons, disp, fprintf
  help matrices    matrix literals, arithmetic, ranges, indexing
  help index       indexed assignment, growing vectors, logical masks
  help vectors     nan/inf, reductions, sort/find/unique, end, reshape, diag
  help stats       rand/randn/rng, std/var/median/mode, skewness/kurtosis, prctile/iqr/zscore, hist, normcdf
  help linalg      qr/lu/chol/svd/eig decompositions; rank/null/orth/cond/pinv; matrix norms
  help logic       comparison and logical operators, masks
  help complex     complex numbers, i/j unit, abs/angle/conj/real/imag
  help strings     char arrays, string objects, strcmp, num2str, ...
  help datetime    datetime/duration types, constructors, arithmetic, formatting
  help files       file I/O: fopen/fclose/fgetl/fgets, dlmread/dlmwrite, isfile, pwd
  help csv         readmatrix, readtable, writetable — CSV with headers and type inference
  help json        jsondecode / jsonencode (requires --features json build)
  help matfile     load('file.mat') — MAT file read (requires --features mat build)
  help control     if/for/while, break/continue, compound assignment, run/source
  help path        addpath/rmpath/path()/genpath() — session search path
  help setops      triu/tril/repmat/kron/cross/dot, set ops, sub2ind/ind2sub/repelem
  help poly        polyval/polyfit/roots/poly, conv/deconv, interp1
  help fft         fft/ifft, fftshift/ifftshift, fftfreq — FFT & signal processing
  help plot        plot/scatter/fill/area/polar, style strings, file export (SVG/PNG)
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
      ^  .^  **   exponentiation (right-associative; -x^2 = -(x^2))
      unary +  -  ~  no-op, negation, logical NOT
      *  /  \\  .*  ./  multiply, divide, left-divide, element-wise
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
    -3 ^ 2             → -9    unary minus lower than ^; same as -(3^2)

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

    Block comments (multi-line):
    %{{
      Everything between %{{ and %}} is ignored.
      %{{ and %}} must be the leading non-whitespace content on their line.
    %}}
    #{{ ... #}}  is the hash-style equivalent.
    A self-contained form %{{ ... %}} on a single line is also valid.

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
    log(x)        natural logarithm (base e)
    log2(x)       base-2 logarithm
    log10(x)      base-10 logarithm
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

Special functions
    erf(x)             Gauss error function: (2/√π) ∫₀ˣ e^(-t²) dt
    erfc(x)            complementary: 1 - erf(x)
    normcdf(x)         standard normal CDF: P(Z ≤ x)   Z~N(0,1)
    normcdf(x, mu, s)  general normal CDF: P(X ≤ x)    X~N(mu,s²)
    normpdf(x)         standard normal PDF
    normpdf(x, mu, s)  general normal PDF

    erf(0) = 0     erf(1) ≈ 0.8427     erfc(x) = 1 - erf(x)
    normcdf(0) = 0.5
    normcdf(1) - normcdf(-1) ≈ 0.6827  (68% of N(0,1) within ±1σ)

    All accept scalars or matrices (element-wise).

See also: help stats      (rand/randn/std/prctile/hist and full stats reference)
          help vectors    (sum, min, max, sort, find, norm, cumsum, ...)
          help complex    (full complex number reference)
          help strings    (char arrays, string objects, predicates, strjoin, ...)
          help regex      (regexp, regexpi, regexprep — requires --features regex)
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
    %x        hexadecimal lowercase  (ff)
    %X        hexadecimal uppercase  (FF)
    %s        string
    %%        literal percent sign
    Width/flags:  %8.3f   %-10s   %+.4e   %05d   %04X

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

Left division (backslash)
    A \\ b            solve A*x = b  (Gaussian elim with partial pivoting)
                      more stable than inv(A) * b
    a \\ b            scalar: equivalent to b / a
    A \\ B            multiple RHS: solve for each column of B independently

    A = [2 1; 5 7];  b = [11; 13];
    x = A \\ b              →  [3; 5]   (solves exactly)
    4 \\ 20                 →  5        (scalar: 20 / 4)

Built-in functions
    zeros(m,n)        m×n matrix of zeros
    zeros(n)          n×n matrix of zeros
    ones(m,n)         m×n matrix of ones
    ones(n)           n×n matrix of ones
    eye(n)            n×n identity matrix
    size(A)           [rows cols] as a 1×2 row vector
    size(A, dim)      rows (dim=1) or cols (dim=2) as scalar
    length(A)         max(rows, cols)
    numel(A)          total element count
    trace(A)          sum of diagonal elements
    det(A)            determinant  (square matrices only)
    inv(A)            inverse  (square, non-singular)

Advanced linear algebra  (see: help linalg)
    [Q,R] = qr(A)     QR decomposition (Householder)
    [L,U,P] = lu(A)   LU with partial pivoting (PA = LU)
    R = chol(A)       Cholesky factor (A = R'*R, SPD only)
    [U,S,V] = svd(A)  full SVD; s = svd(A) — singular values only
    [U,S,V] = svd(A,'econ')  economy SVD
    [V,D] = eig(A)    eigendecomposition; d = eig(A) — eigenvalues only
    rank(A)           numerical rank via SVD
    null(A)           orthonormal null-space basis
    orth(A)           orthonormal column-space basis
    cond(A)           condition number (sigma_max / sigma_min)
    pinv(A)           Moore-Penrose pseudoinverse
    norm(A)           matrix 2-norm (largest singular value)
    norm(A,'fro')     Frobenius norm
    norm(A,1)         max column-sum norm
    norm(A,inf)       max row-sum norm

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

Indexed assignment  (write path — mirrors the read forms above)
    v(3) = 42             set single element
    v(1:3) = [10 20 30]   slice: RHS same length as selection
    v(4:6) = 0            scalar broadcast to all selected positions
    v(:) = 0              reset all elements at once
    A(2,3) = 7            2-D element
    A(:,1) = [1;2;3;4]   entire column
    A(1,:) = [1 2 3 4]   entire row
    A(2:3,2:3) = eye(2)   submatrix

Growing vectors — assigning beyond current length extends with zeros
    v = [];  v(end+1) = x   append: end resolves to current length
    v(7) = 99               pads to length 7, fills gap with zeros
    v(i) = x                auto-creates row vector if v is undefined

Logical (boolean mask) indexing
    v(v > 0)              read: select elements where mask is true
    v(v < 0) = 0          write: modify elements where mask is true
    m = v > 3;  v(m) = 0  using a pre-computed mask variable
    M(M > 5)              2-D matrix: elements in column-major order
    M(M > 5) = 0          2-D masked write

Workspace
    ws  saves only scalar variables — matrices are not persisted.
    who shows dimensions:  A = [2×2 double]

See also: help linalg   (advanced linear algebra reference)
          help index    (full indexed-assignment reference)
          help setops   (triu/tril/repmat/kron, set ops, sub2ind/repelem)"
    );
}

// ---------------------------------------------------------------------------
// help index
// ---------------------------------------------------------------------------

fn print_index_assign() {
    println!(
        "\
INDEXED ASSIGNMENT  (help index)

All read-index forms work as write targets. The left-hand side must be a
variable name — arbitrary expressions are not allowed as assignment targets.

─── Scalar and slice assignment ────────────────────────────────────────────────

  v = zeros(1, 6);
  v(3) = 42;               % set element 3
  v(1:2) = [10, 20];       % slice: RHS length must equal selection
  v(4:6) = 99;             % scalar broadcast to 3 positions
  v(:) = 0;                % reset all elements at once

─── 2-D matrix assignment ──────────────────────────────────────────────────────

  A = zeros(4);
  A(2, 3) = 7;               % scalar at row 2, col 3
  A(:, 1) = [1; 2; 3; 4];   % full column (RHS must be column vector)
  A(1, :) = [10 20 30 40];   % full row
  A(2:3, 2:3) = eye(2);      % 2×2 submatrix

─── Broadcasting rule ──────────────────────────────────────────────────────────

  When RHS is a scalar and the LHS index selects multiple elements,
  the scalar is broadcast to every selected position:

  v(1:5) = 0                 zero five elements
  A(:, 2) = 1                fill column 2 with ones

─── Growing vectors ────────────────────────────────────────────────────────────

  v = [];
  for k = 1:5
    v(end+1) = k^2;          % end = current length, so this appends
  end
  % v = [1 4 9 16 25]

  v = [1 2 3];
  v(7) = 99;                 % → [1 2 3 0 0 0 99]  (gap padded with zeros)

  If the variable does not exist, the first indexed assignment creates it
  as a 1×N row vector.

─── Logical (boolean mask) indexing ────────────────────────────────────────────

  A 0/1 vector whose element count equals the dimension size is a boolean
  mask rather than a list of indices.  This enables compact read/write idioms:

  Read — select elements where mask is 1:
    v = [3, -1, 8, 0, 5, -2, 7];
    pos = v(v > 0);            % → [3  8  5  7]

  Write — modify elements where mask is 1:
    v(v < 0) = 0;              % zero out negatives (half-wave rectifier)

  Using a separate mask variable:
    signal = [0.5, -1.2, 0.8, -0.3, 1.5, -2.0, 0.1];
    noise  = signal < 0;
    signal(noise) = 0;         % zero out noise samples

  2-D matrix logical mask (elements in column-major order):
    M = [1 2 3; 4 5 6; 7 8 9];
    M(M > 5)                   % → [7 8 6 9]
    M(M > 5) = 0;              % zero those elements in place

─── end in index expressions ───────────────────────────────────────────────────

  end resolves to the current length of the dimension being indexed,
  enabling relative addressing from the tail:

    v(end)       last element (read)
    v(end) = x   overwrite last element (write)
    v(end+1) = x append (write — extends the vector)
    v(end-1:end) = [a b]   overwrite last two elements

─── Practical patterns ─────────────────────────────────────────────────────────

  % Build a Fibonacci sequence element by element
  fib = [];
  fib(1) = 1;  fib(2) = 1;
  for k = 3:12
    fib(end+1) = fib(end) + fib(end-1);
  end

  % Collect even numbers, then cap at 10
  evens = [];
  for k = 1:20
    if mod(k, 2) == 0
      evens(end+1) = k;
    end
  end
  evens(evens > 10) = 10;

See also: help matrices  help vectors  help logic
Example:  ccalc examples/indexed_assignment.calc"
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

Reshape, flip, and diagonal
    reshape(A, m, n)    reshape to m×n  (column-major element order)
    fliplr(v)           reverse column order  (mirror left↔right)
    flipud(v)           reverse row order    (mirror up↔down)
    diag(v)             vector → N×N diagonal matrix
    diag(A)             extract main diagonal of A as a column vector

    reshape(1:6, 2, 3)  →  [1 3 5; 2 4 6]
    fliplr([1 2 3])     →  [3 2 1]
    flipud([1;2;3])     →  [3;2;1]
    diag([1 2 3])       →  [1 0 0; 0 2 0; 0 0 3]
    diag(eye(3))        →  [1; 1; 1]

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

See also: help matrices  help functions  help stats  help setops
Example:  ccalc examples/vector_utils.calc"
    );
}

// ---------------------------------------------------------------------------
// help stats
// ---------------------------------------------------------------------------

fn print_stats() {
    println!(
        "\
STATISTICS AND RANDOM NUMBERS  (help stats)

Random number generation
    rand()              scalar uniform in [0, 1)
    rand(n)             n×n uniform matrix
    rand(m, n)          m×n uniform matrix
    randn()             scalar standard-normal sample  N(0, 1)
    randn(n)            n×n standard-normal matrix
    randn(m, n)         m×n standard-normal matrix
    randi(max)          random integer in [1, max]
    randi(max, n)       n×n matrix of integers in [1, max]
    randi(max, m, n)    m×n matrix of integers in [1, max]
    randi([lo hi], ...) integers drawn from [lo, hi]
    rng(seed)           seed RNG — same seed → same sequence
    rng('shuffle')      reseed from system entropy

    rng(42); x = randn(1, 5)    →  reproducible 5-element sequence

Descriptive statistics  (column-wise for M×N matrices, scalar for vectors)
    std(v)              sample standard deviation  (n-1 denominator)
    std(v, 1)           population standard deviation  (n denominator)
    var(v)              sample variance
    var(v, 1)           population variance
    median(v)           median (linear interpolation when length is even)
    mode(v)             most frequent value  (smallest wins on ties)
    cov(v)              variance of a vector  (scalar, n-1 denominator)
    cov(A)              N×N covariance matrix of m×N data matrix A

    v = [2 4 6 8];
    std(v)   →  2.582    var(v)    →  6.667    median(v)  →  5

Shape statistics  (population / biased moment formulas)
    skewness(v)         m3 / m2^(3/2)  — 0 = symmetric, >0 = right tail
    kurtosis(v)         m4 / m2^2      — ≈1.8 uniform, ≈3 normal, >3 heavy tails
                        scalar or constant input: skewness→0, kurtosis→NaN

    skewness([2 4 4 4 5 5 7 9]) →  0.656
    kurtosis([2 4 4 4 5 5 7 9]) →  2.781
    skewness(1:10)              →  0        (symmetric)

Percentiles and spread
    prctile(v, p)       p-th percentile (0–100); p can be a vector
    iqr(v)              interquartile range: prctile(75) - prctile(25)
    zscore(v)           standardise: (v - mean(v)) / std(v)  (same shape)

    prctile([1 2 3 4 5], 50)      →  3
    prctile([1 2 3 4 5], [25 75]) →  [1.5  4.5]    (quartiles)
    iqr([1 2 3 4 5])              →  2

    zscore([2 4 6]) → [-1  0  1]   (mean=4, std=2)

Histogram  (implemented in ccalc-plot plugin — see: help plot)
    hist(v)             ASCII bar chart, Sturges bins; returns Void
    hist(v, n)          ASCII bar chart with n uniform bins
    hist(v, edges)      ASCII bar chart with caller-supplied edge vector
    hist(v, …, 'f.svg') save to SVG/PNG (requires --features plot-svg)
    histc(v, edges)     bin counts returned as row vector (engine built-in)
                        bin i: edges(i) <= x < edges(i+1)
                        last bin: x == edges(end) exactly

    histc([1 1 2 3], [1 2 3])  →  [2  1  1]

Normal distribution  (see also: help functions for erf/erfc)
    normcdf(x)          standard normal CDF: P(Z ≤ x)  Z ~ N(0,1)
    normcdf(x, mu, s)   general normal CDF: P(X ≤ x)   X ~ N(mu, s²)
    normpdf(x)          standard normal PDF: exp(-x²/2) / sqrt(2π)
    normpdf(x, mu, s)   general normal PDF
    erf(x)              Gauss error function: (2/√π) ∫₀ˣ e^(-t²) dt
    erfc(x)             complementary: 1 - erf(x)

    normcdf(0)                     →  0.5
    normcdf(1) - normcdf(-1)       →  0.6827   (68% rule)
    normcdf(2) - normcdf(-2)       →  0.9545   (95% rule)
    normcdf(3) - normcdf(-3)       →  0.9973   (99.7% rule)
    P(40 < X < 60) for X~N(50,10): normcdf(60,50,10) - normcdf(40,50,10)

    Relationship: normcdf(x) = 0.5 * (1 + erf(x / sqrt(2)))

Example
    rng(7)
    data = randn(1, 200) * 10 + 50;   % 200 samples from N(50, 10)
    fprintf('mean   = %.2f\\n', mean(data))
    fprintf('std    = %.2f\\n', std(data))
    fprintf('median = %.2f\\n', median(data))
    fprintf('IQR    = %.2f\\n', iqr(data))
    hist(data, 12)

See also: help vectors  help functions
Full example: ccalc examples/statistics.calc"
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

Complex matrices
    Any matrix literal with at least one complex element becomes a ComplexMatrix.
    A = [1+2i, 3-4i; 5, 6+1i]    % 2×2 ComplexMatrix
    v = [1+i, 2-i, 3]             % 1×3 ComplexMatrix
    isreal(A)  →  0               % always false for ComplexMatrix

    Display: every cell shows both parts  →  5 + 0i,  1 + 1i,  0 + 2i

    Arithmetic: +, -, .*, ./, .^ element-wise;  * is matrix multiply.
    A'     conjugate transpose (Hermitian adjoint)
    A.'    plain transpose (no conjugation)

    Element-wise built-ins on ComplexMatrix:
    real(A)    → real Matrix of real parts
    imag(A)    → real Matrix of imaginary parts
    abs(A)     → real Matrix of element-wise moduli
    conj(A)    → ComplexMatrix of element-wise conjugates
    angle(A)   → real Matrix of arguments (radians)
    norm(A)    → Frobenius norm (scalar)

    Reduction built-ins on ComplexMatrix:
    trace(A)   → Complex scalar (sum of diagonal)
    diag(A)    → ComplexMatrix N×1 column vector of diagonal elements
    diag(v)    → ComplexMatrix diagonal matrix built from a complex vector
    sum(A)     → column sums: ComplexMatrix 1×N; vector → Complex scalar
    prod(A)    → column products: ComplexMatrix 1×N; vector → Complex scalar
    mean(A)    → column means: ComplexMatrix 1×N; vector → Complex scalar

    Indexing and assignment: 1-based column-major, same as real matrices
    X(2)           → second element of a row vector
    A(1,:)         → first row
    A(:,2)         → second column
    A(i,j) = z    → in-place assignment (ComplexMatrix stays ComplexMatrix)
    B(i,j) = z    → auto-upcast: real Matrix → ComplexMatrix when z is complex

    Block concatenation: ComplexMatrix mixes freely with real Matrix blocks
    [CM, M]  /  [M, CM]  /  [CM; M]  /  [M; CM]  all produce ComplexMatrix.

    FFT output: fft() returns a ComplexMatrix (1×N row vector).
    Access bins with X(k); use abs(X) for magnitude spectrum.

    Complex eigenvalues: eig(A) returns a ComplexMatrix column vector when
    A is non-symmetric and has complex conjugate eigenvalue pairs.
    Use real(d) and imag(d) to access parts; all(real(d) < 0) for stability.

    ws/wl do not persist ComplexMatrix (same policy as all matrix types).

Example: ccalc examples/complex_matrix.m
         ccalc examples/complex_matrix_ext.m"
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

    Concatenation with [...] — MATLAB-style dynamic string building
    ['hello' ' world']          →  'hello world'
    ['prefix_' num2str(k)]      →  'prefix_3'   (when k = 3)
    ['A' 66 67]                 →  'ABC'        (numbers → chars by code)
    [65 'B']                    →  [65 66]      (numeric context: chars → codes)

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
    strjoin(c)          join cell array with space → char array
    strjoin(c, delim)   join cell array with delimiter
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

Predicates
    contains(s, pat)                       1 if pat found in s
    contains(s, pat, 'IgnoreCase', 1)      case-insensitive variant
    startsWith(s, pat)                     1 if s begins with pat
    endsWith(s, pat)                       1 if s ends with pat

    strsplit / strjoin examples
    parts = strsplit('a,b,c', ',')   → {{'a', 'b', 'c'}}  (cell array)
    parts{{1}}                       → 'a'
    words = strsplit('hello world')  → {{'hello', 'world'}}
    strjoin({{'x','y','z'}}, '-')    → 'x-y-z'
    strjoin({{'the','fox'}})         → 'the fox'

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

See also: help regex   (regexp, regexpi, regexprep — requires --features regex)

Example: ccalc examples/strings.calc
         ccalc examples/string_regex.calc"
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
    ccalc examples/extended_control_flow/extended_control_flow.calc
    ccalc examples/user_functions.calc
    ccalc examples/cell_arrays.calc
    ccalc examples/structs.calc
    ccalc examples/struct_arrays.calc
    ccalc examples/matrix_ops.calc
    ccalc examples/linear_algebra.calc
    ccalc examples/path_system.calc"
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

  See also: help csv  (readmatrix / readtable / writetable with header support)

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

─── Search path (addpath / rmpath / path / genpath) ───────────────────────────

  addpath('/dir')             prepend directory to the session search path
  addpath('/dir', '-end')     append directory to the session search path
  rmpath('/dir')              remove directory from the session search path
  path()                      display all search path entries
  genpath('/dir')             return '/dir' and all subdirectories as a
                              path-separator-delimited string (';' on Windows,
                              ':' on Unix); combine with addpath to add the
                              whole tree at once

  Search order for run() and script lookup:
    1. Current working directory
    2. Session path entries in order (first entry wins)

  Duplicate entries are silently deduplicated (last addpath wins position).
  path changes are session-only; they are NOT written back to config.toml.
  ~ is expanded to the user's home directory on all platforms.

  To make paths persistent, add them to ~/.config/ccalc/config.toml:
    path = [\"~/.config/ccalc/lib\", \"/home/user/scripts\"]

  A trailing slash on a config entry enables genpath semantics — the directory
  and all its subdirectories are added at startup:
    path = [\"~/.config/ccalc/lib/\", \"/home/user/scripts\"]

  Example:
    addpath('/my/scripts')
    addpath(genpath('/my/libs'))   % add /my/libs and every subdir
    addpath('/my/utils', '-end')
    path()                         % list the current path
    rmpath('/my/utils')            % remove an entry

─── REPL multi-line input ─────────────────────────────────────────────────────

  The REPL detects unclosed blocks by tracking depth changes:
    Keywords that open a block (+1): if  for  while  switch  do  try
    Keywords that close a block (-1): end  until
  Lines accumulate with a continuation prompt until the block is complete.
  Press Ctrl+C to cancel an in-progress block.

See also: help syntax  help logic  help userfuncs  help path  help errors
Examples: ccalc examples/control_flow.calc
          ccalc examples/extended_control_flow.calc
          ccalc examples/error_handling.calc
          ccalc examples/matrix_ops.calc   (backslash linear solve)
          ccalc examples/path_system.calc  (addpath/rmpath/path demo)"
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

─── Function files and autoload ───────────────────────────────────────────────

  A file starting with 'function' is a function file:
    - Only the PRIMARY function is added to the workspace on source().
    - Helper functions after the primary are LOCAL — invisible outside the
      file but available to the primary (MATLAB-style local scoping).

  AUTOLOAD: calling an unknown function name triggers an automatic search
  for <name>.calc / <name>.m on the current directory and session path.
  No explicit source() needed:

    [c, k] = bisect(@(x) x^2-2, 1, 2, 1e-8)   % bisect.calc auto-loaded

  source() still works for explicit loading.

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

─── global and persistent — cross-call state ──────────────────────────────────

  global x     Declares x as shared storage accessible from any function that
               also declares  global x.

  persistent x  Declares x as a per-function variable that retains its value
               between calls.  On the first call x is []; use isempty(x) to
               initialize it.

  See  help scoping  for full documentation with examples.

─── Documentation comments ────────────────────────────────────────────────────

  Place % lines immediately AFTER the function header (MATLAB H1-line style).
  help <name> prints them.

    function y = tri(n)
    % Return the nth triangular number T(n) = n*(n+1)/2.
    % Usage: t = tri(n)
    %
    % Example:
    %   tri(4)  →  10
      y = n * (n + 1) / 2;
    end

    >> help tri
    Return the nth triangular number T(n) = n*(n+1)/2.
    Usage: t = tri(n)

    Example:
      tri(4)  →  10

  Rules:
    - Consecutive % (or #) lines right after the function header form the block.
    - A blank line between the header and the first % breaks the association.
    - One leading space after % is stripped; further indentation is kept.
    - help <name> also works for functions on the path not yet called —
      the file is loaded on demand to extract the doc.

See also: help control  help functions  help cells  help scoping
Example:  ccalc examples/user_functions.calc
          ccalc examples/cell_arrays.calc
          ccalc examples/scoping/scoping.calc"
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
STRUCTS AND STRUCT ARRAYS  (help structs)

A scalar struct groups named fields, each holding any value (scalar, matrix,
string, complex, cell, or another struct).  Fields are ordered by insertion.

─── Scalar struct ─────────────────────────────────────────────────────────────

  s.x = 1               field assignment; creates struct if s doesn't exist yet
  s.y = [1 2 3]         field can hold any Value
  s.a.b = 42            nested field — creates s.a as an empty struct if needed

  s = struct()                  empty struct
  s = struct('x', 1, 'y', 2)   constructor; pairs: string key + value

  s.x                   read field value
  s.a.b                 chained: read nested field (any depth)

─── Built-in utilities ────────────────────────────────────────────────────────

  fieldnames(s)         cell array of field names, insertion order
  isfield(s, 'x')       1 if field 'x' exists, else 0
  rmfield(s, 'x')       copy of s with field 'x' removed; error if absent
  isstruct(v)           1 if v is a struct or struct array, else 0

─── Struct arrays ─────────────────────────────────────────────────────────────

  s(i).field = val      indexed assignment; creates/grows struct array
  s(i).field            read field from element i  (1-based)
  s.field               collect field across ALL elements:
                            all scalars → 1×N matrix
                            mixed types → 1×N cell array

  pts(1).x = 1;  pts(1).y = 0;
  pts(2).x = 3;  pts(2).y = 4;
  pts(3).x = 0;  pts(3).y = 5;

  numel(pts)     →  3
  pts(2).x       →  3

  xs = pts.x     →  [1 3 0]   (field collection)
  ys = pts.y     →  [0 4 5]

  String fields collect into a cell array:
  roster(1).name = 'Alice';  roster(2).name = 'Bob';
  names = roster.name        →  {{'Alice', 'Bob'}}

─── Display ───────────────────────────────────────────────────────────────────

  Scalar struct:
    s =
      scalar structure containing the fields:
        x: 1
        y: [1×3 double]

  Struct array (N > 1):
    pts =
      1×3 struct array with fields:
        x
        y

─── Workspace ─────────────────────────────────────────────────────────────────

  Structs are NOT persisted by ws/save — same policy as matrices and cells.
  who shows: s = [1×1 struct]  or  pts = [1×3 struct]

See also: help cells  help userfuncs  help control
Examples: ccalc examples/structs.calc
          ccalc examples/struct_arrays.calc"
    );
}

// ---------------------------------------------------------------------------
// help errors
// ---------------------------------------------------------------------------

fn print_errors() {
    println!(
        "\
ERROR HANDLING  (help errors)

─── error() and warning() ─────────────────────────────────────────────────────

  error(msg)               raise a runtime error; stops execution in current
                           block (caught by try/catch or propagates to REPL)
  error(fmt, v1, v2, ...)  printf-formatted message (same specifiers as fprintf)
  warning(msg)             print warning to stderr; execution continues
  warning(fmt, v1, ...)    printf-formatted warning

  Examples:
    error('value must be positive')
    error('expected %d arguments, got %d', 2, nargin)
    warning('result may be inaccurate: condition number = %.1e', cond(A))

─── lasterr ───────────────────────────────────────────────────────────────────

  lasterr()       return the message from the most recent runtime error
  lasterr(msg)    set the last-error string; returns the previous value
  lasterr('')     clear the last-error string (returns previous)

  lasterr is set automatically whenever the REPL or a try/catch block
  catches a runtime error.

  Examples:
    inv([1 0; 0 0]);          % triggers an error
    msg = lasterr()           % 'singular matrix'
    lasterr('');              % clear it

─── try / catch / end ─────────────────────────────────────────────────────────

  MATLAB-compatible protected block.  Two forms:

  Anonymous catch — no error variable:
    try
      risky_code()
    catch
      fallback_code()
    end

  Named catch — e is bound to a struct with field 'message':
    try
      result = risky_function(data)
    catch e
      fprintf('caught: %s\\n', e.message)
      result = default_value
    end

  try with no catch — silently swallows the error:
    try
      might_fail()
    end

  Behaviour:
    If the try body completes without error, the catch body is skipped.
    If any statement in the try body raises an error, execution jumps
    immediately to the catch body (remaining try statements are skipped).
    lasterr is set on entry to the catch body.
    break/continue/return inside a try or catch work as normal.

  Example:
    for k = 1:10
      try
        results(k) = compute(data(k))
      catch e
        fprintf('step %d failed: %s\\n', k, e.message)
        results(k) = 0
      end
    end

─── try(expr, default) — inline fallback ──────────────────────────────────────

  x = try(expr, default)

  Evaluates expr; returns its value on success. If expr raises an error,
  evaluates and returns default instead (lazy — default is only evaluated
  on failure).

  Examples:
    x = try(inv(A), eye(n))          % fallback to identity if singular
    n = try(str2num(s), 0)           % fallback to 0 if not a number
    v = try(risky(data), NaN)        % NaN sentinel on error

  Note: try(expr, default) is a special form, not a regular function call.
  The default expression is NOT evaluated unless expr fails.

─── pcall — protected call ────────────────────────────────────────────────────

  [ok, val] = pcall(@func, arg1, arg2, ...)

  Calls @func with the given arguments in a protected context.
  Returns a two-element tuple:
    ok = 1   val = function return value   (on success)
    ok = 0   val = error message string    (on failure)

  Compatible with anonymous functions and named function handles.
  lasterr is set to the error message on failure.

  Examples:
    [ok, x] = pcall(@inv, A)
    if ~ok
      fprintf('inv failed: %s\\n', x)
      x = eye(n)
    end

    [ok, y] = pcall(@(x) sqrt(x), -1)   % ok=0, y='sqrt of negative'

    for k = 1:numel(data)
      [ok, v] = pcall(@process, data(k))
      results(k) = ok * v             % 0 on failure
    end

─── 'e' as a catch variable ───────────────────────────────────────────────────

  The constant 'e' (Euler's number, 2.718...) and the catch variable 'e'
  do not conflict. Variable assignments always shadow built-in constants:

    try
      error('oops')
    catch e
      fprintf('message: %s\\n', e.message)   % e is a struct here
    end
    e                                         % back to 2.718... after block

See also: help control  help userfuncs  help structs
Example:  ccalc examples/error_handling.calc"
    );
}

// ---------------------------------------------------------------------------
// help scoping
// ---------------------------------------------------------------------------

fn print_scoping() {
    println!(
        "\
VARIABLE SCOPING  (help scoping)

Four mechanisms control visibility and lifetime of variables across functions.

─── global — shared workspace storage ─────────────────────────────────────────

  Declare the SAME name in every function that needs to share the value.
  Changes in one function are immediately visible in all others.

    function reset_counter()
      global g_count
      g_count = 0;
    end

    function increment(step)
      global g_count
      g_count = g_count + step;
    end

    reset_counter()
    increment(3)
    increment(7)
    % g_count is now 10 in the base workspace and in any function that
    % also declares  global g_count

  Use case: configuration, counters, shared state across a call graph.
  Anti-pattern: overusing globals creates hidden coupling; prefer passing
  values as arguments when the call chain is shallow.

─── persistent — per-function long-lived storage ──────────────────────────────

  A persistent variable keeps its value between calls to the SAME function.
  On the first call the variable is [], so isempty() is the standard guard.

    function n = how_many_calls()
      persistent call_count
      if isempty(call_count)
        call_count = 0;
      end
      call_count += 1;
      n = call_count;
    end

    how_many_calls()   % 1
    how_many_calls()   % 2
    how_many_calls()   % 3

  Use cases: call counters, memoization caches, lazy initialization.

  Memoized Fibonacci (persistent write-through ensures recursive calls see
  each other's updates immediately):

    function f = fib_memo(n)
      persistent cache
      if isempty(cache)
        cache = zeros(1, 100);
        cache(1) = 1;  cache(2) = 1;
      end
      if cache(n) ~= 0; f = cache(n); return; end
      cache(n) = fib_memo(n-1) + fib_memo(n-2);
      f = cache(n);
    end

─── private/ — directory-scoped helpers ───────────────────────────────────────

  Functions in a private/ sub-directory are visible ONLY to scripts and
  functions in the PARENT directory.  Any other caller sees 'Unknown function'.

  Directory layout:
    mylib/
      main.calc        <- can call clamp() and lerp()
      private/
        clamp.calc     <- invisible outside mylib/
        lerp.calc      <- invisible outside mylib/

  This is the file-system equivalent of making helpers package-private.
  private/ directories are skipped when ccalc builds the autoload path —
  even if mylib/ is on the session path, its private/ folder stays hidden.

    function y = normalize(data, lo, hi)
      % clamp() and lerp() come from private/ — callers cannot use them directly
      span = hi - lo;
      for k = 1:numel(data)
        y(k) = lerp(0, 1, (clamp(data(k), lo, hi) - lo) / span);
      end
    end

─── Packages (+pkg/) — named namespaces ───────────────────────────────────────

  A directory whose name starts with '+' is a PACKAGE.  Functions inside are
  invisible at the top level; call them with the package prefix:

    pkg.function(args)

  Example layout:
    +utils/
      clamp.calc          <- utils.clamp(x, lo, hi)
      lerp.calc           <- utils.lerp(a, b, t)
    +geom/
      circle_area.calc    <- geom.circle_area(r)

  Usage:
    utils.clamp(-3, 0, 10)         % 0
    utils.lerp(0, 100, 0.25)       % 25
    geom.circle_area(1)            % 3.14159...

  Nested packages map subdirectories:
    +geom/+solid/sphere_vol.calc   <- geom.solid.sphere_vol(r)

  Package functions are autoloaded on first call from SCRIPT_DIR_STACK → CWD
  → SESSION_PATH. No explicit source() required.

  Package directories are transparent to addpath and genpath — the search
  path does not include +pkg/ dirs directly; they are only found via the
  qualified call syntax.

─── Interaction summary ───────────────────────────────────────────────────────

  global     — cross-function shared state; requires declaration in each function
  persistent — per-function state; survives between calls; one slot per function
  private/   — file-system visibility guard; MATLAB-compatible
  +pkg/      — named namespace; avoids function-name collisions across libraries

See also: help userfuncs  help control  help path
Example:  ccalc examples/scoping/scoping.calc"
    );
}

// ---------------------------------------------------------------------------
// help linalg
// ---------------------------------------------------------------------------

fn print_linalg() {
    println!(
        "\
ADVANCED LINEAR ALGEBRA  (help linalg)

Multi-output functions use  [a, b, ...] = f(x)  assignment syntax.

Performance — optional BLAS build
    Default build: pure-Rust arithmetic (no system dependencies).
    Fast enough for matrices up to a few hundred rows.

    For larger work, rebuild with OpenBLAS for a significant speedup:
        cargo build --release --features blas          (dynamic link)
        cargo build --release --features blas-static   (static link, no runtime dep)

    Requires libopenblas-dev (Linux) or brew install openblas (macOS).
    The API is identical in both builds.

─── QR decomposition ──────────────────────────────────────────────────────────

  [Q, R] = qr(A)      A = Q * R
                      Q: m×m orthogonal (full Q); R: m×n upper triangular
  R = qr(A)           single-output: returns R only

  Applications: orthogonalisation, least-squares systems.

  Thin (economy) QR from the full factors:
    [Q, R] = qr(A)         % A is m×n, m > n
    Q1 = Q(:, 1:n);        % m×n — orthonormal columns
    R1 = R(1:n, :);        % n×n — square upper triangular
    c  = R1 \\ (Q1' * b)   % least-squares solution

  Verify:  norm(Q' * Q - eye(m), 'fro')  ≈  0
           norm(Q * R - A, 'fro')        ≈  0

─── LU decomposition ──────────────────────────────────────────────────────────

  [L, U, P] = lu(A)   PA = LU  (partial pivoting)
                      L: unit lower triangular; U: upper triangular; P: permutation
  U = lu(A)           single-output: returns U only

  Used internally by backslash (\\). Solving A*x = b:
    [L, U, P] = lu(A)
    x = U \\ (L \\ (P * b))

  Verify:  norm(P * A - L * U, 'fro')  ≈  0

─── Cholesky decomposition ────────────────────────────────────────────────────

  R = chol(A)         A = R' * R  (A must be symmetric positive definite)
                      R: upper triangular

  Faster than LU for SPD systems; also verifies that A is SPD.
  Returns an error if A is not positive definite.

  Example — solve A*x = b for SPD A:
    R = chol(A)
    x = R \\ (R' \\ b)   % back-substitution: cheaper than inv(A)*b

─── SVD — singular value decomposition ────────────────────────────────────────

  s = svd(A)             singular values as a column vector (descending)
  [U, S, V] = svd(A)     full SVD: U (m×m), S (m×n diagonal), V (n×n)
                         A = U * S * V'
  [U, S, V] = svd(A, 'econ')  economy SVD: U (m×k), S (k×k), V (n×k)
                              where k = min(m, n)

  Applications: rank determination, norms, pseudoinverse, low-rank approx.

  Rank-1 approximation (best rank-1 matrix in Frobenius sense):
    [U, S, V] = svd(A)
    A1 = S(1,1) * (U(:,1) * V(:,1)')

  Verify:  norm(U * S * V' - A, 'fro')  ≈  0
           norm(U' * U - eye(m), 'fro')  ≈  0

─── Eigendecomposition ────────────────────────────────────────────────────────

  d = eig(A)             eigenvalues as a column vector
  [V, D] = eig(A)        V: eigenvectors (columns), D: diagonal eigenvalue matrix
                         A * V = V * D  (so A * V(:,k) = D(k,k) * V(:,k))

  Symmetric matrices always yield real eigenvalues (returned as a Matrix).
  Non-symmetric matrices may have complex conjugate pairs: eig(A) then
  returns a ComplexMatrix N×1 column vector.
  [V,D] = eig(A) is available for real eigenvalues only.

  Example (real):
    [V, D] = eig([4 1; 1 3])
    % D(1,1) = 2.382..., D(2,2) = 4.618...
    % V columns are the corresponding eigenvectors

  Example (complex — stability check):
    d = eig([0, -1; 1, 0])       % rotation matrix → [0+1i; 0-1i]
    d = eig([0, 1; -4, -1.2])    % damped oscillator → complex pair
    fprintf('stable: %d\n', all(real(d) < 0))

─── Matrix properties ─────────────────────────────────────────────────────────

  rank(A)       numerical rank (count of singular values > eps * s_max * max(m,n))
  null(A)       orthonormal basis for null space  (columns are right null vectors)
  orth(A)       orthonormal basis for column space  (via left singular vectors)
  cond(A)       condition number:  sigma_max / sigma_min  (Inf for singular)
  pinv(A)       Moore-Penrose pseudoinverse:  A * pinv(A) * A == A

  rank([1 2 3; 4 5 6; 7 8 9])          →  2
  norm(null([1 2; 2 4]) .* [1 2; 2 4]) →  0   (null vector satisfies A*x=0)
  cond(eye(4))                          →  1   (identity: perfectly conditioned)
  norm(A * pinv(A) * A - A, 'fro')     →  ~0   (pseudoinverse identity)

─── Matrix norms ──────────────────────────────────────────────────────────────

  norm(v)        vector: Euclidean (L2) norm  — unchanged
  norm(v, p)     vector: Lp norm
  norm(A)        matrix: spectral 2-norm (largest singular value)
  norm(A, 'fro') Frobenius norm: sqrt(sum of squared elements)
  norm(A, 1)     max column-sum norm
  norm(A, inf)   max row-sum norm

  norm([3 4])           →  5         (L2 vector norm)
  norm([1 2; 3 4])      →  5.4772    (spectral = largest sv)
  norm([1 2; 3 4],'fro')→  5.4772    (Frobenius ≈ spectral here)
  norm([1 2; 3 4], 1)   →  6         (max column sum: max(1+3, 2+4))
  norm([1 2; 3 4], inf) →  7         (max row sum: max(1+2, 3+4))

─── Tip: unary-minus in matrix literals ───────────────────────────────────────

  A space before a minus sign inside [...] can be parsed as subtraction.
  Use commas to separate elements when any element starts with '-':

    A = [2, 1, -1; -3, -1, 2]   % safe: commas disambiguate
    A = [2 1 -1; ...]            % risky: '1 -1' = 1-1 = 0

See also: help matrices  help vectors  help functions
Example:  ccalc examples/linear_algebra.calc"
    );
}

// ---------------------------------------------------------------------------
// help testing
// ---------------------------------------------------------------------------

fn print_testing() {
    println!(
        "\
TESTING — assert built-ins

assert(cond)
    Pass if cond is truthy (nonzero scalar, nonempty string, …).
    Throw an error if cond is falsy (0, NaN, empty).

    assert(1)             % passes
    assert(pi > 3)        % passes
    assert(0)             % error: assertion failed
    assert(nan)           % error: assertion failed (NaN is always falsy)

assert(expected, actual)
    Pass if expected == actual (exact element-wise equality).
    Works on scalars, vectors, and matrices of the same shape.

    assert(4, 2 + 2)                % passes
    assert([1 4 9], [1 2 3].^2)    % passes
    assert(5, 6)                    % error: expected 5, got 6

assert(expected, actual, tol)
    Pass if |expected - actual| <= tol for every element.
    Useful for floating-point results.

    assert(0.3333, 1/3, 1e-4)      % passes
    assert(2, exp(1), 0.5)         % passes  (|2 - 2.718...| = 0.718 > 0.5? no)
    assert(1, 2, 0.1)              % error: |1 - 2| = 1 > 0.1

Practical pattern — doc comment + assert as a test harness
    % Returns the nth triangular number T(n) = n*(n+1)/2.
    function t = tri(n)
      t = n * (n + 1) / 2;
    end

    assert(0,  tri(0))
    assert(1,  tri(1))
    assert(10, tri(4))
    assert(55, tri(10))

See also: help errors  help userfuncs
Example:  ccalc examples/repl_tooling.calc"
    );
}

// ---------------------------------------------------------------------------
// help csv
// ---------------------------------------------------------------------------

fn print_csv() {
    println!(
        "\
CSV — Tables and Matrices

readmatrix — read a numeric CSV file, return Matrix
    A = readmatrix(path)
    A = readmatrix(path, 'Delimiter', d)

  - Auto-detects delimiter: comma (RFC 4180-aware) → tab → whitespace.
  - If the first row contains non-numeric text it is skipped as a header.
    A purely numeric first row is treated as data (never auto-skipped).
  - Empty cells become NaN (unlike dlmread which uses 0.0).

  Example:
    % sensor.csv:  time_s,voltage_V,current_A
    %              0.0,3.300,0.012
    %              0.5,3.281,0.015
    A = readmatrix('sensor.csv')   % header skipped; returns 2×3 Matrix
    A = readmatrix('data.tsv', 'Delimiter', '\\t')

readtable — read a CSV with header row, return Struct of columns
    T = readtable(path)
    T = readtable(path, 'Delimiter', d)

  - First row is always the header (required).
  - Column type inference:
      all cells parseable as numbers → Matrix N×1 column vector
      any non-numeric cell          → Cell of Str
  - Header names are sanitised (non-alphanumeric → _, leading digit → x prefix,
    empty → x{{N}}). Duplicate names get _1 _2 … suffixes.
  - RFC 4180 quoted fields: commas and double-quotes inside \"...\" fields
    are preserved; \"\" inside a quoted field encodes a literal \".

  Example:
    T = readtable('grades.csv')
    scores = T.score          % Matrix N×1
    names  = T.name           % Cell of Str
    nm = names{{1}}             % individual string

writetable — write a Struct to a CSV file with a header row
    writetable(T, path)
    writetable(T, path, 'Delimiter', d)

  - Accepted column types: Matrix (N×1), Cell, Scalar, Str/StringObj.
  - All columns must have the same number of rows.
  - Cells containing the delimiter, \", or newline are automatically
    quoted per RFC 4180; embedded \" is doubled.

  Example:
    T.name  = {{'Alice', 'Bob', 'Carol'}};
    T.score = [91; 85; 78];
    writetable(T, 'out.csv')
    % → out.csv:  name,score
    %             Alice,91
    %             Bob,85
    %             Carol,78

Roundtrip example:
    T  = readtable('in.csv');
    %   ... analyse T ...
    writetable(T, 'out.csv');

Differences from dlmread / dlmwrite
    dlmread    numeric only; empty cells → 0.0; no header handling
    readmatrix numeric only; empty cells → NaN; auto-skips non-numeric header
    readtable  mixed types;  first row always = headers; returns Struct

See also: help files  help structs  help cells
Example:  cargo run --  examples/csv/csv.calc"
    );
}

// ---------------------------------------------------------------------------
// help json
// ---------------------------------------------------------------------------

fn print_json() {
    println!(
        "\
JSON  (requires: cargo build --features json)

Without the feature flag, calling either built-in returns an informative
error message. Both names always appear in tab completion.

jsondecode — parse a JSON string and return a ccalc Value
    val = jsondecode(str)

  Type mapping:
    JSON object  {{…}}          → Struct  (fields in insertion order)
    all-numeric array [n,…]     → Matrix 1×N row vector
    array with nulls only       → Matrix (null → NaN)
    mixed array  [n,\"s\",…]   → Cell
    string                      → Str
    number                      → Scalar
    true / false                → Scalar (1.0 / 0.0)
    null                        → Scalar(NaN)

  Example:
    s = jsondecode('{{\"x\":1,\"y\":[1,2,3]}}')
    s.x          % → 1
    s.y          % → [1  2  3]  (1×3 Matrix)

    nums = jsondecode('[10, 20, 30]')    % → [10  20  30]  (Matrix)
    mix  = jsondecode('[1, \"two\"]')    % → {{1, 'two'}}  (Cell)

jsonencode — encode a ccalc Value to a compact JSON string (Str)
    str = jsonencode(val)

  Type mapping:
    Struct            → object {{…}}         (insertion order preserved)
    Matrix 1×N        → flat array […]
    Matrix M×N        → array of row arrays [[…],[…],…]
    Cell              → array […]
    StructArray       → array of objects [{{…}},…]
    Scalar(NaN)       → null
    Scalar(finite)    → number
    Str / StringObj   → string

  Errors for: Complex, Lambda, Function, Void, Scalar(±Inf).

  Example:
    s.name   = 'Alice';
    s.scores = [88, 92, 75];
    jsonencode(s)     % → '{{\"name\":\"Alice\",\"scores\":[88.0,92.0,75.0]}}'

Reading JSON from a file (fgetl reads one line at a time):
    fid = fopen('data.json', 'r');
    raw = fgetl(fid);
    fclose(fid);
    data = jsondecode(raw);

Build with JSON support:
    cargo build --release --features json

See also: help files  help structs  help cells
Example:  cargo run --features json -- examples/json/json.calc"
    );
}

// ---------------------------------------------------------------------------
// help matfile
// ---------------------------------------------------------------------------

fn print_matfile() {
    println!(
        "\
MAT FILES  (requires: cargo build --features mat)

Without the feature flag, calling load('*.mat') returns an informative
error message. The 'load' name always appears in tab completion.

load — read a MATLAB Level 5/7 .mat file

  Assignment form — returns a Struct of all variables:
    data = load('results.mat')
    data.score        % scalar variable
    data.readings     % matrix variable
    data.label        % char-array variable
    data.sensor.gain  % nested struct field

  Bare form — merges all variables into the current workspace:
    load('results.mat')
    score             % now a direct variable
    readings          % now a direct variable

Type mapping:
    double (1×1)        → Scalar
    double (M×N)        → Matrix  (column-major converted to row-major)
    char array          → Str
    struct              → Struct
    struct array (1)    → Struct  (unwrapped)
    struct array (N)    → StructArray
    cell array          → Cell
    [] / null           → Scalar(NaN)

  Complex and sparse matrices produce an error (not yet supported).

save — writing .mat files is not yet supported:
    save('out.mat')              % error: not yet supported
    save('out.mat', 'x', 'y')   % error: not yet supported

  Use 'save' without a .mat extension (or 'ws') to persist the workspace
  in ccalc's native TOML format.

Build with MAT support:
    cargo build --release --features mat

See also: help files  help structs  help cells
Example:  cargo run --features mat -- examples/mat/mat.calc"
    );
}

// ---------------------------------------------------------------------------
// help regex
// ---------------------------------------------------------------------------

fn print_regex() {
    println!(
        "\
REGULAR EXPRESSIONS  (requires: cargo build --features regex)

Without the feature flag, calling regexp/regexpi/regexprep returns an
informative error. All three names always appear in tab completion.

regexp(s, pat)               1-based index of first match; [] if no match
regexp(s, pat, 'match')      cell array of all matched substrings

regexpi(s, pat)              case-insensitive regexp (prepends (?i))
regexpi(s, pat, 'match')     case-insensitive, return all matches

regexprep(s, pat, rep)       replace all matches with the literal string rep

IMPORTANT: the replacement string in regexprep is always treated as a
literal. Capture-group references ($1, ${{name}}) are NOT expanded.

  regexprep('2024-03-15', '-', '/')     → '2024/03/15'
  regexprep('foo  bar',   '\\s+', '_')  → 'foo_bar'
  regexprep('a', 'a', '$1')             → '$1'   (not expanded)

Pattern examples
  '\\d+'               one or more digits
  '[A-Z][a-z]+'        capital word
  '\\d{{4}}-\\d{{2}}-\\d{{2}}'  ISO 8601 date
  '[0-9]+\\.?[0-9]*'  integer or decimal number
  '\\s+'               one or more whitespace chars

No-match behaviour
  regexp('abc', '\\d+')     → []    (empty 0×0 matrix, displays as [])
  regexp('abc 5', '\\d+')   → 5    (1-based character index)

match form — returns a cell array
  regexp('a1 b2 c3', '\\d', 'match')   → {{'1','2','3'}}

Case-insensitive
  regexpi('Hello', 'hello')             → 1   (match at column 1)
  regexpi('Hello World', 'world', 'match') → {{'World'}}

Build with regex support:
    cargo build --features regex
    cargo build --release --features regex

See also: help strings  (contains, startsWith, endsWith, strjoin, strrep, ...)
Example:  cargo run --features regex -- examples/string_regex.calc"
    );
}

// ---------------------------------------------------------------------------
// help datetime
// ---------------------------------------------------------------------------

fn print_datetime() {
    println!(
        "\
DATETIME & DURATION

UTC datetime and duration values are first-class types.
All timestamps are stored as seconds since the Unix epoch (1970-01-01 00:00:00 UTC).

Value types
    DateTime(f64)          single UTC timestamp  (NaN = NaT)
    Duration(f64)          elapsed time in seconds (fractional)
    DateTimeArray(Vec)     ordered sequence of UTC timestamps (N×1)
    DurationArray(Vec)     ordered sequence of durations (N×1)

    NaT  — Not-a-Time constant, analogous to NaN for scalars.

Datetime constructors
    datetime('2024-06-01')                 from ISO 8601 date string
    datetime('2024-06-01 09:30:00')        date + time string
    datetime(y, m, d)                      from year, month, day
    datetime(y, m, d, H, M, S)            from six components
    datetime(ts, 'ConvertFrom', 'posixtime')  from Unix timestamp

Duration constructors  (all return a Duration value)
    duration(H, M, S)       from hours, minutes, seconds
    hours(n)                n hours
    minutes(n)              n minutes
    seconds(n)              n seconds
    days(n)                 n days
    milliseconds(n)         n milliseconds
    years(n)                n years (365.2425 days each)

    Durations display as HH:MM:SS (e.g. 01:30:00, 02:00:00.500).

Arithmetic
    Expression                Result type
    datetime + duration    →  DateTime
    datetime - duration    →  DateTime
    datetime - datetime    →  Duration
    duration + duration    →  Duration
    duration * scalar      →  Duration
    scalar * duration      →  Duration

    Example:
      t  = datetime(2024, 1, 1);
      t2 = t + hours(3);             % 2024-01-01 03:00:00
      t3 = t2 - minutes(30);         % 2024-01-01 02:30:00
      elapsed = t2 - t;              % Duration: 03:00:00
      fprintf('%g minutes\\n', minutes(elapsed))   % 180

Component extractors  (DateTime → Scalar or DateTimeArray → column vector)
    year(dt)    month(dt)    day(dt)
    hour(dt)    minute(dt)   second(dt)

Duration extractors  (Duration → Scalar, in the named unit)
    hours(d)          total hours
    minutes(d)        total minutes
    seconds(d)        total seconds
    days(d)           total days
    milliseconds(d)   total milliseconds

    Note: hours/minutes/seconds/days/milliseconds are dual-purpose — when called
    with a Duration they extract (return a number); when called with a number they
    construct a new Duration.

Predicates
    isdatetime(x)    1 if x is DateTime or DateTimeArray, else 0
    isduration(x)    1 if x is Duration or DurationArray, else 0
    isnat(x)         1 if x is NaT (DateTime(NaN)), else 0
                     returns 0 (not an error) for non-datetime arguments

Formatting and conversion
    datestr(dt)                 default: '15-Jun-2024 00:00:00'
    datestr(dt, 'yyyy/MM/dd')   custom format pattern
    datevec(dt)                 [y m d H M S] as a 1×6 row vector
    datenum(dt)                 MATLAB serial date number (days since 0000-01-00)
    datenum(y, m, d)            serial date from components
    posixtime(dt)               Unix timestamp as a scalar (seconds)

    datestr pattern tokens:
      yyyy  4-digit year        MMM   3-letter month (Jan, Feb, …)
      MM    2-digit month       dd    2-digit day
      HH    2-digit hour (24h)  mm    2-digit minute
      ss    2-digit second      SSS   3-digit milliseconds

fprintf / sprintf with %s
    fprintf('%s\\n', dt)         2024-06-01 00:00:00  (ISO format)
    fprintf('%s\\n', dur)        01:30:00             (HH:MM:SS)
    s = sprintf('elapsed: %s', elapsed);

Array operations
    Build arrays with matrix literal syntax (all elements same type):
      dates = [datetime(2024,1,1); datetime(2024,1,2); datetime(2024,1,3)]
      durs  = [hours(1); hours(2); hours(3)]

    diff(arr) — successive differences
      DateTimeArray → DurationArray   (each element = next - prev)
      DurationArray → DurationArray

    Example:
      t  = [datetime(2024,9,1); datetime(2024,10,1); datetime(2024,11,1)];
      d  = diff(t);                    % [30 days; 31 days]
      fprintf('%g days\\n', days(d))

Practical example — project timeline:
    kickoff = datetime(2024, 9,  2);
    release = datetime(2025, 1, 20);
    total   = days(release - kickoff);
    fprintf('Project spans %g days\\n', total)
    fprintf('Kickoff: %s\\n', datestr(kickoff, 'dd-MMM-yyyy'))

See also: help format  help strings
Example:  ccalc examples/datetime.calc"
    );
}

// ---------------------------------------------------------------------------
// help setops  (Phase 23 — Matrix utilities and set operations)
// ---------------------------------------------------------------------------

fn print_setops() {
    println!(
        "\
MATRIX UTILITIES AND SET OPERATIONS  (help setops)

── Triangular extraction ────────────────────────────────────────────────────────
    triu(A)        upper triangular (zeros below main diagonal)
    triu(A, k)     upper triangular offset by k  (k>0: above main, k<0: include sub)
    tril(A)        lower triangular (zeros above main diagonal)
    tril(A, k)     lower triangular offset by k

    A = [1 2 3; 4 5 6; 7 8 9];
    triu(A)        % [1 2 3; 0 5 6; 0 0 9]
    triu(A, 1)     % [0 2 3; 0 0 6; 0 0 0]
    tril(A)        % [1 0 0; 4 5 0; 7 8 9]
    tril(A, -1)    % [0 0 0; 4 0 0; 7 8 0]

── Tiling and Kronecker product ─────────────────────────────────────────────────
    repmat(A, m, n)    tile A as m×n block grid → (m*rows)×(n*cols) result
    kron(A, B)         Kronecker (tensor) product

    repmat([1 2; 3 4], 2, 3)           % 4×6 block matrix
    kron([1 0; 0 1], [1 2; 3 4])       % 4×4 block-diagonal identity scaling

── Vector products ───────────────────────────────────────────────────────────────
    cross(a, b)    cross product of two length-3 vectors; result orientation = a
    dot(a, b)      inner (dot) product; returns scalar

    cross([1 0 0], [0 1 0])    % [0 0 1]
    cross([1 2 3], [4 5 6])    % [-3 6 -3]
    dot([1 2 3], [4 5 6])      % 32

── Set operations (sorted, unique; NaN follows IEEE: never a member) ─────────────
    intersect(a, b)    elements in both a and b
    union(a, b)        all unique elements from a and b
    setdiff(a, b)      elements of a not in b
    ismember(x, v)     1 if x is in v; element-wise for vector x

    intersect([1 3 5 7], [3 5 9])    % [3 5]
    union([1 3 5], [3 5 7])          % [1 3 5 7]
    setdiff([1 2 3 4 5], [2 4])      % [1 3 5]
    ismember(3, [1 2 3 4])           % 1
    ismember([1 6 3], [1 2 3 4])     % [1 0 1]

── Index conversion ──────────────────────────────────────────────────────────────
    sub2ind(sz, r, c)      row/col subscripts → linear index (1-based, column-major)
    ind2sub(sz, idx)       linear index → [row; col] returned as tuple

    sub2ind([3 4], 2, 3)           % 8
    sub2ind([3 4], [1 2], [1 3])   % [1 8]
    [r, c] = ind2sub([3 4], 8)     % r=2, c=3
    [r, c] = ind2sub([3 4], [1 7]) % r=[1 1], c=[1 3]

── Element repetition ────────────────────────────────────────────────────────────
    repelem(v, n)       repeat each element of v exactly n times
    repelem(v, nv)      repeat v(i) by nv(i) times (per-element counts)
    repelem(A, m, n)    repeat each element of A in m rows × n cols

    repelem([1 2 3], 3)          % [1 1 1 2 2 2 3 3 3]
    repelem([1 2 3], [2 1 3])    % [1 1 2 3 3 3]
    repelem([1 2; 3 4], 2, 3)    % 4×6 element-wise tiling

See also: help matrices  help vectors  help linalg"
    );
}

// help poly  (Phase 24 — Polynomial operations and interpolation)
// ---------------------------------------------------------------------------

fn print_poly() {
    println!(
        "\
POLYNOMIAL OPERATIONS AND INTERPOLATION  (help poly)

Polynomials are row vectors of coefficients in descending degree order.
  p(x) = x² − 3x + 2  →  [1 -3 2]   (p(1)=0, p(2)=0)

── Evaluation ────────────────────────────────────────────────────────────────
    polyval(p, x)     evaluate polynomial p at scalar or vector x (Horner)

    p = [1 0 1];          % x² + 1
    polyval(p, 0)         % 1
    polyval(p, [0 1 2])   % [1 2 5]

── Fitting ───────────────────────────────────────────────────────────────────
    polyfit(x, y, n)  least-squares polynomial of degree n through (x, y)

    x = [0 1 2 3 4];
    y = [1 2 5 10 17];
    p = polyfit(x, y, 2)  % → [1.0 0.0 1.0]  (x² + 1)

── Roots and monic polynomial ────────────────────────────────────────────────
    roots(p)      roots of p; column vector (Matrix if all real, Cell if complex)
    poly(r)       monic polynomial with given roots; poly(A) char. polynomial

    p = [1 0 1];
    roots(p)           % {{0+1i; 0-1i}}  (Cell — complex roots)
    poly([1 2 3])      % [1 -6 11 -6]  (x-1)(x-2)(x-3)
    poly([1 2; 0 3])   % [1 -4 3]      characteristic polynomial

── Convolution and deconvolution ─────────────────────────────────────────────
    conv(a, b)        polynomial multiplication / discrete convolution
    deconv(c, b)      polynomial long division; returns [q, r] tuple

    conv([1 2 3], [1 1])         % [1 3 5 3]
    [q, r] = deconv([1 3 5 3], [1 1])   % q=[1 2 3], r=[0 0 0 0]
    % invariant: conv(b, q) + r == c

── Interpolation ─────────────────────────────────────────────────────────────
    interp1(x, y, xi)            piecewise linear interpolation at xi
    interp1(x, y, xi, method)   choose method: linear nearest previous next

    x = [0 1 2 3];
    y = [0 1 4 9];
    interp1(x, y, 1.5)                   % 2.5  (linear)
    interp1(x, y, [0.5 1.5 2.5])        % [0.5 2.5 6.5]
    interp1(x, y, 1.5, 'nearest')        % 1    (snap to closest knot)
    interp1(x, y, 1.5, 'previous')       % 1    (left step / ZOH)
    interp1(x, y, 1.5, 'next')           % 4    (right step)
    interp1(x, y, 99)                    % NaN  (out of range)

See also: help linalg  help vectors  help matrices"
    );
}

// help eval  (Phase 25 — Dynamic evaluation and timing)
// ---------------------------------------------------------------------------

fn print_eval() {
    println!(
        "\
DYNAMIC EVALUATION AND TIMING  (help eval)

── eval — string execution ───────────────────────────────────────────────────
    eval(str)            execute str as code in the current workspace
    eval(str, catch_str) execute str; if it errors, execute catch_str instead

Variables defined inside eval() persist in the caller's scope:

    eval('x = sqrt(2)')        % x is now defined
    x                          % → 1.4142…

    % Dynamic variable naming:
    for k = 1:3
      eval(sprintf('v%d = k*k', k))   % creates v1, v2, v3
    end
    v1    % 1
    v2    % 4
    v3    % 9

    % Two-argument form — catch errors:
    eval('1/0', 'disp(''caught error'')')

    % eval() in expression context captures ans of the inner execution:
    y = eval('2 + 2')    % y = 4  (env mutations inside do not persist)

Notes:
  - Nesting depth is capped at 64 (shared with run/source).
  - eval() called as a standalone statement allows env mutations.
  - In expression context (y = eval(...)) mutations inside are discarded.

── tic / toc — elapsed time ──────────────────────────────────────────────────
    tic              start (or restart) the timer
    toc              read elapsed seconds since last tic

    tic
    x = rand(500) * rand(500);
    t = toc              % → elapsed seconds, e.g. 0.0042

    tic
    for k = 1:1000
      x = k^2;
    end
    fprintf('loop: %.4f s\\n', toc)

Notes:
  - tic returns Void (no display).
  - toc returns a scalar; multiple toc calls after one tic are valid.
  - Calling toc before tic is an error.
  - Both tic and toc can be written without parentheses: tic  toc

See also: help control  help script  help errors"
    );
}

// ---------------------------------------------------------------------------
// help fft
// ---------------------------------------------------------------------------

fn print_fft() {
    println!(
        "\
FFT & SIGNAL PROCESSING  (help fft)

fft and ifft require the fft feature flag at build time:
    cargo build --release --features fft
fftshift, ifftshift, and fftfreq are always available.

── Forward FFT ───────────────────────────────────────────────────────────────
    fft(x)      Discrete Fourier Transform of real vector x.
                Returns a ComplexMatrix (1×N row vector).
                X(1) is the DC component (= sum of all input samples).

    fft(x, n)   Zero-pad x to length n before transform (or truncate if n <
                length(x)). Use to control frequency resolution.

    x = [1 2 3 4];
    X = fft(x)
    % X(1) = 10 + 0i     (DC: sum of all samples)
    % X(2) = -2 + 2i
    % X(3) = -2 + 0i
    % X(4) = -2 - 2i

    % Access real and imaginary parts of a bin:
    re = real(X(2));   im = imag(X(2))
    % Element-wise magnitude: abs(X)  →  real Matrix

── Inverse FFT ───────────────────────────────────────────────────────────────
    ifft(X)     Inverse DFT, normalised by 1/N. Accepts a ComplexMatrix.
                When all imaginary parts < 1e-12, returns a real matrix.

    y = ifft(fft([1 2 3 4]))   % → [1 2 3 4]  (real matrix)

── DC shift ──────────────────────────────────────────────────────────────────
    fftshift(x)    Circular shift by floor(N/2) so the DC bin moves to the
                   centre. Works on row vectors, column vectors, and 2-D matrices.
    ifftshift(x)   Inverse: shift by ceil(N/2). Undoes fftshift.

    fftshift([1 2 3 4 5 6])      % → [4 5 6 1 2 3]
    ifftshift([4 5 6 1 2 3])     % → [1 2 3 4 5 6]
    fftshift([1 2 3 4 5])        % → [4 5 1 2 3]   (odd length)

── Frequency axis ────────────────────────────────────────────────────────────
    fftfreq(n, d)   1×n row vector of DFT sample frequencies for n points
                    with sample spacing d seconds (d = 1/fs).
                    Matches NumPy / MATLAB convention.

    n = 8; fs = 1000;
    f = fftfreq(n, 1/fs)
    % → [0 125 250 375 -500 -375 -250 -125]  Hz

    fftshift(f)
    % → [-500 -375 -250 -125 0 125 250 375]  Hz  (centred spectrum)

── Power spectrum example ────────────────────────────────────────────────────
    % Two-tone signal: 10 Hz + 25 Hz, fs = 100 Hz, n = 100 samples.
    % Both tones land on exact bins (resolution = 1 Hz).

    n = 100; fs = 100;
    t = (0:n-1) / fs;
    s = sin(2*pi*10*t) + 0.5*sin(2*pi*25*t);
    S = fft(s);

    % abs() on ComplexMatrix returns a real Matrix of element-wise magnitudes
    mag = abs(S);

    % 10 Hz → bin 11, 25 Hz → bin 26  (1-based; |FFT| = amplitude * n/2)
    fprintf('10 Hz: |S| = %.2f  (expect %.2f)\\n', mag(11), n/2)
    fprintf('25 Hz: |S| = %.2f  (expect %.2f)\\n', mag(26), 0.5*n/2)

See also: help complex  help vectors  examples/fft_demo.calc"
    );
}

fn print_plot() {
    println!(
        "\
PLOT  (help plot)

Two rendering tiers; both use the same annotation API.

── Feature flags ─────────────────────────────────────────────────────────────
    --features plot       ASCII Braille chart printed to terminal (textplots)
    --features plot-svg   SVG + PNG file export (plotters, 800×600 px)
    --features plot-all   both tiers

── Chart types ─────────────────────────────────────────────────────────────
    plot(y)               line chart; x inferred as 1:numel(y)
    plot(x, y)            line chart with explicit x
    plot(x, M)            multi-series: each row of M is one series (SVG/PNG)
    scatter(y)            point cloud; x inferred
    scatter(x, y)         point cloud with explicit x
    bar(y)                vertical bar chart; x inferred
    bar(x, y)             bar chart with explicit x positions
    stem(y)               discrete sequence: line from y=0 to tip + marker
    stem(x, y)            stem with explicit x
    stairs(y)             piecewise-constant step function; x inferred
    stairs(x, y)          stairs with explicit x
    hist(v)               histogram, Sturges bins (max(1, round(sqrt(n))))
    hist(v, n)            histogram with n uniform bins
    hist(v, edges)        histogram with caller-supplied edge vector
    loglog(x, y)          log10 on both axes; non-positive values excluded
    semilogx(x, y)        log10 x-axis, linear y
    semilogy(x, y)        linear x-axis, log10 y

── 3D charts ────────────────────────────────────────────────────────────────
    plot3(x, y, z)        3D line; ASCII uses orthographic projection
    scatter3(x, y, z)     3D point cloud; ASCII uses orthographic projection

    ASCII tier:  projects (x,y,z) with az=-37.5°, el=30° (MATLAB defaults),
                 renders with textplots; x/y/z labels printed as footer lines.
    File tier:   plotters build_cartesian_3d + LineSeries / Circle elements.

── False-colour images ──────────────────────────────────────────────────────
    imagesc(Z)               render matrix Z as a false-colour image (ASCII)
    imagesc(Z, 'f.svg')      save false-colour image to SVG (requires plot-svg)
    imagesc(Z, 'f.png')      save to PNG
    imagesc(Z, 'f.png', W, H) save at custom size W×H pixels
    colormap('name')         set active colormap (consumed by next imagesc)
    colorbar()               append colour-scale legend strip (file export only)

    Supported colormaps:  viridis (default)  inferno  magma  plasma
                          hot  cool  jet  gray

── 3D surface plots ─────────────────────────────────────────────────────────
    meshgrid(x, y)            generate coordinate matrices X (M×N) and Y (M×N)
    meshgrid(x)               square N×N grid — x used for both axes
    [X, Y] = meshgrid(x, y)  multi-output form (standard usage)
    surf(X, Y, Z)             colored 3D surface (ASCII elevation silhouette)
    surf(X, Y, Z, 'f.svg')   surf to SVG file (requires plot-svg)
    surf(X, Y, Z, 'f.png')   surf to PNG file
    mesh(X, Y, Z)             wireframe surface (same as surf in ASCII mode)
    mesh(X, Y, Z, 'f.svg')   mesh to SVG file
    mesh(X, Y, Z, 'f.png')   mesh to PNG file

    X, Y, Z must have the same dimensions (M×N from meshgrid).  In SVG/PNG
    mode surf draws row + column grid lines; mesh draws row lines only.
    colormap() applies to surf/mesh file output.

── Filled polygons and areas ─────────────────────────────────────────────────
    fill(x, y)                   filled polygon (vertices in x/y)
    fill(x, y, 'r')              filled polygon with style string
    fill(x, y, 'out.svg')        save filled polygon to SVG/PNG
    fill(x, y, 'b', 'out.svg')  style + file (style before path)
    area(x, y)                   filled area under curve (baseline y=0)
    area(x, y, 'g--')            area with style string
    area(x, y, 'out.svg')        save area chart to SVG/PNG

    ASCII tier:  bounding-box density grid using ░ characters with
                 ray-casting point-in-polygon (even-odd rule) test.
    File tier:   plotters Polygon element at 40% opacity + full-opacity outline.

── Polar plots ──────────────────────────────────────────────────────────────
    polar(theta, r)              polar chart; theta in radians
    polar(theta, r, 'out.svg')   save polar chart to SVG/PNG

    Coordinate transform: x = r·cos(θ), y = r·sin(θ), then render_line_xy.
    Supports the same file-export and annotation flags as plot().

── Vector field plots ───────────────────────────────────────────────────────
    quiver(x, y, u, v)            plot arrows at (x,y) with direction (u,v)
    quiver(x, y, u, v, 'f.svg')  save vector field to SVG/PNG

    x, y, u, v must all have the same number of elements.  M×N matrices
    (e.g. from meshgrid) are accepted and flattened automatically.

    ASCII tier:  60×20 character grid; arrow direction mapped to one of eight
                 Unicode arrows: → ↗ ↑ ↖ ← ↙ ↓ ↘
    File tier:   shaft drawn as PathElement; arrowhead as a filled Polygon
                 triangle at the tip.  Arrow length is normalised so the
                 longest arrow fills 80% of the minimum grid spacing.

    % Rotational flow  u = -y,  v = x
    [X, Y] = meshgrid(-2:1:2, -2:1:2);
    quiver(X, Y, -Y, X, 'flow.svg')

── Text annotations ─────────────────────────────────────────────────────────
    text(x, y, 'label')   place a text label at data coordinate (x, y)

    Annotations accumulate in FigureState and are flushed (drawn + cleared)
    with the next quiver or savefig call, just like title/xlabel.

    ASCII tier:  annotations are printed below the chart as
                 (x, y): label  lines.
    File tier:   rendered as 12 pt sans-serif text via plotters Text element.

    text(0.0, 0.0, 'origin')
    text(2.0, 2.0, 'tip')
    quiver(x, y, u, v, 'annotated.svg')

── Style strings ────────────────────────────────────────────────────────────
    An optional string argument (before the file path) controls color,
    marker, and line style for plot, scatter, fill, and area.

    Colors (single char):
      'r' red    'g' green    'b' blue    'c' cyan
      'm' magenta  'y' yellow  'k' black  'w' white

    Line styles:
      '-'   solid (default)    '--'  dashed
      '-.'  dash-dot           ':'   dotted

    Marker symbols:
      '.'  point    'o'  circle    'x'  cross    '+'  plus
      '*'  star     's'  square    'd'  diamond  '^'  triangle

    Combinations — color, marker, and linestyle in any order:
      'r--'   red dashed line
      'b.'    blue point markers
      'g-.'   green dash-dot
      'k:'    black dotted
      'c-'    cyan solid

    Style strings affect SVG/PNG output only; ASCII charts are monochrome.

── Multi-panel layout ───────────────────────────────────────────────────────
    subplot(rows, cols, index)   activate panel index (1-based, row-major)
    hold('on')                   start accumulating series without rendering
    hold('off')                  stop accumulating; flush to ASCII if no subplot
    savefig('path.svg')          commit last panel; write all panels to SVG/PNG

    Once subplot or hold('on') is called, all plot calls accumulate instead
    of rendering.  Annotations set after a plot call are collected for the
    current panel and consumed at commit time (next subplot or savefig).
    savefig requires --features plot-svg.

    x = linspace(0, 2*pi, 60);
    subplot(2, 2, 1); title('sin'); plot(x, sin(x));
    subplot(2, 2, 2); title('cos'); plot(x, cos(x));
    subplot(2, 2, 3); bar([1 4 2 5 3]);
    subplot(2, 2, 4); hist(randn(1, 200), 20);
    savefig('out.svg')

Append a file path as the last string argument to save instead of print:
    plot(x, y, 'out.svg')          SVG (requires --features plot-svg)
    plot(x, y, 'out.png')          PNG (requires --features plot-svg)
    plot(x, y, 'ascii')            force ASCII even when plot-svg is active
    hist(v, 20, 'hist.svg')        histogram to file (combined n + path form)
    plot3(x, y, z, 'helix.svg')   3D line to SVG (requires --features plot-svg)
    scatter3(x, y, z, 'pts.png')  3D scatter to PNG

── Annotations ──────────────────────────────────────────────────────────────
    title('text')       chart title
    xlabel('text')      x-axis label
    ylabel('text')      y-axis label
    zlabel('text')      z-axis label (plot3/scatter3; footer in ASCII mode)
    xlim([lo, hi])      override x-axis range
    ylim([lo, hi])      override y-axis range
    zlim([lo, hi])      override z-axis range (plot3/scatter3 file export)
    legend(s1, s2, …)   series labels for multi-series SVG/PNG charts
    grid                toggle grid on/off (default: off)
    grid('on')          enable grid (SVG/PNG only; ASCII ignored)
    grid('off')         disable grid

    Annotations are stored in a thread-local FigureState and consumed (cleared)
    by the next render call.  Set them immediately before the render call:

    title('sin(x)')
    xlabel('x (radians)')
    ylim([-1.2, 1.2])
    grid('on')
    plot(x, sin(x))        % all annotations applied and cleared here

── Examples ─────────────────────────────────────────────────────────────────
    x = linspace(0, 2*pi, 80);
    plot(x, sin(x))

    % Multi-series
    M = [sin(x); cos(x)];
    legend('sin', 'cos')
    plot(x, M, 'trig.svg')

    % Histogram with explicit edges
    hist(randn(1, 500), -3:0.5:3, 'dist.svg')

    % Log-scale
    f = 10 .^ linspace(1, 5, 80);
    G = 1e6 * f .^ (-2);
    loglog(f, G)

    % 3D helix
    t = linspace(0, 4*pi, 120);
    plot3(cos(t), sin(t), t/(4*pi))

    % 3D scatter to file
    scatter3(randn(1,80), randn(1,80), randn(1,80), 'cloud.svg')

    % False-colour image with colormap and colorbar
    Z = reshape(1:64, 8, 8);
    colormap('viridis')
    colorbar()
    imagesc(Z, 'heat.svg')

    % Custom canvas size (one matrix cell = one pixel)
    colormap('inferno')
    imagesc(Z, 'heat.png', 1200, 900)

    % 3D surface
    [X, Y] = meshgrid(-3:0.2:3, -3:0.2:3);
    Z = sin(sqrt(X.^2 + Y.^2));
    colormap('viridis')
    surf(X, Y, Z, 'surface.svg')

    % Wireframe
    [X2, Y2] = meshgrid(-2:0.2:2, -2:0.2:2);
    colormap('jet')
    mesh(X2, Y2, X2.^2 - Y2.^2, 'saddle.svg')

    % 2×2 subplot grid to SVG
    x = linspace(0, 2*pi, 60);
    subplot(2, 2, 1); title('sin'); plot(x, sin(x));
    subplot(2, 2, 2); title('cos'); plot(x, cos(x));
    subplot(2, 2, 3); bar([3 1 4 1 5 9]);
    subplot(2, 2, 4); hist(randn(1, 200), 20);
    savefig('grid.svg')

    % Overlay with hold
    hold('on');
    plot(x, sin(x));
    plot(x, cos(x));
    hold('off')

    % Filled triangle polygon
    fill([0 1 0.5], [0 0 1], 'r', 'triangle.svg')

    % Area under sine curve
    x = linspace(0, 2*pi, 80);
    area(x, sin(x) + 1, 'b', 'sine_area.svg')

    % Polar rose curve  r = |cos(2*theta)|
    theta = linspace(0, 2*pi, 360);
    polar(theta, abs(cos(2*theta)), 'rose.svg')

    % Rotational vector field
    [X, Y] = meshgrid(-2:1:2, -2:1:2);
    title('Rotational flow')
    quiver(X, Y, -Y, X, 'flow.svg')

    % Vector field with text annotations
    text(0.0, 0.0, 'origin')
    text(2.0, 2.0, 'tip')
    quiver([0 1 2], [0 1 2], [1 1 1], [1 1 1], 'annotated.svg')

    % Style strings subplot
    xs = linspace(0, 2*pi, 80);
    s  = sin(xs);
    subplot(2, 4, 1); plot(xs, s, 'r');
    subplot(2, 4, 2); plot(xs, s, 'g');
    subplot(2, 4, 3); plot(xs, s, 'b--');
    subplot(2, 4, 4); plot(xs, s, 'k:');
    savefig('style_demo.svg')

See also: examples/plot_demo.calc               (ASCII demo)
          examples/plot_file/plot_file.calc      (SVG/PNG demo)
          examples/plot_extended.calc            (bar/stem/stairs/hist/loglog)
          examples/plot3_demo.calc               (3D ASCII demo)
          examples/plot3_file/plot3_file.calc    (3D SVG/PNG demo)
          examples/colormap/imagesc_demo.calc    (imagesc/colormap demo)
          examples/surf_demo/surf_demo.calc      (surf — sine wave + Gaussian bell)
          examples/surf_demo/mesh_demo.calc      (mesh — sine wave wireframe + saddle)
          examples/subplot_demo/subplot_demo.calc  (2×2 subplot grid)
          examples/hold_demo/hold_demo.calc        (hold on/off overlay)
          examples/fill_area_polar_demo/fill_area_polar_demo.calc  (fill/area/polar demo)
          examples/quiver_demo/quiver_demo.calc    (quiver — rotational flow field)
          examples/annotations/annotations.calc   (text annotations with quiver)"
    );
}
