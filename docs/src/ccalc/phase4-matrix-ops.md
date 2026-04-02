# Phase 4 — Matrix Operations

**Version:** v0.9.0

## What was added

### Matrix multiplication

`*` between two matrices now performs standard matrix multiplication via
ndarray's `.dot()`. Inner dimensions must agree; otherwise an error is
returned.

```
A = [1 2; 3 4];
b = [1; 1];
A * b          % → [3; 7]
```

### Transpose

Postfix `'` transposes a matrix. It binds tighter than any binary operator,
so `A' * B` parses as `(A') * B`.

```
v = [1; 2; 3];
v'             % → [1  2  3]  (1×3 row vector)
v' * v         % → 14         (dot product, result is 1×1 matrix)
```

### Element-wise operators

`.*`, `./`, `.^` apply the operation to each pair of corresponding elements.
Both operands must have the same shape (or one must be a scalar).

```
A .* B         % Hadamard product
A ./ B         % element-wise division
A .^ 2         % square each element
```

Note: `*` is matrix multiplication; `.*` is element-wise product.

### New built-in functions

| Function        | Description                              |
|-----------------|------------------------------------------|
| `zeros(m, n)`   | m×n matrix of zeros                      |
| `ones(m, n)`    | m×n matrix of ones                       |
| `eye(n)`        | n×n identity matrix                      |
| `size(A)`       | `[rows cols]` as a 1×2 row vector        |
| `size(A, dim)`  | Rows (dim=1) or columns (dim=2) as scalar|
| `length(A)`     | `max(rows, cols)`                        |
| `numel(A)`      | Total element count                      |
| `trace(A)`      | Sum of diagonal elements                 |
| `det(A)`        | Determinant                              |
| `inv(A)`        | Inverse matrix                           |

`det` and `inv` are implemented via Gaussian / Gauss-Jordan elimination
with no external BLAS/LAPACK dependency.

## Parser changes

New tokens:

| Token        | Input | Usage              |
|--------------|-------|--------------------|
| `Apostrophe` | `'`   | Postfix transpose  |
| `DotStar`    | `.*`  | Element-wise `*`   |
| `DotSlash`   | `./`  | Element-wise `/`   |
| `DotCaret`   | `.^`  | Element-wise `^`   |

New AST node: `Expr::Transpose(Box<Expr>)`.

New `Op` variants: `ElemMul`, `ElemDiv`, `ElemPow`.

`is_partial` extended: `.*`, `./`, `.^` prefixes are now recognised as
partial expressions (e.g. `.* 2` expands to `ans .* 2`).

The number tokenizer no longer absorbs `.` into a number when it is
immediately followed by `*`, `/`, or `^`. This means `3.*2` correctly
tokenizes as `Number(3)`, `DotStar`, `Number(2)` rather than
`Number(3.0)`, `Star`, `Number(2)`.

## `split_stmts` fix

`split_stmts` in `repl.rs` previously toggled `in_sq` on every `'`,
which caused `R' * q;` to hide the `;` inside a "string". The function
now checks the left context: `'` preceded by an identifier character,
digit, `)`, `]`, or another `'` is treated as a transpose operator and
does not affect the string-tracking state.

## Evaluator changes

`eval_binop` updated:

- `Matrix * Matrix` → ndarray `.dot()` (was an error in Phase 3)
- `Matrix .* Matrix` → element-wise `&lm * &rm`
- `Matrix ./ Matrix` → element-wise `&lm / &rm`
- `Matrix .^ Matrix` → element-wise `Zip::map_collect(a.powf(b))`

`call_builtin` refactored: the function now returns `Result<Value, String>`
directly from each match arm, replacing the old pattern that extracted an
`f64` result and always wrapped it in `Value::Scalar`. This allows matrix-
returning built-ins (`zeros`, `ones`, `eye`, `size`, `inv`) to coexist with
scalar-returning ones in the same match.
