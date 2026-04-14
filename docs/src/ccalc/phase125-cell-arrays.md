# Phase 12.5 — Cell Arrays

**Version:** v0.17.0+005  
**Prerequisite:** Phase 12 (lambdas needed for `cellfun`/`arrayfun`)  
**Trigger:** Three features blocked on `Value::Cell`:
`case {2, 3}` (Phase 11.5b), `varargin`/`varargout` (deferred from Phase 12),
and `cellfun`/`arrayfun` (higher-order built-ins).

---

## 12.5a — Core cell array infrastructure

### `Value::Cell`

Added to `env.rs` after `Value::Tuple`:

```rust
/// Heterogeneous 1-D container: each element may be any `Value`.
Cell(Vec<Value>),
```

Only 1-D cell vectors for now (`Vec<Value>`). 2-D cell arrays deferred.

### New tokens and AST nodes

**Tokens** (`parser.rs`):
- `Token::LBrace` (`{`) and `Token::RBrace` (`}`)

**Expr variants** (`eval.rs`):
```rust
CellLiteral(Vec<Expr>)        // {e1, e2, e3}
CellIndex(Box<Expr>, Box<Expr>)  // c{i}
FuncHandle(String)             // @funcname
```

**Stmt variant** (`parser.rs`):
```rust
CellSet(String, Expr, Expr)   // c{i} = v
```

### Parser changes

- `parse_primary`: `{` → comma-separated `parse_logical_or` elements → `Expr::CellLiteral`.
- After parsing an identifier, if next token is `LBrace` → `Expr::CellIndex`.
- `c{i} = v` detected in `parse()` lookahead via `try_split_cell_assign()`, produces `Stmt::CellSet`.
- `split_stmts()` now tracks `brace_depth` alongside `bracket_depth` so `;` inside `{...}` is not treated as a statement separator.

### Evaluator changes (`eval.rs`)

- `Expr::CellLiteral` → `Value::Cell(vals)`
- `Expr::CellIndex` → bounds-check (1-based), returns element value
- `Expr::FuncHandle(name)` → `Value::Lambda` that looks up `name` in caller env then falls back to builtins

### Executor changes (`exec.rs`)

- `Stmt::CellSet` → look up cell in env, update element at index, auto-grow if needed
- `is_truthy` extended: `Value::Cell(v) => !v.is_empty()`

### Built-ins

| Function | Description |
|---|---|
| `iscell(v)` | `1.0` if `v` is `Value::Cell`, else `0.0` |
| `cell(n)` | `Value::Cell` of `n` slots, each `Scalar(0.0)` |
| `cell(m, n)` | `Value::Cell` of `m*n` slots |
| `numel(c)` | element count of a cell |
| `length(c)` | same as `numel` for 1-D |
| `size(c)` | `[1, numel(c)]` as 1×2 matrix |

### Display

```
c =
  {
    [1,1]: 42
    [1,2]: hello
    [1,3]: [1×3 double]
  }
```

---

## 12.5b — `varargin` / `varargout`

### `varargin`

When a user function's last parameter is named `varargin`, all extra call
arguments are collected into a `Value::Cell` and bound to `varargin` in the
local scope. Fixed parameters are bound normally.

```matlab
function s = sum_all(varargin)
  s = 0;
  for k = 1:numel(varargin)
    s += varargin{k};
  end
end

sum_all(1, 2, 3)    % varargin = {1, 2, 3}  →  6
sum_all()           % varargin = {}          →  0
```

### `varargout`

When the sole output variable is `varargout`, after the function body executes,
its cell elements are unpacked as return values:

```matlab
function varargout = swap(a, b)
  varargout{1} = b;
  varargout{2} = a;
end

[x, y] = swap(10, 20)   % x = 20, y = 10
```

### Implementation note (injection fix, v0.17.0+005)

The parser previously injected `Expr::Var("ans")` into every empty `f()` call
at the AST level. This made `sum_all()` and `sum_all(1)` indistinguishable
inside `call_user_function` when `fixed_params.is_empty()`, causing varargin
to always be empty.

**Fix:** injection moved from parser to eval-time:
- Builtins and lambdas: inject `ans` when called with empty args (`f()` = `f(ans)`)
- `Value::Function` (user functions): no injection — empty call = no arguments

---

## 12.5c — `case {val1, val2}` in switch

Completes Phase 11.5b. When a switch `case` value evaluates to `Value::Cell`,
the evaluator iterates its elements and tests each with `==` against the switch
expression. First match wins; no fall-through (same semantics as scalar case).

```matlab
switch x
  case {2, 3}
    disp('two or three')
  case {10, 20, 30}
    disp('ten, twenty, or thirty')
end
```

No parser change needed: `case {2, 3}` was already parsed as `Expr::CellLiteral`
once `LBrace`/`RBrace` tokens existed (12.5a).

---

## 12.5d — `cellfun` and `arrayfun`

Higher-order built-ins that apply a function to each element of a collection.

### `cellfun(f, c)`

Applies `f` to each element of cell `c`. Returns `Value::Matrix` when all
results are scalar; returns `Value::Cell` otherwise.

```matlab
cellfun(@sqrt, {1, 4, 9})         % [1  2  3]
cellfun(@(x) x * 2, {1, 4, 9})   % [2  8  18]
```

### `arrayfun(f, v)`

Applies `f` to each element of numeric vector `v`. Returns same-shape `Value::Matrix`.

```matlab
arrayfun(@(x) x^2, [1 2 3 4])     % [1  4  9  16]
arrayfun(@abs, [-1 2 -3 4])        % [1  2   3  4]
```

Both implemented as cases in `call_builtin`; no new AST nodes.

---

## `@funcname` function handles

`@funcname` (without parentheses) creates a `Value::Lambda` that forwards its
arguments to `funcname` — either a builtin or a named user function stored in
the environment.

```
Expr::FuncHandle(name)  →  Value::Lambda wrapping name lookup + call
```

The lambda captures the caller's env at creation time. On each call it first
checks the captured env for a user function, then falls back to `call_builtin`.

```matlab
f = @sqrt;
g = @myFunc;     % wraps a user-defined function

cellfun(@sqrt, {1, 4, 9})   % works: @sqrt passes sqrt as a Lambda
```

---

## Tests

17 new tests in `parser_tests.rs` covering:
- Cell literal and indexing
- Cell assignment and auto-grow
- `iscell`, `cell()`, `numel`, `length`
- `cellfun`, `arrayfun`
- `switch` with `case {…}`
- `varargin`
- Out-of-bounds index error

---

## Known limitations

- 2-D cell arrays are not supported (all cells are 1-D `Vec<Value>`)
- `c{i}` is only supported where `i` is a simple expression — postfix chaining
  `c{k}(args)` is not yet supported (use `f = c{k}; f(args)` instead)
- Workspace save/load skips cell variables (same policy as matrices)
