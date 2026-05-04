# Polynomial Operations & Interpolation

Polynomials are represented as **row vectors of coefficients in descending degree order**.

```
p(x) = x² − 3x + 2  →  [1, -3, 2]
p(x) = x³ − 6x² + 11x − 6  →  [1, -6, 11, -6]
```

## Evaluation — `polyval`

`polyval(p, x)` evaluates polynomial `p` at scalar or vector `x` using
Horner's method (numerically stable, O(n) multiplications).

```matlab
p = [1 0 1];          % x² + 1
polyval(p, 0)         % → 1
polyval(p, [0 1 2])   % → [1 2 5]
```

## Fitting — `polyfit`

`polyfit(x, y, n)` returns the degree-`n` polynomial (n+1 coefficients) that
best fits the data points `(x, y)` in the least-squares sense.

The fit is computed via a Vandermonde matrix and QR decomposition.

```matlab
x = [0 1 2 3 4];
y = [1 2 5 10 17];
p = polyfit(x, y, 2)   % → [1.0  0.0  1.0]  (≈ x² + 1)

% Evaluate the fit at finer points:
xi = linspace(0, 4, 100);
yi = polyval(p, xi);
```

## Roots — `roots`

`roots(p)` finds all roots of polynomial `p` using the Durand–Kerner
(Weierstrass) iteration in complex arithmetic.

- **All roots real** → returns a real column vector (`Matrix`).
- **Any root complex** → returns a `Cell` of `Scalar`/`Complex` values.

```matlab
roots([1 0 1])     % → {0+1i; 0-1i}   (complex pair — Cell)
roots([1 2 1])     % → [-1; -1]        (repeated real root)
```

## Monic polynomial — `poly`

`poly(r)` expands the product `(x − r₁)(x − r₂)…` into a coefficient vector.

`poly(A)` computes the characteristic polynomial of square matrix `A` via the
Faddeev–LeVerrier algorithm.

```matlab
poly([1 2 3])      % → [1 -6 11 -6]    (x-1)(x-2)(x-3)
poly([1 2; 0 3])   % → [1 -4 3]        characteristic polynomial of A
```

## Convolution — `conv`

`conv(a, b)` computes the discrete linear convolution of vectors `a` and `b`.
For polynomials this is equivalent to polynomial multiplication.

Result length = `length(a) + length(b) − 1`.

```matlab
conv([1 2 3], [1 1])   % → [1 3 5 3]
```

## Deconvolution — `deconv`

`[q, r] = deconv(c, b)` performs polynomial long division `c / b`.

Returns quotient `q` and remainder `r` (same length as `c`) satisfying:

```
conv(b, q) + r == c
```

```matlab
[q, r] = deconv([1 3 5 3], [1 1])   % q=[1 2 3], r=[0 0 0 0]
```

## Interpolation — `interp1`

`interp1(x, y, xi)` interpolates the data `(x, y)` at query points `xi`.

`x` must be strictly monotonically increasing. Queries outside `[x(1), x(end)]`
return `NaN` (no extrapolation).

| Method | Description |
|---|---|
| `'linear'` (default) | Linear interpolation between bracketing knots |
| `'nearest'` | Snap to the closest knot (ties go left) |
| `'previous'` | Zero-order hold — left step (floor to left knot) |
| `'next'` | Right step (ceil to right knot) |

```matlab
x = [0 1 2 3];
y = [0 1 4 9];

interp1(x, y, 1.5)                  % → 2.5   (linear)
interp1(x, y, [0.5 1.5 2.5])       % → [0.5 2.5 6.5]
interp1(x, y, 1.5, 'nearest')       % → 1     (closest knot)
interp1(x, y, 1.5, 'previous')      % → 1     (left step)
interp1(x, y, 1.5, 'next')          % → 4     (right step)
interp1(x, y, 99)                   % → NaN   (out of range)
```
