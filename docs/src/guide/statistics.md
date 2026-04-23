# Statistics & Random Numbers

## Random number generation

Use `rng(seed)` at the start of any script that needs reproducible output.

| Function | Description |
|---|---|
| `rand()` | scalar uniform in \[0, 1) |
| `rand(n)` | n×n uniform matrix |
| `rand(m, n)` | m×n uniform matrix |
| `randn()` | scalar standard-normal sample |
| `randn(n)` / `randn(m, n)` | standard-normal matrix |
| `randi(max)` | random integer in \[1, max\] |
| `randi(max, n)` / `randi(max, m, n)` | matrix of random integers |
| `randi([lo hi], ...)` | integers from \[lo, hi\] |
| `rng(seed)` | seed RNG — same seed → same sequence |
| `rng('shuffle')` | reseed from OS entropy |

```
rng(42)
x = randn(1, 5)         % reproducible 5-element sequence
d = randi(6, 1, 10)     % ten dice rolls
```

## Descriptive statistics

All functions operate **column-wise** on M×N matrices and collapse to a
scalar for vectors.

| Function | Description |
|---|---|
| `std(v)` | sample standard deviation (n-1 denominator) |
| `std(v, 1)` | population standard deviation (n denominator) |
| `var(v)` / `var(v, 1)` | sample / population variance |
| `median(v)` | median (linear interpolation for even length) |
| `mode(v)` | most frequent value; smallest wins on ties |
| `cov(v)` | variance of a vector |
| `cov(A)` | N×N covariance matrix of an m×N data matrix |

```
v = [2 4 4 4 5 5 7 9];
mean(v)      % 5.0
std(v)       % sample std ≈ 2.138
std(v, 1)    % population std = 2.0
median(v)    % 4.5
mode(v)      % 4
```

## Percentiles and spread

| Function | Description |
|---|---|
| `prctile(v, p)` | p-th percentile; `p` can be a vector |
| `iqr(v)` | interquartile range: `prctile(75) - prctile(25)` |
| `zscore(v)` | standardise: `(v - mean) / std`, same shape |

```
v = [1 2 3 4 5 6 7 8];
prctile(v, 50)          % 4.5  (median)
prctile(v, [25 75])     % [2.75  6.25]  (quartiles)
iqr(v)                  % 3.5

z = zscore([2 4 6]);    % z = [-1  0  1]
```

### Outlier detection (1.5 × IQR rule)

```
q1 = prctile(data, 25);
q3 = prctile(data, 75);
fence_lo = q1 - 1.5 * iqr(data);
fence_hi = q3 + 1.5 * iqr(data);
outliers = data(data < fence_lo | data > fence_hi);
```

## Histogram

`hist` prints an ASCII bar chart to stdout and returns `Void`.
`histc` returns a count vector for user-supplied bin edges.

```
hist(data)           % 10 bins (default)
hist(data, 20)       % 20 bins

edges = [0 10 20 30 40 50];
counts = histc(data, edges)
```

`histc` bin semantics: bin *i* counts elements where
`edges(i) <= x < edges(i+1)`; the last bin counts `x == edges(end)` exactly.

## Normal distribution

| Function | Description |
|---|---|
| `normcdf(x)` | P(Z ≤ x), Z ~ N(0, 1) |
| `normcdf(x, mu, s)` | P(X ≤ x), X ~ N(mu, s²) |
| `normpdf(x)` | standard normal PDF |
| `normpdf(x, mu, s)` | general normal PDF |
| `erf(x)` | Gauss error function |
| `erfc(x)` | 1 − erf(x) |

All six functions work element-wise on scalars and matrices.

```
normcdf(0)                        % 0.5
normcdf(1) - normcdf(-1)          % 0.6827  (68% rule)
normcdf(2) - normcdf(-2)          % 0.9545  (95% rule)
normcdf(3) - normcdf(-3)          % 0.9973  (99.7% rule)

% Probability that X ~ N(50, 10) falls between 40 and 60:
normcdf(60, 50, 10) - normcdf(40, 50, 10)   % ≈ 0.6827
```

The relationship between `normcdf` and `erf`:

```
normcdf(x) = 0.5 * (1 + erf(x / sqrt(2)))
```

## Full example

```
% Generate 200 samples from N(50, 10) and analyse them.
rng(7)
n    = 200;
data = 50 + 10 * randn(1, n);

fprintf('mean   = %.4f\n', mean(data))
fprintf('std    = %.4f\n', std(data))
fprintf('median = %.4f\n', median(data))
fprintf('IQR    = %.4f\n', iqr(data))

% Percentile table
pct = prctile(data, [5 25 50 75 95]);
fprintf('P5/P25/P50/P75/P95 = %.1f  %.1f  %.1f  %.1f  %.1f\n', ...
  pct(1), pct(2), pct(3), pct(4), pct(5))

% ASCII histogram
hist(data, 12)
```

See the full demo at `examples/statistics.calc`.
