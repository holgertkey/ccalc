# Phase 5 — Range Operator

**Version:** v0.10.0

## What was added

### Range expressions

The `:` operator generates row vectors. Two forms are supported:

| Form | Meaning |
|------|---------|
| `a:b` | start `a`, stop `b`, step 1 |
| `a:step:b` | start `a`, stop `b`, explicit step |

```
1:5              % [1 2 3 4 5]
1:2:9            % [1 3 5 7 9]
0:0.5:2          % [0 0.5 1 1.5 2]
5:-1:1           % [5 4 3 2 1]
5:1              % []   (empty — step in wrong direction)
```

The range operator has **lower precedence than arithmetic**:

```
1+1:2+2          % 2:4 → [2 3 4]
```

### Ranges inside matrix literals

Range elements inside `[...]` are evaluated to row vectors and concatenated
horizontally into the containing row:

```
[1:4]            % [1 2 3 4]           (1×4)
[0, 1:3, 10]     % [0 1 2 3 10]        (1×5)
[1:2:7]          % [1 3 5 7]           (1×4)
[1:3; 4:6]       % [1 2 3; 4 5 6]      (2×3)
```

### linspace

`linspace(a, b, n)` generates `n` evenly spaced values from `a` to `b`
(both endpoints included). This is numerically superior to range expressions
when the number of points matters more than the step size.

```
linspace(0, 1, 5)      % [0  0.25  0.5  0.75  1]
linspace(1, 5, 5)      % [1  2  3  4  5]
linspace(0, 1, 1)      % [1]   (single element returns b, MATLAB convention)
linspace(0, 1, 0)      % []   (empty)
```

## Parser changes

New token: `Token::Colon` — produced by the `:` character.

New AST node: `Expr::Range(Box<Expr>, Option<Box<Expr>>, Box<Expr>)` where
the middle field is the optional step expression.

New parser function `parse_range()` sits above `parse_expr()` in the
precedence hierarchy:

```
parse_range  (lowest)
  parse_expr  (+, -)
    parse_term  (*, /, .*, ./)
      parse_power  (^, .^)
        parse_unary  (unary -)
          parse_primary  (atoms)
```

`parse()` and `parse_matrix()` (element parsing) now call `parse_range()`
instead of `parse_expr()`.

## Evaluator changes

`Expr::Range` evaluation:

1. Evaluate `start`, `stop`, and `step` (must all be scalars).
2. Compute `n = floor((stop - start) / step + ε) + 1`.
3. If `n ≤ 0`, return a 1×0 empty matrix.
4. Generate values as `start + i * step` for `i = 0..n`.

`Expr::Matrix` evaluation updated: when a row element evaluates to a
`Value::Matrix`, its values are appended to the current row (horizontal
concatenation). This enables ranges inside `[...]`. Column vectors and
higher-dimensional sub-matrices are rejected with an error.
