# Phase 11.5 — Extended Control Flow

**Version:** 0.16.0  
**Status:** ✅ Complete

## Motivation

Phase 11 added the core control flow constructs (`if`, `for`, `while`). Phase 11.5
fills in the remaining Octave constructs most commonly found in real `.m` scripts:
`switch`, `do...until`, and the ability to source another script file with `run()`.

## Phase 11.5a — `switch / case / otherwise` (v0.16.0)

### Semantics

`switch` in Octave/MATLAB has **no fall-through**: the first matching `case`
executes and control jumps directly to `end`. `otherwise` is the optional
default branch.

```matlab
switch expr
  case val1
    % ...
  case val2
    % ...
  otherwise
    % ...
end
```

### Matching rules

| Switch value | Case value | Match |
|---|---|---|
| Scalar | Scalar | exact `==` |
| `Str` or `StringObj` | `Str` or `StringObj` | string equality (`Str` and `StringObj` interchangeable) |
| Any other combination | — | no match |

### `break` and `continue`

`break`/`continue` inside a `switch` body propagate **outward** to the nearest
enclosing loop. `switch` itself is not a loop.

### AST

```rust
Stmt::Switch {
    expr: Expr,
    cases: Vec<(Vec<Expr>, Vec<(Stmt, bool)>)>,
    otherwise_body: Option<Vec<(Stmt, bool)>>,
}
```

Each case carries a `Vec<Expr>` for future multi-value case support (`case {1,2,3}`
from Phase 11.5b, deferred until cell arrays are introduced in a later phase).

### Example

```matlab
switch code
  case 200
    msg = 'OK';
  case 301
    msg = 'Moved Permanently';
  case 404
    msg = 'Not Found';
  otherwise
    msg = 'Unknown';
end
fprintf('HTTP %d: %s\n', code, msg)
```

## Phase 11.5b — Multi-value cases (deferred)

`case {val1, val2}` syntax requires `Value::Cell`, which is not yet implemented.
Deferred until cell arrays are introduced.

## Phase 11.5c — `do...until` (v0.16.0)

### Semantics

Octave-specific post-test loop. The body **always executes at least once**, then
the condition is tested. If truthy, the loop exits.

```matlab
do
  body
until (cond)
```

Parentheses around `cond` are optional. `break` exits the loop immediately;
`continue` re-tests the condition.

`until` closes the block without a separate `end`. In the REPL,
`block_depth_delta("until …")` returns `−1`.

### AST

```rust
Stmt::DoUntil {
    body: Vec<(Stmt, bool)>,
    cond: Expr,
}
```

### Execution (`exec.rs`)

```rust
Stmt::DoUntil { body, cond } => loop {
    match exec_stmts(body, env, io, fmt, base, compact)? {
        Some(Signal::Break) => break,
        Some(Signal::Continue) | None => {}
    }
    if is_truthy(&eval_with_io(cond, env, io)?) {
        break;
    }
},
```

### Example

```matlab
% Smallest power of 2 >= n
n = 100;
p = 1;
do
  p *= 2;
until (p >= n)
fprintf('%d\n', p)   % 128
```

## Phase 11.5d — `try/catch` (deferred)

Deferred to **Phase 14** (after structs). `try/catch` requires `Value::Struct`
for the error object, and the error-handling model is under design review.

## Phase 11.5e — Script sourcing `run()` / `source()` (v0.16.0)

### Semantics

Execute a script file in the **caller's workspace** (MATLAB `run` semantics).
Variables defined in the script persist after `run` returns. This is the opposite
of a function call, which would have an isolated scope.

```matlab
run('script')         % search for script.calc, then script.m in CWD
run('script.calc')    % explicit .calc extension
run('script.m')       % explicit .m extension
source('script')      % Octave alias — identical behaviour
```

### Extension resolution for bare names

When no file extension is given, ccalc tries:

1. `<name>.calc` — native ccalc script format (preferred)
2. `<name>.m` — Octave/MATLAB compatibility

Explicit extensions (`.calc`, `.m`, or any other) are used verbatim.

### Implementation

`run()`/`source()` are intercepted in `exec_stmts` by pattern-matching
`Stmt::Expr(Expr::Call("run"|"source", args))` **before** calling `eval_with_io`.
This avoids needing a new AST node:

```rust
if let Expr::Call(fn_name, args) = expr
    && matches!(fn_name.as_str(), "run" | "source")
    && args.len() == 1
{
    // resolve path → read file → parse_stmts → exec_stmts (recursive)
}
```

In pipe/script mode, single-line statements bypass `exec_stmts` and go through
`evaluate()`. A `try_run_source()` helper in `repl.rs` bridges that gap by
routing `run`/`source` calls through `exec_stmts` before `evaluate()` is reached.

### Recursion limit

A `thread_local! RUN_DEPTH` counter prevents infinite recursion. Maximum depth
is 64; exceeding it returns an error.

### Example

```matlab
% euclid_helper.calc — reads a, b from workspace; writes g = gcd(a, b)
g = a;
r = b;
while r ~= 0
  temp = mod(g, r);
  g = r;
  r = temp;
end
```

```matlab
% caller
a = 252; b = 105;
run('euclid_helper')
fprintf('gcd(252, 105) = %d\n', g)   % 21
```

## REPL block depth

`block_depth_delta` was updated for the new keywords:

| Line starts with | Delta |
|---|---|
| `switch` | +1 |
| `do` | +1 |
| `until …` | −1 |
| `end` | −1 (unchanged) |

## Demo

```bash
cd examples
ccalc extended_control_flow.calc
```

The example covers all constructs from this phase: `switch` integer and string
dispatch, `do...until` with break/continue, Euclidean GCD via `run()`, and the
`source()` alias.
