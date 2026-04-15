# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.19.0] - 2026-04-15

### Added

- **Phase 13 — Scalar structs** (`Value::Struct(IndexMap<String, Value>)`):
  - Field assignment: `s.x = 42` creates or updates a field; `s.a.b = 5`
    creates nested structs automatically via `set_nested()` in `exec.rs`.
  - Field read: `s.x`, `s.a.b` — `Expr::FieldGet` postfix chain in parser.
  - `struct('k1', v1, 'k2', v2, ...)` constructor; `struct()` returns empty struct.
  - `fieldnames(s)` — cell array of field names in insertion order.
  - `isfield(s, 'name')` — returns 1/0.
  - `rmfield(s, 'name')` — returns new struct without the named field.
  - `isstruct(v)` — returns 1 if value is a struct, 0 otherwise.
  - Display: `[1×1 struct]` inline; `struct with fields: / field: value` full form.
  - Workspace save/load skips structs (same policy as matrices and complex).
  - 19 regression tests added for all struct operations.

## [0.18.0+001] - 2026-04-14

### Fixed

- **`4i` imaginary literal in pipe/file mode** — `z = 3 + 4i` was raising
  "Unexpected token after expression" in pipe and script mode (worked in REPL
  only by coincidence). Root cause: the tokenizer had no `Ni` suffix rule;
  `4i` tokenized as `Number(4)` followed by `Ident("i")`, and `parse_term`
  had no implicit-multiply path for a trailing identifier. Fix: added
  `push_imag_suffix()` in the tokenizer — after any decimal number literal,
  if the very next character is `i` or `j` (not followed by another
  alphanumeric), consume it and emit `Token::Star + Token::Ident("i")`.
  Multi-character identifiers beginning with `i`/`j` (e.g. `inside`) are
  not affected.

- **`B.';` mis-parsed as string start** — in `split_stmts()`, the `'`
  disambiguation check tested whether the preceding character was alphanumeric,
  `)`, `]`, `'` — but not `.`. So `B.'` was parsed as `B.` followed by the
  start of a char-array literal, causing the `;` that followed to be swallowed
  into the non-terminating string. Fix: added `'.'` to the transpose-detection
  character set in `split_stmts`.

- **`...` line continuation not working in pipe/file mode** — `run_pipe` had
  no `cont_buf` logic, so multi-line expressions joined with `...` silently
  failed. Fix: added the same comment-stripping + `cont_buf` continuation
  logic to `run_pipe` that already existed in `run_repl`.

### Added

- **Phase 12.6 — Language polish and small completions** (v0.18.0):
  - **12.6a** Single-line blocks: `if cond; body; end` on one line — REPL and
    pipe mode bypass block buffering for self-contained blocks; `is_single_line_block()`
    detects them via `split_block_line()` checking the last `;`-separated segment.
  - **12.6b** `...` line continuation: REPL buffers via `cont_buf`; scripts use
    `join_line_continuations()` pre-pass in `parse_stmts`; tokenizer drains
    rest of input on `...`.
  - **12.6c** `&` / `|` element-wise logical operators: `Token::Amp`/`Pipe`,
    `Op::ElemAnd`/`ElemOr`, `parse_elem_or`/`parse_elem_and` levels between
    `parse_logical_and` and `parse_comparison`.
  - **12.6d** `xor(a, b)` and `not(a)` built-ins.
  - **12.6e** Lambda display: `LambdaFn` now carries a source string; lambdas
    display as `@(x) x + 1` instead of `@<lambda>`. `expr_to_string()` helper
    reconstructs source text from the AST at parse time.
  - **12.6f** String utilities: `strsplit(s, delim)` / `strsplit(s)`,
    `int2str(x)`, `mat2str(A)`.
  - **12.6g** `.'` non-conjugate transpose: `Token::DotApostrophe`,
    `Expr::PlainTranspose` — transposes without conjugating the imaginary part.
  - **12.6i** `@funcname` function handles (completed in Phase 12.5).
  - **12.6j** Unary `+` (no-op), `**` exponentiation alias (Octave), `,` as
    non-silent statement separator.
  - **`examples/language_polish.calc`** — 10-section annotated demo script.
  - **4 new regression tests**: `test_imag_literal_4i`,
    `test_imag_literal_in_expression`, `test_imag_literal_not_confused_with_ident`,
    `test_split_stmts_dot_apostrophe_not_string`.

## [0.18.0] - 2026-04-14

### Added

- **Phase 12.6 — Language polish and small completions**:
  - **12.6a** Single-line blocks (`if cond; body; end`, `for k=1:3; disp(k); end`, etc.)
  - **12.6b** `...` line continuation in REPL, pipe mode, and scripts
  - **12.6c** `&` / `|` element-wise logical operators (matrix-compatible, no short-circuit)
  - **12.6d** `xor(a, b)` and `not(a)` built-ins
  - **12.6e** Lambda source display: `@(x) x^2 + 1` shown instead of `@<lambda>`
  - **12.6f** New string utilities: `strsplit(s[, delim])`, `int2str(x)`, `mat2str(A)`
  - **12.6g** `.'` non-conjugate transpose (`Token::DotApostrophe`, `Expr::PlainTranspose`)
  - **12.6j** Unary `+` (no-op), `**` exponentiation alias, `,` as non-silent statement separator
  - `examples/language_polish.calc` — 10-section annotated demo

## [0.17.0+005] - 2026-04-14

### Fixed

- **varargin injection bug**: `sum_all(1)` was returning 0 instead of 1. Root cause: the parser
  injected `ans` into empty `f()` calls at the AST level, making `sum_all(1)` and `sum_all()`
  indistinguishable inside `call_user_function`, causing varargin to always be empty for
  pure-varargin functions. Fix: ans-injection moved from parser to eval-time; builtins and lambdas
  still receive `ans` on empty `f()` calls, but user functions (`Value::Function`) receive the raw
  argument list — empty call means no arguments (MATLAB semantics).

### Added

- **Phase 12.5 — Cell arrays**:
  - `Value::Cell(Vec<Value>)` — heterogeneous 1-D cell array container
  - `{e1, e2, e3}` cell literal syntax; `c{i}` brace-indexing (1-based, content access)
  - `c{i} = v` cell assignment with auto-grow on out-of-bounds index (`Stmt::CellSet`)
  - Built-ins: `iscell(v)`, `cell(n)`, `cell(m,n)` constructor
  - `numel`, `length`, `size` extended to handle Cell values
  - `varargin` / `varargout` support in user-defined functions (variadic args via Cell)
  - `case {v1, v2}` multi-value switch cases — matches if any element equals the switch expression
  - `cellfun(f, c)` — applies function to each cell element; returns Matrix when all results are scalar
  - `arrayfun(f, v)` — applies function to each element of a numeric vector
  - `@funcname` function handle syntax (in addition to existing `@(params) body` lambda syntax)
  - `split_stmts()` updated to track brace depth so `;` inside `{...}` is not a statement separator
  - `examples/cell_arrays.calc` — 9-section annotated example
- **`help cells`** — new help topic covering cell arrays, varargin/varargout, cellfun/arrayfun, @funcname

## [0.17.0] - 2026-04-12

### Added

- **Phase 12 — User-defined functions, multiple return values, and lambdas**:

  - **Named functions** — `function [out1, out2] = name(p1, p2) ... end`
    - Single and multiple return values
    - `return` for early exit
    - `nargin` / `nargout` variables injected into the function scope
    - Fully isolated scope — caller's data variables are not visible inside
    - All `Value::Function` and `Value::Lambda` values from the caller's workspace
      are automatically forwarded, enabling recursion and mutual recursion
    - Functions defined in the workspace persist until `clear` is called

  - **Multi-assignment** — `[a, b, c] = f(x)` destructures a multi-return call;
    `~` discards individual outputs: `[~, mx, ~] = stats(v)`

  - **Anonymous functions (lambdas)** — `@(params) expr`
    - Lexical closure: captures the enclosing environment at creation time
    - Passed as arguments to named functions (higher-order functions)
    - Stored in variables: `sq = @(x) x^2; sq(5)` → `25`
    - Functions returning functions: `make_adder(c)` → `@(x) x + c`

  - **`Value::Lambda` / `Value::Function` / `Value::Tuple`** — new `Value` variants
    in `env.rs` supporting the function system

  - **`Token::At`** (`@`) added to the tokenizer; `Stmt::FunctionDef`,
    `Stmt::Return`, `Stmt::MultiAssign` added to the AST

  - **`exec::init()`** — registers the `FnCallHook` that bridges `eval.rs`
    (function dispatch) and `exec.rs` (function execution)

  - **New example** `examples/user_functions.calc` — 10-section demo:
    recursive factorial, GCD, `nargin` optional args, multiple return values,
    `~` output discard, lambda basics, lexical capture, numerical integration
    (midpoint rule), higher-order functions, iterative Fibonacci

### Fixed

- **Recursion in named functions** — isolated function scope now forwards all
  `Function` and `Lambda` values from the caller's environment, so self-recursion
  and mutual recursion work correctly.  The `FnCallHook` signature was extended
  with `caller_env: &Env` to make this possible.

## [0.16.0] - 2026-04-11

### Added

- **Phase 11.5 — Extended control flow and script sourcing**:

  - **`switch / case / otherwise / end`** — no fall-through; first matching case runs and control jumps to `end`
    - Scalar cases: exact `==` comparison
    - String cases: `Str` and `StringObj` are interchangeable
    - `otherwise` is optional; no match without `otherwise` → body silently skipped
    - `break`/`continue` propagate outward to the nearest enclosing loop (switch itself is not a loop)

  - **`do ... until (cond)`** — Octave post-test loop; body always executes at least once
    - Condition tested *after* each iteration; parentheses around the condition are optional
    - `break` exits the loop immediately; `continue` re-tests the condition
    - `until` closes the block in the REPL (depth delta -1) — no separate `end` needed

  - **`run('file')` / `source('file')`** — execute a script file in the current workspace
    - Script variables persist in the caller's scope (MATLAB `run` semantics — shared `Env`)
    - Extension resolution for bare names: `.calc` is tried first (native ccalc format), then `.m` (Octave/MATLAB compatibility)
    - Explicit extensions (`.calc`, `.m`, or any other) are used verbatim
    - Recursive nesting supported up to depth 64 (tracked via thread-local `RUN_DEPTH`)
    - Works in REPL, pipe/script mode, and inside multi-line blocks
    - `source()` is a full alias for `run()` (Octave convention)

  - **`block_depth_delta` extended**: `switch`/`do` → +1; `until` → −1

- **New example** `examples/extended_control_flow.calc` — 8-section demo: exit-code classifier, unit converter, month-to-season switch, power-of-2 ceiling, digit sum, first prime search, Euclidean GCD via `run()`, `source()` alias
- **New helper script** `examples/euclid_helper.calc` — GCD computation sourced by the demo

### Fixed

- `run()` / `source()` now work correctly in pipe and script mode (single-line statements
  previously bypassed `exec_stmts`; a `try_run_source()` helper in `repl.rs` now bridges
  both execution paths)

## [0.15.2] - 2026-04-10

### Added

- **Comment alias `#`** — `#` is now equivalent to `%` as a comment character (Octave-compatible).
  Works in all contexts: full-line, inline, inside `split_stmts`, and block parsing.
- **Logical NOT alias `!`** — `!expr` is now equivalent to `~expr`.
- **Not-equal alias `!=`** — `!=` is now equivalent to `~=`.

## [0.15.1] - 2026-04-10

### Added

- **Phase 11b — Compound assignment operators**:
  - New tokens: `+=`, `-=`, `*=`, `/=`, `++`, `--`
  - `x += e` → `x = x + e`; `x -= e` → `x = x - e`; `x *= e` → `x = x * e`; `x /= e` → `x = x / e`
  - `x++` / `x--` → `x = x + 1` / `x = x - 1` (suffix)
  - `++x` / `--x` → `x = x + 1` / `x = x - 1` (prefix)
  - All forms desugar at parse time into `Stmt::Assign` — no new AST nodes
  - RHS is a full expression: `x *= 2 + 3` → `x = x * (2 + 3)`
  - **Limitation**: `++`/`--` are statement-level only; using them inside a larger expression is not supported

## [0.15.0] - 2026-04-10

### Added

- **Phase 11a — Multi-line input and core control flow**:
  - **REPL block buffering**: incomplete blocks accumulate lines; `Ctrl+C` cancels; continuation prompt `  >> `
  - **`if` / `elseif` / `else` / `end`**: arbitrary nesting; elseif chains
  - **`for var = range; ...; end`**: iterates over columns of a matrix (row vector → scalars, M×N → M×1 columns)
  - **`while cond; ...; end`**: loops while condition is truthy
  - **`break`** — exits innermost enclosing loop
  - **`continue`** — advances to next iteration of innermost enclosing loop
  - **`block_depth_delta(line)`** — public API for tracking block depth per line
  - **`parse_stmts(input)`** — public API for parsing multi-line block strings
  - **`exec_stmts()`** in new `exec.rs` module — separates block execution from parsing/evaluation; avoids circular dependency
  - `split_stmts()` moved from `repl.rs` to `parser.rs` (made public)
  - **New example** `examples/control_flow.calc` — 7-section demo: grade classifier, sum of squares, odd sums, prime sieve, Newton-Raphson, Collatz sequence

## [0.14.0+006] - 2026-04-09

### Added

- **Phase 10.5 — File I/O and filesystem queries**:
  - **`IoContext`** — file descriptor table in `ccalc-engine/src/io.rs`; passed into `eval_with_io()`
  - **`eval_with_io(expr, env, io)`** — new public API; `eval()` unchanged (no I/O)
  - **10.5a — File handles**:
    - `fopen(path, mode)` — open file; modes `'r'` `'w'` `'a'` `'r+'`; returns fd (≥3) or -1 on failure
    - `fclose(fd)` — close by fd; returns 0 or -1
    - `fclose('all')` — close all open handles
    - `fgetl(fd)` — read one line, strip trailing newline; returns -1 at EOF
    - `fgets(fd)` — read one line, keep trailing newline
    - `fprintf(fd, fmt, ...)` — write formatted output to file descriptor; fd 1 = stdout, 2 = stderr
  - **10.5b — Data file I/O**:
    - `dlmread(path)` — read delimiter-separated numeric data (auto-detect `,` / `\t` / whitespace)
    - `dlmread(path, delim)` — explicit delimiter (`','`, `'\t'`)
    - `dlmwrite(path, A)` — write matrix with comma separator
    - `dlmwrite(path, A, delim)` — explicit delimiter
  - **10.5c — Filesystem queries**:
    - `isfile(path)` — 1 if path exists and is a file, else 0
    - `isfolder(path)` — 1 if path exists and is a directory, else 0
    - `pwd()` — current working directory as a char array
    - `exist(name)` — 1 if variable exists in workspace, 2 if a file on disk
    - `exist(name, 'var')` — check workspace only
    - `exist(name, 'file')` — check filesystem only (returns 2 if found, matching MATLAB)
  - **10.5d — Workspace with explicit path**:
    - `save` / `load` — aliases for `ws` / `wl` (default path)
    - `save('path.mat')` — save all workspace variables to named file
    - `save('path.mat', 'x', 'y')` — save specific variables only
    - `load('path.mat')` — load variables from named file into workspace
    - Path argument can be a variable reference (`save(mat_path)`)
    - Workspace format extended: scalars + char arrays + string objects persisted; matrices/complex still skipped
  - **New example** `examples/file_io.calc` — 10-section demo covering all subphases; writes to `.debug/.TESTS/`

## [0.14.0+001] - 2026-04-08

### Added

- **`format` command** — MATLAB-compatible number display modes:
  - `format short` — 5 significant digits (default MATLAB style, e.g. `3.1416`)
  - `format long` — 15 significant digits (e.g. `3.14159265358979`)
  - `format shortE` — always scientific, 4 decimal places (e.g. `3.1416e+00`)
  - `format longE` — always scientific, 14 decimal places
  - `format shortG` — shorter of fixed/scientific, 5 sig digits
  - `format longG` — shorter of fixed/scientific, 15 sig digits
  - `format bank` — fixed 2 decimal places (e.g. `3.14`)
  - `format rat` — rational approximation via continued fractions (e.g. `355/113` for pi)
  - `format hex` — IEEE 754 double bit pattern as 16 uppercase hex digits
  - `format +` — sign-only display: `+`, `-`, or space
  - `format compact` — suppress blank lines between outputs
  - `format loose` — restore blank lines (default)
  - `format N` — custom N decimal places (legacy behaviour)
  - `format` alone resets to `short`
  - `help format` — new help topic with full mode reference and examples
- **New example** `examples/formatted_output.calc` demonstrating all `fprintf`/`sprintf` specifiers, width/precision/alignment flags, escape sequences, Octave repeat behaviour, and a formatted kinematic data table

### Changed

- `format_scalar`, `format_complex`, `format_value`, `format_value_full` now accept `&FormatMode` parameter (replaces raw `usize` precision)
- `num2str` uses MATLAB-compatible 5-significant-digit formatting by default

## [0.14.0] - 2026-04-08

### Added

- **Phase 10 — C-style I/O and precision overhaul**:
  - `fprintf(fmt, v1, v2, ...)` — full C-style formatted output to stdout; returns `Value::Void` (no result display)
  - `sprintf(fmt, v1, v2, ...)` — same formatting engine, returns result as a char array (`Value::Str`)
  - **Format specifiers**: `%d` `%i` (integer), `%f` (fixed), `%e` (scientific), `%g` (shorter of f/e), `%s` (string), `%%` (literal `%`)
  - **Width and precision**: `%8.3f`, `%-10s`, `%+.4e`, `%05d`
  - **Flags**: `-` (left-align), `+` (force sign), `0` (zero-pad), space (space sign)
  - **Escape sequences** in format strings: `\n`, `\t`, `\\`
  - **Octave repeat behaviour**: when more arguments than specifiers, the format string repeats for remaining args
  - `Value::Void` variant added to the `Value` enum — returned by side-effectful functions; suppresses result display

### Changed

- `sprintf(fmt)` (single-arg, escape-sequences-only) extended to full variadic `sprintf(fmt, ...)` with format specifiers
- `fprintf` moved from special-case string parsing in `repl.rs` into the engine's `call_builtin`, enabling use in any expression context
- `help script` / `help io` updated with full format specifier reference and examples

### Removed

- **`p` / `p<N>` precision directive** removed from the REPL and pipe mode (Phase 10c)
  - Precision is now only settable via `config.toml` or `config reload`
  - Scripts that need specific formatting should use `fprintf` / `sprintf` (portable to Octave)

## [0.13.0] - 2026-04-07

### Added

- **Phase 9 — String data types**:
  - `Value::Str(String)` — char array (single-quoted `'text'`), MATLAB-style, numeric-compatible
  - `Value::StringObj(String)` — string object (double-quoted `"text"`)
  - **Tokenizer**: `'` is context-sensitive — transpose after `ident`/`)`/`]`/number/`'`/string token; char array literal otherwise
  - **Escape sequences**: `''` inside `'...'` = escaped single quote; `\n` `\t` `\\` `\"` inside `"..."`
  - **Char array arithmetic**: char → ASCII codes before binary ops; single char → Scalar, multi-char → 1×N Matrix
  - **String object operations**: `+` concatenates; `==` / `~=` compare whole strings; other ops return an error
  - **AST**: `Expr::StrLiteral(String)` and `Expr::StringObjLiteral(String)` added
  - **New built-in functions**:
    - `num2str(x)` / `num2str(x, N)` — convert number to char array with N decimal digits
    - `str2num(s)` — parse char array as number; error on failure
    - `str2double(s)` — parse char array as number; returns `NaN` on failure
    - `strcat(a, b, ...)` — concatenate two or more strings
    - `strcmp(a, b)` — case-sensitive equality test, returns 0/1
    - `strcmpi(a, b)` — case-insensitive equality test
    - `lower(s)` / `upper(s)` — case conversion
    - `strtrim(s)` — strip leading and trailing whitespace
    - `strrep(s, old, new)` — replace all occurrences of `old` with `new`
    - `sprintf(fmt)` — process escape sequences; single-argument form
    - `ischar(s)` — 1 if char array, else 0
    - `isstring(s)` — 1 if string object, else 0
  - **Updated built-ins**: `length`, `numel`, `size` now handle string arguments
  - **`who`**: shows type annotation — `[1×N char]` for char arrays, `[string]` for string objects
  - **Workspace**: `ws`/`wl` skip string variables (same policy as matrices and complex)
  - New example file `examples/strings.calc` covering all Phase 9 features

## [0.12.0] - 2026-04-06

### Added

- **Phase 8 — Complex numbers**:
  - `Value::Complex(f64, f64)` variant added to the `Value` enum
  - `i` and `j` pre-seeded in `Env` at startup as `Complex(0.0, 1.0)` (Octave semantics; user can reassign)
  - **Syntax**: `3 + 4i` works via implicit multiply: `4 * i` → `Complex(0, 4)`, `3 + 4*i` → `Complex(3, 4)`
  - **Arithmetic**: `+`, `-`, `*`, `/`, `^` / `.^` for all Complex↔Scalar and Complex↔Complex combinations
  - **Unary operators**: `-z` (negate), `~z` (logical NOT), `z'` (conjugate transpose for scalars)
  - **Display**: `a + bi`, `a - bi`, `bi`, `a + i`, `a` (when im is exactly 0)
  - **Comparison**: `==` and `~=` compare both real and imaginary parts; `<`, `>`, `<=`, `>=` return an error
  - **Logical**: `&&` and `||` treat complex as nonzero when `re ≠ 0` or `im ≠ 0`
  - **Built-in functions**:
    - `real(z)` — real part (works on scalars: returns unchanged)
    - `imag(z)` — imaginary part (returns 0 for real scalars)
    - `abs(z)` — modulus `sqrt(re²+im²)` (overloads existing scalar and matrix `abs`)
    - `angle(z)` — argument `atan2(im, re)` in radians
    - `conj(z)` — complex conjugate `a − bi`
    - `complex(re, im)` — construct from two reals; collapses to Scalar when im is 0
    - `isreal(z)` — `1` if `im == 0`, else `0`
  - **Scope boundary**: matrix literals containing Complex elements return an error
  - **Workspace**: `ws`/`wl` skip complex variables (same policy as matrices)
  - `scalar_arg` now accepts `Complex` with `im == 0` as a real scalar for all built-in functions

## [0.11.0+003] - 2026-04-05

### Added

- **Phase 7.5 — Special constants, vector utilities, and indexing enhancements**:
  - `nan` and `inf` as parser-level constants (like `pi`/`e`); `-inf` also works
  - `isnan(x)`, `isinf(x)`, `isfinite(x)` — element-wise predicates (scalar and matrix)
  - `nan(n)` / `nan(m, n)` — matrix filled with NaN (complements `zeros`/`ones`)
  - Vector reductions — for vectors: scalar result; for M×N matrices: 1×N column-wise result:
    - `sum(v)`, `prod(v)`, `mean(v)`, `min(v)`, `max(v)` (1-arg forms)
    - `any(v)`, `all(v)` — reduce to 0/1 logical result
    - `norm(v)` — Euclidean (L2) norm; `norm(v, p)` — general Lp norm
  - Cumulative operations (same shape as input):
    - `cumsum(v)` — cumulative sum; `cumprod(v)` — cumulative product
  - Data manipulation:
    - `sort(v)` — ascending sort (vectors only)
    - `reshape(A, m, n)` — reshape with column-major (MATLAB) element order
    - `fliplr(v)` — reverse column order; `flipud(v)` — reverse row order
    - `find(v)` — 1-based column-major indices of non-zero elements; `find(v, k)` — first `k`
    - `unique(v)` — sorted unique elements as a 1×N row vector
  - `end` keyword in index expressions: resolves to the size of the indexed dimension
    - `v(end)`, `v(end-1)`, `v(3:end)`, `seq(1:2:end)`, `A(end, :)`, `A(1:end-1, 2:end)`
    - Arithmetic on `end` is fully supported: `v(end-2:end)`
- New example file `examples/vector_utils.calc` demonstrating all Phase 7.5 features

## [0.11.0+002] - 2026-04-04

### Added

- **Bitwise functions** (Octave-compatible, scalar integer arguments):
  - `bitand(a, b)` — bitwise AND
  - `bitor(a, b)` — bitwise OR
  - `bitxor(a, b)` — bitwise XOR
  - `bitshift(a, n)` — left shift (`n > 0`) or logical right shift (`n < 0`); returns 0 for `|n| >= 64`
  - `bitnot(a)` — bitwise NOT within 32-bit window (Octave `uint32` default)
  - `bitnot(a, bits)` — bitwise NOT within explicit bit-width window (`bits` in `[1, 53]`)
- All bitwise functions require non-negative integer arguments; non-integers or negatives return an error
- Natural combination with existing hex/bin/oct input literals: `bitand(0xFF, 0x0F)`, `bitor(0b1010, 0b0101)`

## [0.11.0+001] - 2026-04-04

### Added

- **Phase 7 — Comparison and logical operators**:
  - Comparison: `==`, `~=`, `<`, `>`, `<=`, `>=` — return `0.0`/`1.0` (false/true)
  - Logical NOT: `~expr` — unary, returns 1.0 if operand is zero, else 0.0
  - Short-circuit logical: `&&` (AND) and `||` (OR) — scalar and element-wise
  - Element-wise comparison on matrices: `v > 3`, `A == B`
  - Precedence (low to high): `||` → `&&` → comparisons → range → arithmetic
  - `~` (logical NOT) at unary level, same precedence as `-`
  - `Expr::UnaryNot` AST node; `Op::Eq/NotEq/Lt/Gt/LtEq/GtEq/And/Or` variants

## [0.11.0] - 2026-04-03

### Added

- **Phase 6 — Indexing**:
  - `v(i)` — 1-based linear indexing of vectors and matrices (column-major)
  - `v(1:3)` — range as index: extracts sub-vector
  - `v(:)` — all elements as a column vector (column-major order)
  - `A(i, j)` — 2D indexing: returns scalar when both indices are scalars
  - `A(:, j)` — all rows of column `j` → column vector
  - `A(i, :)` — row `i`, all columns → row vector
  - `A(1:2, 2:3)` — sub-matrix via range indices
  - Index expressions can be arbitrary arithmetic: `A(1+1, size(A,2))`
  - `Expr::Colon` AST node for the all-elements selector `:`
  - `parse_call_arg()` — parses bare `:` as `Expr::Colon`; otherwise delegates to `parse_range`; all call/index argument positions now use this

### Fixed

- Range expressions inside grouping parentheses now parse correctly: `2 .^ (0:7)` previously failed with "Expected closing ')'" because the `(...)` parser used `parse_expr` instead of `parse_range`

### Changed

- `Expr::Call` evaluation: if the name resolves to a variable in `Env`, the expression is treated as indexing (variables shadow built-in function names — Octave semantics). Otherwise evaluated as a built-in function call.
- Function call argument parsing switched from `parse_expr` to `parse_call_arg`, enabling range expressions as function/index arguments: `linspace(0:1, 5)` now parses (though semantically an error), and `A(1:3, :)` works correctly.

## [0.10.0] - 2026-04-03

### Added

- **Phase 5 — Range operator**:
  - `a:b` — generates a 1×N row vector from `a` to `b` with step 1
  - `a:step:b` — three-argument form with explicit step (positive or negative)
  - Arithmetic can be used in range bounds: `1+1:2*3` = `2:6`
  - Empty range (step in the wrong direction) produces a 1×0 matrix, displayed as `[]`
  - Ranges work inside matrix literals: `[1:5]` → `[1 2 3 4 5]`, `[1:2:7]` → `[1 3 5 7]`
  - Ranges can be mixed with scalars in brackets: `[0, 1:3, 10]` → `[0 1 2 3 10]`
  - `Token::Colon` added to the tokenizer; `Expr::Range` added to the AST
  - `parse_range()` — new lowest-precedence parser level; `parse()` and `parse_matrix()` updated to use it
- **`linspace(a, b, n)`** — generates `n` linearly spaced values from `a` to `b` (inclusive)

### Changed

- `Expr::Matrix` evaluator: row elements that evaluate to a `Value::Matrix` (row vector) are now concatenated horizontally into the row, enabling range expressions inside `[...]`

## [0.9.0] - 2026-04-02

### Added

- **Phase 4 — Matrix operations**:
  - Matrix multiplication `A * B` (inner-dimension checked, via ndarray `.dot()`)
  - Postfix transpose `A'` — new `Token::Apostrophe`, `Expr::Transpose`; binds tighter than any binary operator
  - Element-wise operators `.*`, `./`, `.^` — new tokens `DotStar`, `DotSlash`, `DotCaret`; same precedence as `*`, `/`, `^` respectively
  - Number tokenizer no longer absorbs `.` before `*`, `/`, `^` (fixes `3.*2` parsing)
- **Built-ins**: `zeros(m,n)`, `ones(m,n)`, `eye(n)`, `size(A)`, `size(A,dim)`, `length(A)`, `numel(A)`, `trace(A)`, `det(A)`, `inv(A)`
  - `det` and `inv` use Gaussian / Gauss-Jordan elimination (no external BLAS/LAPACK dependency)
- **`is_partial`** extended: `.*`, `./`, `.^` prefixes now recognized as partial expressions

### Changed

- `eval_binop`: `Matrix * Matrix` now performs matrix multiplication (was an error); element-wise ops use ndarray broadcast
- `call_builtin` refactored to return `Result<Value, String>` directly (supports both scalar and matrix return values)

## [0.8.0] - 2026-04-01

### Added

- **Phase 3 — Matrix literals**: `[1 2 3]`, `[1; 2; 3]`, `[1 2; 3 4]` and arbitrary-expression elements
- **`Value` enum** in `env.rs`: `Scalar(f64)` | `Matrix(ndarray::Array2<f64>)`; `Env` migrated from `HashMap<String, f64>` to `HashMap<String, Value>`
- **Matrix arithmetic**: scalar × matrix element-wise (`+`, `-`, `*`, `/`, `^`); matrix `+` and `-` (shapes must match)
- **Matrix display**: multi-line right-aligned columns; REPL prompt shows `[ [N×M] ]` when `ans` is a matrix
- **`format_scalar`** — new public formatter for guaranteed-scalar contexts; `format_value_full` for multi-line matrix output
- **`help matrices`** topic — `help matrices` in the REPL prints matrix reference
- **ndarray 0.16** added as a dependency of `ccalc-engine`

### Changed

- `split_stmts()` in `repl.rs` is now bracket-depth-aware: `;` inside `[...]` is parsed as a matrix row separator, not a statement separator
- `eval()` now returns `Result<Value, String>` (was `Result<f64, String>`)
- Workspace save (`ws`) silently skips matrix variables — only scalars are persisted

## [0.7.0+012] - 2026-03-31

### Added

- **Two-level help system** — `help` prints a one-screen cheatsheet; `help <topic>` shows a detailed section:
  - `help syntax` — operators, precedence, implicit multiplication, partial expressions
  - `help functions` — full function reference including `mod` vs `rem` explanation
  - `help bases` — number base input, display switching, inline suffix, `base` command
  - `help vars` — variables, assignment, `who`/`clear`/`ws`/`wl`
  - `help script` — pipe/script mode, `;`, `disp`, `fprintf`, escape sequences
  - `help examples` — practical usage examples
- **`?` shortcut** — alias for `help` in the REPL
- **`-h` / `--help` flag** — now shows usage and modes only (no math reference); full reference accessible via `help` in the REPL
- **REPL banner** — updated to `ccalc vX.Y.Z  (type 'help' for reference)`

## [0.7.0+011] - 2026-03-31

### Added

- **Phase 2 — Multi-argument functions**: `atan2(y,x)`, `mod(a,b)`, `rem(a,b)`, `max(a,b)`, `min(a,b)`, `hypot(a,b)`, `log(x,base)`
- **Inverse trig**: `asin(x)`, `acos(x)`, `atan(x)`
- **`sign(x)`** — returns −1, 0, or 1
- **`Token::Comma`** — comma is now a valid token; function calls accept comma-separated argument lists: `fn(a, b, c)`
- **`mod` vs `rem` semantics**: `mod(-1, 3) = 2` (sign follows divisor, Octave convention); `rem(-1, 3) = -1` (sign follows dividend)
- **`examples/ac_impedance.ccalc`** — demonstrates `hypot`, `atan2`, `mod`, `max`, `min`, `log`, `log(x,base)` in an AC circuit calculation

### Changed

- `Expr::Call(String, Box<Expr>)` → `Expr::Call(String, Vec<Expr>)` — variadic argument list
- Evaluator dispatch moved from inline `match` to `call_builtin(name, args: &[f64])` using slice pattern matching; one-argument functions are backward-compatible

## [0.7.0+008] - 2026-03-28

### Added

- **Variable expansion in REPL** — when an expression contains known variables, the expanded form is printed before the result: `ans + x + y` → prints `13 + 10 + 20` then `[ 43 ]:`

### Fixed

- **Double output for assignments** — `w = ans` was printing `w = 110` twice (once from expansion display, once from assignment handler); expansion display is now suppressed for assignment statements

## [0.7.0+007] - 2026-03-28

## [0.7.0+006] - 2026-03-28

### Removed

- **`c` command** — reset-ans command removed; use `ans = 0` to reset manually if needed

## [0.7.0+005] - 2026-03-28

### Changed

- **`q` → `exit`** — quit command renamed to `exit`; `quit` also accepted as an alias

## [0.7.0+004] - 2026-03-28

### Added

- **Script file argument** — `ccalc script.m` runs a file directly without shell redirection; if the argument is an existing file it is executed as a script, otherwise it is evaluated as an expression (existing behaviour)

## [0.7.0+003] - 2026-03-28

### Changed

- **Comment symbol `#` → `%`** — aligns with Octave/MATLAB convention; `%` starts a comment both as a full line and inline after an expression
- **`%` operator removed** — modulo (`17 % 5`) and percentage postfix (`20%`) are no longer supported; `%` is now exclusively a comment character
- **REPL welcome line** — version banner printed on startup: `ccalc v0.7.0+003  (type q to quit, -h for help)`

### Removed

- `Op::Mod` from the AST and evaluator
- `Token::Percent` from the tokenizer

## [0.7.0+002] - 2026-03-28

### Changed

- **Variable system** — replaced fixed memory cells (`m1`–`m9`) with a full named-variable environment:
  - `x = expr` assignment syntax (any valid identifier)
  - `ans` replaces `acc` as the implicit result of the last expression (Octave/MATLAB convention)
  - `who` lists all defined variables (replaces `m`)
  - `clear` / `clear x` clears all variables or a single one (replaces `mc` / `mc1`)
  - `ws` / `wl` save/load the workspace (replaces `ms` / `ml`)
  - `c` resets `ans` to `0` (behavior unchanged)
- **Engine restructure** — `memory.rs` removed; new `env.rs` module provides `Env` type (`HashMap<String, f64>`), workspace I/O, and identifier validation

## [0.7.0+001] - 2026-03-28

### Changed

- **Cargo workspace** — project restructured into two crates:
  - `crates/ccalc-engine` — new library crate containing the parser, evaluator, and memory modules; serves as the foundation for the upcoming Octave/MATLAB compatibility layer
  - `crates/ccalc` — binary crate (CLI), now depends on `ccalc-engine`
- **Single version source** — version is now defined once in `[workspace.package]` and inherited by both crates via `version.workspace = true`

### Added

- **mdBook documentation skeleton** — `docs/` directory with `book.toml` and `src/SUMMARY.md`; sections: User Guide, Architecture, Octave Compatibility

## [0.7.0+009] - 2026-03-26

### Added

- **Comments in pipe/file mode** — lines starting with `#` are skipped; inline `#` trims the rest of the line:
  ```
  # full-line comment
  10 * 5  # inline comment — the expression still evaluates
  ```
- **Semicolon suppression** — a trailing `;` evaluates the expression and updates the accumulator but prints nothing:
  ```
  0.06 / 12;   # silent intermediate step
  m1;
  1 + m1;      # still updates accumulator
  print "Monthly payment ($):"
  ```
- **`print` command** — explicit output control in pipe/file mode:
  - `print` — prints the current accumulator value
  - `print "label"` — prints `label value` (the label is the full quoted string, including any `:` the user writes)
- **Section headers** — `print "label"` after a blank line (or at the start) prints the label only, without the value, acting as a section separator:
  ```
  print "=== Results ==="

  10 + 5
  print "Sum:"
  ```
  Output:
  ```
  === Results ===
  15
  Sum: 15
  ```
- **`examples/` directory** — four annotated formula files demonstrating comments, `;`, and `print`:
  - `cylinder.ccalc` — volume and surface area of a cylinder
  - `mortgage.ccalc` — monthly mortgage payment
  - `data_storage.ccalc` — storage unit conversion (real GiB in a "500 GB" drive)
  - `resistors.ccalc` — Ohm's law: series, parallel, voltage divider, power

### Fixed

- Compound memory directives (`2 + 2 + 2 m1-`) now display the evaluated RHS value instead of the raw expression string:
  was: `10 - (2 + 2 + 2)` → now: `10 - 6`

## [0.7.0+003]

### Changed

- Compound memory directives (`m1+`, `m2*`, etc.) now display the full operation before the result:
  `80 m1-` with m1=850 prints `850 - 80` then `[ 770 ]:`
- Operation order is `cell op expr` to match the actual computation
- Multi-token expressions on the right-hand side are wrapped in parentheses:
  `2 * 10 + 2 + 2 m1+` with m1=10 prints `10 + (2 * 10 + 2 + 2)` then `[ 34 ]:`

## [0.7.0] - 2026-03-25

### Added

- Memory persistence: `ms` saves all non-zero memory cells to `~/.config/ccalc/memory.toml`; `ml` loads them back (clears all cells first, then restores from file)
- `dirs` dependency for cross-platform config directory resolution
- Expression conversion display: when the current base is non-decimal and the expression
  contains literals in other bases, the expression is printed with all values converted
  to the accumulator's base before the result
  - `[ 0b110 ]: 2 + 0b110 + 0xa` prints `0b10 + 0b110 + 0b1010` then `[ 0b10010 ]:`
  - `[ 0x6 ]: 0b11 + 0b11` prints `0x3 + 0x3` then `[ 0x6 ]:`

### Fixed

- `base` command and `expr base` suffix now display hex with `0x` prefix (e.g. `0xA` instead of `A`)
- Hex display in expression conversion now uses `0x` prefix, consistent with bin (`0b`) and oct (`0o`)

## [0.6.0] - 2026-03-25

### Added

- Percentage postfix operator: `N%` evaluates to `N * accumulator / 100`
  - `[1500]: 20%` → `300`
  - `[1500]: + 20%` → `1800` (add 20% of accumulator)
  - `[1800]: - 10%` → `1620` (subtract 10% of accumulator)
  - Disambiguated from modulo by lookahead: `17 % 5` still means modulo
- Implicit multiplication: a number or `)` immediately before `(` multiplies without an explicit `*`
  - `2(3 + 1)` → `8`
  - `(2+1)(4-1)` → `9`

## [0.5.0] - 2026-03-24

### Added

- Hex, binary, and octal input literals: `0xFF`, `0b1010`, `0o17` — parsed directly in expressions
- Display base commands: `hex`, `dec`, `bin`, `oct` — change how all subsequent results are shown (including the prompt)
- Inline base suffix: `0xFF + 0b1010 hex` evaluates the expression and switches the display base in one step
- `base` command — prints the current accumulator value in all four bases simultaneously
- Configurable decimal precision: `p` shows current precision, `p<N>` sets it (0–15 decimal places, default 10)
- Scientific notation display for very large (`|n| >= 1e15`) and very small (`|n| < 1e-9`) numbers
- All new formatting and base commands work identically in REPL, pipe, and single-expression modes

### Changed

- `memory.display_nonzero` now accepts a format closure, allowing memory cells to be printed in the current display base

## [0.4.0] - 2026-03-23

### Added

- Pipe / non-interactive mode: when stdin is not a terminal, ccalc runs silently (no prompt, one result per line)
- Single-expression argument mode: `ccalc "expr"` evaluates and prints the result, exits with code 1 on error
- File redirect support: `ccalc < formulas.txt` (handled by the same pipe path)
- Accumulator carries over across lines in pipe mode — multi-step calculations work naturally
- Commands `q`, `c`, `mc`, `mc[1-9]`, `m[1-9]` all work in pipe mode; `cls` and `m` are silently ignored

### Changed

- Refactored `repl.rs`: extracted shared `evaluate()` / `evaluate_expanded()` / `apply_compound()` helpers used by all three modes
- `main.rs` now detects mode via `std::io::IsTerminal` (no extra dependency)

## [0.3.0] - 2026-03-23

### Added

- Line editing via `rustyline`: ← → Home End cursor movement, Ctrl+W word delete, Ctrl+U line clear
- History navigation: ↑ ↓ to browse previous inputs, Ctrl+R for reverse search
- Ctrl+C and Ctrl+D as additional quit shortcuts (in addition to `q`)
- `acc` — explicit alias for the current accumulator value in expressions (e.g. `sqrt(acc)`, `acc * 2`)
- Empty function call `fn()` uses the accumulator as argument (e.g. `sqrt()` → `sqrt(accumulator)`)
- Compound assignment directives `m[1-9]OP` for operators `+`, `-`, `*`, `/`, `%`, `^`: `expr m1+` means `m1 = m1 + expr`; accumulator is set to the new cell value

### Removed

- Memory add/subtract commands `ma[1-9]` and `ms[1-9]` (replaced by the more general compound assignment directives)

## [0.2.0] - 2026-03-22

### Added

- Power operator `^` (right-associative, higher precedence than `*` and `/`), e.g. `2 ^ 10` → `1024`
- Modulo operator `%` (same precedence as `*` and `/`), e.g. `17 % 5` → `2`
- Constants `pi` and `e` usable in any expression, e.g. `sin(pi / 6)` → `0.5`
- Math functions: `sqrt`, `abs`, `floor`, `ceil`, `round`, `log` (base 10), `ln`, `exp`, `sin`, `cos`, `tan`
- Partial expressions now also accept `^` and `%` as leading operators
- New AST nodes: `Expr::Call(name, arg)`, `Op::Pow`, `Op::Mod`
- New `Ident(String)` token in the lexer — architectural prerequisite for functions and constants
- 38 new unit tests covering all new operators, constants, functions, precedence, and edge cases

## [0.1.0+004] - 2026-03-09

### Added

- CLI flag `-h` / `--help` — prints full usage reference with examples
- Unknown CLI flags now print an error message and exit with code 1

## [0.1.0+003] - 2026-03-09

### Added

- Memory cells `m1`–`m9` for storing intermediate values
- Store accumulator into cell: `m[1-9]` (standalone)
- Store expression result into cell: `expr m[1-9]` (trailing directive)
- Recall cell value inside any expression: `m[1-9]` used as an operand
- Add to cell: `ma[1-9]` (standalone) and `expr ma[1-9]` (trailing); prints new cell value
- Subtract from cell: `ms[1-9]` (standalone) and `expr ms[1-9]` (trailing); prints new cell value
- Show all non-zero cells: command `m`
- Clear all cells: command `mc`
- Clear a specific cell: `mc[1-9]`
- Expression display when memory refs are expanded (e.g. `m1 + 8 + m1` → prints `6 + 8 + 6`)
- New module `memory` encapsulating `Memory` struct, standalone command parser, directive extractor, and ref expander
- 26 unit tests covering all memory operations and parsing rules

## [0.1.0+002] - 2026-03-09

### Added

- CLI flag `-v` / `--version` — prints program version and exits

## [0.1.0+001] - 2026-03-07

### Added

- CLI calculator REPL with prompt `[result]:` acting as a numeric display
- Arithmetic operations: `+`, `-`, `*`, `/` with correct operator precedence
- Parenthesized expressions support, e.g. `(3 + 3) * 2`
- Partial expressions: input starting with an operator uses the current accumulator as the left operand (e.g. `+ 2`, `* 100`)
- Unary minus support (e.g. `-5`, `-(3 + 2)`)
- Command `c` — resets the accumulator to 0
- Command `cls` — clears the console screen
- Command `q` — exits the program
- Smart number formatting: integers displayed without decimal point; floats trimmed to 10 significant fractional digits with trailing zeros removed
- Module structure: `repl` (I/O loop), `parser` (tokenizer + recursive descent parser), `eval` (AST types + evaluator)
- 18 unit tests covering eval, formatting, parsing, operator precedence, parentheses, error cases, and partial-expression detection
