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

| Function      | Returns 1 when…                  |
|---------------|----------------------------------|
| `isnan(x)`    | `x` is NaN                       |
| `isinf(x)`    | `x` is ±Inf                      |
| `isfinite(x)` | `x` is neither NaN nor ±Inf      |

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
nan(3)        % 3×3 matrix of NaN
nan(2, 4)     % 2×4 matrix of NaN
```

---

## Reductions

For **vectors** (1×N or N×1) these return a scalar.  
For **M×N matrices** (M>1, N>1) they operate column-wise and return a 1×N row vector.

| Function      | Description                                          |
|---------------|------------------------------------------------------|
| `sum(v)`      | Sum of elements                                      |
| `prod(v)`     | Product of elements                                  |
| `mean(v)`     | Arithmetic mean                                      |
| `min(v)`      | Minimum element (see also 2-arg `min(a, b)`)         |
| `max(v)`      | Maximum element (see also 2-arg `max(a, b)`)         |
| `any(v)`      | 1.0 if any element is non-zero, else 0.0             |
| `all(v)`      | 1.0 if all elements are non-zero, else 0.0           |
| `norm(v)`     | Euclidean (L2) norm: √(Σxᵢ²)                        |
| `norm(v, p)`  | General Lp norm; `p = inf` → max of absolute values  |

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

| Function     | Description                                          |
|--------------|------------------------------------------------------|
| `cumsum(v)`  | Running sum: element `i` = sum of first `i` elements |
| `cumprod(v)` | Running product                                      |

```
cumsum([1 2 3 4])    % → [1  3  6  10]
cumprod([1 2 3 4])   % → [1  2  6  24]

% Compound interest: balance after each year
rates = [1.05, 1.08, 1.03, 1.10];
cumprod(rates)       % → cumulative growth factors
```

---

## Sorting and searching

| Function        | Description                                                |
|-----------------|------------------------------------------------------------|
| `sort(v)`       | Sort in ascending order (vectors only)                     |
| `find(v)`       | 1-based column-major indices of non-zero elements          |
| `find(v, k)`    | First `k` non-zero indices                                 |
| `unique(v)`     | Sorted unique elements as a 1×N row vector                 |

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

| Function           | Description                                              |
|--------------------|----------------------------------------------------------|
| `reshape(A, m, n)` | Reshape to m×n using column-major (MATLAB) element order |
| `fliplr(v)`        | Reverse column order (left↔right mirror)                 |
| `flipud(v)`        | Reverse row order (up↔down mirror)                       |

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
