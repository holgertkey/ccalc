# Phase 26 — FFT & Signal Processing

**Trigger:** Signal processing workflows — spectrum analysis, filter design,
frequency-domain operations — require an FFT. The standard interface is
`fft(x)` / `ifft(X)` (MATLAB/Octave/NumPy compatible).

**Prerequisite:** Phase 7.5 (vector utilities — `length`, `numel`, `zeros`);
Phase 8 (complex scalars — FFT output is complex).

> **Feature flag:** `fft` and `ifft` are gated behind the `fft` Cargo feature
> (pulls in `rustfft`). Build with:
> ```bash
> cargo build --release --features fft
> ```
> `fftshift`, `ifftshift`, and `fftfreq` are always available.

---

## 26a — Forward and inverse FFT

`fft(x)` computes the DFT of a real row vector using the Cooley-Tukey
radix-2 algorithm (`rustfft`). `fft(x, n)` zero-pads (or truncates) to length
`n` before transforming.

`ifft(X)` computes the inverse DFT, normalised by 1/N. When all imaginary
parts are < 1e-12, the result is returned as a real `Matrix` instead.

**Note (Phase 27):** the return type changed from `Cell` to `ComplexMatrix`
in v0.32.0. Access bins with `X(k)` (parenthesis indexing), not `X{k}`.

---

## 26b — `fftshift` / `ifftshift`

`fftshift(x)` performs a circular shift by `floor(N/2)` so that the DC
component moves from index 1 to the centre. Used to produce a zero-centred
spectrum plot.

`ifftshift(x)` undoes the shift by `ceil(N/2)`. Works on row vectors,
column vectors, and 2-D matrices (shifts both dimensions).

---

## 26c — `fftfreq`

`fftfreq(n, d)` returns a 1×n row vector of DFT sample frequencies for `n`
points with sample spacing `d` seconds (so `d = 1/fs`). The formula matches
NumPy and MATLAB:

```
f = [0, 1, …, floor((n-1)/2), -floor(n/2), …, -1] / (n·d)
```

---

## Files changed

| File | Change |
|------|--------|
| `crates/ccalc-engine/src/eval.rs` | `fft`, `fft(x,n)`, `ifft`, `fftshift`, `ifftshift`, `fftfreq` in `call_builtin`; `complex_pairs_to_complex_matrix` helper; gated under `#[cfg(feature = "fft")]` |
| `crates/ccalc-engine/src/eval_tests.rs` | FFT regression tests |
| `crates/ccalc/src/help.rs` | `print_fft()` topic |
| `docs/src/guide/fft.md` | User guide page |
| `docs/src/SUMMARY.md` | Added entry |
| `examples/fft_demo.calc` | Full worked example |

**Version:** v0.31.0
