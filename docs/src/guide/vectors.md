# Vector & Data Utilities

## Special constants

`nan` and `inf` are built-in constants — they behave like numeric literals
and cannot be overwritten.

| Constant | Value                                         |
|----------|-----------------------------------------------|
| `nan`    | IEEE 754 Not-a-Number                         |
| `inf`    | Positive infinity (`-inf` for negative)       |

```
nan + 5         % → NaN    (NaN propagates through all arithmetic)
nan == nan      % → 0      (IEEE 754: NaN is never equal to itself)
inf * 2         % → inf
-inf < inf      % → 1
```

### NaN predicates (element-wise)

| Function      | Signature       | Returns 1 when…                  |
|---------------|-----------------|----------------------------------|
| `isnan(x)`    | `isnan(x)`      | `x` is NaN                       |
| `isinf(x)`    | `isinf(x)`      | `x` is ±Inf                      |
| `isfinite(x)` | `isfinite(x)`   | `x` is neither NaN nor ±Inf      |

All three accept a scalar or a matrix and apply element-wise, returning a result of the same shape
with each element replaced by `1.0` (true) or `0.0` (false).
Use these instead of `== nan` or `== inf`, which do not work as expected:
`nan == nan` always returns 0.

```
isnan(nan)       % → 1
isinf(inf)       % → 1
isfinite(42)     % → 1
isfinite(nan)    % → 0

v = [1 nan 3 inf];
isnan(v)         % → [0 1 0 0]
isfinite(v)      % → [1 0 1 0]
```

### NaN matrix constructor

```
nan(n)        % n×n matrix of NaN
nan(m, n)     % m×n matrix of NaN
```

`nan(n)` is shorthand for `nan(n, n)`.
The result is a matrix where every element is `NaN`, useful for pre-allocating
arrays that must be filled before use.

---

## Reductions

For **vectors** (1×N or N×1) these return a **scalar**.  
For **M×N matrices** (M>1, N>1) they operate **column-wise** and return a **1×N row vector**.

| Function      | Signature          | Description                                           |
|---------------|--------------------|-------------------------------------------------------|
| `sum(v)`      | `sum(v)`           | Sum of all elements                                   |
| `prod(v)`     | `prod(v)`          | Product of all elements                               |
| `mean(v)`     | `mean(v)`          | Arithmetic mean (sum divided by element count)        |
| `min(v)`      | `min(v)`           | Minimum element; for 2-scalar form see `min(a, b)`    |
| `max(v)`      | `max(v)`           | Maximum element; for 2-scalar form see `max(a, b)`    |
| `any(v)`      | `any(v)`           | `1.0` if at least one element is non-zero, else `0.0` |
| `all(v)`      | `all(v)`           | `1.0` if every element is non-zero, else `0.0`        |
| `norm(v)`     | `norm(v)`          | Euclidean (L2) norm: √(Σxᵢ²)                         |
| `norm(v, p)`  | `norm(v, p)`       | General Lp norm; `p = inf` → max of absolute values  |

### Notes

**`min(v)`, `max(v)`** — The 1-argument form finds the extreme element of a vector or matrix.
The 2-argument forms `min(a, b)` and `max(a, b)` compare two scalars.
Both forms are available; which is used depends on the number of arguments.

**`any(v)`, `all(v)`** — Treat any non-zero value (including negative numbers) as true,
and zero as false. NaN is non-zero, so `any([nan]) → 1` and `all([0 nan]) → 0`.

**`norm(v, p)`** — Common values of `p`:
- `p = 1` → L1 norm: sum of absolute values
- `p = 2` (default) → L2 Euclidean norm
- `p = inf` → L∞ norm: maximum absolute value

For scalars, `norm(x)` returns `abs(x)`.

```
v = [1 2 3 4 5];

sum(v)            % → 15
prod(v)           % → 120
mean(v)           % → 3
min(v)            % → 1
max(v)            % → 5
any(v > 4)        % → 1
all(v > 0)        % → 1
norm(v)           % → sqrt(1+4+9+16+25) ≈ 7.416
norm([3 4])       % → 5
norm([1 2 3], 1)  % → 6   (L1 = sum of absolute values)
```

Column-wise on a matrix:

```
M = [1 2 3; 4 5 6];
sum(M)    % → [5  7  9]    one sum per column
mean(M)   % → [2.5  3.5  4.5]
min(M)    % → [1  2  3]
max(M)    % → [4  5  6]
```

---

## Cumulative operations

These return an array of the **same shape** as the input.

| Function     | Signature      | Description                                           |
|--------------|----------------|-------------------------------------------------------|
| `cumsum(v)`  | `cumsum(v)`    | Running sum: element `i` = sum of first `i` elements  |
| `cumprod(v)` | `cumprod(v)`   | Running product: element `i` = product of first `i`   |

Both functions accept a scalar (returned unchanged) or a vector/matrix.
For a matrix the operation runs along all elements in column-major order,
returning a matrix of the same shape.

```
cumsum([1 2 3 4])    % → [1  3  6  10]
cumprod([1 2 3 4])   % → [1  2  6  24]

% Compound interest: balance after each year
rates = [1.05, 1.08, 1.03, 1.10];
cumprod(rates)       % → cumulative growth factors
```

---

## Sorting and searching

| Function        | Signature          | Description                                                  |
|-----------------|--------------------|--------------------------------------------------------------|
| `sort(v)`       | `sort(v)`          | Sort elements in ascending order; vectors only               |
| `find(v)`       | `find(v)`          | 1-based column-major indices of all non-zero elements        |
| `find(v, k)`    | `find(v, k)`       | First `k` non-zero indices; `k` must be non-negative         |
| `unique(v)`     | `unique(v)`        | Sorted unique elements as a 1×N row vector                   |

### Notes

**`sort(v)`** — Sorts in ascending order only. Accepts a scalar (returned unchanged) or a vector.
Passing a 2D matrix (more than one row *and* more than one column) returns an error;
use `sort` on individual rows or columns instead.

**`find(v)`** — Returns a row vector of 1-based indices of elements that are non-zero (including `±Inf` and `NaN`).
Indices follow column-major order (columns first), matching MATLAB/Octave convention.
Returns an empty matrix `[]` when no elements match.

**`find(v, k)`** — Limits the result to the first `k` indices. `k = 0` returns `[]`.
`k` must be a non-negative integer.

**`unique(v)`** — Returns a 1×N row vector of distinct values, sorted in ascending order.
Accepts scalars, vectors, or matrices (elements are flattened in column-major order before deduplication).

```
v = [3 1 4 1 5 9 2 6];

sort(v)                    % → [1 1 2 3 4 5 6 9]
unique(v)                  % → [1 2 3 4 5 6 9]

find(v > 4)                % → [5  6  8]   indices where v > 4
find(v > 4, 2)             % → [5  6]      first 2 such indices

% Typical pattern: use find with a comparison mask
idx = find(v > 3);
v(idx)                     % → elements of v greater than 3
```

---

## Reshape and flip

| Function           | Signature            | Description                                              |
|--------------------|----------------------|----------------------------------------------------------|
| `reshape(A, m, n)` | `reshape(A, m, n)`   | Reshape to m×n using column-major element order          |
| `fliplr(v)`        | `fliplr(A)`          | Reverse column order (left↔right mirror)                 |
| `flipud(v)`        | `flipud(A)`          | Reverse row order (up↔down mirror)                       |

### Notes

**`reshape(A, m, n)`** — Rearranges the elements of `A` into a matrix with `m` rows and `n` columns.
The total number of elements must be preserved: `m * n` must equal `numel(A)`, otherwise an error is raised.
Elements are read and written in **column-major order** (column by column), matching MATLAB/Octave.

**`fliplr(A)`** — Reverses the order of columns. For a row vector this reverses all elements.
A scalar is returned unchanged.

**`flipud(A)`** — Reverses the order of rows. For a column vector this reverses all elements.
A scalar is returned unchanged.

```
reshape(1:6, 2, 3)    % fills column-by-column:
                      % [1 3 5]
                      % [2 4 6]

reshape(1:6, 3, 2)    % [1 4]
                      % [2 5]
                      % [3 6]

fliplr([1 2 3])       % → [3 2 1]
fliplr([1 2 3; 4 5 6]) % → [3 2 1; 6 5 4]

flipud([1 2; 3 4])    % → [3 4; 1 2]
```

---

## Example file

`examples/vector_utils.calc` demonstrates all of these features:

```bash
ccalc examples/vector_utils.calc
```
