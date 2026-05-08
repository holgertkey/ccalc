# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.30.0+005] - 2026-05-08

### Fixed

- **`prctile` / `iqr` interpolation method** — switched from Type 7 (R's
  formula, `idx = (n-1)*p/100`) to **Type 5 / Hazen** (`idx = n*p/100 - 0.5`,
  clamped to `[0, n-1]`), which matches Octave and MATLAB.  Examples that
  exposed the difference:
  - `prctile([1 1 2 2 3 4 7 12 20], 25)` → **1.75** (was 2)
  - `prctile([1 1 2 2 3 4 7 12 20], 75)` → **8.25** (was 7), IQR **6.5** (was 5)
  - `prctile([2 4 4 4 5 5 7 9], 75)` → **6** (was 5.5), IQR **2** (was 1.5)
  - `iqr([1 2 3 4 5])` → **2.5** (was 2)

- **`skewness` for constant vectors** — when all values are equal, `std = 0`
  and the formula `m3 / m2^(3/2)` reduces to `0/0`.  Now returns **NaN** to
  match Octave (was returning 0).  Consistent with `kurtosis`, which already
  returned NaN in this case.

  3 Octave-pinning regression tests added for `prctile`; 902 tests total.

## [0.30.0+004] - 2026-05-08

### Performance

- **O(n²) → O(n) loop performance** — three bottlenecks fixed that caused
  loops with indexed reads/writes and lambda calls to degrade quadratically:

  1. **`exec_index_set` matrix clone** — the indexed-write path
     (`y(k) = val`) previously cloned the entire target matrix on every
     iteration via `env.get(name).clone()`.  Now uses `env.remove()` after
     index-expression resolution to take ownership (move, not copy).
     For a pre-allocated 10 001-element vector this eliminated ~800 MB of
     unnecessary data copying per `trapz_rule(n=10000)` call.

  2. **`eval_inner` indexed-read clone** — `Expr::Call` dispatch checked
     `env.get(name).cloned()` to determine whether a name is a variable to
     index or a function to call.  For matrix variables this cloned the
     whole array.  Now the code uses a borrow for non-function values
     (`eval_index(env_val, args, env)`) and only clones for `Lambda`/`Function`
     (cheap: `Rc` reference count or `String` fields).

  3. **Autoload miss cache** — when a name that is a built-in (e.g. `sin`,
     `cos`, `exp`) is called inside a lambda, `eval_inner` previously fired
     `try_autoload(name)` on every call.  That function searches the session
     path with multiple `Path::exists()` filesystem stat() calls, and the
     negative result was never cached.  Added `AUTOLOAD_MISS_CACHE`
     (`thread_local HashSet<String>`) so each name is searched at most once
     per session.  This reduced per-lambda-call overhead from ~2.4 ms to
     ~0.06 ms for typical built-in functions.

  Combined effect: `trapz_rule_demo.m` (including the n = 10 000 convergence
  table in Example 4) dropped from **≈ 28 s → ≈ 0.3 s** (~97× speedup).
  These optimizations apply broadly to any loop that does repeated indexed
  reads, indexed writes, or anonymous-function calls on vectors.

  - 4 regression tests added in `loop_performance_regression_tests` (899 total).

## [0.30.0+003] - 2026-05-07

### Fixed

- **`fprintf` / `disp` output stutter** — `IoContext::write_to_fd` (the code
  path used by all REPL and script execution) now flushes stdout/stderr only
  when the output string contains a newline character, instead of after every
  single write. Previously, each `fprintf('%d ', v(k))` call inside a loop
  triggered a separate OS `flush` syscall, causing visible character-by-character
  output. Now all numbers on a line appear atomically when the trailing
  `fprintf('\n')` flushes the buffer.

### Performance

- **`contains_end` optimization — up to ~700× fewer env clones in index-heavy
  loops** — added `pub(crate) fn contains_end(expr: &Expr) -> bool` in
  `eval.rs` that recursively checks whether an index expression references the
  identifier `end`. All 11 `env_with_end` call sites in `eval_index` and all
  4 `write_env_with_end` call sites in `exec.rs` now skip the full
  `HashMap::clone()` when `end` is absent. For typical loops with indices like
  `v(j)` or `v(j+1)`, **zero** environment clones occur — previously each
  indexed read or write cloned the entire variable environment.
  - 11 regression tests added in `end_index_regression_tests` (895 total).

## [0.30.0+002] - 2026-05-07

### Added

- **`near line N` error messages** — runtime errors that occur inside block
  statements, function bodies, and scripts executed via `run()`/`source()`
  now include the 1-based source line number of the failing statement:
  ```
  Error: Undefined variable: 'bad_var' near line 3
  ```
  - Each `(Stmt, bool)` in the parsed AST now carries a `usize` line number,
    recorded during parsing from the `pos` counter in `parse_stmts_from_lines`.
  - `exec_stmts` wraps every `eval_with_io` call with
    `.map_err(|e| annotate_line(e, stmt_line))?`; errors that already contain
    `"near line"` are passed through unchanged (innermost location wins).
  - `catch` variable `e.message` strips the suffix before storing, matching
    MATLAB/Octave semantics where `e.message` is the clean message and
    location info lives in `e.stack`.
  - Single-line block expansions (`if cond; body; end`) map all virtual lines
    back to the original physical source line.
  - 7 new regression tests (884 total).

## [0.30.0+001] - 2026-05-06

### Added

- **Char-array matrix literals** — `['str' expr 'str']` now concatenates
  char arrays horizontally, matching MATLAB/Octave semantics:
  - **String context** (first element is a char array): numeric elements become
    characters via Unicode code point — `['A' 66 67]` → `'ABC'`.
  - **Numeric context** (first element is numeric): char-array elements
    contribute their code values — `[65 'B']` → `[65 66]`.
  - Common dynamic-string idiom now works: `['v' num2str(k) ' = k^2']`.

### Fixed

- **`eval(...)` statement context in pipe/script mode**: standalone `eval`
  calls on their own line (outside a block) now correctly mutate the caller's
  workspace. Previously they fell through to `evaluate()` → `call_builtin`,
  which ran on a cloned env and discarded all variable assignments.
- **`'` transpose disambiguation**: a `'` preceded by whitespace is now
  always tokenized as the start of a new char-array literal, never as a
  transpose. This is the correct MATLAB rule and fixes `['a' 'b']` being
  mis-parsed as `('a')' b`.
- **Clippy** — `env.get("caught").is_none()` replaced with `!env.contains_key("caught")`.
- 11 new tests in `mod char_array_literal_tests` (877 total).

## [0.30.0] - 2026-05-05

### Added

- **Phase 25 — Dynamic evaluation and timing** (3 new built-in functions):
  - **25a — `eval`:**
    - `eval(str)` — executes a string as code in the current workspace; variable mutations persist in the caller's scope (MATLAB semantics).
    - `eval(str, catch_str)` — two-argument form: if `str` errors, execute `catch_str` instead; original error stored in `lasterr()`.
    - In expression context (`y = eval('...')`), returns `ans` of the inner execution; env mutations inside are discarded.
    - Nesting depth capped at 64 (shared with `run`/`source`).
  - **25b — `tic` / `toc`:**
    - `tic` — starts (or restarts) a wall-clock timer; returns `Void`.
    - `toc` — returns elapsed seconds since last `tic` as a scalar; multiple calls after one `tic` are valid; calling before `tic` is an error.
    - Both work with or without parentheses (`tic`, `tic()`).
  - **Engine change:** `Expr::Var` handler now falls back to `call_builtin(name, &[], ...)` for unresolved names, enabling zero-arg built-ins to be called without parentheses.
  - 11 new tests in `mod phase25_tests` (866 total).
  - New guide page `docs/src/guide/eval.md`; `help eval` topic added.

## [0.29.0] - 2026-05-04

### Added

- **Phase 24 — Polynomial operations and interpolation** (7 new built-in functions, no new syntax):
  - **24a — Evaluation, fitting, roots:**
    - `polyval(p, x)` — evaluate polynomial `p` at scalar or matrix `x` (Horner's method).
    - `polyfit(x, y, n)` — least-squares degree-`n` fit via Vandermonde matrix + QR decomposition.
    - `roots(p)` — all roots via Durand–Kerner iteration; returns real `Matrix` (column) when all roots are real, `Cell` of `Scalar`/`Complex` otherwise.
    - `poly(r)` — monic polynomial from a root vector; `poly(A)` computes the characteristic polynomial of a square matrix (Faddeev–LeVerrier algorithm).
  - **24b — Convolution, deconvolution, interpolation:**
    - `conv(a, b)` — discrete linear convolution; result length = `m + n − 1`.
    - `deconv(c, b)` — polynomial long division; returns `[q, r]` tuple with `conv(b,q)+r==c`.
    - `interp1(x, y, xi[, method])` — piecewise interpolation at query points; methods: `'linear'` (default), `'nearest'`, `'previous'`, `'next'`; out-of-range → `NaN`.
  - **Implementation note:** `roots` uses the Durand–Kerner algorithm in complex arithmetic (not the companion-matrix/eig approach from the CDP plan, which is limited to real eigenvalues).
  - 33 new tests in `mod phase24_tests` (855 total).
  - New guide page `docs/src/guide/polynomials.md`; `help poly` topic added.

## [0.28.0] - 2026-05-04

### Added

- **Phase 23 — Matrix utilities and set operations** (13 new built-in functions, no new syntax):
  - **23a — Triangular extraction and tiling:** `triu(A[,k])`, `tril(A[,k])`, `repmat(A,m,n)`, `kron(A,B)`.
  - **23b — Vector products:** `cross(a,b)` (3-element vectors; result orientation matches `a`), `dot(a,b)` (inner product → scalar).
  - **23c — Set operations:** `intersect(a,b)`, `union(a,b)`, `setdiff(a,b)`, `ismember(x,v)`. All results are sorted and deduplicated; NaN is never a member (IEEE semantics).
  - **23d — Index utilities:** `sub2ind(sz,r,c)`, `ind2sub(sz,idx)` (returns a tuple for use with `[r,c] = ind2sub(...)`), `repelem(v,n)` / `repelem(v,nv)` / `repelem(A,m,n)`.
  - 33 new tests in `mod phase23_tests` (822 total).
  - New guide page `docs/src/guide/set-operations.md`; `help setops` topic added.

## [0.27.0+001] - 2026-05-03

### Fixed

- `fprintf`/`sprintf` `%s` now formats `DateTime` and `Duration` values (previously returned an error).
- `isnat(x)` on any non-datetime value (scalar, duration, string, etc.) now returns `0` instead of throwing an error (MATLAB-compatible).
- `[datetime(...); datetime(...)]` and `[hours(1); hours(2)]` matrix literals now produce `DateTimeArray` / `DurationArray`; mixing types raises an error.

### Added

- `examples/datetime.calc` — Phase 22 demo script covering constructors, arithmetic, extractors, predicates, formatting, and project-timeline example.

## [0.27.0] - 2026-05-03

### Added

- **Phase 22 — Datetime & Duration:**

  - **New value types:**
    - `Value::DateTime(f64)` — UTC timestamp (seconds since Unix epoch).
    - `Value::Duration(f64)` — elapsed time in seconds (fractional).
    - `Value::DateTimeArray(Vec<f64>)` — ordered sequence of UTC timestamps.
    - `Value::DurationArray(Vec<f64>)` — ordered sequence of durations.
    - `NaT` — parser-level Not-a-Time constant; evaluates to `DateTime(NaN)`.

  - **New module `ccalc-engine::datetime`** — pure-Rust UTC calendar arithmetic (no external crate). Implements the Howard Hinnant proleptic Gregorian algorithm: `days_from_civil`, `civil_from_days`, `timestamp_to_civil`, `civil_to_timestamp`, `parse_iso8601`, `format_datetime`, `format_duration`, `format_datestr`, `now_timestamp`, `today_timestamp`, `to_datenum`, `from_datenum`.

  - **Constructors:**
    - `datetime('yyyy-MM-dd')` / `datetime('yyyy-MM-dd HH:mm:ss')` — parse ISO 8601 string.
    - `datetime(y, m, d)` — from year/month/day scalars.
    - `datetime(y, m, d, H, M, S)` — from six components.
    - `datetime(ts, 'ConvertFrom', 'posixtime')` — from Unix timestamp.
    - `duration(H, M, S)` — from hours/minutes/seconds.
    - `hours(n)`, `minutes(n)`, `seconds(n)`, `days(n)`, `milliseconds(n)`, `years(n)` — scalar-to-Duration constructors.

  - **Component extractors:**
    - `year(dt)`, `month(dt)`, `day(dt)`, `hour(dt)`, `minute(dt)`, `second(dt)` — scalar or array forms.

  - **Duration extractors (Duration → Scalar):**
    - `hours(d)`, `minutes(d)`, `seconds(d)`, `days(d)`, `milliseconds(d)` — Duration-to-scalar conversions.

  - **Predicates:**
    - `isdatetime(x)`, `isduration(x)`, `isnat(x)`.

  - **Formatting / conversion:**
    - `datestr(dt)` — default format `dd-MMM-yyyy HH:mm:ss`; `datestr(dt, fmt)` — custom pattern tokens: `yyyy`, `MMM`, `MM`, `dd`, `HH`, `mm`, `ss`, `SSS`.
    - `datevec(dt)` — returns 1×6 row vector `[y m d H M S]`.
    - `datenum(dt)` / `datenum(y, m, d)` — MATLAB serial date number.
    - `posixtime(dt)` — Unix timestamp as scalar.

  - **Arithmetic:** `DateTime ± Duration → DateTime`, `DateTime − DateTime → Duration`, `Duration ± Duration → Duration`, `Duration × Scalar → Duration`; array broadcasting between DateTimeArray / DurationArray.

  - **`diff(arr)`** — successive differences for `DateTimeArray` (→ `DurationArray`), `DurationArray` (→ `DurationArray`), and numeric `Matrix`.

  - **49 new tests** in `eval_tests.rs::datetime_tests` covering constructors, extractors, predicates, arithmetic, formatting, and array operations.

## [0.26.0] - 2026-04-30

### Added

- **Phase 21 — String completions and regex:**

  - **21a — String predicates and joining:**
    - `contains(s, pat)` — returns `1` if `pat` is a substring of `s`, `0` otherwise.
    - `contains(s, pat, 'IgnoreCase', tf)` — 4-argument form for case-insensitive search.
    - `startsWith(s, pat)` — prefix check; returns `1`/`0`.
    - `endsWith(s, pat)` — suffix check; returns `1`/`0`.
    - `strjoin(c)` / `strjoin(c, delim)` — joins a cell array of strings into a single char array. Default delimiter is a space. Rejects non-string cell elements with a clear error.
    - All added as cases in `call_builtin`. No new tokens or AST nodes.

  - **21b — Regular expressions (feature-gated):**
    - `regexp(s, pat)` — returns the 1-based start index of the first match, or an empty matrix `[]` if no match.
    - `regexp(s, pat, 'match')` — returns a `Cell` of `Str` with all matched substrings.
    - `regexpi(s, pat)` / `regexpi(s, pat, 'match')` — case-insensitive variants; prepend `(?i)` to pattern.
    - `regexprep(s, pat, rep)` — replace all non-overlapping matches with the **literal** string `rep` (capture-group expansion `$1`/`${name}` is suppressed via `regex::NoExpand`).
    - Gated behind `--features regex` (adds `regex = "1"` optional dep to `ccalc-engine`). Without the feature, calling any of the three functions returns an informative error. All names always appear in `builtin_names()` for tab completion.
    - `regex` passthrough feature added to the `ccalc` binary crate.
    - 10 new tests in `eval_tests.rs::regex_tests` (gated `#[cfg(feature = "regex")]`).
    - 14 new tests for 21a builtins.

## [0.25.0] - 2026-04-27

### Added

- **Phase 20.5 — MAT file read:**
  - `load('file.mat')` — reads a MATLAB Level 5/7 MAT file and returns a `Struct` whose fields are the variable names stored in the file. Assignment form: `data = load('results.mat')`. Bare form: `load('results.mat')` merges all variables directly into the current workspace.
  - Type mapping: scalar (1×1) → `Scalar`, M×N matrix → `Matrix` (column-major layout converted correctly), char array → `Str`, struct → `Struct`, struct array → `StructArray`, cell array → `Cell`, null → `Scalar(NaN)`.
  - Gated behind the `mat` Cargo feature (keeps the default binary lean): `cargo build --features mat`. When the feature is disabled, calling `load('*.mat')` returns an informative error message.
  - `save('file.mat', ...)` gracefully errors with "writing .mat files is not yet supported".
  - `load` appears in tab completion (`builtin_names`) regardless of the feature flag.
  - Backed by `matrw = "=0.1.4"` (optional dependency in `ccalc-engine`).
  - 5 new tests in `eval_tests.rs` under `mod mat_tests` (gated `#[cfg(feature = "mat")]`).
  - Example script: `examples/mat/mat.calc`.

## [0.24.0] - 2026-04-26

### Added

- **Phase 20c — CSV improvements:**
  - `readmatrix(path)` / `readmatrix(path, 'Delimiter', d)` — reads a delimiter-separated file and returns a `Matrix`. Auto-detects comma or tab delimiter; falls back to whitespace splitting. If the first row contains non-numeric text it is automatically skipped as a header. Empty cells become `NaN` (unlike `dlmread`, which uses `0.0`).
  - `readtable(path)` / `readtable(path, 'Delimiter', d)` — reads a CSV file with a mandatory header row and returns a `Struct` of columns. Numeric columns become `Matrix` (N×1); columns with any non-numeric cell become `Cell` of `Str`. Handles RFC 4180 quoted fields (commas and embedded `"` inside quoted cells).
  - `writetable(T, path)` / `writetable(T, path, 'Delimiter', d)` — writes a struct table to a CSV file with a header row. Accepts `Matrix` (N×1), `Cell`, `Scalar`, and `Str`/`StringObj` columns. Cell values are quoted per RFC 4180 when they contain the delimiter, a double-quote, or a newline.
  - Auto-detection uses the CSV-aware split for comma (respects quoted fields), then tab, then whitespace fallback.
  - 15 new tests in `eval_tests.rs` under `mod csv_tests`.
  - Example script: `examples/csv/csv.calc` (6 sections: writetable, readtable analysis, summary table, readmatrix with header skip, RFC 4180 quoting, tab-separated).
  - In-REPL help: `help csv`.
  - Docs: `docs/src/guide/csv.md`, `docs/src/ccalc/phase20c-csv.md`.

- **Phase 20a — JSON encode/decode:**
  - `jsondecode(str)` — parses a JSON string and returns a ccalc value. Mapping: JSON object → `Struct`, all-numeric array → `Matrix` row vector, mixed array → `Cell`, string → `Str`, number → `Scalar`, boolean → `Scalar` (1/0), null → `Scalar(NaN)`.
  - `jsonencode(val)` — encodes a ccalc value to a compact JSON string (`Str`). Mapping: `Struct` → object, `Matrix` row vector → flat array, `Matrix` M×N → array of row arrays, `Cell` → array, `Scalar(NaN)` → `null`. `Complex`, `Lambda`, `Function`, and `Inf` values produce an error.
  - Both built-ins are gated behind the `json` feature flag (keeps the default binary lean): `cargo build --features json`. When the feature is disabled, calling either built-in returns an informative error message.
  - Both names appear in tab completion (`builtin_names`) regardless of the feature flag.
  - Backed by `serde_json = "1"` (optional dependency in `ccalc-engine`).
  - Example script: `examples/json/json.calc` (8 sections: primitives, arrays, objects, nesting, encoding, roundtrip, file I/O, dataset statistics).
  - In-REPL help: `help json`.
  - Docs: `docs/src/guide/json.md`, `docs/src/ccalc/phase20a-json.md`.

## [0.23.0] - 2026-04-25

### Added

- **Phase 19 — REPL tooling:**
  - **19a — Tab completion**: `Tab` key completes variable names and built-in function names in the REPL. Candidates are updated after each statement from the current environment plus the full built-in list. Implemented via a custom `rustyline` helper (`CcalcHelper`).
  - **19b — Inline help for user functions**: `%`-comment lines immediately **after** the `function` header (MATLAB H1-line style) are extracted as a doc string and stored in `Value::Function { doc }`. One leading space after `%` is stripped; remaining indentation is preserved. `help <name>` searches the workspace first, then triggers the autoload hook on demand so that `help bisect` works before `bisect()` is ever called. `resolve_autoloaded()` added to `ccalc-engine::eval` as a public API for this lookup.
  - **19c — "Did you mean?" suggestions**: Undefined-variable and unknown-function errors now suggest the closest known name (Levenshtein distance ≤ 2) from the current environment and the built-in function list.
  - **19d — Assertion built-ins**:
    - `assert(cond)` — throws `"assert: condition is false"` when the condition is falsy.
    - `assert(expected, actual)` — checks equality (scalars exact, matrices element-wise).
    - `assert(expected, actual, tol)` — checks `|expected - actual| ≤ tol` element-wise.
  - `builtin_names()` — new public function in `ccalc-engine::eval` returning the complete list of built-in function names (used for completion and suggestions).
  - **Block comments** (`%{ … %}` / `#{ … #}`): multi-line block comments are now stripped before parsing. The opening marker and all content up to the closing marker are replaced with blank lines (line numbers preserved for error reporting). Same-line `%{ … %}` is also supported. Unterminated block comments produce a parse error. `block_depth_delta` updated so the REPL correctly buffers lines inside `%{…%}` blocks.

## [0.22.0+003] - 2026-04-24

### Fixed

- `run()`/`source()` inside a script no longer aborts the outer script after the first call — all statements after the `run()` now execute correctly.
- Mixed script+function files (functions defined at the top, script body below) are now executed correctly; only files where *every* statement is a function definition are treated as pure function libraries.

### Changed

- `examples/file_io.calc` moved to `examples/file_io/file_io.calc` (self-contained example with its own `tmp/` scratch directory; `.gitignore` updated accordingly).

## [0.22.0] - 2026-04-24

### Added

- **Phase 18 — Advanced linear algebra (pure-Rust, no BLAS):**
  - **Decompositions:**
    - `[Q, R] = qr(A)` — QR decomposition via Householder reflectors; `R = qr(A)` returns R only.
    - `[L, U, P] = lu(A)` — LU factorisation with partial pivoting (PA = LU); `U = lu(A)` single-output.
    - `R = chol(A)` — Cholesky factor (upper triangular, A = R'*R); errors if not positive definite.
    - `[U, S, V] = svd(A)` — full SVD (U m×m, S m×n, V n×n); `s = svd(A)` returns singular values column vector.
    - `[U, S, V] = svd(A, 'econ')` — economy SVD (U m×k, S k×k, V n×k where k = min(m,n)).
    - `[V, D] = eig(A)` — eigenvalue decomposition (QR iteration with Wilkinson shift); `d = eig(A)` returns eigenvalue column vector.
  - **Matrix properties:**
    - `rank(A)` — numerical rank via SVD threshold.
    - `null(A)` — orthonormal basis for null space (right singular vectors for near-zero singular values).
    - `orth(A)` — orthonormal basis for column space (left singular vectors for non-zero singular values).
    - `cond(A)` — condition number (σ_max / σ_min); returns `Inf` for singular matrices.
    - `pinv(A)` — Moore–Penrose pseudoinverse via SVD.
  - **Updated `norm`:**
    - `norm(A)` — matrix 2-norm (largest singular value) for non-vector matrices; vector behaviour unchanged.
    - `norm(A, 'fro')` — Frobenius norm.
    - `norm(A, 1)` / `norm(A, Inf)` — max column-sum / max row-sum for matrices.
  - **`nargout` support:** new `set_nargout(n)` thread-local API lets multi-output built-ins
    (`eig`, `svd`, `lu`, `qr`) return either a single value or a `Value::Tuple` depending on
    the number of targets on the LHS.  Called by `exec_stmts` (for block/script contexts) and
    `evaluate()` (for REPL/pipe single-line context).
  - 25 regression tests added to `eval_tests.rs` covering all new functions.

## [0.21.0+018] - 2026-04-23

### Added

- **Phase 17e — Shape statistics:**
  - `skewness(v)` — population skewness coefficient: `m3 / m2^(3/2)`.
    Returns `0.0` for a scalar, single-element, or constant vector; `NaN` for
    empty input.  Column-wise on M×N matrices.
  - `kurtosis(v)` — population kurtosis: `m4 / m2^2`.  A standard normal
    distribution produces kurtosis ≈ 3; uniform data ≈ 1.8.  Returns `NaN`
    for n < 2 or constant input.  Column-wise on M×N matrices.
  - Both implemented as cases in `call_builtin` using the existing `apply_stat`
    helper.  No new tokens or AST nodes required.
  - 7 regression tests added to `eval_tests.rs` (symmetry, right-skew, scalar,
    constant data, kurtosis value, NaN cases).

- **ccalc-scripts:** `math/descriptive.calc` and `math/descriptive_demo.calc` —
  prints n, min, max, range, mean, median, mode, std, var, Q1, Q3, IQR,
  skewness, and kurtosis for any numeric vector.  Auto-loaded via the session
  path; no explicit `source` needed when run from the scripts folder.

## [0.21.0+017] - 2026-04-23

### Added

- **Phase 17c — Percentiles and distributions:**
  - `prctile(v, p)` — p-th percentile with linear interpolation; `p` may be a
    scalar (→ scalar) or a vector (→ row vector of same length).  For M×N matrix
    input `prctile` operates column-wise, returning an n\_p×N result.
  - `iqr(v)` — interquartile range (`prctile(75) - prctile(25)`), column-wise
    for matrices.
  - `zscore(v)` — standardise: `(v - mean(v)) / std(v)`; returns the same shape
    as the input.  Constant columns map to zero to avoid division by zero.

- **Phase 17d — Mathematical special functions:**
  - `erf(x)` — Gauss error function; delegates to the `libm` crate.
  - `erfc(x)` — complementary error function: `1 - erf(x)`.
  - `normcdf(x)` — standard normal CDF: `0.5 * (1 + erf(x / √2))`.
  - `normcdf(x, mu, sigma)` — general normal CDF.
  - `normpdf(x)` — standard normal PDF: `exp(-x²/2) / √(2π)`.
  - `normpdf(x, mu, sigma)` — general normal PDF.
  - All six functions work element-wise on scalars and matrices.
  - `libm = "0.2"` added as a dependency to `ccalc-engine`.

- `examples/statistics.calc` — 9-section demo covering all Phase 17 built-ins.
- `help stats` / `help random` / `help distribution` — new in-app help topic.

## [0.21.0+016] - 2026-04-23

### Added

- **Phase 17b — Descriptive statistics:**
  - `std(v)` / `std(v, 1)` — sample (n-1) and population (n) standard deviation.
  - `var(v)` / `var(v, 1)` — sample and population variance.
  - `median(v)` — median with linear interpolation for even-length inputs.
  - `mode(v)` — most frequent value; smallest wins on ties.
  - `cov(v)` — scalar variance of a vector (n-1 denominator).
  - `cov(A)` — N×N covariance matrix of an m×N data matrix.
  - `hist(v)` / `hist(v, n)` — ASCII bar chart to stdout; returns `Void`.
  - `histc(v, edges)` — bin counts matching MATLAB semantics.
  - All functions operate column-wise on M×N matrices.
  - Helper functions added to `eval.rs`: `numeric_vec`, `stat_var_vec`,
    `apply_stat`, `percentile_sorted`.

## [0.21.0+015] - 2026-04-23

### Added

- **Phase 17a — Random number generation:**
  - `rand()` / `rand(n)` / `rand(m, n)` — uniform [0, 1) scalars and matrices.
  - `randn()` / `randn(n)` / `randn(m, n)` — standard-normal samples via
    Box-Muller transform (no extra dependencies).
  - `randi(max)` / `randi(max, n)` / `randi(max, m, n)` — random integers in
    [1, max].  `randi([lo, hi], ...)` — arbitrary integer range.
  - `rng(seed)` — seed for reproducible output; returns `Void`.
  - `rng('shuffle')` — reseed from OS entropy.
  - Thread-local `SmallRng` (from `rand = "0.8"`, feature `small_rng`) seeded
    at startup from OS entropy.

### Fixed

- `rand`, `randn`, and `rng` added to the `no_ans_inject` list so that
  zero-argument calls do not silently receive `ans` as a phantom argument
  (previously `rand()` returned a 0×0 matrix instead of a scalar).

## [0.21.0+014] - 2026-04-22

### Fixed

- `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `exp`, `sqrt`, `log`, `log2`,
  `log10`, `floor`, `ceil`, `round`, `sign` are now element-wise on vectors and
  matrices (previously scalar-only). `Complex(re, 0)` is also accepted and
  treated as a real scalar. Five regression tests added.

### Added

- `lagrange_interp.calc` reference script: Lagrange polynomial interpolation.
  Published to `ccalc-scripts/math/`.

## [0.21.0+013] - 2026-04-22

### Fixed

- Matrix literal `[A, b]` and `[A; B]` now work when elements are matrices
  (previously only scalars and row vectors were accepted). The evaluator now
  performs full horizontal concatenation across a row and vertical concatenation
  across rows, matching MATLAB/Octave semantics. Five regression tests added.

### Added

- `gauss_elim.calc` reference script: Gaussian elimination with partial pivoting,
  solves Ax=b. Published to `ccalc-scripts/math/`.

## [0.21.0+012] - 2026-04-22

### Added

- `diag(v)` built-in: vector → square diagonal matrix; `diag(A)` → column
  vector of the main diagonal of a matrix. Handles row vectors, column vectors,
  square matrices, and non-square matrices (`min(rows, cols)` elements extracted).
  Scalar input returns a 1×1 matrix. Six regression tests added.

## [0.21.0+011] - 2026-04-21

### Added

- **Phase 16 — Package namespaces (`+pkg/` directories):**
  - Directories whose name starts with `+` (e.g., `+utils`, `+geom`) are
    packages. Functions inside are invisible at the top level and must be
    called with the package prefix: `utils.clamp(x, 0, 10)`.
  - Nested packages are supported: `+geom/+solid/sphere_vol.calc` is called
    as `geom.solid.sphere_vol(r)`.
  - Package functions are autoloaded on the first call — no `source()` needed.
    The search order is the calling script's directory, then CWD, then the
    session path.
  - New `Expr::DotCall(Vec<String>, Vec<Expr>)` AST node; parser detects
    `ident{.ident}*(` in the postfix loop and produces `DotCall`.
  - New `try_autoload_pkg()` in `exec.rs`: resolves qualified names to
    `+pkg/func.calc` paths and caches under the qualified name.
  - Example: `examples/scoping/scoping.calc` section 8 demonstrates packages
    with `+utils/` and `+geom/` package directories.
  - New `help scoping` / `help packages` topic.

- **Phase 15.6 — Variable scoping:**
  - **`global` variables**: `global x` declares a variable shared across all
    functions and the base workspace that declare the same name.
  - **`persistent` variables**: `persistent x` keeps a per-function value
    between calls. IndexSet and Assign on persistent variables now write
    through to the persistent store immediately so recursive callers see
    updates (fixes memoization patterns like `fib_memo`).
  - **`private/` directory scoping**: functions in a `private/` sub-directory
    are visible only to scripts and functions in the parent directory.
    `collect_dirs_recursive` skips `private/` directories; `resolve_script_path`
    only adds the `private/` look-aside for the calling script's own directory.
  - **`silence_all`**: recursive function in `exec.rs` walks the full statement
    tree and marks every statement silent so function bodies never print output.
  - **Single-line block fix**: the REPL's `if cond; body; end` bypass only
    activates at `block_depth == 0`, preventing premature execution inside
    buffered function definitions.
  - Example: `examples/scoping/scoping.calc` (in its own directory) covers
    global counters, global configuration, persistent call counters, persistent
    memoization, Welford running statistics, combined global+persistent, and
    `private/` directory helpers.

## [0.21.0+005] - 2026-04-20

### Added

- **Autoload**: calling an unknown function triggers automatic search for
  `<name>.calc` / `<name>.m` on CWD and session path — no explicit `source()`
  needed (MATLAB/Octave-compatible).
- **MATLAB-style local function scoping**: a function file may contain multiple
  `function` definitions; only the primary function is exposed to the caller's
  workspace; all other functions are local helpers, bundled inside
  `Value::Function.locals` and invisible outside the file.
- **`log2(x)`** and **`log10(x)`** built-ins added.
- **`Inf`** and **`NaN`** (capital variants) recognised as aliases for `inf` / `nan`.

### Fixed

- **`log(x)` now returns the natural logarithm** (base *e*), matching
  MATLAB/Octave. Previously it incorrectly computed log base 10.
- **Stack overflow on deep recursion** fixed: `main` is now spawned on a
  dedicated thread with a 64 MB stack, supporting hundreds of recursive
  function calls.

## [0.21.0] - 2026-04-20

### Added

- **Phase 15 — Indexed assignment** (`A(i,j) = v`, `v(1:3) = 0`):
  - **15a — Scalar and slice assignment**: `v(i) = x`, `v(1:3) = [1 2 3]`, `A(i,j) = x`, `A(:,j) = col`, `A(1:2,1:2) = eye(2)`. Scalar RHS is broadcast to all selected positions.
  - **15b — Growing vectors**: `v(end+1) = x` extends a row vector by one element (filling gaps with zeros); `v(n) = x` where `n > length(v)` pads to length `n`. Empty variable `v(i) = x` auto-creates a row vector.
  - **15c — Cell grow already supported** via Phase 12.5 `CellSet`.
  - **15d — Logical indexing**: a 0/1 vector whose length equals the dimension is treated as a boolean mask in both read (`v(v > 0)`) and write (`v(v < 0) = 0`) contexts.
  - New `Stmt::IndexSet { name, indices, value }` AST node; detected at parse time by `try_split_index_assign` (string-level lookahead, same strategy as `FieldSet`).
  - `exec_index_set` in `exec.rs` handles growth, broadcasting, and 2-D submatrix writes.

### Fixed

- **`zeros(n)` and `ones(n)` with a single argument** now correctly create an `n×n` matrix (previously required `zeros(n,n)` form).

## [0.20.0] - 2026-04-20

### Added

- **Phase 14 — Error handling:**
  - **`error(fmt, args...)`** — raises a runtime error with a printf-formatted message.
  - **`warning(fmt, args...)`** — prints a warning to stderr and continues execution.
  - **`lasterr()`** — returns the message from the most recent runtime error.
  - **`lasterr(msg)`** — sets the last-error string, returns the previous value.
  - **`try/catch/end` block** — MATLAB-compatible protected block; anonymous (`catch`) and named (`catch e`) forms; `e` is a struct with field `message`.
  - **`try(expr, default)`** — inline functional fallback; evaluates `default` only if `expr` raises an error.
  - **`pcall(@func, args...)`** — protected call; returns `[ok, result]` tuple where `ok=1` on success and `ok=0` with the error message on failure.
  - **`e` constant now variable-shadowing**: `e` (Euler's number) falls back gracefully when `e` is defined as a variable (e.g. `catch e`).

- **`genpath(dir)`** built-in: returns `dir` and all its subdirectories
  (recursively, sorted) as a path-separator-delimited string (`;` on Windows,
  `:` on Unix). Designed to be composed with `addpath`:
  `addpath(genpath('/my/libs'))`.
- **Trailing-slash convention in `config.toml`**: a `path` entry ending with
  `/` (or `\` on Windows) triggers genpath semantics at startup — the directory
  and all its subdirectories are added to the session search path.
  `path = ["~/.config/ccalc/lib/"]` is equivalent to calling
  `addpath(genpath('~/.config/ccalc/lib'))` at session start.

## [0.20.0] - 2026-04-18

### Added

- **Phase 13.6a — Backslash operator `\` (left division / linear solve):**
  - `a \ b` for scalars returns `b / a`.
  - `A \ b` for a square matrix `A` and column vector (or matrix) `b` solves the
    linear system `A * x = b` using Gaussian elimination with partial pivoting.
  - `scalar \ matrix` broadcasts as `matrix / scalar`.
  - Same operator precedence as `*` and `/` (left-associative).
- **Phase 13.6b — Path system (`addpath` / `rmpath` / `path`):**
  - `addpath('dir')` — prepend a directory to the session search path.
  - `addpath('dir', '-end')` — append instead of prepend.
  - `rmpath('dir')` — remove a directory from the session search path.
  - `path()` — display the current search path.
  - `run()` / `source()` now search the session path after the current directory.
  - `path` array in `~/.config/ccalc/config.toml` sets the initial path at startup.
  - `~` is expanded to the home directory (cross-platform).
  - 13 new regression tests.

## [0.19.0+003] - 2026-04-17

### Changed

- **`inv(A)` and `det(A)` upgraded to partial pivoting** (pure Rust, zero new
  dependencies). Both functions previously searched for the first non-zero pivot
  element; they now select the row with the largest absolute value (`max_by |abs|`),
  matching the strategy used by LAPACK `dgetrf`. This improves numerical stability
  for ill-conditioned matrices at no cost to portability.
- `--features blas` scope narrowed: the flag now accelerates only `A*B` matrix
  multiply (`ndarray/blas`). The `inv`/`det` path is pure Rust regardless of features.
- Benchmark `inv/{100,500}` added to `benches/engine.rs` to track `inv` performance
  (baseline: ~16 ms / ~240 ms on this machine, release build, no BLAS).

## [0.19.0+002] - 2026-04-17

### Added

- **Criterion benchmark suite** (`crates/ccalc-engine/benches/engine.rs`):
  - `scalar_ops_sum_1M` — `sum(1:1000000)`: range construction + 1 M reductions.
  - `fib/fib_30` — naive recursive `fib(30)` (~2.7 M interpreter calls); exercises
    function call overhead and body cache. Configured with `sample_size=10` and
    `measurement_time=90s` due to long per-iteration time (~7 s).
  - `loop_10k` — `for k=1:10000; s+=k; end`: interpreter loop throughput.
  - `matmul/{100,500,1000}` — `ones(N,N)*ones(N,N)` at three matrix sizes.
  - `fn_calls_1000` — 1 000 calls to a trivial 1-line named function via a loop.
  - HTML reports written to `target/criterion/` after each run.
  - Run all: `cargo bench`; run one: `cargo bench --bench engine -- loop_10k`.

## [0.19.0+001] - 2026-04-16

### Added

- **Phase 13.5 — Struct arrays** (`Value::StructArray(Vec<IndexMap<String, Value>>)`):
  - Element assignment: `s(i).field = val` creates or grows a 1-based struct array;
    `s(i).a.b = val` sets nested fields.
  - Array indexing read: `s(i)` returns element `i` as a `Value::Struct`.
  - Field collection: `s.field` across a struct array returns a `1×N` matrix when
    all values are scalar, or a cell array otherwise.
  - `s(:)` returns the full struct array unchanged.
  - Built-ins extended: `isstruct`, `fieldnames`, `isfield`, `rmfield`, `numel`,
    `size`, `length` all handle `StructArray`.
  - Display: `[1×N struct]` inline; multi-line shows field names for N>1, full
    field values for N=1 (same as scalar struct display).
  - 8 regression tests added covering creation, read, field collection, `numel`,
    `isstruct`, `fieldnames`, auto-growing, and mixed-type field collection.

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
