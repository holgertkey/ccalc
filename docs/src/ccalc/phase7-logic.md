# Phase 7 — Comparison & Logical Operators

**Version:** v0.11.0+001

## What was added

### Comparison operators

All six relational operators from Octave/MATLAB are now supported.
They return `1.0` (true) or `0.0` (false):

```
3 > 2        % → 1
3 == 3       % → 1
3 == 4       % → 0
5 ~= 3       % → 1  (not equal)
4 <= 4       % → 1
4 >= 5       % → 0
```

Comparison has lower precedence than arithmetic, so the operands are
evaluated first:

```
1 + 1 == 2   % → 1   (evaluates as (1+1) == 2)
2 * 3 > 5    % → 1   (evaluates as (2*3) > 5)
```

Comparison is **non-associative** — chaining like `a < b < c` is a parse
error. Write `a < b && b < c` instead.

### Logical NOT — `~`

Unary `~` negates a truth value: zero becomes `1`, any non-zero becomes `0`.
It binds at the same precedence level as unary minus:

```
~0           % → 1
~1           % → 0
~(3 == 3)    % → 0
~(3 == 4)    % → 1
```

### Short-circuit logical AND/OR — `&&`, `||`

`&&` and `||` evaluate both operands and return `0.0` or `1.0`.
`&&` binds tighter than `||`:

```
1 && 1       % → 1
1 && 0       % → 0
0 || 1       % → 1
0 || 0       % → 0

% '&&' before '||':
1 || 0 && 0  % → 1   (1 || (0 && 0))
```

Combining comparisons:

```
x = 2.7;
x >= 0 && x <= 3.3   % → 1   (in range check)
x < 0  || x > 3.3   % → 0   (out-of-range flag)
```

### Element-wise on matrices

All operators work element-wise on matrices of the same shape, producing
a `0`/`1` mask of the same dimensions:

```
v = [1 2 3 4 5];

v > 3          % → [0 0 0 1 1]
v == 3         % → [0 0 1 0 0]
v ~= 3         % → [1 1 0 1 1]
~(v > 3)       % → [1 1 1 0 0]
```

Scalar–matrix mixed comparisons broadcast the scalar:

```
v >= 2         % → [0 1 1 1 1]
3 < v          % → [0 0 0 1 1]
```

### Soft masking via `.*`

Since masks are `0`/`1` matrices, multiplying by them zeroes out unwanted
elements — a common pattern for conditional selection:

```
v .* (v > 3)               % → [0 0 0 4 5]  keep elements > 3

% Combine two masks (element-wise AND):
lo = v >= 2;
hi = v <= 4;
v .* (lo .* hi)            % → [0 2 3 4 0]  keep 2–4 only
```

## Precedence summary

Full precedence table from lowest to highest:

| Level | Operators | Notes |
|-------|-----------|-------|
| 1 (lowest) | `\|\|` | logical OR |
| 2 | `&&` | logical AND |
| 3 | `==` `~=` `<` `>` `<=` `>=` | comparison, non-associative |
| 4 | `:` | range (`a:b`, `a:step:b`) |
| 5 | `+` `-` | additive |
| 6 | `*` `/` `.*` `./` | multiplicative |
| 7 | `^` `.^` | power (right-associative) |
| 8 | unary `-` `~` | negation, logical NOT |
| 9 (highest) | postfix `'` | transpose |

## Parser changes

Three new parser levels were inserted above `parse_range`:

```
parse_logical_or        % '||'
  parse_logical_and     % '&&'
    parse_comparison    % == ~= < > <= >=
      parse_range       % a:b, a:step:b  (existing)
        parse_expr      % + -
          ...
```

`parse_logical_or` is now the top-level entry point for `parse()`,
`parse_call_arg()`, and grouped expressions `(...)` in `parse_primary`.

New tokens: `EqEq`, `NotEq`, `Lt`, `Gt`, `LtEq`, `GtEq`, `AmpAmp`,
`PipePipe`, `Tilde`.

A bare `=` in expression context (not a valid assignment left-hand side)
is now a tokenizer error, making `3 = 3` produce a clear message instead
of a silent parse failure.

## Evaluator changes

New `Expr` variant:

- `Expr::UnaryNot(Box<Expr>)` — evaluates to `1.0` if inner is `0.0`, else `0.0`.
  Works element-wise on matrices.

New `Op` variants:

| Variant | Operation |
|---------|-----------|
| `Op::Eq` | `==` |
| `Op::NotEq` | `~=` |
| `Op::Lt` | `<` |
| `Op::Gt` | `>` |
| `Op::LtEq` | `<=` |
| `Op::GtEq` | `>=` |
| `Op::And` | `&&` |
| `Op::Or` | `\|\|` |

`eval_binop` handles all four combinations (Scalar×Scalar,
Matrix×Matrix, Scalar×Matrix, Matrix×Scalar) for every new operator.
Scalar×Scalar returns a `Value::Scalar(0.0|1.0)`; any matrix combination
returns a `Value::Matrix` of the same shape.

Helper functions added to `eval.rs`:
- `bool_to_f64(b: bool) -> f64` — converts a boolean to `0.0`/`1.0`
- `cmp_op(op: &Op, a: f64, b: f64) -> bool` — applies comparison or logical op to two scalar values
