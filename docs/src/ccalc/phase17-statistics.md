# Phase 17 — Statistics & Random Numbers

**Version:** v0.21.0+015 – v0.21.0+017

Purely a built-in library addition — no new tokens, AST nodes, or parser changes.
Depends on Phase 15 (indexed assignment) for statistical algorithms that build
result matrices element-by-element.

---

## 17a — Random number generation (v0.21.0+015)

**New crate dependency:** `rand = { version = "0.8", features = ["small_rng"] }`
added to `crates/ccalc-engine/Cargo.toml`.

**Thread-local RNG** (`eval.rs`):

```rust
thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}
```

`rand_uniform()` uses `gen_range(0.0_f64..1.0)` (not `gen::<f64>()` — `gen` is
a reserved keyword in Rust 2024 edition).

`rand_normal()` uses the Box-Muller transform — avoids the `rand_distr` crate:

```rust
fn rand_normal() -> f64 {
    let u1 = rand_uniform().max(f64::EPSILON);
    let u2 = rand_uniform();
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}
```

**`no_ans_inject` list** updated to include `"rand" | "randn" | "rng"` — without
this, zero-argument calls like `rand()` injected `ans` (scalar 0.0) and matched
the 1-argument case (n=0), producing a 0×0 matrix instead of a scalar.

**New built-ins:** `rand`, `randn`, `randi`, `rng`

---

## 17b — Descriptive statistics (v0.21.0+016)

**New crate dependency:** `libm = "0.2"` added for `erf`/`erfc` (Rust std does
not expose these). Used in Phase 17d.

**Helpers added to `eval.rs`:**

- `numeric_vec(v, fname)` — extracts a `Vec<f64>` from Scalar/Matrix, error on Complex/Str
- `apply_stat(v, f, fname)` — column-wise reduction helper (same shape rules as `apply_reduction`)
- `stat_var_vec(vals, population)` — shared variance/std computation

`apply_stat` reuses the same column-wise pattern as `apply_reduction`: vectors
collapse to a scalar, M×N matrices collapse each column to produce a 1×N row vector.

**`hist` terminal-width awareness:** reads `COLUMNS` env var, falls back to 80.

**New built-ins:** `std`, `var`, `cov`, `median`, `mode`, `hist`, `histc`

---

## 17c — Percentiles and distributions (v0.21.0+016)

**`percentile_sorted(sorted, p)`** — linear interpolation using index `p/100 * (n-1)`:

```rust
fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let idx = p / 100.0 * (sorted.len() - 1) as f64;
    let lo = sorted[idx.floor() as usize];
    let hi = sorted[idx.ceil() as usize];
    lo + (hi - lo) * idx.fract()
}
```

`zscore` returns a zero vector for constant input to avoid division by zero.

`prctile` handles both scalar `p` and vector `p` as the second argument.

**New built-ins:** `prctile`, `iqr`, `zscore`

---

## 17d — Normal distribution functions (v0.21.0+017)

`erf` and `erfc` delegate to `libm::erf` / `libm::erfc`.
Element-wise on matrices via the existing `apply_elem` helper.

`normcdf` and `normpdf` are pure one-liners on top of `erf`:

```
normcdf(x) = 0.5 * (1 + erf(x / sqrt(2)))
normpdf(x) = exp(-x^2 / 2) / sqrt(2 * pi)
```

No additional dependencies beyond `libm`.

**New built-ins:** `erf`, `erfc`, `normcdf`, `normpdf`

---

## Tests

54 new tests in `crates/ccalc-engine/src/eval_tests.rs`:
- 11 for Phase 17a (rand/randn/randi/rng reproducibility and shape)
- 21 for Phase 17b (std/var/cov/median/mode/hist/histc)
- 22 for Phase 17c+17d (prctile/iqr/zscore/erf/erfc/normcdf/normpdf)

## Example

See `examples/statistics.calc` for a full demo (200-sample simulation,
percentile table, ASCII histogram, covariance matrix, normal distribution checks).
