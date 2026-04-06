# Phase 8 — Complex Numbers

**Version:** v0.12.0

## What was added

### Value::Complex

A third variant was added to the `Value` enum in `env.rs`:

```rust
pub enum Value {
    Scalar(f64),
    Matrix(Array2<f64>),
    Complex(f64, f64),   // re, im
}
```

`make_complex(re, im)` in `eval.rs` collapses the result to `Scalar(re)`
when `im == 0.0` exactly, so operations like `(1+i)*(1-i)` produce `Scalar(2)`
rather than `Complex(2, 0)`.

### Imaginary units i and j

`i` and `j` are pre-seeded in `Env` at startup as `Complex(0.0, 1.0)`,
matching Octave/MATLAB behaviour. The user can overwrite them with
`i = 5` (the new value shadows the imaginary unit for the session).
`clear i` removes the variable; the imaginary unit is not automatically
restored (document limitation).

### Syntax — no new tokens

`4i` tokenizes as `Number(4) Ident("i")`, which triggers implicit
multiplication. With `i = Complex(0.0, 1.0)` in `Env`:

```
4*i       →  Complex(0.0, 4.0)
3 + 4*i   →  Complex(3.0, 4.0)
```

No tokenizer or parser changes were needed.

### Arithmetic

`complex_binop(re1, im1, op, re2, im2)` handles all four combination groups:

| Left | Right | Routing |
|------|-------|---------|
| Complex | Complex | `complex_binop` directly |
| Complex | Scalar  | `complex_binop(re, im, op, s, 0.0)` |
| Scalar  | Complex | `complex_binop(s, 0.0, op, re, im)` |
| Complex | Matrix  | error |

Supported operations:

| Op | Formula |
|----|---------|
| `+` | `(a+c) + (b+d)i` |
| `-` | `(a-c) + (b-d)i` |
| `*` / `.*` | `(ac-bd) + (ad+bc)i` |
| `/` / `./` | `((ac+bd) + (bc-ad)i) / (c²+d²)` |
| `^` / `.^` integer | binary exponentiation (exact) |
| `^` / `.^` general | polar form `exp((c+di)·ln(a+bi))` |
| `==` | `1` if both parts equal |
| `~=` | `1` if either part differs |
| `<`/`>`/`<=`/`>=` | error: "Ordering is not defined for complex numbers" |
| `&&` / `\|\|` | nonzero test on `re != 0 || im != 0` |

### Powers — integer exactness

Integer powers use **binary exponentiation** (repeated squaring) to avoid
polar-form floating-point error:

```
i^2  =  -1         (exact, not -1 + 1.22e-16i)
i^3  =  -i         (exact)
i^4  =   1         (exact)
```

General non-integer powers fall back to the polar form
`exp((c+di) * (ln|z| + i*arg(z)))`.

### Unary operators

| Op | Result |
|----|--------|
| `-z` | `Complex(-re, -im)` |
| `~z` | `0` if `re≠0` or `im≠0`, else `1` |
| `z'` | `Complex(re, -im)` — conjugate transpose |

### Display

`format_complex(re, im, precision)` in `eval.rs`:

| Condition | Display |
|-----------|---------|
| `im == 0` | `a` (same as scalar) |
| `re == 0, im > 0` | `bi` (or `i` when `im == 1`) |
| `re == 0, im < 0` | `-bi` (or `-i`) |
| `im > 0`  | `a + bi` |
| `im < 0`  | `a - \|b\|i` |

The REPL prompt shows the complex value directly (e.g. `[ 3 + 4i ]:`).

### Built-in functions

| Function | Description |
|----------|-------------|
| `real(z)` | Real part; returns `z` unchanged for scalars |
| `imag(z)` | Imaginary part; returns `0` for scalars |
| `abs(z)` | Modulus `sqrt(re²+im²)`; overloads scalar and matrix `abs` |
| `angle(z)` | Argument `atan2(im, re)` in radians |
| `conj(z)` | Complex conjugate `re - im*i` |
| `complex(re, im)` | Construct from two real scalars |
| `isreal(z)` | `1` if `im == 0`, else `0` |

### scalar_arg compatibility

`scalar_arg(v, name, pos)` now accepts `Complex` with `im == 0` as a real
scalar, so `sqrt(complex(4, 0))` returns `2` without error.

### Scope boundary

Complex matrices (e.g. `[1+2i, 3+4i]`) are **out of scope** for this phase.
A matrix literal containing a `Complex` element returns:

```
Error: Complex elements in matrix literals are not supported yet
```

Complex variables are **not persisted** by `ws`/`wl` (same policy as matrices).

## Files changed

| File | Change |
|------|--------|
| `crates/ccalc-engine/src/env.rs` | Added `Value::Complex(f64, f64)` |
| `crates/ccalc-engine/src/eval.rs` | `complex_binop`, `make_complex`, `format_complex`, built-ins, exhaustive match arms |
| `crates/ccalc/src/repl.rs` | `new_env` seeds `i`/`j`; all output paths handle `Value::Complex` |
| `crates/ccalc-engine/src/eval_tests.rs` | 38 new tests covering all operations |
| `examples/complex_numbers.calc` | New annotated example file |
