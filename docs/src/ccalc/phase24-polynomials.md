# Phase 24 — Polynomial Operations & Interpolation

**Version:** 0.29.0  
**Prerequisite:** Phase 8 (complex numbers — roots can be complex); Phase 18 (QR decomposition — used by `polyfit`).

## Overview

Phase 24 adds 7 new built-in functions for polynomial arithmetic, root finding,
and piecewise interpolation. No new tokens or AST nodes were needed.

| Function | Signature | Notes |
|---|---|---|
| `polyval` | `polyval(p, x)` | Horner evaluation |
| `polyfit` | `polyfit(x, y, n)` | Vandermonde + QR solve |
| `roots` | `roots(p)` | Durand–Kerner iteration |
| `poly` | `poly(r)` / `poly(A)` | Expand from roots / char. poly |
| `conv` | `conv(a, b)` | O(mn) convolution |
| `deconv` | `deconv(c, b)` | Polynomial long division |
| `interp1` | `interp1(x, y, xi[, method])` | Piecewise interpolation |

## 24a — Polynomial evaluation, fitting, and roots

### `polyval(p, x)` — Horner evaluation

Polynomials are row vectors `[c_n, c_{n-1}, …, c_0]` (highest degree first).
Evaluation uses Horner's method for numerical stability:

```
p(x) = c_n x^n + … + c_1 x + c_0
     = (…((c_n x + c_{n-1}) x + c_{n-2}) x + …) x + c_0
```

`x` can be a scalar or any-shape matrix; the result has the same shape as `x`.

### `polyfit(x, y, n)` — Least-squares polynomial fit

Builds an m×(n+1) Vandermonde matrix `V` (rows `[x_i^n, …, x_i, 1]`), then
solves `V c ≈ y` via `qr_decompose` (Phase 18) + back-substitution. Returns
the (n+1) coefficients as a 1×(n+1) row vector.

### `roots(p)` — Root finding via Durand–Kerner

The CDP plan called for building a companion matrix and calling `eig_compute`,
but `eig_compute` only handles real eigenvalues (real Wilkinson shifts). For
polynomials with complex roots (e.g. `x² + 1`), a companion matrix approach
would stall.

**Implementation deviation:** uses the **Durand–Kerner (Weierstrass) iteration**
directly in complex `(f64, f64)` arithmetic (~70 lines, no eig dependency).

Key implementation details:
- Normalizes to monic polynomial first.
- Cauchy root bound as initial radius: `r = 1 + max|c_i|`.
- Initial guesses rotated by `0.25/n` turns to avoid the real axis, preventing
  stall on polynomials with purely imaginary roots.
- 2000 iterations max; terminates when max correction falls below `1e-12`.
- Sorted by descending real part, then descending imaginary part (MATLAB order).

**Return type:** since `Value::Matrix` is `Array2<f64>` (real only):
- All roots real (imaginary parts < `1e-9`) → `Value::Matrix` (n×1 column).
- Any root complex → `Value::Cell` of `Value::Scalar`/`Value::Complex` elements.

### `poly(r)` — Monic polynomial from roots or characteristic polynomial

- **Vector argument:** iteratively convolves `[1.0]` with `[1.0, -r_i]` for
  each root. Requires `poly_conv` (see 24b).
- **Square matrix argument:** Faddeev–LeVerrier algorithm, O(n³) matrix
  multiplications, no eigenvalue computation needed.

## 24b — Convolution, deconvolution, interpolation

### `conv(a, b)` — Discrete linear convolution

Direct O(mn) double loop. Result length = `m + n − 1`. Accepts row or column
vectors; always returns a row vector.

### `deconv(c, b)` — Polynomial long division

Returns `Value::Tuple(vec![q_val, r_val])` for `[q, r] = deconv(c, b)`.
The remainder `r` has the **same length as `c`** (MATLAB convention), so the
invariant `conv(b, q) + r == c` holds element-wise.

Near-zero rounding residuals (< `1e-10` × max input coefficient) are zeroed.

### `interp1(x, y, xi[, method])` — Piecewise interpolation

Uses `partition_point` for O(log n) bracket search. Four methods:

| Method | Implementation |
|---|---|
| `linear` | `y[lo] + t*(y[lo+1]-y[lo])` where `t=(xi-x[lo])/(x[lo+1]-x[lo])` |
| `nearest` | Snap to `x[lo]` or `x[lo+1]`, tie goes left |
| `previous` | `y[lo]` (left step / ZOH) |
| `next` | `y[lo+1]` unless at exact knot, then `y[lo]` |

The last-knot edge case (`xi == x[n-1]`) is handled specially to ensure all
methods return `y[n-1]` correctly.

Extrapolation (query outside `[x[0], x[n-1]]`) always returns `NaN`.

## Tests

33 new tests in `mod phase24_tests` (855 total).

A `ep_p(src, coeffs)` helper is used throughout to pre-seed the environment
with a polynomial variable `p`, bypassing the parser ambiguity where
`[1 -3 2]` is tokenized as `[1-3, 2] = [-2, 2]` (binary minus in matrix
context). This is a known pre-existing parser limitation.
