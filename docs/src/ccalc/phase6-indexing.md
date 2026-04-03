# Phase 6 — Indexing

**Version:** v0.11.0

## What was added

### Vector indexing

All indices are 1-based (Octave/MATLAB convention).

```
v = [10 20 30 40 50];

v(3)         % → 30          scalar element
v(2:4)       % → [20 30 40]  sub-vector via range
v(:)         % → column vector [10;20;30;40;50]
```

`v(:)` follows Octave convention: all elements are returned as a column vector
in column-major order.

### Matrix indexing

```
A = [1 2 3; 4 5 6; 7 8 9];

A(2, 3)      % → 6           scalar at row 2, col 3
A(1, :)      % → [1 2 3]     entire row 1     (1×3)
A(:, 2)      % → [2;5;8]     entire column 2  (3×1)
A(1:2, 2:3)  % → [2 3; 5 6]  submatrix
A(:, :)      % → copy of A   (all rows, all cols)
```

Scalar result: `A(i,j)` returns `Value::Scalar` when both `i` and `j` are
scalars. Sub-matrix results return `Value::Matrix`.

### Linear (1D) indexing

`v(i)` on a matrix uses column-major linear indexing — columns are counted
before rows. For a 1×N row vector this is simply sequential:

```
v = [10 20 30 40 50];
v(3)    % 3rd element → 30
```

For a 2D matrix, linear indexing counts down each column before moving to
the next:

```
A = [1 2; 3 4];   % linear order: 1, 3, 2, 4
A(3)              % → 2   (3rd in column-major order)
```

### Call vs. index disambiguation

When `name(args)` is parsed, the evaluator checks `Env` at eval time:

- **Name is in `Env`** → indexing (`eval_index`)
- **Name is not in `Env`** → built-in function call (`call_builtin`)

This means variables shadow built-in function names, matching Octave
semantics. Assigning `zeros = [1 2; 3 4]` makes `zeros(1,2)` an indexing
operation on that variable, not a call to `zeros(m,n)`.

## Parser changes

New AST node: `Expr::Colon` — represents a bare `:` used as an
all-elements index selector. Evaluating `Expr::Colon` outside an indexing
context returns an error.

New parser function `parse_call_arg()`:

```
parse_call_arg:
  if current token is ':' → consume, return Expr::Colon
  else                    → parse_range(...)
```

All function call / index argument positions now use `parse_call_arg`
instead of `parse_expr`. This enables:
- `A(:, j)` — bare colon as first arg
- `A(1:3, :)` — range as first arg, colon as second
- `f(1:5)` — range as function argument

### Bug fix: range in grouping parentheses

`parse_primary` for grouped expressions `(...)` previously called
`parse_expr`, which does not handle `:`. This was changed to `parse_range`,
enabling expressions like `2 .^ (0:7)`.

## Evaluator changes

`eval_index(val, args, env)` — dispatches based on argument count:

| Args | Form | Result |
|------|------|--------|
| 1, `Colon` | `v(:)` | All elements as column vector (column-major) |
| 1, scalar | `v(i)` | `Value::Scalar` |
| 1, vector | `v(1:3)` | `Value::Matrix` 1×N |
| 2, both scalar | `A(i,j)` | `Value::Scalar` |
| 2, otherwise | `A(:,j)`, `A(i,:)`, `A(1:2,2:3)` | `Value::Matrix` |

`resolve_dim(expr, dim_size, env)` — converts one index argument to a list
of 0-based indices:

- `Expr::Colon` → `DimIdx::All`
- `Value::Scalar(n)` → validates `1 ≤ n ≤ dim_size`, returns `[n-1]`
- `Value::Matrix` (vector) → validates each element, returns `[i-1, ...]`
