# Octave Compatibility Roadmap

ccalc aims for maximum practical compatibility with Octave/MATLAB `.m` files.
The work is divided into phases in order of architectural dependency.

## Phase summary

| Phase | Goal | Status |
|---|---|---|
| 1 | Variables and assignment (`x = 5`, `who`, `clear`, `ws`/`wl`) | ✅ Done |
| 2 | Multi-argument functions (`atan2`, `mod`, `max`, `min`) | ✅ Done |
| 3 | Matrix literals (`[1 2 3]`, `[1; 2; 3]`) | ✅ Done |
| 4 | Matrix operations (`A * B`, `A'`, `A .* B`) | ✅ Done |
| 5 | Range operator (`1:5`, `1:2:10`, `linspace`) | ✅ Done |
| 6 | Indexing (`A(1,1)`, `v(2:4)`) | ✅ Done |
| 7 | Comparison and logical operators (`==`, `~=`, `&&`) | ✅ Done |
| 7.5 | Vector utilities, `end` indexing, `NaN`/`Inf`, `sort`, `find` | Planned |
| 8 | Complex numbers (`3 + 4i`, `abs(z)`, `angle(z)`) | Planned |
| 9 | String data types (`'char array'`, `"string object"`) | Planned |
| 10 | C-style I/O (`fprintf('%.2f\n', x)`, `sprintf`) | Planned |
| 11 | Control flow (`if`, `for`, `while`, `switch`, `try`/`catch`, `+=`) | Planned |
| 12 | User-defined functions, multiple return values, `@(x)` lambdas | Planned |

## Key architectural decisions

**Phase 1** introduced `Env` (`HashMap<String, f64>`) and `Stmt`
(assignment vs expression), which are load-bearing for every subsequent phase.

**Phase 2** migrated `Expr::Call(String, Box<Expr>)` to
`Expr::Call(String, Vec<Expr>)` and introduced `call_builtin` dispatch.
New functions: `atan2`, `mod`, `rem`, `max`, `min`, `hypot`, `log(x,base)`,
`asin`, `acos`, `atan`, `sign`. Empty args `fn()` still passes `ans`.

**Phase 3** adds `ndarray` and a `Value` enum (`Scalar(f64)` | `Matrix(...)`),
migrating `Env` from `f64` to `Value`. Matrix literals `[1 2; 3 4]`,
element-wise arithmetic with scalars, and matrix `+`/`-` are implemented.
`split_stmts()` in `repl.rs` became bracket-depth-aware so `;` inside
`[...]` is parsed as a row separator, not a statement separator.

**Phase 4** adds matrix multiplication (`A * B` via ndarray `.dot()`),
postfix transpose (`A'` — new token `Apostrophe`, new `Expr::Transpose`),
and element-wise operators `.*`, `./`, `.^` (new tokens `DotStar`,
`DotSlash`, `DotCaret`). New built-ins: `zeros(m,n)`, `ones(m,n)`, `eye(n)`,
`size(A)`, `size(A,dim)`, `length(A)`, `numel(A)`, `trace(A)`, `det(A)`
(Gaussian elimination), `inv(A)` (Gauss-Jordan). `split_stmts()` updated to
distinguish transpose `'` from string-literal `'` by left-context.
`call_builtin` refactored to return `Result<Value, String>` directly.

**Phase 5** adds `Token::Colon` and `Expr::Range(start, step?, stop)`. A new
`parse_range()` layer sits above `parse_expr()` with lower precedence, so
`1+1:5` = `2:5`. The `Expr::Matrix` evaluator is updated to concatenate
row-vector elements horizontally, making `[1:5]` work. New built-in:
`linspace(a, b, n)`.

**Phase 6** adds `Expr::Colon` and `parse_call_arg()`. The `Expr::Call`
evaluator checks `Env` first: if the name resolves to a variable, the
expression is treated as indexing (variables shadow built-in function names,
matching Octave semantics). `eval_index()` + `resolve_dim()` handle 1D
(column-major linear) and 2D indexing, all 1-based. A bug fix also landed
here: range expressions inside grouping parentheses `(a:b)` now parse
correctly.

**Phase 7** adds comparison tokens (`==`, `~=`, `<`, `>`, `<=`, `>=`) and
logical operators (`~`, `&&`, `||`). Comparisons return `0.0`/`1.0` and work
element-wise on matrices. New parse levels `parse_logical_or` →
`parse_logical_and` → `parse_comparison` sit above `parse_range` in the
precedence hierarchy. `Expr::UnaryNot` and `Op::Eq/NotEq/Lt/Gt/LtEq/GtEq/And/Or`
are added to the AST.

**Phase 7.5** adds special floating-point constants (`nan`, `inf` pre-seeded
in `Env`), `isnan`/`isinf`/`isfinite` built-ins, vector reductions (`sum`,
`prod`, `cumsum`, `any`, `all`, 1-arg `min`/`max`, `mean`, `norm`), the
`end` keyword in indexing contexts (`v(end)`, `A(1:end, 2)` — requires a
new `Expr::End` AST node and context-passing in `eval_index`), and data
utility functions (`sort`, `reshape`, `fliplr`, `flipud`, `find`, `unique`).
No new `Value` variants are needed.

**Phase 8** adds `Value::Complex(f64, f64)` as a third `Value` variant.
No new tokens are required — `4i` already parses as implicit multiplication
`4 * i`, where `i` is pre-seeded in `Env` as `Complex(0.0, 1.0)`. The phase
adds complex arithmetic in `eval_binop`, display formatting, and built-ins
`real`, `imag`, `abs`, `angle`, `conj`, `complex`, `isreal`. Complex matrices
are out of scope and deferred.

**Phase 9** adds string types: `Value::CharMatrix` (single-quote char arrays,
numeric-compatible) and `Value::StringMatrix` (double-quote string objects).
The `'` disambiguation — transpose vs string literal — is resolved by one
token of left context in the tokenizer.

**Phase 10** adds `fprintf(fmt, ...)` and `sprintf(fmt, ...)` using the
string infrastructure from Phase 9. The ad-hoc `p`/`p<N>` precision command
is deprecated and removed in this phase. Placed before control flow so that
loop scripts have formatted output from the start.

**Phase 11** adds multi-line input buffering to the REPL and six control
flow constructs: `if`/`elseif`/`else`, `for`, `while`, `switch`
(scalar and string `case` values; `case {…}` deferred to cell arrays),
`do...until` (Octave-only), and `try`/`catch` error handling.
Compound assignment operators (`+=`, `-=`, `*=`, `/=`, `++`, `--`) are
desugared at parse time — no new AST nodes.
Multi-value `case {2, 3}` requires cell arrays and is deferred.

**Phase 12** adds user-defined functions with single and multiple return
values (`[a, b] = f(x)`), and anonymous functions `@(x) expr` (closures).
Requires Phase 11 for `return`, `break`, and multi-statement bodies.

## Compatibility notes

- `%` is a **comment** character (Octave/MATLAB convention). It terminates
  tokenization at that point. This is already implemented.
- `ans` is the sole implicit variable (Octave/MATLAB convention). The old
  accumulator (`acc`) and memory cells (`m1`–`m9`) were removed in Phase 1.
- 1-based indexing for matrices (Octave/MATLAB convention) — implemented in Phase 6.
- Where MATLAB and Octave differ, ccalc follows the **modern MATLAB standard
  (R2016b+)**.
- Full toolbox compatibility (Signal Processing, Optimization, etc.) is
  out of scope.
