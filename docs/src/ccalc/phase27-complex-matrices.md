# Phase 27 — Complex Matrices

**Trigger:** FFT output (Phase 26) is naturally complex; control-theory
transfer functions and non-symmetric eigenvalue problems also produce complex
matrix results. Phase 8 complex scalars are insufficient once matrix-level
complex output is needed.

**Prerequisite:** Phase 8 (complex scalars — arithmetic contract);
Phase 26 (FFT — primary consumer of complex matrix output).

---

## 27a — `Value::ComplexMatrix` and literals

A new `Value::ComplexMatrix(Array2<Complex<f64>>)` variant is added to `env.rs`.
Requires `num-complex = "0.4"` in `ccalc-engine/Cargo.toml`.

Any matrix literal where at least one element evaluates to `Value::Complex`
causes all elements to be upcast to `Complex<f64>` and the whole literal
returns `Value::ComplexMatrix`. Pure-real literals remain `Value::Matrix`.

```matlab
A = [1+2i, 3-4i; 5, 6+1i]     % 2×2 ComplexMatrix
v = [1+i, 2-i, 3]              % 1×3 ComplexMatrix
R = [1, 2; 3, 4]               % 2×2 Matrix  (stays real)
```

`isreal` returns `0` for any `ComplexMatrix`, `1` for a plain `Matrix`.

**Display:** each cell always shows both parts — `5 + 0i`, `1 + 1i`, `0 + 2i`.

**FFT integration:** `fft` output switches from the interim `Cell`
representation (Phase 26) to `Value::ComplexMatrix`. Access bins with
`X(k)` (parenthesis indexing), not `X{k}`.

**Workspace:** `ComplexMatrix` is skipped on `ws`/`wl` save (same policy as
all non-scalar types).

---

## 27b — Arithmetic and decomposition

`eval_binop` is extended for all combinations involving `ComplexMatrix`:

| Left | Right | Operation |
|------|-------|-----------|
| ComplexMatrix | ComplexMatrix | element-wise or matrix multiply |
| ComplexMatrix | Matrix | auto-promote right to complex |
| Matrix | ComplexMatrix | auto-promote left to complex |
| ComplexMatrix | Scalar / Complex | scalar broadcast |
| Scalar / Complex | ComplexMatrix | scalar broadcast |

`Expr::Transpose` (conjugate, `A'`) and `Expr::PlainTranspose` (`A.'`) both
handle `ComplexMatrix`. The conjugate transpose is the Hermitian adjoint.

Element-wise built-ins extended to `ComplexMatrix`:

| Function | Returns |
|----------|---------|
| `real(A)` | real `Matrix` |
| `imag(A)` | real `Matrix` |
| `abs(A)` | real `Matrix` (element-wise modulus) |
| `conj(A)` | `ComplexMatrix` |
| `angle(A)` | real `Matrix` (argument in radians) |
| `isreal(A)` | `Scalar(0.0)` |

Shape functions `size`, `numel`, `length`, and `norm` (Frobenius) all work.

Column-major 1-based indexing (both scalar and range) works identically to
real matrices.

---

## Files changed

| File | Change |
|------|--------|
| `crates/ccalc-engine/src/env.rs` | `Value::ComplexMatrix(Array2<Complex<f64>>)` variant |
| `crates/ccalc-engine/src/eval.rs` | Literal upcasting; `format_complex_cell` / `format_complex_matrix`; all arithmetic combinations; conjugate & plain transpose; `real`/`imag`/`abs`/`conj`/`angle`/`isreal` element-wise; `size`/`numel`/`length`/`norm`; indexing; `complex_pairs_to_complex_matrix` for FFT |
| `crates/ccalc-engine/src/exec.rs` | `is_truthy` and `print_value` for `ComplexMatrix` |
| `crates/ccalc/src/repl.rs` | Prompt display, `who`, `handle_disp` for `ComplexMatrix` |
| `crates/ccalc/src/repl_tests.rs` | Pattern updates for `ComplexMatrix` |
| `crates/ccalc-engine/src/eval_tests.rs` | Updated `fft_of` helper; 16 tests in `mod phase27_tests` |
| `crates/ccalc/src/help.rs` | `print_complex()` — removed limitations note; added matrix section |
| `docs/src/guide/complex.md` | Added "Complex Matrices" section; removed outdated "Limitations" |
| `docs/src/guide/fft.md` | Updated Cell → ComplexMatrix; `X{k}` → `X(k)`; `abs(S)` example |
| `docs/src/SUMMARY.md` | Added Phase 26 and 27 entries |
| `examples/complex_matrix.m` | Full Phase 27 demo script (Octave-compatible) |
| `examples/fft_demo.calc` | Updated Cell → ComplexMatrix API |

**Version:** v0.32.0 | **Test count:** 16 new in `phase27_tests`
