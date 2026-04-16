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
| 7.5 | Vector utilities, `end` indexing, `NaN`/`Inf`, `sort`, `find` | ✅ Done |
| 8 | Complex numbers (`3 + 4i`, `abs(z)`, `angle(z)`) | ✅ Done |
| 9 | String data types (`'char array'`, `"string object"`) | ✅ Done |
| 10 | C-style I/O (`fprintf('%.2f\n', x)`, `sprintf`) | ✅ Done |
| 10.5 | File I/O (`fopen`, `dlmread`, `isfile`, `save`/`load` with path) | ✅ Done |
| 11 | Core control flow (`if`, `for`, `while`, `break`, `continue`, `+=`) | ✅ Done |
| 11.5 | Extended control flow (`switch`, `do...until`, `run`/`source`; `try`/`catch` deferred to Phase 14) | ✅ Done |
| 12 | User-defined functions, multiple return values, `@(x)` lambdas | ✅ Done |
| 12.5 | Cell arrays, `varargin`/`varargout`, `cellfun`/`arrayfun`, `@funcname` | ✅ Done |
| 12.6 | Language polish: `&`/`\|`, `...`, single-line blocks, `.'`, `**`, string utils | ✅ Done |
| 13 | Scalar structs (`s.field`, `struct()`, `fieldnames`, `isfield`, `rmfield`) | ✅ Done |
| 13.5 | Struct arrays (`s(i).field`, field collection, `numel`/`isstruct` extended) | ✅ Done |

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

**Phase 7.5** adds special floating-point constants (`nan`, `inf` as
parser-level constants), `isnan`/`isinf`/`isfinite` built-ins, vector
reductions (`sum`, `prod`, `cumsum`, `any`, `all`, 1-arg `min`/`max`,
`mean`, `norm`), the `end` keyword in indexing contexts (`v(end)`,
`A(1:end, 2)` — `env_with_end()` injects the dimension size into a cloned
env before evaluating index expressions), and data utility functions
(`sort`, `reshape`, `fliplr`, `flipud`, `find`, `unique`).
No new `Value` variants are needed.

**Phase 8** adds `Value::Complex(f64, f64)` as a third `Value` variant.
No new tokens are required — `4i` already parses as implicit multiplication
`4 * i`, where `i` and `j` are pre-seeded in `Env` as `Complex(0.0, 1.0)`.
`complex_binop()` handles all arithmetic combinations; integer powers use
binary exponentiation for exact results (`i^2 = -1` exactly); non-integer
powers use the polar form `exp((c+di)·ln(a+bi))`.
`make_complex(re, im)` collapses to `Scalar(re)` when `im == 0.0` exactly.
`z'` (conjugate transpose) returns the complex conjugate for scalar complex.
Built-ins added: `real`, `imag`, `abs` (overloaded), `angle`, `conj`,
`complex`, `isreal`. `scalar_arg` accepts `Complex` with `im == 0` as a
real scalar. Complex matrices are out of scope and deferred.

**Phase 9** adds two string value types. `Value::Str(String)` represents
single-quoted char arrays; `Value::StringObj(String)` represents double-quoted
string objects. The `'` disambiguation — transpose vs char array literal — is
resolved by one token of left context in the tokenizer: after
`Number`/`Ident`/`RParen`/`RBracket`/`Apostrophe`/`Str` it is a transpose;
otherwise it opens a char array. Char array arithmetic converts characters to
their ASCII codes before the operation, matching MATLAB behaviour.
String objects use `+` for concatenation and `==`/`~=` for comparison.
New built-ins: `num2str`, `str2num`, `str2double`, `strcat`, `strcmp`,
`strcmpi`, `lower`, `upper`, `strtrim`, `strrep`, `sprintf` (1-arg),
`ischar`, `isstring`. `length`/`numel`/`size` updated for strings.

**Phase 10** adds `fprintf(fmt, ...)` and `sprintf(fmt, ...)` using the
string infrastructure from Phase 9. The ad-hoc `p`/`p<N>` precision command
is deprecated and removed in this phase. Placed before control flow so that
loop scripts have formatted output from the start.

**Phase 10** adds `fprintf(fmt, ...)` and `sprintf(fmt, ...)`.
The ad-hoc `p`/`p<N>` precision command is removed. Phase 10.5 extends this
with file I/O (`fopen`/`fclose`/`fgetl`/`fgets`), data files (`dlmread`/`dlmwrite`),
filesystem queries (`isfile`, `isfolder`, `pwd`, `exist`), and workspace
persistence with explicit paths (`save('f.mat')`, `load('f.mat')`).

**Phase 11** adds multi-line REPL buffering and core control flow constructs:
`if`/`elseif`/`else`, `for` (column iteration), `while`, `break`, `continue`.
Compound assignment operators (`+=`, `-=`, `*=`, `/=`, `++`, `--`) are
desugared at parse time to plain `Stmt::Assign` — no new AST nodes.
Syntax aliases: `#` for `%` (comment), `!` for `~` (NOT), `!=` for `~=`.
Extended control flow (`switch`, `do...until`, `run`/`source`) was completed in Phase 11.5.

**Phase 11.5** adds `switch`/`case`/`otherwise` (no fall-through; scalar exact `==`,
string equality), `do...until` (Octave post-test loop; `until` closes the block
without `end`), and `run()`/`source()` script sourcing (MATLAB run semantics —
scripts execute in the caller's workspace). Extension resolution for bare names
tries `.calc` (native ccalc format) before `.m` (Octave/MATLAB compatibility).
A `thread_local! RUN_DEPTH` counter caps recursion at 64 levels.
`try`/`catch` is deferred to Phase 14 (after structs).

**Phase 12** adds user-defined functions with single and multiple return
values (`[a, b] = f(x)`), and anonymous functions `@(x) expr` (closures).

Named functions use `function [out1, out2] = name(p1, p2) ... end` syntax and
are stored as `Value::Function { outputs, params, body_source }` in `Env`.
`body_source` is stored as a string and re-parsed on each call to avoid a
circular dependency between `eval.rs` and `parser.rs`.
Anonymous functions are stored as `Value::Lambda(Rc<dyn Fn>)` — a closure
compiled at definition time that captures the enclosing environment lexically.
A thread-local `FnCallHook` bridges `eval.rs` (which dispatches the call) and
`exec.rs` (which executes the body in an isolated scope).
Each call gets a fresh environment seeded with `i`, `j`, `ans`, the declared
parameters, `nargin`, and all callable values (`Function`/`Lambda`) from the
caller's workspace — the last point enables self-recursion and mutual recursion
without exposing caller data.
`Stmt::FunctionDef`, `Stmt::Return`, `Stmt::MultiAssign`, and `Token::At` were
added to the parser; `Signal::Return` to the executor.

**Phase 12.5** adds `Value::Cell(Vec<Value>)` — a heterogeneous 1-D cell array.
Cell literals `{e1, e2}`, brace indexing `c{i}`, and brace assignment `c{i} = v`
use new `Token::LBrace`/`RBrace` and `Expr::CellLiteral`/`CellIndex`/`Stmt::CellSet`.
`varargin`/`varargout` collect extra call arguments into a `Value::Cell`.
`case {v1, v2}` multi-value switch cases iterate the cell and test each element.
`cellfun(f, c)` and `arrayfun(f, v)` apply a function to each element.
`@funcname` desugars to `Expr::FuncHandle(name)` — a lambda wrapping any named function.
`split_stmts()` updated to track brace depth so `;` inside `{...}` is not a separator.

**Phase 12.6** delivers language polish across nine sub-items (12.6h deferred):
- **12.6a** Single-line blocks: `if cond; body; end` — `is_single_line_block()`
  detects self-contained blocks; REPL/pipe bypass the block buffer for them.
- **12.6b** `...` line continuation: `cont_buf` in REPL and `run_pipe`;
  `join_line_continuations()` pre-pass in `parse_stmts`; tokenizer drains rest of line.
- **12.6c** `&`/`|` element-wise logical: new `Token::Amp`/`Pipe`, `Op::ElemAnd`/`ElemOr`,
  and `parse_elem_or`/`parse_elem_and` precedence levels between `parse_logical_and` and
  `parse_comparison`.
- **12.6d** `xor(a,b)` and `not(a)` built-ins.
- **12.6e** Lambda display: `LambdaFn` carries a source string; `expr_to_string()` helper
  reconstructs readable source text from the AST at parse time.
- **12.6f** `strsplit(s[,delim])`, `int2str(x)`, `mat2str(A)` built-ins.
- **12.6g** `.'` plain transpose: `Token::DotApostrophe`, `Expr::PlainTranspose` — no
  complex conjugation (contrast with `'` which is the Hermitian conjugate transpose).
- **12.6j** Unary `+` (no-op), `**` alias for `^` (Octave), `,` non-silent separator.
- **Bug fixes**: `4i` imaginary literal now works via tokenizer `push_imag_suffix()`;
  `split_stmts` `'` disambiguation extended to recognise `.` as a transpose indicator
  (fixing `B.';` mis-parse); `run_pipe` gained `cont_buf` for `...` continuation.

**Phase 13** adds `Value::Struct(IndexMap<String, Value>)` — scalar structs
with insertion-order-preserving fields (using the `indexmap` crate).
`Token::Dot` is emitted only when `.` is followed by an ASCII letter/underscore,
leaving `DotStar`/`DotSlash`/`DotCaret`/`DotApostrophe` unaffected.
`Expr::FieldGet` handles chained reads (`s.a.b`); `Stmt::FieldSet(String,
Vec<String>, Expr)` handles writes with arbitrary depth paths via the
`set_nested()` recursive helper in `exec.rs`.
Built-ins: `struct()`, `fieldnames`, `isfield`, `rmfield`, `isstruct`.
19 regression tests added; 488 total.

**Phase 13.5** adds `Value::StructArray(Vec<IndexMap<String, Value>>)` — a
separate variant for 1-D arrays of structs, keeping `Value::Struct` for scalar
structs unchanged. `s(i).field = val` is intercepted at string level by
`try_split_struct_array_field_assign()` before tokenization and parsed into a
new `Stmt::StructArrayFieldSet(base, idx_expr, path, rhs)` statement. The
executor in `exec.rs` resolves the index, grows the array if needed (filling
gaps with empty field maps), and calls the existing `set_nested()` helper to
write nested field paths. `s(i)` indexing returns a `Value::Struct` for a
single element and a `Value::StructArray` for a slice or `:`. `s.field` on a
struct array collects the field across all elements, returning `Value::Matrix`
when all elements are scalar, or `Value::Cell` when types are mixed.
Extended built-ins: `isstruct`, `fieldnames`, `isfield`, `rmfield`, `numel`,
`size`, `length`. 8 regression tests added.

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
