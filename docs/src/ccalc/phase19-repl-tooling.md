# Phase 19 — REPL Tooling

Introduced in **v0.23.0**.

Phase 19 adds four developer-experience features: tab completion, inline function
help, "did you mean?" error hints, and assertion built-ins.

---

## 19a — Tab completion

Press `Tab` in the REPL to complete the current word against:

- All **variable names** defined in the current session.
- All **~90 built-in function names** (`sqrt`, `mean`, `assert`, …).

When multiple candidates match, they are listed and the longest common prefix is
inserted. Type more characters and press `Tab` again to narrow down.

```
>> inv<Tab>       → inv(
>> my_fun<Tab>    → my_function   (if defined)
```

Tab completion is an interactive REPL feature and cannot be demonstrated in a
script.

**Implementation**: `rustyline` is upgraded from `DefaultEditor` to a typed
`Editor<CcalcHelper, DefaultHistory>`. `CcalcHelper` implements the `Completer`
trait with prefix-based matching over `env.keys()` and `builtin_names()`.
`Hinter`, `Highlighter`, and `Validator` are required no-op stubs (rustyline
demands all four traits). The helper is updated before each `readline()` call
so newly defined variables appear immediately.

---

## 19b — Inline help for user functions

Any function prefixed by consecutive `%`-comment lines (with no blank line
between the comments and the `function` keyword) gets those lines as its doc
string. `help <name>` in the REPL prints it.

```matlab
% Return the nth triangular number T(n) = n*(n+1)/2.
% Usage: t = tri(n)
%
% Example:
%   tri(4)  →  10
function t = tri(n)
  t = n * (n + 1) / 2;
end
```

```
>> help tri
Return the nth triangular number T(n) = n*(n+1)/2.
Usage: t = tri(n)

Example:
  tri(4)  →  10
```

- Any number of consecutive `%` (or `#`) lines form the doc block.
- A **blank line** between the comment block and the `function` keyword breaks
  the association — only lines that directly precede the keyword are collected.

**Implementation**: `Stmt::FunctionDef` and `Value::Function` gain an
`Option<String>` `doc` field. `parse_stmts_from_lines` scans backward from the
`function` keyword through raw (un-stripped) lines, collecting comment text until
it hits a non-comment line. The REPL `help <name>` handler checks
`Value::Function { doc: Some(d), .. }` before falling through to built-in topics.

---

## 19c — "Did you mean?" error hints

When a name is not found, ccalc computes the Levenshtein edit distance from the
misspelled name to every variable in the current environment and every built-in
function name. If the closest match is within 2 edits, it is appended to the
error message.

```
>> sqrtt(4)
Error: Unknown function 'sqrtt'; did you mean 'sqrt'?

>> my_valu + 1
Error: Undefined variable 'my_valu'; did you mean 'my_value'?
```

No suggestion is printed when no close match exists.

**Implementation**: `levenshtein(a, b)` — O(m × n) DP implementation, no external
crate. `suggest_similar(name, env)` in `eval.rs` iterates `env.keys()` and
`builtin_names()`, picks the minimum, and returns `Some(name)` when ≤ 2.
The hint is appended inline in the "Undefined variable" branch of `eval` and in
the "Unknown function" fallthrough of `call_builtin`.

---

## 19d — `assert` built-ins

Three overloads for lightweight unit testing inside scripts:

| Call | Behaviour |
|---|---|
| `assert(cond)` | Pass when `cond` is truthy; error otherwise |
| `assert(expected, actual)` | Exact element-wise equality check |
| `assert(expected, actual, tol)` | Tolerance check: `|expected - actual| <= tol` |

All three work on scalars, vectors, and matrices.

```matlab
assert(pi > 3)
assert(4, 2 + 2)
assert(0.3333, 1/3, 1e-4)
assert([1 4 9], [1 2 3].^2)
```

**Implementation**: Three cases added to `call_builtin` in `eval.rs` keyed on
`("assert", 1)`, `("assert", 2)`, `("assert", 3)`. The shared
`assert_values_equal(a, b, tol)` helper handles shape checking and element-wise
comparison for both scalars and matrices.

---

---

## 19e — `near line N` in error messages

Introduced in **v0.30.0+002**.

Runtime errors that occur inside block statements, function bodies, or scripts
executed via `run()`/`source()` now include the 1-based source line number of
the failing statement:

```
Error: Undefined variable: 'bad_var' near line 3
```

### Where it applies

| Context | Has line number? |
|---------|-----------------|
| Inside `for`/`while`/`if`/`switch`/`do`-`until` body | ✓ |
| Inside `try`/`catch` body | ✓ |
| Inside a user-defined function body | ✓ |
| Script run via `run('file.m')` or `source('file.m')` | ✓ |
| Single statements typed at the REPL or piped line-by-line | — |

### `try/catch` and `e.message`

The catch variable stores the **original** message without the line suffix,
matching MATLAB/Octave semantics. Location information in Octave lives in
`e.stack.line`; in ccalc it is only surfaced in the printed error.

```matlab
try
  x = bad_var;
catch e
  disp(e.message)   % "Undefined variable: 'bad_var'"  (no "near line")
end
```

### Innermost location wins

When errors propagate through nested blocks, the location of the innermost
failing statement is reported and outer wrappers do not re-annotate:

```matlab
for k = 1:3
  if bad_var > 0   % line 2 — this line is reported
    x = 1;
  end
end
% Error: Undefined variable: 'bad_var' near line 2
```

**Implementation**: `(Stmt, bool)` throughout the AST became `(Stmt, bool, usize)`.
`parse_stmts_from_lines` records `*pos + 1` (1-based) at the start of each loop
iteration. Single-line block expansions (`if cond; body; end`) remap all virtual
inner lines back to the physical source line. `exec_stmts` wraps `eval_with_io`
calls with `.map_err(|e| annotate_line(e, stmt_line))?`; `annotate_line` is a
no-op when `line == 0` (synthetic REPL statements) or when the message already
contains `"near line"`.

---

## Example

```bash
ccalc examples/repl_tooling.calc
```

The example file demonstrates assert forms, doc-comment-driven test harnesses,
and "did you mean?" error recovery.

See also: [User-defined Functions](../guide/user-functions.md),
[Error Handling](../guide/error-handling.md), [`help testing`](../guide/repl.md).
