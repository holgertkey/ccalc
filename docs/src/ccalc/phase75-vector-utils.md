# Phase 7.5 — Vector Utilities

**Version:** v0.11.0+003

## What was added

### 7.5a — Special floating-point constants

`nan` and `inf` are parser-level constants (like `pi` and `e`) — they require
no variable definition and cannot be overwritten by assignment.

```
nan            % Not-a-Number
inf            % positive infinity
-inf           % negative infinity
nan + 5        % → NaN   (NaN propagates through all arithmetic)
nan == nan     % → 0     (always false in IEEE 754)
```

**Matrix constructor:**

```
nan(3)         % 3×3 matrix filled with NaN
nan(2, 4)      % 2×4 matrix filled with NaN
```

**Element-wise predicates** — work on both scalars and matrices:

| Function       | Description                         |
|----------------|-------------------------------------|
| `isnan(x)`     | 1.0 if NaN, else 0.0                |
| `isinf(x)`     | 1.0 if ±Inf, else 0.0               |
| `isfinite(x)`  | 1.0 if finite (not NaN, not Inf)    |

```
isnan(nan)        % → 1
isinf(inf)        % → 1
isfinite(42)      % → 1
isfinite(nan)     % → 0

v = [1 nan 3 inf];
isnan(v)          % → [0  1  0  0]
isfinite(v)       % → [1  0  1  0]
```

### 7.5b — Vector reductions

All reduction functions follow the same rule:

- **Vector** (1×N or N×1): collapse all elements to a scalar.
- **M×N matrix** (M>1, N>1): operate column-wise, return a 1×N row vector.

This matches Octave/MATLAB behaviour.

| Function      | Description                                          |
|---------------|------------------------------------------------------|
| `sum(v)`      | Sum of elements                                      |
| `prod(v)`     | Product of elements                                  |
| `mean(v)`     | Arithmetic mean                                      |
| `min(v)`      | Minimum element (1-arg form; `min(a,b)` still works) |
| `max(v)`      | Maximum element (1-arg form)                         |
| `any(v)`      | 1.0 if any element is non-zero                       |
| `all(v)`      | 1.0 if all elements are non-zero                     |
| `norm(v)`     | Euclidean (L2) norm                                  |
| `norm(v, p)`  | General Lp norm; `p = inf` → max of absolute values  |

```
sum([1 2 3 4])        % → 10
mean([1 2 3 4])       % → 2.5
min([3 1 4 1 5])      % → 1
max([3 1 4 1 5])      % → 5
any([0 0 1 0])        % → 1
all([1 2 3] > 0)      % → 1
norm([3 4])           % → 5
norm([1 2 3], 1)      % → 6  (L1 = sum of absolute values)
```

Column-wise on a matrix:

```
M = [1 2 3; 4 5 6]
sum(M)    % → [5  7  9]
mean(M)   % → [2.5  3.5  4.5]
```

**Cumulative operations** — return the same shape as the input:

| Function     | Description                                    |
|--------------|------------------------------------------------|
| `cumsum(v)`  | Cumulative sum along the vector / each column  |
| `cumprod(v)` | Cumulative product                             |

```
cumsum([1 2 3 4])    % → [1  3  6  10]
cumprod([1 2 3 4])   % → [1  2  6  24]
```

### 7.5c — `end` keyword in indexing

Inside any index expression `(...)`, the keyword `end` resolves to the size
of the dimension being indexed. Arithmetic on `end` is fully supported.

```
v = [10 20 30 40 50];
v(end)           % → 50
v(end-1)         % → 40
v(end-2:end)     % → [30 40 50]
v(1:2:end)       % → [10 30 50]

A = [1 2 3; 4 5 6; 7 8 9];
A(end, :)        % → [7 8 9]      last row
A(:, end)        % → [3; 6; 9]    last column
A(1:end-1, 2:end) % → [2 3; 5 6]  submatrix
```

**Implementation note:** `end` is a context-sensitive value injected into the
evaluation environment by `eval_index`. Outside an indexing context it is
an undefined variable.

### 7.5d — Sort, reshape, and find

| Function           | Description                                                |
|--------------------|------------------------------------------------------------|
| `sort(v)`          | Sort ascending (vectors only)                              |
| `reshape(A, m, n)` | Reshape to m×n using column-major (MATLAB) element order   |
| `fliplr(v)`        | Reverse column order (left↔right mirror)                   |
| `flipud(v)`        | Reverse row order (up↔down mirror)                         |
| `find(v)`          | 1-based column-major indices of non-zero elements          |
| `find(v, k)`       | First `k` non-zero indices                                 |
| `unique(v)`        | Sorted unique elements as a 1×N row vector                 |

```
sort([3 1 4 1 5 9])       % → [1  1  3  4  5  9]
reshape(1:6, 2, 3)        % → [1 3 5; 2 4 6]
fliplr([1 2 3])           % → [3 2 1]
flipud([1;2;3])           % → [3; 2; 1]
find([0 3 0 5 0])         % → [2 4]
find([1 0 2 0 3], 2)      % → [1 3]
unique([3 1 4 1 5 3])     % → [1 3 4 5]
```

`reshape` uses **column-major** order — elements fill the output matrix
column-by-column, matching MATLAB/Octave behaviour:

```
reshape([1 2 3 4 5 6], 2, 3)
% col 0: [1 2], col 1: [3 4], col 2: [5 6]
% → [1 3 5]
%   [2 4 6]
```

## Example file

`examples/vector_utils.calc` covers all Phase 7.5 features with annotated
output. Run it with:

```bash
ccalc examples/vector_utils.calc
```

## Implementation details

- `nan` / `inf` are handled in `parse_primary` (parser.rs) as named constants
  → `Expr::Number(f64::NAN)` / `Expr::Number(f64::INFINITY)`, exactly like
  `pi` and `e`. This allows `nan(m,n)` to work as a builtin call without
  the variable shadowing the function.
- `apply_elem`, `apply_reduction`, `apply_cumulative`, `find_nonzero` are
  private helpers in `eval.rs` that keep the builtin match arms concise.
- `end` support: `eval_index` creates a cloned environment with
  `"end" = dim_size` via `env_with_end()` before calling `resolve_dim`.
  No new AST node is required — `end` is just a variable that exists only
  within the scope of the index evaluation.
